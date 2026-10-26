//! Real HttpAdapter service and HTTP/1 duplex tests: no TCP listener exists here.
use super::*;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Construct a real immutable app using a tiny independently stored asset inventory.
async fn host(limits: GatewayLimits) -> (tempfile::TempDir, Arc<Gateway>) {
    let directory = assets::tests::fixture();
    let assets = assets::AssetBundle::load(directory.path()).unwrap();
    let layers = fixture_layers(directory.path()).await;
    let host = Arc::new(
        Gateway::with_config(BasePath::new("/demo").unwrap(), assets, limits, layers)
            .await
            .unwrap(),
    );
    (directory, host)
}

/// Install fixture data explicitly before the gateway opens its read-only service.
async fn fixture_layers(_directory: &std::path::Path) -> ConfigLayers {
    #[cfg(feature = "sqlite")]
    {
        let path = _directory.join("profiles.sqlite");
        stackpress_example_sql_profile::setup::install_and_seed(&path)
            .await
            .unwrap();
        let mut layers = ConfigLayers::new();
        layers
            .value("fixture", serde_json::json!({"sqlite":{"path":path}}))
            .unwrap();
        layers
    }
    #[cfg(not(feature = "sqlite"))]
    ConfigLayers::new()
}

/// Pages, strict POST errors and exact static paths all traverse HttpAdapter.
#[tokio::test]
async fn page_api_assets_and_errors() {
    let (_directory, host) = host(GatewayLimits::default()).await;
    for (method, path, body, status, contains) in [
        ("GET", "/demo/ui/profiles?id=abc", "", 200, "Ada Lovelace"),
        (
            "GET",
            "/demo/ui/profiles/abc456?return=abc",
            "",
            200,
            "Grace Hopper",
        ),
        (
            "GET",
            "/demo/ui/profiles/missing?return=abc",
            "",
            404,
            "Profile not found",
        ),
        (
            "POST",
            "/demo/ui-api/v1/profile-search",
            r#"{"revision":1,"id":"zzz"}"#,
            200,
            "\"total\"",
        ),
        (
            "POST",
            "/demo/ui-api/v1/profile-search",
            r#"{"revision":999,"id":"abc"}"#,
            400,
            "Refresh to continue",
        ),
        ("GET", "/assets/start.js", "", 404, "Not Found"),
    ] {
        let req = Request::builder()
            .method(method)
            .uri(path)
            .header(
                "content-type",
                if method == "POST" {
                    "application/json"
                } else {
                    "text/plain"
                },
            )
            .body(Full::new(Bytes::from(body)))
            .unwrap();
        let response = host.handle(req).await;
        assert_eq!(response.status(), status, "{method} {path}");
        assert_eq!(response.headers()[header::CACHE_CONTROL], "no-store");
        let text = String::from_utf8(
            response
                .into_body()
                .collect()
                .await
                .unwrap()
                .to_bytes()
                .to_vec(),
        )
        .unwrap();
        assert!(text.contains(contains), "{text}");
    }
    let directory = assets::tests::fixture();
    let bundle = assets::AssetBundle::load(directory.path()).unwrap();
    let path = format!("/demo{}", bundle.module());
    let response = host
        .handle(
            Request::builder()
                .uri(path)
                .body(Full::new(Bytes::new()))
                .unwrap(),
        )
        .await;
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers()[header::CACHE_CONTROL],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(
        response.into_body().collect().await.unwrap().to_bytes(),
        "export {};"
    );
}

