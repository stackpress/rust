//! Opt-in Rustls over supplied IO; no certificate file discovery or plaintext fallback.

use crate::{ConnectionReport, HttpServer, external};
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use stackpress_lib::Result;
use std::{future::Future, net::SocketAddr, sync::Arc};
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
    /// Admit supplied IO before a finite configured handshake, then serve bounded HTTP/1.1.
    ///
    /// Stop during handshake drops it immediately. Failure never falls back to plaintext
    /// and never invokes application lifecycle callbacks. Reports retain the native cause.
    /// Connection and handshake budgets are acquired synchronously before native TLS work.
    /// Excess IO closes immediately; the report records 503 without plaintext fallback.
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
        let Some(admission) = self.admit(true) else {
            drop(io);
            return crate::report::rejected(
                stackpress_lib::Error::new("TLS admission exhausted"),
                503,
            );
        };
        self.serve_tls_admitted(io, peer, tls, stop, admission, tokio::time::Instant::now())
            .await
    }
    /// Retain a connection permit and release the handshake permit before HTTP starts.
    pub(crate) async fn serve_tls_admitted<I, F>(
        &self,
        io: I,
        peer: SocketAddr,
        tls: TlsConfig,
        stop: F,
        mut admission: crate::Admission,
        started: tokio::time::Instant,
    ) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let measured = std::time::Instant::now();
        tokio::pin!(stop);
        let acceptor = tokio_rustls::TlsAcceptor::from(tls.0);
        // Both budgets begin at admission; task scheduling and TLS cannot reset connection age.
        let deadline = (started + self.limits.handshake_timeout)
            .min(started + self.limits.connection_lifetime);
        let handshake = tokio::select! {
            biased;
            ()=&mut stop=>return ConnectionReport{exchanges:Vec::new(),failure:None,shutdown:true,forced:false},
            result=tokio::time::timeout_at(deadline,acceptor.accept(io))=>result,
        };
        match handshake {
            Ok(Ok(io)) => {
                admission._handshake.take();
                self.serve_connection(io, peer, "https", stop, started)
                    .await
            }
            result => {
                let error = match result {
                    Ok(Err(cause)) => external("TLS handshake failed", cause),
                    Err(cause) => external("TLS handshake deadline exceeded", cause).with_code(408),
                    Ok(Ok(_)) => unreachable!("successful handshake returned above"),
                };
                let status = if error.code() == 408 { 408 } else { 400 };
                let mut report = crate::report::rejected(error, status);
                report.exchanges[0].elapsed = measured.elapsed();
                report
            }
        }
    }
}
