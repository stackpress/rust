//! Optional buffered HTTP conversion over the common host lifecycle.
//!
//! Callers own input buffering; these limits do not bound network reads. No socket,
//! executor, task or stream enters the framework request. Only the checked method
//! and exact URI pathname select the operation; applications own authorization.
//!
//! Session cookie revisions accept scalar strings. List values require an explicit
//! codec and currently produce an Output failure naming the cookie, with sanitized
//! empty status-500 output. An explicit codec option on [`HttpOptions`] is future work.

mod common;
#[cfg(feature = "http-stream")]
pub mod stream;

use common::{load_body, prepare};
use http::HeaderValue;
use http::header::CONTENT_LENGTH;
use stackpress_lib::{DeliveryState, Error, ParseLimits, Request, Response, Result};

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
///         .map_err(|cause| stackpress_lib::Error::new("buffered HTTP request construction failed")
///             .with_kind(stackpress_lib::ErrorKind::External)
///             .with_code(500)
///             .with_public_message("Internal Server Error")
///             .with_cause(cause))?;
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
    ///     .map_err(|cause| stackpress_lib::Error::new("buffered HTTP request construction failed")
    ///         .with_kind(stackpress_lib::ErrorKind::External)
    ///         .with_code(500)
    ///         .with_public_message("Internal Server Error")
    ///         .with_cause(cause))?;
    /// let reply = HttpAdapter::new(HttpOptions::default())?.handle(&app, input).await?;
    /// assert!(matches!(reply.output, HttpOutput::Response(response) if response.status() == 404));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn handle(&self, app: &Server, input: BufferedRequest) -> Result<HttpReply> {
        self.handle_with_options(app, input, HandleOptions::default())
            .await
    }

    /// Run the buffered adapter with trusted metadata inherited by nested dispatch.
    ///
    /// # Errors
    /// Has the same reporting and cancellation contract as [`Self::handle`].
    ///
    /// # Examples
    /// Supply only caller-trusted metadata; input headers do not populate this context.
    /// ```
    /// use stackpress_server::{HandleOptions, ServerBuilder, http::{BufferedRequest, HttpAdapter}};
    /// use std::sync::Arc;
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// let app = ServerBuilder::new().build().await.map_err(|failure| failure.error)?;
    /// let options = HandleOptions { adapter_context: Some(Arc::new(String::from("trusted caller"))) };
    /// let reply = HttpAdapter::new(Default::default())?
    ///     .handle_with_options(&app, BufferedRequest::new(Vec::new()), options).await?;
    /// assert!(!reply.report.output_eligible);
    /// # Ok(()) }
    /// ```
    pub async fn handle_with_options(
        &self,
        app: &Server,
        input: BufferedRequest,
        options: HandleOptions,
    ) -> Result<HttpReply> {
        // Preserve HEAD suppression even when normalization cannot produce a Request.
        let head = input.method().as_str().eq_ignore_ascii_case("HEAD");
        let (parts, bytes) = input.into_parts();
        let reply = match common::normalize(&parts, bytes.len(), &self.options, &self.origin) {
            Ok((mut request, event)) => {
                // Start a caller-local response with trusted budgets, separate from input policy.
                let mut response = Response::with_limits(self.options.response_limits);
                // Move the original buffer into one loader; no native reader can be retried.
                let limits = self.options.limits;
                // The loader stages body/post atomically inside the common Input runner.
                let report = app
                    .handle_input(
                        &event,
                        &mut request,
                        &mut response,
                        options,
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