/// Streamed unknown-length input has the same ceiling as declared Content-Length.
#[tokio::test]
async fn bounded_headers_and_bodies() {
    let (_directory, host) = host(GatewayLimits {
        body_bytes: 32,
        header_bytes: 128,
        headers: 4,
        ..Default::default()
    })
    .await;
    let req = Request::builder()
        .uri("/demo/ui/profiles")
        .header("x", "a".repeat(129))
        .body(Full::new(Bytes::new()))
        .unwrap();
    assert_eq!(host.handle(req).await.status(), 431);
    let req = Request::builder()
        .uri("/demo/ui/profiles")
        .body(Full::new(Bytes::from(vec![0; 33])))
        .unwrap();
    assert_eq!(host.handle(req).await.status(), 413);
    // Real HTTP/1 parser and chunked framing over an in-memory connection.
    let (mut client, server) = tokio::io::duplex(4096);
    let (stop, _) = watch::channel(false);
    let task = tokio::spawn(Arc::clone(&host).serve(server, stop.subscribe()));
    client.write_all(b"POST /demo/ui-api/v1/profile-search HTTP/1.1\r\nHost: test\r\nTransfer-Encoding: chunked\r\n\r\n21\r\nxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx\r\n0\r\n\r\n").await.unwrap();
    let mut bytes = Vec::new();
    timeout(Duration::from_secs(2), client.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert!(String::from_utf8_lossy(&bytes).starts_with("HTTP/1.1 413"));
    task.await.unwrap().unwrap();
    assert_eq!(host.active_connections(), 0);
}

/// Partial headers, slow bodies, disconnect and graceful draining all finish inside deadlines.
#[tokio::test]
async fn deadlines_disconnect_and_shutdown() {
    let (_directory, host) = host(GatewayLimits {
        header_timeout: Duration::from_millis(40),
        body_timeout: Duration::from_millis(40),
        drain_timeout: Duration::from_millis(60),
        ..Default::default()
    })
    .await;
    for mode in ["headers", "body", "disconnect", "shutdown"] {
        let (mut client, server) = tokio::io::duplex(4096);
        let (stop, _) = watch::channel(false);
        let task = tokio::spawn(Arc::clone(&host).serve(server, stop.subscribe()));
        if mode == "headers" {
            client.write_all(b"GET /demo/ui/profiles").await.unwrap();
        } else {
            client.write_all(b"POST /demo/ui-api/v1/profile-search HTTP/1.1\r\nHost: test\r\nContent-Length: 20\r\n\r\nx").await.unwrap();
        }
        if mode == "disconnect" {
            drop(client);
        } else if mode == "shutdown" {
            stop.send(true).unwrap();
        }
        let _ = timeout(Duration::from_secs(2), task)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(host.active_connections(), 0, "{mode}");
    }
    let (client, server) = tokio::io::duplex(4096);
    let (stop, _) = watch::channel(false);
    let task = tokio::spawn(Arc::clone(&host).serve(server, stop.subscribe()));
    let (mut sender, connection) =
        hyper::client::conn::http1::handshake::<_, Full<Bytes>>(TokioIo::new(client))
            .await
            .unwrap();
    let client_task = tokio::spawn(connection);
    let response = sender
        .send_request(
            Request::builder()
                .uri("/demo/ui/profiles?id=abc")
                .body(Full::new(Bytes::new()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    response.into_body().collect().await.unwrap();
    stop.send(true).unwrap();
    timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    drop(sender);
    client_task.await.unwrap().unwrap();
    assert_eq!(host.active_connections(), 0);
}

/// No executable startup silently invents a port or admits expanded policy.
#[test]
fn launch_and_policy() {
    assert!(LaunchOptions::parse(&[], None).is_err());
    for args in [
        vec!["--port", "0"],
        vec!["--port", "x"],
        vec!["--unknown", "1"],
        vec!["--port"],
        vec!["--port", "3000", "--port", "3001"],
        vec!["--port", "3000", "--duration-seconds", "0"],
        vec!["--port", "3000", "--config"],
        vec!["--port", "3000", "--config", "a", "--config", "b"],
    ] {
        assert!(
            LaunchOptions::parse(
                &args.into_iter().map(str::to_owned).collect::<Vec<_>>(),
                None
            )
            .is_err()
        );
    }
    assert_eq!(LaunchOptions::parse(&[], Some("3002")).unwrap().port, 3002);
    let options = LaunchOptions::parse(
        &[
            "--port",
            "3001",
            "--assets",
            "fixture",
            "--base",
            "/demo",
            "--duration-seconds",
            "1",
            "--config",
            "config.json",
        ]
        .map(String::from),
        Some("3002"),
    )
    .unwrap();
    assert_eq!(options.port, 3001);
    assert_eq!(options.duration, Duration::from_secs(1));
    assert_eq!(options.base.as_str(), "/demo");
    assert_eq!(
        options.config.as_deref(),
        Some(std::path::Path::new("config.json"))
    );
    assert!(
        GatewayLimits {
            connections: 9,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
    assert!(
        GatewayLimits {
            body_timeout: Duration::ZERO,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

/// Unknown-length frames, invalid/trailer data and whole-request deadlines are bounded.
#[tokio::test]
async fn acquisition_failure_table() {
    use std::{
        collections::VecDeque,
        pin::Pin,
        task::{Context, Poll},
    };
    struct Frames {
        frames: VecDeque<Result<hyper::body::Frame<Bytes>, &'static str>>,
        pending: bool,
    }
    impl Body for Frames {
        type Data = Bytes;
        type Error = &'static str;
        fn poll_frame(
            mut self: Pin<&mut Self>,
            _: &mut Context<'_>,
        ) -> Poll<Option<Result<hyper::body::Frame<Bytes>, Self::Error>>> {
            if self.pending {
                Poll::Pending
            } else {
                Poll::Ready(self.frames.pop_front())
            }
        }
    }
    let (_directory, host) = host(GatewayLimits {
        body_bytes: 4,
        body_timeout: Duration::from_millis(30),
        ..Default::default()
    })
    .await;
    for (frames, status) in [
        (
            vec![
                Ok(hyper::body::Frame::data(Bytes::from("123"))),
                Ok(hyper::body::Frame::data(Bytes::from("45"))),
            ],
            413,
        ),
        (vec![Err("PRIVATE_BODY_CAUSE")], 400),
        (
            vec![Ok(hyper::body::Frame::trailers(hyper::HeaderMap::new()))],
            400,
        ),
    ] {
        let response = host
            .handle(Request::new(Frames {
                frames: frames.into(),
                pending: false,
            }))
            .await;
        assert_eq!(response.status(), status);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        assert!(!String::from_utf8_lossy(&body).contains("PRIVATE_BODY_CAUSE"));
    }
    let response = host
        .handle(Request::new(Frames {
            frames: VecDeque::new(),
            pending: true,
        }))
        .await;
    assert_eq!(response.status(), 408);
    let (_short_directory, short) = host_for_request_deadline().await;
    assert_eq!(
        short
            .handle(Request::new(Frames {
                frames: VecDeque::new(),
                pending: true
            }))
            .await
            .status(),
        504
    );
}

/// Give the entire request a shorter budget than its body acquisition.
async fn host_for_request_deadline() -> (tempfile::TempDir, Arc<Gateway>) {
    host(GatewayLimits {
        request_timeout: Duration::from_millis(20),
        ..Default::default()
    })
    .await
}

/// The actual manager accepts finite duplex streams and drains on EOF, failure and stop.
#[tokio::test]
async fn runner_owns_all_admitted_connections() {
    use std::collections::VecDeque;
    let (_directory, host) = host(GatewayLimits {
        connections: 1,
        drain_timeout: Duration::from_millis(30),
        ..Default::default()
    })
    .await;
    for fail in [false, true] {
        let mut streams = VecDeque::new();
        let mut clients = Vec::new();
        for _ in 0..4 {
            let (client, server) = tokio::io::duplex(1024);
            clients.push(client);
            streams.push_back(server);
        }
        let result = Arc::clone(&host)
            .run(
                move || {
                    std::future::ready(match streams.pop_front() {
                        Some(io) => Ok(Some(io)),
                        None if fail => Err(io::Error::other("accept fixture")),
                        None => Ok(None),
                    })
                },
                std::future::pending(),
            )
            .await;
        assert_eq!(result.is_err(), fail);
        assert_eq!(host.active_connections(), 0);
        drop(clients);
    }
    let result = Arc::clone(&host)
        .run(
            std::future::pending::<io::Result<Option<tokio::io::DuplexStream>>>,
            async { Ok(()) },
        )
        .await;
    assert!(result.is_ok());
    assert_eq!(host.active_connections(), 0);
}

/// A pre-stopped connection cannot wait forever for another notification.
#[tokio::test]
async fn already_stopped_connection() {
    let (_directory, host) = host(GatewayLimits::default()).await;
    let (_client, server) = tokio::io::duplex(1024);
    let (stop, signal) = watch::channel(true);
    let result = timeout(
        Duration::from_secs(1),
        Arc::clone(&host).serve(server, signal),
    )
    .await
    .unwrap();
    assert!(result.is_ok());
    assert_eq!(host.active_connections(), 0);
    drop(stop);
}

/// Disconnect and expired graceful draining release an already-entered handler's state.
#[tokio::test]
async fn disconnect_cancels_entered_handler() {
    for graceful in [false, true] {
        cancel_entered_handler(graceful).await;
    }
}

/// Keep a real operation pending until the selected connection exit path cancels it.
async fn cancel_entered_handler(graceful: bool) {
    use stackpress_lib::{Error, Flow, handler};
    struct Ticket(Arc<AtomicUsize>);
    impl Drop for Ticket {
        fn drop(&mut self) {
            self.0.fetch_sub(1, Ordering::SeqCst);
        }
    }
    let entered = Arc::new(tokio::sync::Notify::new());
    let live = Arc::new(AtomicUsize::new(0));
    let mut app = ServerBuilder::new();
    let entry = Arc::clone(&entered);
    let count = Arc::clone(&live);
    app.get(
        "/pending",
        handler(move |_, _, _| {
            let entry = Arc::clone(&entry);
            let count = Arc::clone(&count);
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                let _ticket = Ticket(count);
                entry.notify_one();
                std::future::pending::<Result<Flow, Error>>().await
            })
        }),
    )
    .unwrap();
    let host = Arc::new(Gateway {
        app: app.build().await.unwrap(),
        adapter: HttpAdapter::new(Default::default()).unwrap(),
        limits: GatewayLimits {
            drain_timeout: Duration::from_millis(30),
            ..GatewayLimits::default()
        },
        active: AtomicUsize::new(0),
    });
    let (mut client, server) = tokio::io::duplex(4096);
    let (stop, _) = watch::channel(false);
    let task = tokio::spawn(Arc::clone(&host).serve(server, stop.subscribe()));
    client
        .write_all(b"GET /pending HTTP/1.1\r\nHost: test\r\n\r\n")
        .await
        .unwrap();
    timeout(Duration::from_secs(1), entered.notified())
        .await
        .unwrap();
    assert_eq!(live.load(Ordering::SeqCst), 1);
    if graceful {
        stop.send(true).unwrap();
    } else {
        drop(client);
    }
    let _ = timeout(Duration::from_secs(2), task)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(live.load(Ordering::SeqCst), 0);
    assert_eq!(host.active_connections(), 0);
}
