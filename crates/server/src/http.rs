//! Optional buffered HTTP conversion over the common host lifecycle.
//!
//! Callers own input buffering; these limits do not bound network reads. No socket,
//! executor, task or stream enters the framework request. Only the checked method
//! and exact URI pathname select the operation; applications own authorization.
//!
//! Session cookie revisions accept scalar strings. List values require an explicit
//! codec and currently produce an Output failure naming the cookie, with sanitized
//! empty status-500 output. An explicit codec option on [`HttpOptions`] is future work.

use http::header::{CONTENT_LENGTH, CONTENT_TYPE, SET_COOKIE, TRANSFER_ENCODING};
use http::{HeaderMap, HeaderName, HeaderValue, Uri};
use stackpress_lib::data::parse_body;
use stackpress_lib::{
    Body, DeliveryState, Error, Headers, LoadResult, ParseLimits, Request, Response, Result,
    Session, SessionRevision, SessionValue,
};

use crate::{HandleOptions, HandleReport, HostReply, OutputGuard, Server};

/// Caller-owned already-buffered HTTP input; this type does not bound network reads.
pub type BufferedRequest = http::Request<Vec<u8>>;
/// Adapter-produced buffered native output; no stream/backpressure support is implied.
pub type BufferedResponse = http::Response<Vec<u8>>;

/// Trusted configuration, retained immutably after validation.
///
/// All budgets are byte/count ceilings, including zero; none means unlimited.
/// Request totals count method, URI, every header occurrence and body, without
/// inventing a particular HTTP version's wire framing overhead.
///
/// # Examples
/// ```
/// use stackpress_server::http::{HttpAdapter, HttpOptions};
/// let defaults = HttpOptions::default();
/// assert_eq!(defaults.base_url, "http://unknownhost/");
/// assert_eq!(defaults.max_headers, 256);
/// // Change trusted policy before creating the immutable adapter.
/// let options = HttpOptions {
///     base_url: "https://profiles.example/".into(),
///     max_header_bytes: 16 * 1024,
///     ..defaults
/// };
/// let adapter = HttpAdapter::new(options)?;
/// # Ok::<(), stackpress_lib::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct HttpOptions {
    /// Absolute HTTP(S) origin with root path, without credentials, query or fragment.
    /// Host and forwarded headers never override it for origin-form input.
    pub base_url: String,
    /// Request data budgets; `max_bytes` also bounds the body and URI separately.
    pub limits: ParseLimits,
    /// Initial response budgets; a replacement Response owns its replacement limits.
    pub response_limits: ParseLimits,
    /// Aggregate input bytes, checked before copying headers or parsing data.
    pub max_request_bytes: usize,
    /// Header name/value bytes, counting the name for every occurrence.
    pub max_header_bytes: usize,
    /// Maximum header occurrences, including repeated names.
    pub max_headers: usize,
}

impl Default for HttpOptions {
    /// Choose finite input/response policies and the explicit unknown-host origin.
    fn default() -> Self {
        // Independent aggregate/header ceilings prevent many small fields bypassing body limits.
        Self {
            base_url: "http://unknownhost/".into(),
            limits: ParseLimits::default(),
            response_limits: ParseLimits::default(),
            max_request_bytes: 2 * 1024 * 1024,
            max_header_bytes: 64 * 1024,
            max_headers: 256,
        }
    }
}

