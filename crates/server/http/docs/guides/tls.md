# Serve HTTPS from files

Select HTTPS explicitly when this process terminates TLS. Enable the host's
`tls` feature and supply operator-managed certificate/key files; failure to
prepare them fails startup before bind and never falls back to plaintext.

## Prepare HTTPS options

This independent example prepares a host without opening a listener. Call
`HttpServer::bind` with the same options when the application is ready to serve:

```rust
use stackpress_server::Server;
use stackpress_server_http::{HttpServer, NetworkOptions, TlsFiles};
use std::sync::Arc;

/// Validate and retain HTTPS material before accepting any connection.
fn prepare(app: Arc<Server>) -> Result<HttpServer, Box<dyn std::error::Error>> {
    let files = TlsFiles::pem("deploy/fullchain.pem", "deploy/private-key.pem");
    let options = NetworkOptions::https(
        "0.0.0.0:8443".parse()?,
        "https://profiles.example:8443/",
        files,
    );
    Ok(HttpServer::from_options(app, options)?)
}
```

The files are read synchronously during preparation. A regular PEM certificate
chain plus exactly one private key is supported. `TlsFiles` with
`format: TlsFileFormat::Der` selects DER input instead. Invalid, missing, or
mismatched material returns a contextual Error with its typed cause retained.

## Keep startup files stable

Certificate bytes are capped at 64 KiB and key bytes at 16 KiB. Files must be
regular and every path component must be free of symlinks; parent traversal is
rejected. Keep the deployment tree operator-owned and unchanged while reading.
Portable metadata/open checks do not provide atomic protection against hostile
concurrent replacement. No request reopens the files after preparation.

Rustls uses the explicit ring provider, TLS 1.2/1.3 and HTTP/1.1 ALPN. Admission
bounds handshakes before task spawn and the finite handshake deadline starts
at admission. There is no mTLS, HTTP/2, encrypted-key password interface,
certificate discovery, issuance, or reload. Rebuild/restart with new material
for rotation. Repository test certificates are public fixtures, never deployment
credentials.

See [TLS reference](../api/tls.md), [limits](../api/limits.md), and the
[proxy guide](proxy.md) when HTTPS terminates upstream instead.
