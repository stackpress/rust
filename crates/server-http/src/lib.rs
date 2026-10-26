//! HTTP/1.1 ownership over caller-supplied IO, with no listener or runtime ownership.
//!
//! This M0 surface establishes real streaming, cancellation and final report ownership.
//! It serves one exchange per connection. Admission, full deadline/limit policy,
//! bounded keep-alive, proxy policy, assets and binding remain later milestones.
//! [`HttpDelivery::Completed`] means producer EOF handed to Hyper, never remote ACK.
//! No callback receives an IO handle. Dropping the serving future drops all active work.

mod body;
mod report;
#[cfg(feature = "tls")]
mod tls;

pub use report::{ConnectionReport, HttpDelivery, HttpExchangeReport};
#[cfg(feature = "tls")]
pub use tls::TlsConfig;

use body::TrackedBody;
use http_body_util::BodyExt;
use hyper::{body::Incoming, server::conn::http1, service::service_fn};
use hyper_util::rt::{TokioIo, TokioTimer};
use report::{Exchange, ExchangeSlot, lock};
use stackpress_lib::{Error, ErrorKind, Result};
use stackpress_server::{
    HandleOptions, Server,
    http::{
        HttpOptions,
        stream::{StreamingHttpAdapter, StreamingHttpOutput},
    },
};
use std::{
    future::Future,
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncWrite};

/// Idempotent, read-only cancellation view for cooperative handler-owned work.
#[derive(Clone, Default, Debug)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    /// Check cancellation without granting native IO or task-spawning authority.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    /// Connection/body ownership marks cancellation exactly once or repeatedly without harm.
    fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}