/// Native availability, independent of lifecycle success or cooperative Abort.
///
/// # Examples
/// ```
/// use stackpress_lib::{DeliveryState, Flow, Invocation, Request, Response, Result};
/// use stackpress_server::{ServerBuilder, http::{HttpAdapter, HttpOptions, HttpOutput}};
/// async fn suppress(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.stop()?;
///     Ok(Flow::Abort)
/// }
/// async fn finalize(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
///     res.dispatch().await?;
///     Ok(Flow::Abort)
/// }
/// # async fn example() -> Result<()> {
/// let mut builder = ServerBuilder::new();
/// builder.get("/suppressed", suppress)?;
/// builder.get("/elsewhere", finalize)?;
/// let app = builder.build().await.map_err(|error| error.error)?;
/// let adapter = HttpAdapter::new(HttpOptions::default())?;
/// for path in ["/missing", "/suppressed", "/elsewhere"] {
///     let input = http::Request::builder().uri(path).body(Vec::new())
///         .map_err(stackpress_lib::Error::from_source)?;
///     let reply = adapter.handle(&app, input).await?;
///     match reply.output {
///         HttpOutput::Response(native) => assert_eq!(native.status(), 404),
///         HttpOutput::Suppressed => assert_eq!(path, "/suppressed"),
///         // Foundation finalization does not supply this adapter's native response.
///         HttpOutput::Unavailable { state } => assert_eq!(state, DeliveryState::Sent),
///     }
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug)]
pub enum HttpOutput {
    /// The single response produced by this adapter's output authority.
    Response(BufferedResponse),
    /// Explicitly stopped output; no native response was fabricated.
    Suppressed,
    /// Foreign, failed or interrupted delivery without this adapter's native result.
    Unavailable {
        /// Observed framework state; it never authorizes a second write.
        state: DeliveryState,
    },
}

/// Buffered native output and separately disclosed host diagnostics.
#[derive(Debug)]
pub struct HttpReply {
    /// Explicit native bytes, suppression or foreign finalization.
    pub output: HttpOutput,
    /// Ordered diagnostics; never serialized automatically.
    pub report: HandleReport,
}

/// Immutable converter, with separate output authority for each invocation.
pub struct HttpAdapter {
    /// Validated trusted policy, inaccessible to request fields and handlers.
    options: HttpOptions,
    /// Scheme/authority prefix; concatenation preserves repeated leading slashes.
    origin: String,
}

