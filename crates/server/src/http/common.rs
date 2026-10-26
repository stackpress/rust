//! Private HTTP normalization, strict loading and buffered conversion shared by adapters.

use http::header::{CONTENT_LENGTH, CONTENT_TYPE, SET_COOKIE, TRANSFER_ENCODING};
use http::{HeaderMap, HeaderName, HeaderValue, Uri};
use stackpress_lib::data::parse_body;
use stackpress_lib::{
    Body, Error, ErrorKind, Headers, LoadResult, ParseLimits, Request, Response, Result, Session,
    SessionRevision, SessionValue,
};

use super::{BufferedResponse, HttpOptions};

/// Validate bounded input and retain the native pathname as route authority.
pub(super) fn normalize(
    input: &http::request::Parts,
    body_bytes: usize,
    options: &HttpOptions,
    origin: &str,
) -> Result<(Request, String)> {
    // Reject raw excess before header copies, URL parsing or body decoding.
    check_input_limits(input, body_bytes, options)?;
    // Borrow the native pathname before any URL library can canonicalize it.
    let uri = &input.uri;
    let path = uri.path();
    if !path.starts_with('/') {
        // Authority-form CONNECT and asterisk-form OPTIONS belong to real transports.
        return Err(Error::invalid("HTTP request target must have a pathname"));
    }
    // Construct informational URL authority independently of Host/forwarded header values.
    let url = request_url(uri, origin)?;
    // Append every occurrence so both ordinary repeats and separate Cookie lines survive.
    let mut headers = Headers::new();
    for (name, value) in &input.headers {
        // Reject opaque binary header values instead of guessing a text codec.
        let value = value.to_str().map_err(|cause| {
            Error::new("HTTP request header decoding failed")
                .with_kind(ErrorKind::External)
                .with_code(500)
                .with_public_message("Internal Server Error")
                .with_cause(cause)
        })?;
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
        .method(input.method.as_str())
        .headers(headers)
        .mimetype(mime)
        .limits(options.limits)
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

/// Bound raw buffers and every header occurrence before normalized allocations.
pub(super) fn check_input_limits(
    input: &http::request::Parts,
    body_bytes: usize,
    options: &HttpOptions,
) -> Result<()> {
    // Charge the whole native target, including query text, before decoded allocations.
    let uri_bytes = input.uri.to_string().len();
    // Count wire occurrences, including repeated names, rather than only distinct header keys.
    let mut header_bytes = 0usize;
    for (name, value) in &input.headers {
        // Saturation makes overflow exceed any practical finite configured ceiling.
        header_bytes = header_bytes.saturating_add(name.as_str().len());
        header_bytes = header_bytes.saturating_add(value.as_bytes().len());
    }
    // The total is a buffer budget; it makes no claim about version-specific network framing.
    let total = input
        .method
        .as_str()
        .len()
        .saturating_add(uri_bytes)
        .saturating_add(header_bytes)
        .saturating_add(body_bytes);
    // Equality is permitted; zero is a strict ceiling rather than an unlimited sentinel.
    if uri_bytes > options.limits.max_bytes
        || body_bytes > options.limits.max_bytes
        || header_bytes > options.max_header_bytes
        || input.headers.len() > options.max_headers
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
pub(super) fn load_body(mime: &str, bytes: Vec<u8>, limits: ParseLimits) -> Result<LoadResult> {
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

/// Preflight foundation budgets, HTTP legality and framing before native publication.
pub(super) fn prepare(response: &Response, head: bool) -> Result<BufferedResponse> {
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
pub(super) fn frame_response(
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
        let length = HeaderValue::from_str(&body.len().to_string()).map_err(|cause| {
            Error::new("HTTP response content length encoding failed")
                .with_kind(ErrorKind::External)
                .with_code(500)
                .with_public_message("Internal Server Error")
                .with_cause(cause)
        })?;
        headers.insert(CONTENT_LENGTH, length);
    }
    // HEAD never exposes its computed representation bytes or silently chooses a GET handler.
    let mut native = BufferedResponse::new(if head { Vec::new() } else { body });
    // Install checked status and completed headers before returning the preflight candidate.
    *native.status_mut() = http::StatusCode::from_u16(code).map_err(|cause| {
        Error::new("HTTP response status conversion failed")
            .with_kind(ErrorKind::External)
            .with_code(500)
            .with_public_message("Internal Server Error")
            .with_cause(cause)
    })?;
    *native.headers_mut() = headers;
    Ok(native)
}

/// Validate and append repeated headers/cookies without comma folding.
pub(super) fn response_headers(response: &Response, limits: &ParseLimits) -> Result<HeaderMap> {
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
            let name = HeaderName::from_bytes(name.as_bytes()).map_err(|cause| {
                Error::new("HTTP response header name conversion failed")
                    .with_kind(ErrorKind::External)
                    .with_code(500)
                    .with_public_message("Internal Server Error")
                    .with_cause(cause)
            })?;
            if name == TRANSFER_ENCODING {
                // Transfer coding requires real transport ownership beyond this buffer.
                return Err(Error::invalid(
                    "buffered HTTP output forbids Transfer-Encoding",
                ));
            }
            // Reject invalid native field values while every candidate is still unpublished.
            let value = HeaderValue::from_str(value).map_err(|cause| {
                Error::new("HTTP response header value conversion failed")
                    .with_kind(ErrorKind::External)
                    .with_code(500)
                    .with_public_message("Internal Server Error")
                    .with_cause(cause)
            })?;
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
        let value = HeaderValue::from_str(&cookie).map_err(|cause| {
            Error::new("HTTP response cookie header conversion failed")
                .with_kind(ErrorKind::External)
                .with_code(500)
                .with_public_message("Internal Server Error")
                .with_cause(cause)
        })?;
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
            (
                serde_json::to_vec(&json).map_err(|cause| {
                    Error::new("HTTP response JSON serialization failed")
                        .with_kind(ErrorKind::External)
                        .with_code(500)
                        .with_public_message("Internal Server Error")
                        .with_cause(cause)
                })?,
                Some("application/json"),
            )
        }
    };
    // An absent MIME keeps an explicit header; structured envelopes always supply JSON MIME.
    if let Some(mime) = mime {
        // Explicit MIME wins over stale user headers; an envelope always declares JSON.
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(mime).map_err(|cause| {
                Error::new("HTTP response content type conversion failed")
                    .with_kind(ErrorKind::External)
                    .with_code(500)
                    .with_public_message("Internal Server Error")
                    .with_cause(cause)
            })?,
        );
    }
    Ok(body)
}
