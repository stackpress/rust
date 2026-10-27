# TLS API

Available only with `stackpress-server-http/tls`; default off. See the
[TLS guide](../guides/tls.md) for prepared HTTPS options. Source:
[tls.rs](../../src/tls.rs).

## Files and configuration

TlsFiles is Clone/Debug with public `certificates: PathBuf`,
`private_key: PathBuf`, and `format: TlsFileFormat` fields.
TlsFileFormat is Clone/Copy/Debug with Pem and Der variants.

| Operation | Parameters | Returns and effects |
| --- | --- | --- |
| TlsFiles::pem | certificate path, key path (each Into<PathBuf>) | Self; chooses Pem without IO. |
| TlsConfig::from_files | &TlsFiles | Result<TlsConfig>; bounded synchronous regular-file reads, then chain/key validation. |
| TlsConfig::new | Vec<rustls::pki_types::CertificateDer<'static>>, rustls::pki_types::PrivateKeyDer<'static> | Result<TlsConfig>; validates caller-owned material without file IO. |
| NetworkOptions::https | SocketAddr, public origin (Into<String>), TlsFiles | Self; selects HTTPS for preparation/bind. |

TlsConfig is Clone and retains an Arc-backed immutable server configuration.
Certificate and key caps are 64 KiB/16 KiB. PEM accepts a certificate chain and
exactly one key; explicit DER supports certificate/key bytes. Missing, malformed,
mismatched, oversized, symlink or nonregular files fail startup without bind or
downgrade. Keep files operator-owned and unchanged while loading; no request
reopens them.

## Protocol and lifetime

Rustls uses explicit ring, TLS 1.2/1.3 and HTTP/1.1 ALPN. Handshake admission and
time limits apply before host tasks begin TLS work. `serve_tls_io` exposes the
same policy over supplied IO; prepared NetworkOptions HTTPS also applies to
serve_io. See [host reference](host.md) and [limits](limits.md).

There is no mTLS, HTTP/2, certificate discovery/issuance/reload, or password
interface for encrypted keys. Test certificate fixtures are public test data.