impl HttpAdapter {
    /// Validate a trusted base and retain finite policies without opening resources.
    ///
    /// # Errors
    /// Rejects non-HTTP(S), credential-bearing or non-origin base URLs.
    ///
    /// # Examples
    /// ```
    /// use stackpress_server::http::{HttpAdapter, HttpOptions};
    /// let adapter = HttpAdapter::new(HttpOptions {
    ///     base_url: "https://example.test/".into(),
    ///     ..Default::default()
    /// })?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(options: HttpOptions) -> Result<Self> {
        // Reuse foundation URL parsing rather than adding another dependency/policy.
        let base = Request::builder().url(options.base_url.clone()).build()?;
        // Require a web origin; URL parsing alone also accepts unrelated schemes/relative input.
        if !matches!(base.url.scheme(), "http" | "https")
            || base.url.host_str().is_none()
            // Request credentials must never be inherited from trusted base configuration.
            || !base.url.username().is_empty()
            || base.url.password().is_some()
            // A base supplies authority only; request targets supply their own path/query.
            || base.url.path() != "/"
            || base.url.query().is_some()
            || base.url.fragment().is_some()
            // Foundation construction can prefix relative paths, so check explicit origin syntax.
            || !options.base_url.contains("://")
        {
            // Origin-form requests must have exactly one explicitly trusted origin.
            return Err(Error::invalid(
                "HTTP base must be an absolute HTTP(S) origin",
            ));
        }
        // Remove only the validated root suffix before literal request-target concatenation.
        let origin = base.url.as_str().trim_end_matches('/').to_owned();
        Ok(Self { options, origin })
    }

    /// Normalize input, run one lifecycle and publish at most one buffered response.
    ///
    /// JSON requires an object root. JSON, urlencoded and multipart use strict lib
    /// parsing; other MIME types retain raw bytes. Normalized parse failures enter
    /// the Input error offer; normalization failures return 400 without callbacks.
    /// Conversion failure records Output and publishes one fixed empty 500. Dropping
    /// this future stops further work; no background task completes it.
    ///
    /// # Errors
    /// Outer errors mean reporting could not start, e.g. exhausted correlation IDs.
    /// Normal input/handler/conversion failures are retained in the reply's report.
    /// Handler panics propagate under the ordinary host contract.
    ///
    /// # Examples
    /// ```
    /// use stackpress_server::{ServerBuilder, http::{HttpAdapter, HttpOptions, HttpOutput}};
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// let app = ServerBuilder::new().build().await.map_err(|error| error.error)?;
    /// let input = http::Request::builder().uri("/missing").body(Vec::new())
    ///     .map_err(stackpress_lib::Error::from_source)?;
    /// let reply = HttpAdapter::new(HttpOptions::default())?.handle(&app, input).await?;
    /// assert!(matches!(reply.output, HttpOutput::Response(response) if response.status() == 404));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle(&self, app: &Server, input: BufferedRequest) -> Result<HttpReply> {
        // Preserve HEAD suppression even when normalization cannot produce a Request.
        let head = input.method().as_str().eq_ignore_ascii_case("HEAD");
        let reply = match self.normalize(&input) {
            Ok((mut request, event)) => {
                // Start a caller-local response with trusted budgets, separate from input policy.
                let mut response = Response::with_limits(self.options.response_limits);
                // Move the original buffer into one loader; no native reader can be retried.
                let limits = self.options.limits;
                let bytes = input.into_body();
                // The loader stages body/post atomically inside the common Input runner.
                let report = app
                    .handle_input(
                        &event,
                        &mut request,
                        &mut response,
                        HandleOptions::default(),
                        move |view| async move { load_body(&view.mimetype, bytes, limits) },
                    )
                    .await?;
                // Retain response ownership beside diagnostics until the later output phase.
                HostReply { response, report }
            }
            Err(error) => {
                // No normalized Request exists; do not fabricate one for callbacks (G17).
                app.input_failure("<invalid HTTP input>", error)?
            }
        };
        // Native publication remains outside handler reach, including Response replacement.
        Ok(finish(reply, head).await)
    }

    /// Validate bounded input and retain the native pathname as route authority.
    fn normalize(&self, input: &BufferedRequest) -> Result<(Request, String)> {
        // Reject raw excess before header copies, URL parsing or body decoding.
        check_input_limits(input, &self.options)?;
        // Borrow the native pathname before any URL library can canonicalize it.
        let uri = input.uri();
        let path = uri.path();
        if !path.starts_with('/') {
            // Authority-form CONNECT and asterisk-form OPTIONS belong to real transports.
            return Err(Error::invalid("HTTP request target must have a pathname"));
        }
        // Construct informational URL authority independently of Host/forwarded header values.
        let url = request_url(uri, &self.origin)?;
        // Append every occurrence so both ordinary repeats and separate Cookie lines survive.
        let mut headers = Headers::new();
        for (name, value) in input.headers() {
            // Reject opaque binary header values instead of guessing a text codec.
            let value = value.to_str().map_err(Error::from_source)?;
            headers.append(name.as_str(), value)?;
        }
        // An absent type uses the foundation default; multiple types cannot select one parser.
        let types = headers.get_all("content-type").unwrap_or_default();
        if types.len() > 1 {
            // Ambiguous parser selection cannot silently choose the first occurrence.
            return Err(Error::invalid("multiple HTTP content types"));
        }
        // Own the MIME before moving its borrowed header storage into the request builder.
        let mime = types
            .first()
            .map_or("text/plain", String::as_str)
            .to_owned();
        // Foundation construction validates method, MIME, query, headers and initial cookies.
        let request = Request::builder()
            .url(url)
            .method(input.method().as_str())
            .headers(headers)
            .mimetype(mime)
            .limits(self.options.limits)
            .build()?;
        // Refuse canonicalization or hidden credentials instead of dispatching a changed target.
        if request.url.path() != path
            || !request.url.username().is_empty()
            || request.url.password().is_some()
            || request.url.fragment().is_some()
        {
            // WHATWG dot/backslash normalization must never silently change a route.
            return Err(Error::invalid(
                "HTTP target changes during URL normalization",
            ));
        }
        // Route authority uses the native pathname, never query/post/header event suggestions.
        let event = format!("{} {path}", request.method.as_str());
        Ok((request, event))
    }
}

