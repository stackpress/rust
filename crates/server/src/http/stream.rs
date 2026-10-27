//! Explicit one-shot HTTP output without executors, listeners or hidden collection.
//!
//! Streams are owned, `Send` and `'static`; response hooks finish before first poll.
//! Cloned response views share allocation identity, never another stream or output permit.
//! This module enforces byte/chunk bounds. An IO host must enforce the supplied finite
//! time bounds while driving [`HttpBody`]; an in-process caller owns that timer policy.

use super::{HttpAdapter, HttpOptions, common};
use crate::{HandleOptions, HandleReport, HostReply, OutputGuard, Server};
use futures_core::Stream;
use http::header::{CONNECTION, CONTENT_LENGTH, CONTENT_TYPE};
use stackpress_lib::{Body, DeliveryState, Error, Response, Result, SharedNative, Value};
use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};

/// Finite producer and IO-host budgets; zero is invalid, never unlimited.
///
/// # Examples
/// These time controls travel with the body; its driving IO host must enforce them.
/// ```
/// use stackpress_server::http::stream::StreamLimits;
/// use std::time::Duration;
/// let limits = StreamLimits {
///     max_chunk_bytes: 4096,
///     total_timeout: Duration::from_secs(15),
///     ..Default::default()
/// };
/// assert!(limits.max_chunk_bytes <= limits.max_bytes);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct StreamLimits {
    /// Largest admitted producer chunk, before handing it to the transport.
    pub max_chunk_bytes: usize,
    /// Aggregate admitted bytes across all chunks.
    pub max_bytes: usize,
    /// Absolute body lifetime enforced by the IO host.
    pub total_timeout: Duration,
    /// Maximum interval without body/write progress, enforced by the IO host.
    pub idle_timeout: Duration,
}
impl Default for StreamLimits {
    /// Choose the accepted finite HTTP defaults.
    fn default() -> Self {
        Self {
            max_chunk_bytes: 64 * 1024,
            max_bytes: 64 * 1024 * 1024,
            total_timeout: Duration::from_secs(60),
            idle_timeout: Duration::from_secs(10),
        }
    }
}
impl StreamLimits {
    /// Reject zero controls before installing a producer in a response.
    fn validate(self) -> Result<()> {
        // Durations must also fit the monotonic timer used by native hosts.
        if self.max_chunk_bytes == 0
            || self.max_bytes == 0
            || self.total_timeout.is_zero()
            || self.idle_timeout.is_zero()
            || std::time::Instant::now()
                .checked_add(self.total_timeout)
                .is_none()
            || std::time::Instant::now()
                .checked_add(self.idle_timeout)
                .is_none()
        {
            return Err(Error::invalid(
                "HTTP stream limits must be positive and finite",
            ));
        }
        Ok(())
    }
}
/// Exclusive pinned producer; no reference into a request or invocation can escape.
type OwnedStream = Pin<Box<dyn Stream<Item = Result<Vec<u8>>> + Send + 'static>>;
/// Private typed identity; a mutex transfers ownership and is never held during polling.
struct HttpBodySlot {
    /// Only the selected final response may take this producer, after successful preflight.
    stream: Mutex<Option<OwnedSource>>,
    /// Exact size is available only for internally owned immutable bytes.
    length: Option<usize>,
    /// Immutable limits travel with the exclusive producer when taken.
    limits: StreamLimits,
}

/// Owned alternatives share one take-once publication guard; neither runs under the slot lock.
enum OwnedSource {
    /// An application producer retains its explicit runtime/byte policy.
    Stream(OwnedStream),
    /// An immutable representation shares its storage and has an exact checked length.
    Bytes(Arc<[u8]>),
}

