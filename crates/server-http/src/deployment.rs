//! Validated deployment configuration and a thin TCP adapter over the generic supervisor.
use crate::{
    AcceptedIo, Acceptor, DrainReport, ExchangeObserver, HttpServer, NetworkLimits, ProxyPolicy,
    external, proxy::RequestPolicy,
};
use stackpress_lib::Result;
use stackpress_server::{Server, http::HttpOptions};
use std::{future::Future, net::SocketAddr, sync::Arc};
use tokio::net::{TcpListener, TcpStream};

/// Explicit startup policy; construction/edits open no IO. Preparation validates before bind.
#[must_use]
pub struct NetworkOptions {
    /// Explicit bind address; applications normally supply a loopback development address.
    pub address: SocketAddr,
    /// Trusted origin and value-conversion budgets; no request can replace this policy.
    pub http: HttpOptions,
    /// Finite admission/parser/lifecycle/drain bounds.
    pub limits: NetworkLimits,
    /// Additional literal authorities; public-origin authority is always included.
    pub allowed_authorities: Vec<String>,
    /// None ignores all forwarding fields. Some selects one strict, exact-hop trust policy.
    pub proxy: Option<ProxyPolicy>,
    /// Optional synchronous redacted sink, with no library-created observation queue.
    pub observer: Option<Arc<dyn ExchangeObserver>>,
    #[cfg(feature = "tls")]
    /// Explicit HTTPS files, validated during preparation before the bind syscall.
    pub tls: Option<crate::TlsFiles>,
}
impl NetworkOptions {
    /// Select deliberate plaintext (including deployment behind a trusted TLS proxy).
    ///
    /// ```
    /// use stackpress_server_http::{NetworkOptions, ProxyHeaders, ProxyPolicy};
    /// let mut options = NetworkOptions::http(
    ///     "127.0.0.1:8080".parse()?, "https://profiles.example/",
    /// );
    /// options.proxy = Some(ProxyPolicy {
    ///     peers: vec!["127.0.0.1/32".parse()?],
    ///     hops: 1,
    ///     headers: ProxyHeaders::Forwarded,
    /// });
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn http(address: SocketAddr, public_origin: impl Into<String>) -> Self {
        Self {
            address,
            http: HttpOptions {
                base_url: public_origin.into(),
                ..Default::default()
            },
            limits: Default::default(),
            allowed_authorities: Vec::new(),
            proxy: None,
            observer: None,
            #[cfg(feature = "tls")]
            tls: None,
        }
    }
    /// Select HTTPS explicitly; invalid/missing certificate/key never downgrades to plaintext.
    #[cfg(feature = "tls")]
    pub fn https(
        address: SocketAddr,
        public_origin: impl Into<String>,
        tls: crate::TlsFiles,
    ) -> Self {
        let mut options = Self::http(address, public_origin);
        options.tls = Some(tls);
        options
    }
    /// Select explicit proxy trust before preparing/sharing the host.
    pub fn with_proxy(mut self, proxy: ProxyPolicy) -> Self {
        self.proxy = Some(proxy);
        self
    }
    /// Attach a nonblocking observer before IO admission starts.
    pub fn with_observer(mut self, observer: Arc<dyn ExchangeObserver>) -> Self {
        self.observer = Some(observer);
        self
    }
}
impl HttpServer {
    /// Validate deployment policy and certificate files without binding a socket.
    ///
    /// The same prepared host drives duplex proofs and real TCP accepts. Immutable asset bundles
    /// are loaded/registered before ServerBuilder::build, so they remain ordinary lifecycle routes.
    /// Startup certificate reads are synchronous and bounded; no worker task escapes preparation.
    /// # Errors
    /// Rejects invalid origin/authority/proxy/limit/TLS configuration with original typed causes.
    pub fn from_options(app: Arc<Server>, options: NetworkOptions) -> Result<Self> {
        let policy = RequestPolicy::new(
            &options.http.base_url,
            &options.allowed_authorities,
            options.proxy,
        )?;
        // Reject invalid time/allocation policy before reading certificates or binding any IO.
        options.limits.validate()?;
        #[cfg(feature = "tls")]
        let tls = options
            .tls
            .as_ref()
            .map(crate::TlsConfig::from_files)
            .transpose()?;
        // Only validated immutable policy reaches a shareable host; failure publishes no listener.
        let mut host = Self::new(app, options.http)?.with_limits(options.limits)?;
        host.policy = Some(policy);
        host.observer = options.observer;
        #[cfg(feature = "tls")]
        {
            host.deployment_tls = tls;
        }
        Ok(host)
    }
    /// Validate all startup policy/TLS before binding one TCP listener; never starts serving.
    ///
    /// Assets must already be loaded and registered on the built application. This library owns
    /// no runtime or process signal handler. Repository development must use devmetrics.
    /// # Errors
    /// Returns configuration errors before bind, or a contextual original native bind failure.
    ///
    /// This example is compile-only; repository listeners run through devmetrics:
    /// ```no_run
    /// use std::sync::Arc;
    /// use stackpress_server::{ServerBuilder};
    /// use stackpress_server_http::{HttpServer, NetworkOptions};
    /// # async fn example() -> stackpress_lib::Result<()> {
    /// let app = Arc::new(ServerBuilder::new().build().await.map_err(|failure| failure.error)?);
    /// let options = NetworkOptions::http("127.0.0.1:8080".parse().unwrap(), "http://localhost:8080/");
    /// let bound = HttpServer::bind(Arc::clone(&app), options).await?;
    /// let (_send, stop) = tokio::sync::oneshot::channel::<()>();
    /// let report = bound.serve_until(async { let _ = stop.await; }).await;
    /// // Close process/database services after HTTP drain, while the runtime remains alive.
    /// drop(app);
    /// # let _ = report; Ok(()) }
    /// ```
    pub async fn bind(app: Arc<Server>, options: NetworkOptions) -> Result<BoundHttpServer> {
        let address = options.address;
        let host = Arc::new(Self::from_options(app, options)?);
        let listener = TcpListener::bind(address)
            .await
            .map_err(|cause| external("HTTP TCP bind failed", cause))?;
        Ok(BoundHttpServer { host, listener })
    }
}
/// Bound socket and prepared host; serving remains an explicitly awaited operation.
pub struct BoundHttpServer {
    /// Shared policy/application outlives every supervised connection.
    host: Arc<HttpServer>,
    /// The one bound source is dropped before active connections drain.
    listener: TcpListener,
}
impl BoundHttpServer {
    /// Inspect the actual native address, including a caller-selected ephemeral production port.
    /// # Errors
    /// Retains the original local-address query failure.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        self.listener
            .local_addr()
            .map_err(|cause| external("HTTP bound address query failed", cause))
    }
    /// Stop admission, drain, then abort/reap before returning the final bounded report.
    ///
    /// Fatal accept failures are retained in DrainReport::failure after cleanup. Await this before
    /// closing database/process services. Stop is caller-owned; no detached signal task exists.
    pub async fn serve_until<F: Future<Output = ()> + Send>(self, stop: F) -> DrainReport {
        // TCP contributes ownership/peer metadata only; the generic seam owns stop and drain.
        let source = TcpAcceptor {
            listener: self.listener,
            #[cfg(feature = "tls")]
            tls: self.host.deployment_tls.clone(),
        };
        self.host.serve_acceptor_until(source, stop).await
    }
}
/// TCP supplies owned IO/peer only; all admission, TLS and draining stay in the tested supervisor.
struct TcpAcceptor {
    /// Dropped as soon as admission stops, including fatal accept failure.
    listener: TcpListener,
    #[cfg(feature = "tls")]
    tls: Option<crate::TlsConfig>,
}
impl Acceptor for TcpAcceptor {
    type Io = TcpStream;
    /// Dropping Tokio accept is cancellation-safe; no host task is created here.
    async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
        let (io, peer) = self
            .listener
            .accept()
            .await
            .map_err(|cause| external("HTTP TCP accept failed", cause))?;
        #[cfg(feature = "tls")]
        if let Some(tls) = &self.tls {
            return Ok(Some(AcceptedIo::tls(io, peer, tls.clone())));
        }
        Ok(Some(AcceptedIo::plain(io, peer)))
    }
}