/// Bound raw buffers and every header occurrence before normalized allocations.
fn check_input_limits(input: &BufferedRequest, options: &HttpOptions) -> Result<()> {
    // Charge the whole native target, including query text, before decoded allocations.
    let uri_bytes = input.uri().to_string().len();
    // Count wire occurrences, including repeated names, rather than only distinct header keys.
    let mut header_bytes = 0usize;
    for (name, value) in input.headers() {
        // Saturation makes overflow exceed any practical finite configured ceiling.
        header_bytes = header_bytes.saturating_add(name.as_str().len());
        header_bytes = header_bytes.saturating_add(value.as_bytes().len());
    }
    // The total is a buffer budget; it makes no claim about version-specific network framing.
    let total = input
        .method()
        .as_str()
        .len()
        .saturating_add(uri_bytes)
        .saturating_add(header_bytes)
        .saturating_add(input.body().len());
    // Equality is permitted; zero is a strict ceiling rather than an unlimited sentinel.
    if uri_bytes > options.limits.max_bytes
        || input.body().len() > options.limits.max_bytes
        || header_bytes > options.max_header_bytes
        || input.headers().len() > options.max_headers
        || total > options.max_request_bytes
    {
        // No normalized request is published, so no callback sees oversized input.
        return Err(Error::limit("buffered HTTP input limit exceeded"));
    }
    Ok(())
}

/// Form an absolute informational URL without relative resolution/forwarded trust.
fn request_url(uri: &Uri, origin: &str) -> Result<String> {
    match (uri.scheme_str(), uri.authority()) {
        (None, None) => {
            // Literal concatenation preserves //path instead of treating it as another host.
            Ok(format!("{origin}{uri}"))
        }
        (Some(scheme), Some(_))
            if scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https") =>
        {
            // An absolute web target keeps its own checked authority and exact path spelling.
            Ok(uri.to_string())
        }
        _ => {
            // Never fall back for malformed absolute/proxy targets or unsupported schemes.
            Err(Error::invalid(
                "HTTP target requires HTTP(S) authority or origin form",
            ))
        }
    }
}

/// Stage a complete raw body and optional strict structured fields together.
fn load_body(mime: &str, bytes: Vec<u8>, limits: ParseLimits) -> Result<LoadResult> {
    // Ignore MIME parameters only for dispatch; the strict parser still receives the whole type.
    let kind = mime
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase();
    // JSON aliases/suffixes share the object-root policy; multipart retains boundary parameters.
    let structured = matches!(
        kind.as_str(),
        "application/json"
            | "text/json"
            | "application/x-www-form-urlencoded"
            | "multipart/form-data"
    ) || kind.ends_with("+json");
    // Only supported formats populate post; unknown MIME retains exact bytes without fields.
    let post = if structured {
        // Any parse error leaves both body and post uninstalled in Request::load_with.
        Some(parse_body(mime, &bytes, &limits)?)
    } else {
        // Text, binary and custom formats are opaque bytes, not silently empty form objects.
        None
    };
    // Publish one staged result so aggregate merge validation can still reject it atomically.
    Ok(LoadResult {
        body: Some(Body::Bytes(bytes)),
        post,
    })
}

/// Convert all fallible state before consuming the invocation's private output authority.
async fn finish(mut reply: HostReply, head: bool) -> HttpReply {
    // This authority belongs to the adapter call, independently of any handler Response value.
    let mut guard = OutputGuard::default();
    match reply.response.delivery_state() {
        DeliveryState::Stopped => {
            // Suppression consumes authority without invoking a native publisher.
            let _ = guard
                .dispatch(&mut reply.response, |_| async { Ok(()) })
                .await;
            // Stopped is deliberate suppression, not a native write or successful HTTP body.
            reply.report.record_output(&reply.response);
            return HttpReply {
                output: HttpOutput::Suppressed,
                report: reply.report,
            };
        }
        // Only an open response can proceed to fallible conversion and native publication.
        DeliveryState::Open => {}
        state => {
            // Foundation finalization did not produce this adapter's native buffered output.
            reply.report.record_output_failure(
                Error::new("HTTP response finalized outside adapter authority"),
                &reply.response,
            );
            return HttpReply {
                output: HttpOutput::Unavailable { state },
                report: reply.report,
            };
        }
    }
    // Complete conversion/fallback before the guard can enter native publication.
    let prepared = prepare_or_fallback(&mut reply, head);
    // Publication has no hook dispatch or retry path and consumes the same private authority.
    let output = publish(&mut reply, &mut guard, prepared).await;
    HttpReply {
        output,
        report: reply.report,
    }
}