/// Trusted metadata inherited through HandleOptions, never derived from forwarded headers.
#[derive(Clone, Debug)]
pub struct HttpRequestContext {
    /// Caller-attested peer; supplying IO does not authenticate this address.
    pub peer: SocketAddr,
    /// Explicit connection scheme, HTTPS only after a successful selected TLS handshake.
    pub scheme: &'static str,
    /// Validated configured origin; forwarded headers never override this value.
    pub public_origin: String,
    /// Cooperative signal set on connection/body drop and deadline cancellation.
    pub cancellation: Cancellation,
}
/// Immutable host configuration over an already built application; no sockets are opened.
pub struct HttpServer {
    /// Application lifetime is explicit and shared, never rebuilt per request.
    app: Arc<Server>,
    /// Common normalization/lifecycle/output implementation.
    adapter: StreamingHttpAdapter,
    /// Finite request buffering ceiling checked incrementally before each owned copy.
    max_body_bytes: usize,
    /// Configured origin copied only into trusted context.
    origin: String,
    /// Caller stop drains one owned connection before forced cancellation.
    grace: Duration,
}
impl HttpServer {
    /// Validate the converter and retain the application's explicit shared ownership.
    ///
    /// M0 handles one exchange per supplied IO, with fixed finite head/body controls.
    /// Complete transport policy and listener binding are not established by this seam.
    /// # Errors
    /// Rejects invalid trusted HTTP origins.
    ///
    /// ```
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// let app = stackpress_server::ServerBuilder::new().build().await.map_err(|error| error.error)?;
    /// let host = stackpress_server_http::HttpServer::new(std::sync::Arc::new(app), Default::default())?;
    /// # let _ = host;
    /// # Ok(()) }
    /// ```
    pub fn new(app: Arc<Server>, options: HttpOptions) -> Result<Self> {
        let max_body_bytes = options.limits.max_bytes;
        let origin = options.base_url.clone();
        Ok(Self {
            app,
            adapter: StreamingHttpAdapter::new(options)?,
            max_body_bytes,
            origin,
            grace: Duration::from_secs(30),
        })
    }
    /// Select a positive finite graceful-drain deadline without changing application state.
    /// # Errors
    /// Rejects zero or monotonic-clock overflow.
    pub fn with_shutdown_grace(mut self, grace: Duration) -> Result<Self> {
        if grace.is_zero() || std::time::Instant::now().checked_add(grace).is_none() {
            return Err(Error::invalid(
                "HTTP shutdown grace must be positive and finite",
            ));
        }
        self.grace = grace;
        Ok(self)
    }
    /// Serve caller-supplied plaintext IO and return final reports after dropping the connection.
    ///
    /// `stop` initiates graceful shutdown, drains within the configured grace and then
    /// drops overdue work. There are no spawned tasks to detach or reap. Caller drop
    /// cancels active work. No listener is bound and no process signals are installed.
    pub async fn serve_io<I, F>(&self, io: I, peer: SocketAddr, stop: F) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        self.serve_connection(io, peer, "http", stop).await
    }
    /// Drive one HTTP/1 connection; the scope ends before consuming final diagnostics.
    async fn serve_connection<I, F>(
        &self,
        io: I,
        peer: SocketAddr,
        scheme: &'static str,
        stop: F,
    ) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let slots = Arc::new(Mutex::new(Vec::<ExchangeSlot>::new()));
        // Guard survives until after Hyper is dropped, including panic and serving-future drop.
        let guard = ConnectionCancellation(Arc::clone(&slots));
        let mut shutdown = false;
        let mut forced = false;
        let result;
        {
            let registry = Arc::clone(&slots);
            let service = service_fn(move |request| {
                let slot = {
                    let mut registry = lock(&registry);
                    let slot = Arc::new(Mutex::new(Exchange::new(registry.len() as u64 + 1)));
                    registry.push(Arc::clone(&slot));
                    slot
                };
                self.handle_request(request, peer, scheme, slot)
            });
            let mut builder = http1::Builder::new();
            // Fixed M0 controls bound parser buffering; full configurable policy belongs to M1.
            builder
                .keep_alive(false)
                .half_close(false)
                .max_headers(64)
                .max_buf_size(32 * 1024)
                .timer(TokioTimer::new())
                .header_read_timeout(Duration::from_secs(10));
            let connection = builder.serve_connection(TokioIo::new(io), service);
            tokio::pin!(connection);
            tokio::pin!(stop);
            // Stop wins if already signalled; do not start a request before graceful shutdown.
            result = tokio::select! {
                biased;
                () = &mut stop => {
                    shutdown = true;
                    connection.as_mut().graceful_shutdown();
                    match tokio::time::timeout(self.grace, &mut connection).await {
                        Ok(result) => result.map_err(|cause| external("HTTP connection drain failed", cause)),
                        Err(cause) => {
                            forced = true;
                            Err(external("HTTP graceful shutdown deadline exceeded", cause))
                        }
                    }
                }
                result = &mut connection => result.map_err(|cause| external("HTTP connection failed", cause)),
            };
        }
        // The connection, service future, body and IO are gone before reports become final.
        drop(guard);
        report::finish(slots, result, shutdown, forced)
    }

    /// One normalized Input load and one lifecycle; no external event override or writer escapes.
    async fn handle_request(
        &self,
        request: http::Request<Incoming>,
        peer: SocketAddr,
        scheme: &'static str,
        slot: ExchangeSlot,
    ) -> Result<http::Response<TrackedBody>> {
        let cancellation = lock(&slot).cancellation.clone();
        let options = HandleOptions {
            adapter_context: Some(Arc::new(HttpRequestContext {
                peer,
                scheme,
                public_origin: self.origin.clone(),
                cancellation,
            })),
        };
        let (parts, mut body) = request.into_parts();
        let reply = self
            .adapter
            .handle_body(&self.app, parts, options, || async move {
                let mut bytes = Vec::new();
                // Copy each admitted data frame only after checking actual decoded length.
                while let Some(frame) = body.frame().await {
                    let frame =
                        frame.map_err(|cause| external("HTTP request body read failed", cause))?;
                    let data = frame
                        .into_data()
                        .map_err(|_| Error::invalid("HTTP request trailers are unsupported"))?;
                    if data.len() > self.max_body_bytes.saturating_sub(bytes.len()) {
                        return Err(Error::limit("HTTP request body limit exceeded"));
                    }
                    bytes.extend_from_slice(&data);
                }
                Ok(bytes)
            })
            .await?;
        lock(&slot).report.host = Some(reply.report);
        match reply.output {
            StreamingHttpOutput::Response(native) => {
                lock(&slot).report.delivery = HttpDelivery::Published;
                Ok(native.map(|source| TrackedBody::new(source, slot)))
            }
            StreamingHttpOutput::Suppressed => {
                lock(&slot).report.delivery = HttpDelivery::Suppressed;
                // Error closes IO without fabricating response bytes for framework suppression.
                Err(Error::new("HTTP output suppressed"))
            }
            StreamingHttpOutput::Unavailable { .. } => {
                let error = Arc::new(Error::new("HTTP output unavailable"));
                lock(&slot).fail(error);
                Err(Error::new("HTTP output unavailable"))
            }
        }
    }
}
/// Scope guard cancels contexts even when the serving future itself is dropped or panics.
struct ConnectionCancellation(Arc<Mutex<Vec<ExchangeSlot>>>);
impl Drop for ConnectionCancellation {
    /// No await or user callback runs under a diagnostic lock during cancellation.
    fn drop(&mut self) {
        for slot in lock(&self.0).iter() {
            lock(slot).cancellation.cancel();
        }
    }
}
/// Keep native errors in a real cause chain, with separate sanitized public context.
fn external<E: std::error::Error + Send + Sync + 'static>(context: &str, cause: E) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}
