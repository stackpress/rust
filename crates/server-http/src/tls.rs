//! Opt-in Rustls over supplied IO; no certificate file discovery or plaintext fallback.

use crate::{ConnectionReport, HttpServer, external};
use rustls::pki_types::pem::PemObject;
use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer},
};
use stackpress_lib::Result;
use std::{future::Future, net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::io::{AsyncRead, AsyncWrite};

/// Validated HTTP/1-only server configuration with explicit ring TLS 1.2/1.3 support.
#[derive(Clone)]
pub struct TlsConfig(Arc<ServerConfig>);
/// Pki-types returns a static reason for DER format failure rather than an Error implementation.
#[derive(Debug)]
struct DerKeyError(&'static str);
impl std::fmt::Display for DerKeyError {
    /// Preserve that exact reason only in the explicit diagnostic source, never context formatting.
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.0)
    }
}
impl std::error::Error for DerKeyError {}
/// Explicit file format, never auto-detected or used to select plaintext fallback.
#[derive(Clone, Copy)]
pub enum TlsFileFormat {
    /// PEM certificate chain and exactly one PEM private key.
    Pem,
    /// One DER certificate and one DER private key.
    Der,
}
/// Startup-only certificate/key locations; no file access occurs during a request.
#[derive(Clone)]
pub struct TlsFiles {
    /// Regular certificate/chain file; at most 64KiB, no symlink components.
    pub certificates: PathBuf,
    /// Regular key file; at most 16KiB, no symlink components. Never logged by this library.
    pub private_key: PathBuf,
    /// Deliberately selected encoding.
    pub format: TlsFileFormat,
}
impl TlsFiles {
    /// Select conventional PEM deployment files without opening them.
    ///
    /// # Examples
    /// Select locations without reading files; host preparation validates them before bind.
    /// ```
    /// use stackpress_server_http::{TlsFiles, TlsFileFormat};
    /// let files = TlsFiles::pem("deploy/fullchain.pem", "deploy/private-key.pem");
    /// assert!(matches!(files.format, TlsFileFormat::Pem));
    /// ```
    pub fn pem(certificates: impl Into<PathBuf>, private_key: impl Into<PathBuf>) -> Self {
        Self {
            certificates: certificates.into(),
            private_key: private_key.into(),
            format: TlsFileFormat::Pem,
        }
    }
}
impl TlsConfig {
    /// Read bounded regular files and validate their chain/key before any socket bind.
    /// Files must remain operator-owned and unchanged during preparation; portable metadata/open
    /// cannot exclude hostile concurrent replacement, as with immutable asset startup reads.
    /// # Errors
    /// Preserves original IO, PEM/DER or Rustls key/configuration failures without displaying them.
    ///
    /// # Examples
    /// ```no_run
    /// use stackpress_server_http::{TlsConfig, TlsFiles};
    /// let config = TlsConfig::from_files(&TlsFiles::pem("deploy/fullchain.pem", "deploy/private-key.pem"))?;
    /// // Reuse this prepared immutable configuration; requests do not reopen the files.
    /// # let _ = config; Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_files(files: &TlsFiles) -> Result<Self> {
        let certificates = crate::startup::read_regular(&files.certificates, 64 * 1024)?;
        let key = crate::startup::read_regular(&files.private_key, 16 * 1024)?;
        let (certificates, key) = match files.format {
            TlsFileFormat::Der => (
                vec![CertificateDer::from(certificates)],
                PrivateKeyDer::try_from(key)
                    .map_err(|cause| external("TLS DER key decoding failed", DerKeyError(cause)))?,
            ),
            TlsFileFormat::Pem => {
                let certificates = CertificateDer::pem_slice_iter(&certificates)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|cause| external("TLS certificate PEM decoding failed", cause))?;
                let mut keys = PrivateKeyDer::pem_slice_iter(&key)
                    .collect::<std::result::Result<Vec<_>, _>>()
                    .map_err(|cause| external("TLS key PEM decoding failed", cause))?;
                if keys.len() != 1 {
                    return Err(stackpress_lib::Error::invalid(
                        "TLS requires exactly one private key",
                    ));
                }
                (certificates, keys.remove(0))
            }
        };
        Self::new(certificates, key)
    }
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
        let registry = crate::report::registry();
        let slots = Arc::clone(&registry);
        let work = async move {
            let Some(admission) = self.admit(true) else {
                drop(io);
                return crate::report::rejected(
                    stackpress_lib::Error::new("TLS admission exhausted"),
                    503,
                );
            };
            self.serve_tls_admitted(
                io,
                tls,
                stop,
                admission,
                crate::ConnectionContext {
                    peer,
                    started: tokio::time::Instant::now(),
                    slots,
                },
            )
            .await
        };
        self.observed(work, registry).await
    }
    /// Retain a connection permit and release the handshake permit before HTTP starts.
    pub(crate) async fn serve_tls_admitted<I, F>(
        &self,
        io: I,
        tls: TlsConfig,
        stop: F,
        mut admission: crate::Admission,
        context: crate::ConnectionContext,
    ) -> ConnectionReport
    where
        I: AsyncRead + AsyncWrite + Unpin + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let crate::ConnectionContext {
            peer,
            started,
            slots,
        } = context;
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
        // Collapse timeout/native failures once; the serving match then has no impossible success arm.
        let handshake = match handshake {
            Ok(result) => result.map_err(|cause| external("TLS handshake failed", cause)),
            Err(cause) => Err(external("TLS handshake deadline exceeded", cause).with_code(408)),
        };
        match handshake {
            Ok(io) => {
                admission._handshake.take();
                self.serve_connection(
                    io,
                    "https",
                    stop,
                    crate::ConnectionContext {
                        peer,
                        started,
                        slots,
                    },
                )
                .await
            }
            Err(error) => {
                // Keep timeout classification separate from native handshake rejection without losing its cause.
                let status = if error.code() == 408 { 408 } else { 400 };
                let mut report = crate::report::rejected(error, status);
                report.exchanges[0].elapsed = measured.elapsed();
                report
            }
        }
    }
}