/// Choose a fully prepared response, retaining conversion failure before sanitized replacement.
fn prepare_or_fallback(reply: &mut HostReply, head: bool) -> BufferedResponse {
    // Preflight touches no native writer; an error can still safely choose the sole output.
    match prepare(&reply.response, head) {
        Ok(response) => response,
        Err(error) => {
            // Preserve the failed conversion and old delivery state before replacing any values.
            reply.report.record_output_failure(error, &reply.response);
            // Drop all handler state, including unusable zero-node budgets, for a fixed fallback.
            reply.response = Response::new();
            // Fresh state is Open; setting this standard status cannot fail its guarded setter.
            let _ = reply.response.status_code(500, None);
            // Match framework status with a fixed empty native response, disclosing no old data.
            let mut response = BufferedResponse::new(Vec::new());
            *response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            response
                .headers_mut()
                .insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
            response
        }
    }
}

/// Publish an already converted buffer once, retaining any final guard/state failure.
async fn publish(
    reply: &mut HostReply,
    guard: &mut OutputGuard,
    prepared: BufferedResponse,
) -> HttpOutput {
    // Only successful entry into the guarded callback may fill this caller-local output slot.
    let mut native = None;
    // Publication is an infallible move after preflight, with no external I/O or retry.
    let result = guard
        .dispatch(&mut reply.response, |_| async {
            // Moving prepared bytes is the publication step; no conversion remains here.
            native = Some(prepared);
            Ok(())
        })
        .await;
    if let Err(error) = result {
        // Preserve honest availability if a final foundation state check ever fails.
        reply.report.record_output_failure(error, &reply.response);
    } else {
        // Successful dispatch records Sent after the guard has consumed its one attempt.
        reply.report.record_output(&reply.response);
    }
    // Missing publication never invents a native response from foreign or failed state.
    native.map_or(
        HttpOutput::Unavailable {
            state: guard.state(),
        },
        HttpOutput::Response,
    )
}

/// Preflight foundation budgets, HTTP legality and framing before native publication.
fn prepare(response: &Response, head: bool) -> Result<BufferedResponse> {
    // Handler replacement can change budgets, so use the final Response's policy every time.
    let limits = response.limits();
    // Mirror dispatch validation before its callback, including publicly replaceable stores.
    response.body().validate(&limits)?;
    response.data().value().validate(&limits)?;
    response.errors().value().validate(&limits)?;
    // The host normally supplies status; absent status still represents an empty completion.
    let code = response.status().map_or(204, |status| status.code);
    // Buffered output represents a completed response, never interim 1xx or application codes.
    if !(200..=599).contains(&code) {
        return Err(Error::invalid(
            "completed HTTP status must be between 200 and 599",
        ));
    }
    // Headers/cookies are checked even when the selected status forbids a body.
    let mut headers = response_headers(response, &limits)?;
    // These statuses cannot carry payload bytes; HEAD is handled later as representation
    // suppression.
    let body = if matches!(code, 204 | 205 | 304) {
        // Forbidden-body statuses need no representation serialization.
        Vec::new()
    } else {
        // Compute the selected representation before HEAD suppresses its wire body.
        representation(response, &mut headers)?
    };
    // Framing owns the final body/headers; it cannot mutate framework response state.
    frame_response(code, headers, body, head)
}

