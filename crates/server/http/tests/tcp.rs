//! Ephemeral TCP proof for the thin native adapter; policy/handshake proofs remain in duplex tests.
mod support;

use stackpress_lib::{Body, Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;
use stackpress_server_http::{HttpDelivery, HttpServer, NetworkOptions};
use std::{sync::Arc, time::Duration};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Keep the native-adapter proof independent of stream and asset policy.
async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.set("text/plain", Body::Text("healthy".into()))?;
    Ok(Flow::Continue)
}

/// Only real TCP can exercise bind/local_addr/accept and prove that awaited stop closes the port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run only by the TCP parent watchdog"]
async fn tcp_child() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut builder = ServerBuilder::new();
    builder.get("/health", health)?;
    let app = Arc::new(builder.build().await?);
    let options = NetworkOptions::http("127.0.0.1:0".parse()?, "http://example.test/");
    // This test strictly needs one ephemeral listener; no development server is started.
    let bound = HttpServer::bind(Arc::clone(&app), options).await?;
    let address = bound.local_addr()?;
    assert!(address.ip().is_loopback());
    assert_ne!(address.port(), 0);
    let (stop, stopping) = tokio::sync::oneshot::channel();
    let serving = tokio::spawn(bound.serve_until(async {
        let _ = stopping.await;
    }));

    // Request connection-close so producer handoff/flush settle before asking the source to stop.
    let mut client = tokio::net::TcpStream::connect(address).await?;
    client
        .write_all(b"GET /health HTTP/1.1\r\nHost: example.test\r\nConnection: close\r\n\r\n")
        .await?;
    let mut wire = Vec::new();
    client.read_to_end(&mut wire).await?;
    assert!(wire.starts_with(b"HTTP/1.1 200"));
    assert!(wire.ends_with(b"healthy"));
    drop(client);

    // Awaited stop must reap the admitted task and drop its TCP source before services close.
    stop.send(()).unwrap();
    let report = serving.await?;
    assert_eq!(report.admitted, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(report.aborted, 0);
    assert_eq!(report.task_failures, 0);
    assert!(report.failure.is_none());
    assert_eq!(report.reports.len(), 1);
    assert_eq!(
        report.reports[0].exchanges[0].delivery,
        HttpDelivery::Completed
    );
    // A closed listening endpoint refuses a fresh connection; do not create a replacement listener.
    let reconnect = tokio::time::timeout(
        Duration::from_secs(1),
        tokio::net::TcpStream::connect(address),
    )
    .await
    .expect("post-stop connection attempt is bounded");
    assert!(reconnect.is_err());
    drop(app);
    Ok(())
}

/// The parent can kill/reap even if native accept, connect, flush or drain stalls.
#[test]
fn tcp_watchdog() {
    support::watchdog::run("tcp_child", Duration::from_secs(10), &[]);
}