/// Install a typed stream through the existing explicit native-value seam.
pub trait HttpResponseExt {
    /// Select a one-shot producer on an Open response, validating MIME and budgets first.
    ///
    /// Replacing the body detaches its producer. Retained views can extend allocation
    /// lifetime, but cannot poll or extract it. No bytes are produced by this call.
    ///
    /// # Errors
    /// Rejects finalized response, invalid MIME or zero/nonfinite stream controls.
    ///
    /// ```compile_fail
    /// use std::{pin::Pin, rc::Rc, task::{Context, Poll}};
    /// use futures_core::Stream;
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// struct Local(Rc<()>);
    /// impl Stream for Local {
    ///     type Item = stackpress_lib::Result<Vec<u8>>;
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> { Poll::Pending }
    /// }
    /// stackpress_lib::Response::new().stream("text/plain", Local(Rc::new(())), StreamLimits::default()).unwrap();
    /// ```
    ///
    /// Borrowed request state cannot outlive the handler in a producer:
    /// ```compile_fail
    /// use std::{pin::Pin, task::{Context, Poll}};
    /// use futures_core::Stream;
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// struct Borrowed<'a>(&'a str);
    /// impl Stream for Borrowed<'_> {
    ///     type Item = stackpress_lib::Result<Vec<u8>>;
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> { Poll::Pending }
    /// }
    /// fn attach(req: &stackpress_lib::Request, res: &mut stackpress_lib::Response) {
    ///     res.stream("text/plain", Borrowed(req.method.as_str()), StreamLimits::default()).unwrap();
    /// }
    /// ```
    /// A borrowed response view also cannot escape:
    /// ```compile_fail
    /// use std::{pin::Pin, task::{Context, Poll}};
    /// use futures_core::Stream;
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// struct Borrowed<'a>(&'a stackpress_lib::ResponseView);
    /// impl Stream for Borrowed<'_> {
    ///     type Item = stackpress_lib::Result<Vec<u8>>;
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> { Poll::Pending }
    /// }
    /// fn attach(view: &stackpress_lib::ResponseView, res: &mut stackpress_lib::Response) {
    ///     res.stream("text/plain", Borrowed(view), StreamLimits::default()).unwrap();
    /// }
    /// ```
    /// An invocation remains scoped to dispatch rather than becoming producer-owned:
    /// ```compile_fail
    /// use std::{pin::Pin, task::{Context, Poll}};
    /// use futures_core::Stream;
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// struct Borrowed<'a>(stackpress_lib::Invocation<'a>);
    /// impl Stream for Borrowed<'_> {
    ///     type Item = stackpress_lib::Result<Vec<u8>>;
    ///     fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> { Poll::Pending }
    /// }
    /// fn attach(inv: stackpress_lib::Invocation<'_>, res: &mut stackpress_lib::Response) {
    ///     res.stream("text/plain", Borrowed(inv), StreamLimits::default()).unwrap();
    /// }
    /// ```
    ///
    /// # Examples
    /// Own the producer state; this call attaches it without polling or spawning work.
    /// ```
    /// use futures_core::Stream;
    /// use stackpress_lib::{Response, Result};
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// use std::{pin::Pin, task::{Context, Poll}};
    /// struct Once(Option<Vec<u8>>);
    /// impl Stream for Once {
    ///     type Item = Result<Vec<u8>>;
    ///     fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    ///         Poll::Ready(self.0.take().map(Ok))
    ///     }
    /// }
    /// let mut response = Response::new();
    /// response.stream("text/plain", Once(Some(b"hello".to_vec())), StreamLimits::default())?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn stream<S>(&mut self, mime: &str, stream: S, limits: StreamLimits) -> Result<&mut Self>
    where
        S: Stream<Item = Result<Vec<u8>>> + Send + 'static;

    /// Share immutable representation bytes without a whole-response copy.
    ///
    /// Publication remains one-shot. Native output copies at most 64KiB per demand poll;
    /// HEAD reports the known length without polling or copying representation bytes.
    /// Explicit limits bound bytes/chunks and IO lifetime, as for application streams.
    /// Callers own the preexisting immutable allocation.
    /// # Errors
    /// Rejects a finalized response, invalid MIME, zero/nonfinite controls, oversized bytes
    /// or foundation descriptor budgets.
    ///
    /// ```
    /// use std::sync::Arc;
    /// use stackpress_lib::Response;
    /// use stackpress_server::http::stream::{HttpResponseExt, StreamLimits};
    /// let mut response = Response::new();
    /// let bytes: Arc<[u8]> = Arc::from(b"verified bytes".as_slice());
    /// response.immutable_bytes("text/plain", bytes, StreamLimits::default())?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    fn immutable_bytes(
        &mut self,
        mime: &str,
        bytes: Arc<[u8]>,
        limits: StreamLimits,
    ) -> Result<&mut Self>;
}
impl HttpResponseExt for Response {
    /// Transfer the producer into a private take-once slot after checking its policy.
    fn stream<S>(&mut self, mime: &str, stream: S, limits: StreamLimits) -> Result<&mut Self>
    where
        S: Stream<Item = Result<Vec<u8>>> + Send + 'static,
    {
        limits.validate()?;
        // Response::set atomically checks Open state, MIME and foundation node budgets.
        self.set(
            mime,
            Body::Value(Value::Shared(SharedNative::new(HttpBodySlot {
                stream: Mutex::new(Some(OwnedSource::Stream(Box::pin(stream)))),
                length: None,
                limits,
            }))),
        )
    }
    /// Only owned immutable storage can supply a trusted native representation length.
    fn immutable_bytes(
        &mut self,
        mime: &str,
        bytes: Arc<[u8]>,
        limits: StreamLimits,
    ) -> Result<&mut Self> {
        limits.validate()?;
        let length = bytes.len();
        if length > limits.max_bytes {
            return Err(Error::limit("immutable HTTP byte limit exceeded"));
        }
        self.set(
            mime,
            Body::Value(Value::Shared(SharedNative::new(HttpBodySlot {
                stream: Mutex::new(Some(OwnedSource::Bytes(bytes))),
                length: Some(length),
                limits,
            }))),
        )
    }
}

