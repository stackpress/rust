//! Bounded HTTP/1 hosting over caller-supplied IO and generic acceptors.
//!
//! Hyper owns parsing and framing. The host owns admission, deadlines, sequential reuse,
//! cancellation, bounded drain and final reports. Binding is explicit; runtime/process signals
//! remain caller-owned. Deployment policy adds strict authority/proxy trust and optional Rustls.
//! Immutable assets register as ordinary routes before application build.

mod assets;
mod body;
mod connection;
mod deployment;
mod input;
mod io;
mod monitor;
mod observe;
mod options;
mod proxy;
mod report;
mod startup;
mod supervisor;
mod timer;
#[cfg(feature = "tls")]
mod tls;

pub use assets::{AssetBundle, AssetCache, AssetEntry, AssetLimits};
pub use deployment::{BoundHttpServer, NetworkOptions};
pub use observe::{ExchangeObservation, ExchangeObserver};
pub use options::NetworkLimits;
pub use proxy::{IpCidr, ProxyHeaders, ProxyPolicy};
pub use report::{ConnectionReport, HttpDelivery, HttpExchangeReport};
pub use supervisor::{AcceptedIo, Acceptor, DrainReport, Shutdown};
#[cfg(feature = "tls")]
pub use tls::{TlsConfig, TlsFileFormat, TlsFiles};

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
        atomic::{AtomicBool, AtomicU64, Ordering},
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
    /// Validated configured or trusted-proxy origin; untrusted forwarding fields cannot replace it.
    pub public_origin: String,
    /// Cooperative signal set on connection/body drop and deadline cancellation.
    pub cancellation: Cancellation,
    /// Original client asserted by a validated proxy chain; informational, never authorization.
    pub forwarded_client: Option<std::net::IpAddr>,
}
/// Immutable policy and explicit application ownership; construction opens no resources.
pub struct HttpServer {
    /// One built host is shared; per-request startup/registration never occurs.
    app: Arc<Server>,
    /// Existing normalization, lifecycle and one-shot publication authority.
    adapter: StreamingHttpAdapter,
    /// Cloned only when a validated proxy origin selects a different informational URL base.
    http_options: HttpOptions,
    /// Deployment-only authority/proxy policy; legacy buffered compatibility remains opt-in.
    policy: Option<proxy::RequestPolicy>,
    /// Immutable, optional fixed-metadata observer; callbacks hold no internal locks.
    observer: Option<Arc<dyn ExchangeObserver>>,
    /// Checked local observation identity, never sourced from remote headers.
    next_observation: AtomicU64,
    #[cfg(feature = "tls")]
    /// Prepared deployment HTTPS selection; absence is intentional plaintext.
    deployment_tls: Option<TlsConfig>,
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
            http_options: options.clone(),
            policy: None,
            observer: None,
            next_observation: AtomicU64::new(1),
            #[cfg(feature = "tls")]
            deployment_tls: None,
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
    /// Admit supplied IO using the prepared transport, then serve finite sequential HTTP/1 exchanges.
    /// The legacy constructor selects plaintext; deployment HTTPS performs TLS before HTTP.
    /// Excess IO is closed promptly; its report records 503 without inventing wire bytes.
    /// Stop initiates a bounded graceful drain. Await completion before closing services.
    /// Dropping this future drops active work; no host task is spawned by this method.
    pub async fn serve_io<I, F>(&self, io: I, peer: SocketAddr, stop: F) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let registry = report::registry();
        let slots = Arc::clone(&registry);
        let work = async move {
            #[cfg(feature = "tls")]
            if let Some(tls) = &self.deployment_tls {
                let Some(admission) = self.admit(true) else {
                    drop(io);
                    return report::rejected(Error::new("TLS admission exhausted"), 503);
                };
                return self
                    .serve_tls_admitted(
                        io,
                        tls.clone(),
                        stop,
                        admission,
                        ConnectionContext {
                            peer,
                            started: tokio::time::Instant::now(),
                            slots,
                        },
                    )
                    .await;
            }
            let Some(admission) = self.admit(false) else {
                drop(io);
                return report::rejected(Error::new("HTTP connection admission exhausted"), 503);
            };
            let _admission = admission;
            self.serve_connection(
                io,
                "http",
                stop,
                ConnectionContext {
                    peer,
                    started: tokio::time::Instant::now(),
                    slots,
                },
            )
            .await
        };
        self.observed(work, registry).await
    }
    /// Finalize only after the inner future/IO/admission has dropped, including caller cancellation.
    async fn observed<F: Future<Output = ConnectionReport> + Send>(
        &self,
        future: F,
        slots: report::Registry,
    ) -> ConnectionReport {
        let Some(observer) = &self.observer else {
            return future.await;
        };
        let Ok(id) =
            self.next_observation
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        else {
            drop(future);
            return report::rejected(Error::limit("HTTP observation identity exhausted"), 503);
        };
        observe::Observed::new(future, slots, Arc::clone(observer), id).await
    }
    /// Admission rejection has no task/registry; publish only fixed metadata outside its gate.
    fn observe_rejection(&self) {
        if let Some(observer) = &self.observer
            && let Ok(id) =
                self.next_observation
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        {
            observe::notify(
                &**observer,
                id,
                &report::rejected(Error::limit("HTTP admission exhausted"), 503),
            );
        }
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
/// Attested metadata and report ownership survive TLS's change of native IO type.
struct ConnectionContext {
    /// Actual immediate peer, never a forwarding-header claim.
    peer: SocketAddr,
    /// Admission timestamp fixes lifetime across scheduling and handshaking.
    started: tokio::time::Instant,
    /// Bounded report state shared with the final observation owner.
    slots: report::Registry,
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