/// Set adapter-owned framing and suppress HEAD bytes after measuring the representation.
fn frame_response(
    code: u16,
    mut headers: HeaderMap,
    body: Vec<u8>,
    head: bool,
) -> Result<BufferedResponse> {
    // Caller-supplied lengths never control actual buffered framing.
    headers.remove(CONTENT_LENGTH);
    // 204/304 omit length; 205 receives zero because prepare already discarded its body.
    if !matches!(code, 204 | 304) {
        // Count bytes before HEAD suppression, retaining the selected representation's length.
        let length = HeaderValue::from_str(&body.len().to_string()).map_err(Error::from_source)?;
        headers.insert(CONTENT_LENGTH, length);
    }
    // HEAD never exposes its computed representation bytes or silently chooses a GET handler.
    let mut native = BufferedResponse::new(if head { Vec::new() } else { body });
    // Install checked status and completed headers before returning the preflight candidate.
    *native.status_mut() = http::StatusCode::from_u16(code).map_err(Error::from_source)?;
    *native.headers_mut() = headers;
    Ok(native)
}

/// Validate and append repeated headers/cookies without comma folding.
fn response_headers(response: &Response, limits: &ParseLimits) -> Result<HeaderMap> {
    // Keep each native value separate; no field joins can accidentally fold Set-Cookie.
    let mut headers = HeaderMap::new();
    let mut bytes = 0usize;
    let mut count = 0usize;
    for (name, values) in response.headers().entries() {
        // Match foundation storage budgets: name once, each value occurrence once.
        bytes = bytes.saturating_add(name.len());
        for value in values {
            // Each value consumes one entry and its own bytes, matching dispatch validation.
            bytes = bytes.saturating_add(value.len());
            count = count.saturating_add(1);
            // Convert names with the native HTTP validator before applying transport policy.
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(Error::from_source)?;
            if name == TRANSFER_ENCODING {
                // Transfer coding requires real transport ownership beyond this buffer.
                return Err(Error::invalid(
                    "buffered HTTP output forbids Transfer-Encoding",
                ));
            }
            // Reject invalid native field values while every candidate is still unpublished.
            let value = HeaderValue::from_str(value).map_err(Error::from_source)?;
            headers.append(name, value);
        }
    }
    // Recheck final Response limits; Headers mutation used its independent foundation defaults.
    if bytes > limits.max_bytes || count > limits.max_entries {
        return Err(Error::limit("response header limit exceeded"));
    }
    // Cookie encoding is a separate boundary from ordinary HTTP header conversion.
    append_cookie_headers(response.session(), &mut headers)?;
    Ok(headers)
}

/// Append checked scalar cookie revisions, retaining scoped removals and diagnostic identity.
fn append_cookie_headers(session: &Session, headers: &mut HeaderMap) -> Result<()> {
    // Diagnose the offending key before scalar serialization can return an unnamed codec error.
    for (name, revision) in session.revisions() {
        if matches!(
            revision,
            SessionRevision::Set {
                value: SessionValue::List(_),
                ..
            }
        ) {
            // Name the unsupported revision only in diagnostics; the wire fallback stays empty.
            return Err(Error::invalid(format!(
                "cookie {name:?}: session list value requires an explicit codec"
            )));
        }
    }
    // Foundation serialization validates attributes and forces scoped removal expiry/max-age.
    for cookie in session.cookie_headers()? {
        // Checked scalar serialization retains deletion scope and separate revisions.
        let value = HeaderValue::from_str(&cookie).map_err(Error::from_source)?;
        headers.append(SET_COOKIE, value);
    }
    Ok(())
}

/// Preserve text/bytes; structured and status-only output use the public envelope.
fn representation(response: &Response, headers: &mut HeaderMap) -> Result<Vec<u8>> {
    // Explicit raw bodies bypass the envelope, retaining both empty buffers and non-UTF-8 bytes.
    let (body, mime) = match response.body() {
        Body::Text(text) => (text.as_bytes().to_vec(), response.mimetype()),
        Body::Bytes(bytes) => (bytes.clone(), response.mimetype()),
        Body::Empty | Body::Value(_) => {
            // Keep the owning Response limits and presence of null/false/zero/total=0.
            let json = response.try_to_json()?;
            (serde_json::to_vec(&json)?, Some("application/json"))
        }
    };
    // An absent MIME keeps an explicit header; structured envelopes always supply JSON MIME.
    if let Some(mime) = mime {
        // Explicit MIME wins over stale user headers; an envelope always declares JSON.
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime).map_err(Error::from_source)?,
        );
    }
    Ok(body)
}