/// Exclusive native body; implements demand-driven chunks and drops its producer on failure.
/// It deliberately implements neither Clone nor Debug of producer contents.
/// See [`StreamingHttpAdapter::handle_body`] for polling, limits and known-completion usage.
pub struct HttpBody {
    /// Exhaustive ownership state prevents producer/limit/buffer combinations that cannot occur.
    source: BodySource,
}
/// Each state carries only the fields required to drive that particular representation.
enum BodySource {
    /// One already converted buffer, yielded at most once.
    Buffered(Option<Vec<u8>>),
    /// Shared immutable storage is copied only into one bounded demand chunk at a time.
    Immutable {
        bytes: Arc<[u8]>,
        offset: usize,
        limits: StreamLimits,
    },
    /// Exclusive producer with its inseparable validated policy and admitted-byte count.
    Streaming {
        stream: OwnedStream,
        limits: StreamLimits,
        bytes: usize,
    },
    /// EOF or error has been observed; all subsequent polls return None.
    Finished,
}
impl HttpBody {
    /// Wrap already converted buffered output without collecting a producer.
    fn buffered(bytes: Vec<u8>) -> Self {
        Self {
            source: BodySource::Buffered(Some(bytes)),
        }
    }
    /// Borrow finite streaming deadlines before the producer completes.
    pub fn limits(&self) -> Option<StreamLimits> {
        match &self.source {
            BodySource::Streaming { limits, .. } | BodySource::Immutable { limits, .. } => {
                Some(*limits)
            }
            _ => None,
        }
    }
    /// Inspect known local completion without polling; empty buffered output is already complete.
    /// A streaming producer remains incomplete until its EOF/error is actually observed.
    pub fn is_finished(&self) -> bool {
        match &self.source {
            BodySource::Buffered(bytes) => bytes.as_ref().is_none_or(Vec::is_empty),
            BodySource::Streaming { .. } => false,
            BodySource::Immutable { bytes, offset, .. } => *offset == bytes.len(),
            BodySource::Finished => true,
        }
    }
}
impl Stream for HttpBody {
    type Item = Result<Vec<u8>>;
    /// Poll the exclusive producer outside all locks, enforcing bounds before yielding bytes.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Buffered output never enters the producer path or receives producer byte policy.
        if let BodySource::Buffered(bytes) = &mut self.source {
            return Poll::Ready(bytes.take().map(Ok));
        }
        if let BodySource::Immutable {
            bytes,
            offset,
            limits,
        } = &mut self.source
        {
            // An exact known length lets Hyper finish without requiring a separate EOF poll.
            if *offset == bytes.len() {
                return Poll::Ready(None);
            }
            let end = offset
                .saturating_add(limits.max_chunk_bytes.min(64 * 1024))
                .min(bytes.len());
            let chunk = bytes[*offset..end].to_vec();
            *offset = end;
            return Poll::Ready(Some(Ok(chunk)));
        }
        let BodySource::Streaming {
            stream,
            limits,
            bytes: admitted,
        } = &mut self.source
        else {
            return Poll::Ready(None);
        };
        let item = match stream.as_mut().poll_next(cx) {
            Poll::Pending => return Poll::Pending,
            Poll::Ready(item) => item,
        };
        match item {
            Some(Ok(bytes)) => {
                // Checked addition rejects overflow and exact equality remains an allowed boundary.
                let total = admitted
                    .checked_add(bytes.len())
                    .filter(|total| *total <= limits.max_bytes);
                if bytes.len() > limits.max_chunk_bytes || total.is_none() {
                    self.source = BodySource::Finished;
                    return Poll::Ready(Some(Err(Error::limit("HTTP stream byte limit exceeded"))));
                }
                // The matched Some total was validated before exposing the new chunk to the caller.
                if let Some(total) = total {
                    *admitted = total;
                }
                Poll::Ready(Some(Ok(bytes)))
            }
            // Retain the original typed error once, then drop the producer and fuse output.
            Some(Err(error)) => {
                self.source = BodySource::Finished;
                Poll::Ready(Some(Err(error)))
            }
            None => {
                self.source = BodySource::Finished;
                Poll::Ready(None)
            }
        }
    }
}

