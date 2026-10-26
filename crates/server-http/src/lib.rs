//! Bounded HTTP/1 hosting over caller-supplied IO and generic acceptors.
//!
//! Hyper owns parsing and framing. The host owns admission, deadlines, sequential reuse,
//! cancellation, bounded drain and final reports. No listener/runtime/process signals are owned.
//! `tls` selects explicit Rustls; proxy/assets/TCP binding remain separate M2 work.

mod body;
mod connection;
mod input;
mod io;
mod monitor;
mod options;
mod report;
mod supervisor;
mod timer;
#[cfg(feature = "tls")]
mod tls;

pub use options::NetworkLimits;
pub use report::{ConnectionReport, HttpDelivery, HttpExchangeReport};
pub use supervisor::{AcceptedIo, Acceptor, DrainReport, Shutdown};
#[cfg(feature = "tls")]
pub use tls::TlsConfig;

use stackpress_lib::{Error, ErrorKind, Result};
use stackpress_server::{
    Server,
    http::{HttpOptions, stream::StreamingHttpAdapter},
};
use std::{
    future::Future,
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncRead, AsyncWrite},
    sync::{OwnedSemaphorePermit, Semaphore},
};

/// Idempotent, read-only cancellation view for cooperative handler-owned work.
#[derive(Clone, Default, Debug)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    /// Check cancellation without granting native IO or task-spawning authority.
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    /// Connection/body ownership marks cancellation exactly once or repeatedly without harm.
    pub(crate) fn cancel(&self) {
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
/// Immutable policy and explicit application ownership; construction opens no resources.
pub struct HttpServer {
    /// One built host is shared; per-request startup/registration never occurs.
    app: Arc<Server>,
    /// Existing normalization, lifecycle and one-shot publication authority.
    adapter: StreamingHttpAdapter,
    /// Trusted configured origin; request/forwarded headers cannot replace it.
    origin: String,
    /// Native limits remain distinct from compatibility-preserving buffered HttpOptions.
    limits: NetworkLimits,
    /// Admission permits include active handshakes and are acquired before host spawn.
    connections: Arc<Semaphore>,
    /// A second finite budget protects TLS work, also before spawning.
    handshakes: Arc<Semaphore>,
}
impl HttpServer {
    /// Validate buffered conversion and choose finite native defaults without opening IO.
    /// # Errors
    /// Rejects invalid trusted HTTP origins.
    /// ```
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// let app=stackpress_server::ServerBuilder::new().build().await.map_err(|e|e.error)?;
    /// let host=stackpress_server_http::HttpServer::new(std::sync::Arc::new(app),Default::default())?;
    /// # let _=host; Ok(()) }
    /// ```
    pub fn new(app: Arc<Server>, options: HttpOptions) -> Result<Self> {
        let origin = options.base_url.clone();
        let limits = NetworkLimits::default();
        Ok(Self {
            app,
            adapter: StreamingHttpAdapter::new(options)?,
            origin,
            connections: Arc::new(Semaphore::new(limits.max_connections)),
            handshakes: Arc::new(Semaphore::new(limits.max_handshakes)),
            limits,
        })
    }
    /// Replace startup policy before sharing/admission; never changes running requests.
    /// # Errors
    /// Rejects zero/unrepresentable byte/count/time controls and a parser buffer below 8192.
    pub fn with_limits(mut self, limits: NetworkLimits) -> Result<Self> {
        limits.validate()?;
        self.connections = Arc::new(Semaphore::new(limits.max_connections));
        self.handshakes = Arc::new(Semaphore::new(limits.max_handshakes));
        self.limits = limits;
        Ok(self)
    }
    /// Select a finite drain budget, preserving the M0 builder surface.
    /// # Errors
    /// Rejects zero or monotonic-clock overflow.
    pub fn with_shutdown_grace(mut self, grace: Duration) -> Result<Self> {
        let mut limits = self.limits.clone();
        limits.shutdown_grace = grace;
        limits.validate()?;
        self.limits = limits;
        Ok(self)
    }
    /// Read immutable native policy; mutation requires constructing a new host.
    pub fn limits(&self) -> &NetworkLimits {
        &self.limits
    }
    /// Observe active owned IO and handshakes without acquiring or queuing a permit.
    pub fn active(&self) -> (usize, usize) {
        (
            self.limits.max_connections - self.connections.available_permits(),
            self.limits.max_handshakes - self.handshakes.available_permits(),
        )
    }
    /// Admit supplied plaintext IO immediately, then serve finite sequential HTTP/1 exchanges.
    /// Excess IO is closed promptly; its report records 503 without inventing wire bytes.
    /// Stop initiates a bounded graceful drain. Await completion before closing services.
    /// Dropping this future drops active work; no host task is spawned by this method.
    pub async fn serve_io<I, F>(&self, io: I, peer: SocketAddr, stop: F) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let Some(admission) = self.admit(false) else {
            drop(io);
            return report::rejected(Error::new("HTTP connection admission exhausted"), 503);
        };
        let _admission = admission;
        self.serve_connection(io, peer, "http", stop, tokio::time::Instant::now())
            .await
    }
    /// Acquire both budgets synchronously; failure never allocates a waiter or spawns work.
    fn admit(&self, tls: bool) -> Option<Admission> {
        let connection = Arc::clone(&self.connections).try_acquire_owned().ok()?;
        let handshake = if tls {
            Some(Arc::clone(&self.handshakes).try_acquire_owned().ok()?)
        } else {
            None
        };
        Some(Admission {
            _connection: connection,
            _handshake: handshake,
        })
    }
}
/// RAII owns admission through handshake/HTTP/cancellation; no native resource is recycled early.
struct Admission {
    _connection: OwnedSemaphorePermit,
    _handshake: Option<OwnedSemaphorePermit>,
}
/// A contextual wrapper retains the original native cause once, never interpolating its Display.
fn external<E: std::error::Error + Send + Sync + 'static>(context: &str, cause: E) -> Error {
    Error::new(context)
        .with_kind(ErrorKind::External)
        .with_cause(cause)
}
