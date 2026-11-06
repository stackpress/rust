//! Native HTTP policy shared by the listening host and finite adapter proofs.
use stackpress_server::http::HttpOptions;
use stackpress_server_http::NetworkOptions;

/// Match devmetrics' localhost URL while binding only IPv4 loopback.
/// The documented authority allowlist also admits the literal loopback smoke URL.
pub fn network(port: u16) -> NetworkOptions {
    let address = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    let origin = format!("http://localhost:{port}/");
    let mut network = NetworkOptions::http(address, origin.clone());
    network.http = options(&origin);
    network
        .allowed_authorities
        .push(format!("127.0.0.1:{port}"));
    network
}

/// Admit this example's unoptimized browser WASM while retaining default request policies.
/// The dev build is about23MiB; this explicit response allowance belongs to the app adapter.
pub fn options(origin: &str) -> HttpOptions {
    HttpOptions {
        base_url: origin.into(),
        response_limits: stackpress_lib::ParseLimits {
            max_bytes: 32 * 1024 * 1024,
            ..Default::default()
        },
        ..Default::default()
    }
}