/// Native availability from the sole publication authority.
/// See [`StreamingHttpAdapter::handle_body`] for explicit matching before polling output.
pub enum StreamingHttpOutput {
    /// Fully preflighted headers and an unpolled one-shot body.
    Response(http::Response<HttpBody>),
    /// Deliberate framework stop; no HTTP response is invented.
    Suppressed,
    /// Foreign/failed finalization cannot grant a second output attempt.
    Unavailable {
        /// Observed foundation state.
        state: DeliveryState,
    },
}
/// Publication diagnostics, before any IO host drives the body to completion.
/// See [`StreamingHttpAdapter::handle_body`] for obtaining and inspecting the native reply.
pub struct StreamingHttpReply {
    /// Native output availability.
    pub output: StreamingHttpOutput,
    /// Lifecycle and adapter publication report; never remote delivery acknowledgement.
    pub report: HandleReport,
}
/// Additive converter sharing the buffered adapter's normalization and representation rules.
pub struct StreamingHttpAdapter {
    /// Reuse the validated origin/policies without exposing common lifecycle internals.
    adapter: HttpAdapter,
}
impl StreamingHttpAdapter {
    /// Validate trusted configuration without opening resources.
    /// # Errors
    /// Rejects the same invalid origins as [`HttpAdapter::new`].
    ///
    /// # Examples
    /// ```
    /// use stackpress_server::http::{HttpOptions, stream::StreamingHttpAdapter};
    /// let adapter = StreamingHttpAdapter::new(HttpOptions {
    ///     base_url: "https://profiles.example/".into(),
    ///     ..Default::default()
    /// })?;
    /// # let _ = adapter; Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(options: HttpOptions) -> Result<Self> {
        Ok(Self {
            adapter: HttpAdapter::new(options)?,
        })
    }
    /// Normalize the head before invoking a one-attempt, Send body loader inside Input.
    ///
    /// The loaded body is checked against aggregate head+body budgets before parsing.
    /// Loader errors enter G12; malformed heads use G17 without callbacks or loading.
    /// Dropping this future drops active loading/handling; no background work is spawned.
    /// # Errors
    /// Outer errors mean the host could not allocate a report. Ordinary failures stay in it.
    ///
    /// # Examples
    /// Drive a known representation in process; a network host additionally enforces timers.
    /// ```
    /// use futures_core::Stream;
    /// use stackpress_lib::{Flow, Invocation, Request, Response, Result};
    /// use stackpress_server::{HandleOptions, ServerBuilder, http::{BufferedRequest, stream::{
    ///     HttpResponseExt, StreamLimits, StreamingHttpAdapter, StreamingHttpOutput,
    /// }}};
    /// use std::{pin::Pin, sync::Arc};
    /// async fn asset(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    ///     res.immutable_bytes("text/plain", Arc::from(b"hello".as_slice()), StreamLimits::default())?;
    ///     Ok(Flow::Continue)
    /// }
    /// # async fn example() -> Result<()> {
    /// let mut builder = ServerBuilder::new();
    /// builder.get("/", asset)?;
    /// let app = builder.build().await.map_err(|failure| failure.error)?;
    /// let (head, _) = BufferedRequest::new(Vec::new()).into_parts();
    /// let reply = StreamingHttpAdapter::new(Default::default())?
    ///     .handle_body(&app, head, HandleOptions::default(), || async { Ok(Vec::new()) }).await?;
    /// let StreamingHttpOutput::Response(mut native) = reply.output else { panic!("expected output") };
    /// assert!(native.body().limits().is_some());
    /// let chunk = std::future::poll_fn(|cx| Pin::new(native.body_mut()).poll_next(cx)).await;
    /// assert_eq!(chunk.transpose()?, Some(b"hello".to_vec()));
    /// assert!(native.body().is_finished());
    /// # Ok(()) }
    /// ```
    pub async fn handle_body<F, Fut>(
        &self,
        app: &Server,
        parts: http::request::Parts,
        options: HandleOptions,
        load: F,
    ) -> Result<StreamingHttpReply>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<Vec<u8>>> + Send,
    {
        // HEAD changes only native body publication, never operation selection.
        let head = parts.method == http::Method::HEAD;
        let policy = &self.adapter.options;
        // Pre-normalization failures cannot run loaders or fabricate callback requests (G17).
        let reply = match common::normalize(&parts, 0, policy, &self.adapter.origin) {
            Ok((mut request, event)) => {
                // Each lifecycle begins with its own response budgets and output identity.
                let mut response = Response::with_limits(policy.response_limits);
                // No second lifecycle is needed for a failed loader or aggregate body check.
                let report = app
                    .handle_input(
                        &event,
                        &mut request,
                        &mut response,
                        options,
                        move |view| async move {
                            // Await exactly one loader, then charge raw head+body before parsing.
                            let bytes = load().await?;
                            common::check_input_limits(&parts, bytes.len(), policy)?;
                            common::load_body(&view.mimetype, bytes, policy.limits)
                        },
                    )
                    .await?;
                HostReply { response, report }
            }
            Err(error) => app.input_failure("<invalid HTTP input>", error)?,
        };
        Ok(finish(reply, head).await)
    }
}
/// Extract only the private extension identity; arbitrary SharedNative values stay unsupported.
fn descriptor(response: &Response) -> Option<&HttpBodySlot> {
    match response.body() {
        Body::Value(Value::Shared(value)) => value.downcast_ref(),
        _ => None,
    }
}
/// Validate all headers and response fields before taking exclusive producer ownership.
fn prepare(response: &Response, head: bool) -> Result<http::Response<HttpBody>> {
    let Some(slot) = descriptor(response) else {
        // Buffered output keeps its exact converter and bytes; only network hop headers differ.
        let native = common::prepare(response, head)?;
        let (mut parts, bytes) = native.into_parts();
        strip_hop_headers(&mut parts.headers)?;
        return Ok(http::Response::from_parts(parts, HttpBody::buffered(bytes)));
    };
    // A replacement response owns its own budgets; validate it rather than an earlier snapshot.
    let limits = response.limits();
    response.body().validate(&limits)?;
    response.data().value().validate(&limits)?;
    response.errors().value().validate(&limits)?;
    // HTTP output represents a final status, so informational/application-only codes are invalid.
    let code = response.status().map_or(204, |status| status.code);
    if !(200..=599).contains(&code) {
        return Err(Error::invalid(
            "completed HTTP status must be between 200 and 599",
        ));
    }
    let status = http::StatusCode::from_u16(code).map_err(|cause| {
        Error::new("HTTP stream status conversion failed")
            .with_kind(stackpress_lib::ErrorKind::External)
            .with_cause(cause)
    })?;
    // Convert repeated headers/cookies before altering adapter-owned framing.
    let mut headers = common::response_headers(response, &limits)?;
    strip_hop_headers(&mut headers)?;
    // Stream lengths are unknown; stale caller lengths must never frame producer bytes.
    headers.remove(CONTENT_LENGTH);
    // The final typed MIME wins over an earlier explicit Content-Type field.
    if let Some(mime) = response.mimetype() {
        headers.insert(
            CONTENT_TYPE,
            http::HeaderValue::from_str(mime).map_err(|cause| {
                Error::new("HTTP stream MIME conversion failed")
                    .with_kind(stackpress_lib::ErrorKind::External)
                    .with_cause(cause)
            })?,
        );
    }
    // Immutable bytes have a trustworthy length, including HEAD; forbidden statuses omit it.
    if let Some(length) = slot.length
        && !matches!(code, 204 | 205 | 304)
    {
        headers.insert(CONTENT_LENGTH, http::HeaderValue::from(length as u64));
    }
    // Reset Content always has zero length, independently of the selected representation.
    if code == 205 {
        headers.insert(CONTENT_LENGTH, http::HeaderValue::from_static("0"));
    }
    // Take happens only after every fallible header/status/budget check, never under user code.
    let stream = slot
        .stream
        .lock()
        .map_err(|_| Error::new("HTTP stream ownership lock poisoned"))?
        .take()
        .ok_or_else(|| Error::new("HTTP stream ownership already consumed"))?;
    let body = if head || matches!(code, 204 | 205 | 304) {
        // Bodyless output actively discards the producer even if another view retains the slot.
        drop(stream);
        HttpBody::buffered(Vec::new())
    } else {
        HttpBody {
            source: match stream {
                OwnedSource::Stream(stream) => BodySource::Streaming {
                    stream,
                    limits: slot.limits,
                    bytes: 0,
                },
                OwnedSource::Bytes(bytes) => BodySource::Immutable {
                    bytes,
                    offset: 0,
                    limits: slot.limits,
                },
            },
        }
    };
    let mut native = http::Response::new(body);
    *native.status_mut() = status;
    *native.headers_mut() = headers;
    Ok(native)
}
/// Remove fixed and Connection-nominated hop fields; transport owns connection framing.
fn strip_hop_headers(headers: &mut http::HeaderMap) -> Result<()> {
    // Parse all nominations before removing Connection itself or mutating its borrowed values.
    let mut nominated = Vec::new();
    for value in headers.get_all(CONNECTION) {
        let text = value.to_str().map_err(|cause| {
            Error::new("HTTP connection header decoding failed")
                .with_kind(stackpress_lib::ErrorKind::External)
                .with_cause(cause)
        })?;
        for name in text.split(',') {
            nominated.push(
                http::HeaderName::from_bytes(name.trim().as_bytes()).map_err(|cause| {
                    Error::new("HTTP connection field conversion failed")
                        .with_kind(stackpress_lib::ErrorKind::External)
                        .with_cause(cause)
                })?,
            );
        }
    }
    // Remove both nominated extension fields and the standard per-connection fields.
    for name in nominated {
        headers.remove(name);
    }
    for name in [
        "connection",
        "keep-alive",
        "proxy-authenticate",
        "proxy-authorization",
        "te",
        "trailer",
        "upgrade",
    ] {
        headers.remove(name);
    }
    Ok(())
}
/// Preflight, consume one output guard, then return an unpolled native body to the caller.
async fn finish(mut reply: HostReply, head: bool) -> StreamingHttpReply {
    // Authority is call-local and cannot be reset by any handler response replacement.
    let mut guard = OutputGuard::default();
    match reply.response.delivery_state() {
        DeliveryState::Stopped => {
            // Suppression consumes the sole opportunity without fabricating native bytes.
            let _ = guard
                .dispatch(&mut reply.response, |_| async { Ok(()) })
                .await;
            reply.report.record_output(&reply.response);
            return StreamingHttpReply {
                output: StreamingHttpOutput::Suppressed,
                report: reply.report,
            };
        }
        DeliveryState::Open => {}
        state => {
            // Foreign foundation finalization cannot authorize another adapter write.
            reply.report.record_output_failure(
                Error::new("HTTP response finalized outside adapter authority"),
                &reply.response,
            );
            return StreamingHttpReply {
                output: StreamingHttpOutput::Unavailable { state },
                report: reply.report,
            };
        }
    }
    // Every fallible conversion precedes publication; failure can still choose one empty 500.
    let prepared = match prepare(&reply.response, head) {
        Ok(native) => native,
        Err(error) => {
            reply.report.record_output_failure(error, &reply.response);
            // Discard handler state and producer association; no private body/header/cookie escapes.
            reply.response = Response::new();
            let _ = reply.response.status_code(500, None);
            let mut native = http::Response::new(HttpBody::buffered(Vec::new()));
            *native.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;
            native
                .headers_mut()
                .insert(CONTENT_LENGTH, http::HeaderValue::from_static("0"));
            native
        }
    };
    let mut output = None;
    // Publication only moves the prepared value; polling starts in the caller after this await.
    let result = guard
        .dispatch(&mut reply.response, |_| async {
            output = Some(prepared);
            Ok(())
        })
        .await;
    if let Err(error) = result {
        reply.report.record_output_failure(error, &reply.response);
    } else {
        reply.report.record_output(&reply.response);
    }
    StreamingHttpReply {
        output: output.map_or(
            StreamingHttpOutput::Unavailable {
                state: guard.state(),
            },
            StreamingHttpOutput::Response,
        ),
        report: reply.report,
    }
}
