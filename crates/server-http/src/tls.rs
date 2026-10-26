//! Opt-in Rustls over supplied IO; no certificate file discovery or plaintext fallback.

use crate::{ConnectionReport, HttpServer, external};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use stackpress_lib::Result;
use std::{future::Future, net::SocketAddr, sync::Arc, time::Duration};
use tokio::io::{AsyncRead, AsyncWrite};

/// Validated HTTP/1-only server configuration with explicit ring TLS 1.2/1.3 support.
#[derive(Clone)]
pub struct TlsConfig(Arc<ServerConfig>);
impl TlsConfig {
    /// Check DER certificates and matching private key before any handshake is started.
    ///
    /// ALPN advertises HTTP/1.1 only. Trust/expiry/name checks belong to the client.
    /// No client-authentication or certificate reload policy is implied.
    /// # Errors
    /// Returns a contextual error retaining Rustls's original certificate/key failure.
    pub fn new(
        certificates: Vec<CertificateDer<'static>>,
        key: PrivateKeyDer<'static>,
    ) -> Result<Self> {
        // Explicit provider avoids process-global crypto-provider installation or AWS-LC defaults.
        let mut config =
            ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .map_err(|cause| external("TLS protocol configuration failed", cause))?
                .with_no_client_auth()
                .with_single_cert(certificates, key)
                .map_err(|cause| external("TLS certificate/key configuration failed", cause))?;
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(Self(Arc::new(config)))
    }
}
impl HttpServer {
    /// Handshake supplied IO with a fixed five-second M0 deadline, then serve HTTP/1.1.
    ///
    /// Stop during handshake drops it immediately. Failure never falls back to plaintext
    /// and never invokes application lifecycle callbacks. Reports retain the native cause.
    /// This seam establishes TLS behavior; configurable handshake admission is M1/M2.
    pub async fn serve_tls_io<I, F>(
        &self,
        io: I,
        peer: SocketAddr,
        tls: TlsConfig,
        stop: F,
    ) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        // Measure the rejected handshake too; it has no framework HandleReport of its own.
        let started = std::time::Instant::now();
        tokio::pin!(stop);
        let acceptor = tokio_rustls::TlsAcceptor::from(tls.0);
        // Stop wins before handshake polling, so an already-stopped connection allocates no host.
        let handshake = tokio::select! {
            biased;
            () = &mut stop => return ConnectionReport { exchanges: Vec::new(), failure: None, shutdown: true, forced: false },
            result = tokio::time::timeout(Duration::from_secs(5), acceptor.accept(io)) => result,
        };
        match handshake {
            Ok(Ok(io)) => self.serve_connection(io, peer, "https", stop).await,
            result => {
                let error = match result {
                    Ok(Err(cause)) => external("TLS handshake failed", cause),
                    Err(cause) => external("TLS handshake deadline exceeded", cause),
                    Ok(Ok(_)) => unreachable!("successful handshake returned above"),
                };
                // No HTTP request parts exist at this boundary; never fabricate a host report.
                let error = Arc::new(error);
                ConnectionReport {
                    exchanges: vec![crate::HttpExchangeReport {
                        exchange_id: 1,
                        host: None,
                        delivery: crate::HttpDelivery::RejectedBeforeHost,
                        bytes: 0,
                        elapsed: started.elapsed(),
                        failure: Some(Arc::clone(&error)),
                    }],
                    failure: Some(error),
                    shutdown: false,
                    forced: false,
                }
            }
        }
    }
}
