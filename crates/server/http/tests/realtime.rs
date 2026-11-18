//! Finite real-transport SSE/WS proofs; an independent parent kills and reaps a hung child.
#![cfg(all(feature = "sse", feature = "websocket"))]
mod support;
use futures_core::Stream;
use futures_util::{SinkExt, StreamExt};
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    ServerBuilder,
    http::sse::{SseEvent, SseOptions, SseResponseExt},
};
use stackpress_server_http::{
    HttpServer, Shutdown, SseHostLimits, WebSocketHostLimits,
    websocket::{WebSocketOptions, WebSocketResponseExt},
};
use std::{
    future::pending,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
static DROPS: AtomicUsize = AtomicUsize::new(0);
static OPENS: AtomicUsize = AtomicUsize::new(0);
struct Events {
    sent: bool,
}
impl Drop for Events {
    fn drop(&mut self) {
        DROPS.fetch_add(1, Ordering::SeqCst);
    }
}
impl Stream for Events {
    type Item = Result<SseEvent>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.sent {
            Poll::Pending
        } else {
            self.sent = true;
            Poll::Ready(Some(SseEvent::data("first")?.id("cursor")?.event("ready")))
        }
    }
}
async fn updates(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.sse(Events { sent: false }, SseOptions::default())?;
    Ok(Flow::Continue)
}
async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.html("healthy")?;
    Ok(Flow::Continue)
}
async fn websocket(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.websocket(
        req,
        WebSocketOptions::default(),
        |mut connection| async move {
            OPENS.fetch_add(1, Ordering::SeqCst);
            if let Some(message) = connection.recv().await? {
                connection.send(message).await?;
            }
            Ok(())
        },
    )?;
    Ok(Flow::Continue)
}
async fn host() -> Result<HttpServer> {
    let mut builder = ServerBuilder::new();
    builder.get("/updates", updates)?;
    builder.get("/ws", websocket)?;
    builder.get("/health", health)?;
    let app = Arc::new(builder.build().await.map_err(|e| e.error)?);
    HttpServer::new(app, Default::default())?
        .with_shutdown_grace(Duration::from_millis(100))?
        .with_sse_limits(SseHostLimits::new(1, SseOptions::default())?)?
        .with_websocket_limits(WebSocketHostLimits::default().with_max_sessions(1)?)
}
async fn head<I: AsyncRead + Unpin>(io: &mut I) -> Vec<u8> {
    let mut bytes = Vec::new();
    while !bytes.ends_with(b"\r\n\r\n") {
        let byte = io.read_u8().await.unwrap();
        bytes.push(byte);
        assert!(bytes.len() < 16384);
    }
    bytes
}
fn handshake() -> Vec<u8> {
    b"GET /ws HTTP/1.1\r\nHost: localhost\r\nConnection: keep-alive\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\r\n".to_vec()
}
async fn ws_exchange<I: AsyncRead + AsyncWrite + Unpin>(mut client: I) {
    let mut bytes = handshake();
    // First masked frame shares the handshake write, proving Hyper read-ahead ownership.
    bytes.extend_from_slice(&[0x81, 0x82, 1, 2, 3, 4, b'h' ^ 1, b'i' ^ 2]);
    client.write_all(&bytes).await.unwrap();
    let headers = head(&mut client).await;
    assert!(
        headers.starts_with(b"HTTP/1.1 101"),
        "{}",
        String::from_utf8_lossy(&headers)
    );
    assert!(String::from_utf8_lossy(&headers).contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="));
    let mut websocket = tokio_tungstenite::WebSocketStream::from_raw_socket(
        client,
        tungstenite::protocol::Role::Client,
        None,
    )
    .await;
    assert_eq!(
        websocket.next().await.unwrap().unwrap(),
        tungstenite::Message::Text("hi".into())
    );
    let frame = websocket.next().await.unwrap().unwrap();
    assert!(
        matches!(frame,tungstenite::Message::Close(Some(frame)) if u16::from(frame.code)==1000)
    );
    websocket.flush().await.unwrap();
}
async fn tcp_sessions() -> Result<()> {
    let host = host().await?;
    let stats = host.realtime_stats();
    let bound = host.bind_prepared("127.0.0.1:0".parse().unwrap()).await?;
    let address = bound.local_addr()?;
    let stop = Shutdown::new();
    let stopping = stop.clone();
    let serving = tokio::spawn(bound.serve_until(async move { stopping.cancelled().await }));
    let mut client = tokio::net::TcpStream::connect(address).await.unwrap();
    client
        .write_all(b"GET /updates HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    assert!(head(&mut client).await.starts_with(b"HTTP/1.1 200"));
    let mut body = Vec::new();
    while !String::from_utf8_lossy(&body).contains("data: first") {
        body.push(client.read_u8().await.unwrap());
        assert!(body.len() < 65536);
    }
    assert_eq!(stats.active(), (1, 0));
    let mut excess = tokio::net::TcpStream::connect(address).await.unwrap();
    excess
        .write_all(b"GET /updates HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    assert!(head(&mut excess).await.starts_with(b"HTTP/1.1 503"));
    drop(excess);
    let mut ordinary = tokio::net::TcpStream::connect(address).await.unwrap();
    ordinary
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    let mut wire = Vec::new();
    ordinary.read_to_end(&mut wire).await.unwrap();
    assert!(wire.ends_with(b"healthy"));
    drop(ordinary);
    ws_exchange(tokio::net::TcpStream::connect(address).await.unwrap()).await;
    stop.stop();
    let mut remaining = Vec::new();
    client.read_to_end(&mut remaining).await.unwrap();
    let report = serving.await.unwrap();
    assert_eq!(report.aborted, 0);
    assert_eq!(stats.active(), (0, 0));
    assert!(DROPS.load(Ordering::SeqCst) >= 2);
    assert_eq!(OPENS.load(Ordering::SeqCst), 1);
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    Ok(())
}
async fn malformed_and_unselected() -> Result<()> {
    let host = Arc::new(host().await?);
    for request in [b"GET /ws HTTP/1.1\r\nHost: localhost\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Version: 13\r\nSec-WebSocket-Key: invalid\r\n\r\n".as_slice(),b"GET /ws HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".as_slice()] {
        let (io,mut client)=tokio::io::duplex(4096);let host=Arc::clone(&host);let server=tokio::spawn(async move {host.serve_io(io,"127.0.0.1:1".parse().unwrap(),pending()).await});client.write_all(request).await.unwrap();assert!(head(&mut client).await.starts_with(b"HTTP/1.1 400"));drop(client);server.await.unwrap();
    }
    Ok(())
}
#[cfg(feature = "tls")]
async fn tls_sessions() -> Result<()> {
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
    };
    let tls = stackpress_server_http::TlsConfig::new(
        vec![CertificateDer::from(
            include_bytes!("fixtures/server.der").to_vec(),
        )],
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            include_bytes!("fixtures/server-key.der").to_vec(),
        )),
    )?;
    let mut roots = RootCertStore::empty();
    roots
        .add(CertificateDer::from(
            include_bytes!("fixtures/ca.der").to_vec(),
        ))
        .unwrap();
    let config =
        ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_root_certificates(roots)
            .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
    for websocket in [true, false] {
        // Each test-owned ephemeral listener accepts exactly one encrypted transport.
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let host = host().await?;
        let stats = host.realtime_stats();
        let stop = Shutdown::new();
        let stopping = stop.clone();
        let tls = tls.clone();
        let server = tokio::spawn(async move {
            let (io, peer) = listener.accept().await.unwrap();
            drop(listener);
            host.serve_tls_io(io, peer, tls, async move { stopping.cancelled().await })
                .await
        });
        let client = tokio::net::TcpStream::connect(address).await.unwrap();
        let mut client = connector
            .connect(ServerName::try_from("localhost").unwrap(), client)
            .await
            .unwrap();
        if websocket {
            ws_exchange(client).await;
        } else {
            client
                .write_all(b"GET /updates HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();
            assert!(head(&mut client).await.starts_with(b"HTTP/1.1 200"));
            assert_eq!(chunk(&mut client).await, b": connected\n\n");
            assert!(
                String::from_utf8(chunk(&mut client).await)
                    .unwrap()
                    .contains("data: first")
            );
            stop.stop();
            assert!(chunk(&mut client).await.is_empty());
        }
        let report = server.await.unwrap();
        assert!(report.failure.is_none(), "{report:?}");
        assert_eq!(stats.active(), (0, 0));
        assert!(tokio::net::TcpStream::connect(address).await.is_err());
    }
    Ok(())
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "invoked only by the parent kill/reap watchdog"]
async fn realtime_child() -> Result<()> {
    tcp_sessions().await?;
    malformed_and_unselected().await?;
    websocket_admission_and_stop().await?;
    lifecycle_selection().await?;
    #[cfg(feature = "tls")]
    tls_sessions().await?;
    Ok(())
}
#[test]
fn realtime_watchdog() {
    support::watchdog::run("realtime_child", Duration::from_secs(20), &[]);
}

/// Bounded chunk decoder for this proof only; Hyper remains the production HTTP parser.
async fn chunk<I: AsyncRead + Unpin>(io: &mut I) -> Vec<u8> {
    let mut line = Vec::new();
    while !line.ends_with(b"\r\n") {
        line.push(io.read_u8().await.unwrap());
        assert!(line.len() < 32);
    }
    let count =
        usize::from_str_radix(std::str::from_utf8(&line[..line.len() - 2]).unwrap(), 16).unwrap();
    assert!(count <= 65536);
    let mut data = vec![0; count];
    io.read_exact(&mut data).await.unwrap();
    let mut end = [0; 2];
    io.read_exact(&mut end).await.unwrap();
    assert_eq!(&end, b"\r\n");
    data
}
struct Quiet;
impl Stream for Quiet {
    type Item = Result<SseEvent>;
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Pending
    }
}
async fn quiet(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.sse(Quiet, SseOptions::default())?;
    Ok(Flow::Continue)
}
async fn quiet_host() -> HttpServer {
    let mut builder = ServerBuilder::new();
    builder.get("/", quiet).unwrap();
    let app = Arc::new(builder.build().await.unwrap());
    HttpServer::new(app, Default::default())
        .unwrap()
        .with_sse_limits(
            SseHostLimits::new(
                1,
                SseOptions::new(
                    Duration::from_secs(310),
                    Duration::from_secs(15),
                    Duration::from_secs(30),
                    65536,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
}
#[tokio::test(start_paused = true)]
async fn sse_clock_survives_ordinary_deadlines_then_rotates_cleanly() {
    let host = quiet_host().await;
    let stats = host.realtime_stats();
    let (io, mut client) = tokio::io::duplex(4096);
    let server = tokio::spawn(async move {
        host.serve_io(io, "127.0.0.1:1".parse().unwrap(), pending())
            .await
    });
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    assert!(head(&mut client).await.starts_with(b"HTTP/1.1 200"));
    let start = tokio::time::Instant::now();
    assert_eq!(chunk(&mut client).await, b": connected\n\n");
    for step in 1..=20 {
        assert_eq!(chunk(&mut client).await, b": heartbeat\n\n");
        assert_eq!(start.elapsed(), Duration::from_secs(step * 15));
        assert_eq!(stats.active(), (1, 0));
    }
    assert!(chunk(&mut client).await.is_empty());
    assert_eq!(start.elapsed(), Duration::from_secs(310));
    let report = server.await.unwrap();
    assert!(report.failure.is_none(), "{report:?}");
    assert_eq!(stats.active(), (0, 0));
}
#[tokio::test(start_paused = true)]
async fn sse_stalled_native_write_expires_despite_producer_progress() {
    let host = quiet_host().await;
    let stats = host.realtime_stats();
    let (io, mut client) = tokio::io::duplex(128);
    let server = tokio::spawn(async move {
        host.serve_io(io, "127.0.0.1:1".parse().unwrap(), pending())
            .await
    });
    client
        .write_all(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    // Leave the response head/event unwritable; heartbeats cannot refresh actual-write idle.
    let report = server.await.unwrap();
    assert!(report.failure.is_some());
    assert_eq!(stats.active(), (0, 0));
}

async fn websocket_admission_and_stop() -> Result<()> {
    let host = host().await?;
    let stats = host.realtime_stats();
    let bound = host.bind_prepared("127.0.0.1:0".parse().unwrap()).await?;
    let address = bound.local_addr()?;
    let stop = Shutdown::new();
    let stopping = stop.clone();
    let serving = tokio::spawn(bound.serve_until(async move { stopping.cancelled().await }));
    let mut first = tokio::net::TcpStream::connect(address).await.unwrap();
    first.write_all(&handshake()).await.unwrap();
    assert!(head(&mut first).await.starts_with(b"HTTP/1.1 101"));
    assert_eq!(stats.active(), (0, 1));
    let mut excess = tokio::net::TcpStream::connect(address).await.unwrap();
    excess.write_all(&handshake()).await.unwrap();
    assert!(head(&mut excess).await.starts_with(b"HTTP/1.1 503"));
    drop(excess);
    let mut first = tokio_tungstenite::WebSocketStream::from_raw_socket(
        first,
        tungstenite::protocol::Role::Client,
        None,
    )
    .await;
    stop.stop();
    let message = first.next().await.unwrap().unwrap();
    assert!(
        matches!(message,tungstenite::Message::Close(Some(frame)) if u16::from(frame.code)==1001)
    );
    first.flush().await.unwrap();
    let report = serving.await.unwrap();
    assert_eq!(report.aborted, 0);
    assert_eq!(stats.active(), (0, 0));
    assert_eq!(stats.rejected(), 1);
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    Ok(())
}
async fn selections(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.url.path() == "/bare" {
        res.status_code(101, None)?;
        return Ok(Flow::Continue);
    }
    res.websocket(req, WebSocketOptions::default(), |_| async {
        panic!("denied callback must never open")
    })?;
    match req.url.path() {
        "/deny" => {
            res.status_code(403, None)?;
        }
        "/replace" => {
            res.html("replacement")?;
        }
        "/headers" => {
            res.headers_mut()?.set("transfer-encoding", "chunked")?;
        }
        _ => {}
    }
    Ok(Flow::Continue)
}
async fn lifecycle_selection() -> Result<()> {
    let mut builder = ServerBuilder::new();
    for path in ["/bare", "/deny", "/replace", "/headers", "/unselected"] {
        builder.get(path, selections)?;
    }
    let host = Arc::new(HttpServer::new(
        Arc::new(builder.build().await.map_err(|e| e.error)?),
        Default::default(),
    )?);
    for (path, status) in [
        ("/bare", 500),
        ("/deny", 403),
        ("/replace", 200),
        ("/headers", 500),
        ("/unselected", 503),
        ("/missing", 404),
    ] {
        let (io, mut client) = tokio::io::duplex(4096);
        let host = Arc::clone(&host);
        let serving = tokio::spawn(async move {
            host.serve_io(io, "127.0.0.1:1".parse().unwrap(), pending())
                .await
        });
        let request = String::from_utf8(handshake()).unwrap().replace("/ws", path);
        client.write_all(request.as_bytes()).await.unwrap();
        let response = head(&mut client).await;
        assert!(
            response.starts_with(format!("HTTP/1.1 {status}").as_bytes()),
            "{path}: {}",
            String::from_utf8_lossy(&response)
        );
        drop(client);
        serving.await.unwrap();
    }
    Ok(())
}
