//! Real Hyper/TLS in-memory proofs, isolated under a parent kill-and-reap watchdog.
use futures_core::Stream;
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
    http::{
        HttpOptions,
        stream::{HttpResponseExt, StreamLimits},
    },
};
use stackpress_server_http::{
    Cancellation, ConnectionReport, HttpDelivery, HttpRequestContext, HttpServer,
};
use std::{
    future::pending,
    pin::Pin,
    process::{Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Notify,
};

/// Independent scenario signals; tests never rely on fixed sleeps for handler/producer entry.
#[derive(Default)]
struct Probe {
    polls: AtomicUsize,
    drops: AtomicUsize,
    calls: AtomicUsize,
    hooks: AtomicUsize,
    released: AtomicBool,
    entered: Notify,
    polled: Notify,
    waker: Mutex<Option<Waker>>,
    cancellation: Mutex<Option<Cancellation>>,
}
/// Real demand-driven producer; pending mode retains no detached producer task.
struct Producer {
    probe: Arc<Probe>,
    step: u8,
    mode: u8,
}
impl Stream for Producer {
    type Item = Result<Vec<u8>>;
    /// Emit a prefix then either EOF, a native-cause error or a cooperatively released suffix.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.probe.polls.fetch_add(1, Ordering::SeqCst);
        if self.step == 1 && self.mode != 0 && !self.probe.released.load(Ordering::SeqCst) {
            *self.probe.waker.lock().unwrap() = Some(cx.waker().clone());
            self.probe.polled.notify_one();
            return Poll::Pending;
        }
        let step = self.step;
        self.step += 1;
        Poll::Ready(match step {
            0 => Some(Ok(b"first".to_vec())),
            1 if self.mode == 1 => {
                Some(Err(Error::new("producer failed")
                    .with_cause(std::io::Error::other("private typed IO"))))
            }
            1 => Some(Ok(b"second".to_vec())),
            _ => None,
        })
    }
}
impl Drop for Producer {
    /// Confirm resource release by the actual Hyper body owner.
    fn drop(&mut self) {
        self.probe.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// Handler entry drop marker for cancellation during lifecycle computation.
struct HandlerDrop(Arc<Probe>);
impl Drop for HandlerDrop {
    /// Prove cancellation drops the active future, rather than merely setting metadata.
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// All production paths go through the real Send handler and trusted context lookup.
async fn route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    let context = inv.adapter_context::<HttpRequestContext>().unwrap();
    assert_eq!(context.peer, "127.0.0.1:1234".parse().unwrap());
    assert_eq!(context.public_origin, "http://localhost/");
    *probe.cancellation.lock().unwrap() = Some(context.cancellation.clone());
    probe.calls.fetch_add(1, Ordering::SeqCst);
    probe.entered.notify_one();
    match req.url.path() {
        "/handler-pending" => {
            let _drop = HandlerDrop(Arc::clone(probe));
            pending::<()>().await;
        }
        "/empty" => {
            // Hyper may omit every body poll for 204; its final report must still complete.
            res.status_code(204, None)?;
            return Ok(Flow::Continue);
        }
        "/buffered" => {
            res.html("buffered")?;
            return Ok(Flow::Continue);
        }
        "/suppress" => {
            res.stop()?;
            return Ok(Flow::Abort);
        }
        "/foreign" => {
            res.dispatch().await?;
            return Ok(Flow::Abort);
        }
        "/panic" => panic!("intentional handler unwind"),
        _ => {}
    }
    let mode = match req.url.path() {
        "/stream-error" => 1,
        "/pending" => 2,
        _ => 0,
    };
    res.stream(
        "text/plain",
        Producer {
            probe: Arc::clone(probe),
            step: 0,
            mode,
        },
        StreamLimits::default(),
    )?;
    Ok(Flow::Continue)
}
/// Publication cannot poll the producer until response hooks have finished.
async fn response_hook(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    probe.hooks.fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}
/// Build an actual application with startup-injected state and a small shutdown grace.
async fn host(probe: Arc<Probe>) -> Result<Arc<HttpServer>> {
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = Arc::clone(&probe);
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.all("/:case", route)?;
    builder.on("response", response_hook)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    Ok(Arc::new(
        HttpServer::new(
            Arc::new(app),
            HttpOptions {
                base_url: "http://localhost/".into(),
                ..Default::default()
            },
        )?
        .with_shutdown_grace(Duration::from_millis(80))?,
    ))
}
/// Every server future is actually spawned on a multithread runtime, proving Send.
async fn ordinary(path: &str) -> Result<(Vec<u8>, ConnectionReport, Arc<Probe>)> {
    let probe = Arc::new(Probe::default());
    let host = host(Arc::clone(&probe)).await?;
    let (server, mut client) = tokio::io::duplex(128);
    let serving = tokio::spawn(async move {
        host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), pending())
            .await
    });
    client
        .write_all(format!("GET {path} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
        .await
        .unwrap();
    let mut wire = Vec::new();
    if path == "/stream-error" {
        // Make publication observable before releasing the producer's original failure.
        while !String::from_utf8_lossy(&wire).contains("first") {
            let mut chunk = [0; 64];
            let count = client.read(&mut chunk).await.unwrap();
            assert_ne!(
                count, 0,
                "connection ended before the prefix reached the peer"
            );
            wire.extend_from_slice(&chunk[..count]);
        }
        probe.released.store(true, Ordering::SeqCst);
        if let Some(waker) = probe.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
    client.read_to_end(&mut wire).await.unwrap();
    Ok((wire, serving.await.unwrap(), probe))
}
/// EOF/error observations are returned exactly once after the real connection/body are dropped.
async fn completion_and_failure() -> Result<()> {
    for path in [
        "/normal",
        "/empty",
        "/buffered",
        "/stream-error",
        "/suppress",
        "/foreign",
    ] {
        let (wire, report, probe) = ordinary(path).await?;
        assert_eq!(report.exchanges.len(), 1);
        let exchange = &report.exchanges[0];
        assert_eq!(exchange.exchange_id, 1);
        assert!(exchange.host.is_some());
        let text = String::from_utf8_lossy(&wire);
        match path {
            "/suppress" => {
                assert!(wire.is_empty());
                assert_eq!(exchange.delivery, HttpDelivery::Suppressed);
            }
            "/foreign" => {
                assert!(wire.is_empty());
                assert_eq!(exchange.delivery, HttpDelivery::Failed);
            }
            "/stream-error" => {
                assert_eq!(exchange.delivery, HttpDelivery::Failed);
                let error = exchange.failure.as_ref().unwrap();
                assert_eq!(error.message(), "producer failed");
                // Default formatting follows the existing host report's diagnostic redaction.
                let diagnostic = format!("{report:?}");
                assert!(!diagnostic.contains("private typed IO"));
                assert!(!diagnostic.contains("producer failed"));
                assert!(
                    std::error::Error::source(error.as_ref())
                        .unwrap()
                        .downcast_ref::<std::io::Error>()
                        .is_some()
                );
                // A real prefix reached the peer; failure cannot append JSON or another response.
                assert!(text.contains("first"));
                assert!(!text.contains("private typed IO"));
                assert!(!text.contains("Internal Server Error"));
                assert_eq!(text.matches("HTTP/1.1").count(), 1);
            }
            _ => {
                assert_eq!(exchange.delivery, HttpDelivery::Completed);
                assert!(report.failure.is_none());
                if path == "/normal" {
                    assert!(text.contains("first"));
                    assert!(text.contains("second"));
                    assert_eq!(exchange.bytes, 11);
                } else if path == "/empty" {
                    assert!(text.starts_with("HTTP/1.1 204"));
                    assert!(text.ends_with("\r\n\r\n"));
                    assert_eq!(exchange.bytes, 0);
                } else {
                    assert!(text.ends_with("buffered"));
                    assert_eq!(exchange.bytes, 8);
                }
            }
        }
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
        assert!(
            probe
                .cancellation
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .is_cancelled()
        );
        if matches!(path, "/normal" | "/stream-error") {
            assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
        }
    }
    Ok(())
}
/// Stop during active streaming drains when released, cancels otherwise, and never spawns work.
async fn shutdown(drain: bool) -> Result<()> {
    let probe = Arc::new(Probe::default());
    let host = host(Arc::clone(&probe)).await?;
    let (server, mut client) = tokio::io::duplex(128);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let stop_observed = Arc::new(Notify::new());
    let observed = Arc::clone(&stop_observed);
    let serving = tokio::spawn(async move {
        host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), async {
            let _ = stopped.await;
            // Acknowledge stop consumption before the peer may release normal stream completion.
            observed.notify_one();
        })
        .await
    });
    client
        .write_all(b"GET /pending HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    let reading = tokio::spawn(async move {
        let mut bytes = Vec::new();
        let _ = client.read_to_end(&mut bytes).await;
        bytes
    });
    probe.polled.notified().await;
    stop.send(()).unwrap();
    stop_observed.notified().await;
    if drain {
        probe.released.store(true, Ordering::SeqCst);
        if let Some(waker) = probe.waker.lock().unwrap().take() {
            waker.wake();
        }
    }
    let report = serving.await.unwrap();
    let wire = reading.await.unwrap();
    assert!(report.shutdown);
    assert_eq!(report.forced, !drain);
    assert_eq!(report.exchanges.len(), 1);
    assert_eq!(
        report.exchanges[0].delivery,
        if drain {
            HttpDelivery::Completed
        } else {
            HttpDelivery::Cancelled
        }
    );
    if drain {
        assert!(String::from_utf8_lossy(&wire).contains("second"));
    }
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert!(
        probe
            .cancellation
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .is_cancelled()
    );
    Ok(())
}
/// Dropping the peer during output releases the exact producer, not only its diagnostics.
async fn disconnect() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let host = host(Arc::clone(&probe)).await?;
    let (server, mut client) = tokio::io::duplex(128);
    let serving = tokio::spawn(async move {
        host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), pending())
            .await
    });
    client
        .write_all(b"GET /pending HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    probe.polled.notified().await;
    drop(client);
    let report = serving.await.unwrap();
    assert!(matches!(
        report.exchanges[0].delivery,
        HttpDelivery::Failed | HttpDelivery::Cancelled
    ));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert!(
        probe
            .cancellation
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .is_cancelled()
    );
    Ok(())
}
/// Dropping serve_io and forcing shutdown both release an active handler and cancel its context.
async fn handler_cancel(abort: bool) -> Result<()> {
    let probe = Arc::new(Probe::default());
    let host = host(Arc::clone(&probe)).await?;
    let (server, mut client) = tokio::io::duplex(128);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let serving = tokio::spawn(async move {
        host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), async {
            let _ = stopped.await;
        })
        .await
    });
    client
        .write_all(b"GET /handler-pending HTTP/1.1\r\nHost: localhost\r\n\r\n")
        .await
        .unwrap();
    probe.entered.notified().await;
    if abort {
        serving.abort();
        assert!(serving.await.unwrap_err().is_cancelled());
    } else {
        stop.send(()).unwrap();
        let report = serving.await.unwrap();
        assert!(report.forced);
        assert!(report.exchanges[0].host.is_none());
        assert_eq!(report.exchanges[0].delivery, HttpDelivery::Cancelled);
    }
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert!(
        probe
            .cancellation
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .is_cancelled()
    );
    Ok(())
}
/// Parser failures precede host reports; normalized malformed targets retain G17 reporting.
async fn input_boundaries() -> Result<()> {
    for (raw, host_expected) in [
        (b"INVALID\r\n\r\n".as_slice(), false),
        (
            b"GET /a/../buffered HTTP/1.1\r\nHost: localhost\r\n\r\n",
            true,
        ),
    ] {
        let probe = Arc::new(Probe::default());
        let host = host(Arc::clone(&probe)).await?;
        let (server, mut client) = tokio::io::duplex(256);
        let serving = tokio::spawn(async move {
            host.serve_io(server, "127.0.0.1:1234".parse().unwrap(), pending())
                .await
        });
        client.write_all(raw).await.unwrap();
        let mut wire = Vec::new();
        client.read_to_end(&mut wire).await.unwrap();
        let report = serving.await.unwrap();
        assert_eq!(report.exchanges.len(), 1);
        assert_eq!(report.exchanges[0].host.is_some(), host_expected);
        assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
        if host_expected {
            assert_eq!(
                report.exchanges[0].host.as_ref().unwrap().failures[0].stage,
                stackpress_server::LifecycleStage::Input
            );
        } else {
            assert_eq!(
                report.exchanges[0].delivery,
                HttpDelivery::RejectedBeforeHost
            );
        }
    }
    // An already-signalled stop must win without loading or dispatching any exchange.
    let probe = Arc::new(Probe::default());
    let host = host(probe).await?;
    let (server, _client) = tokio::io::duplex(128);
    let report = host
        .serve_io(server, "127.0.0.1:1234".parse().unwrap(), async {})
        .await;
    assert!(report.shutdown);
    assert!(!report.forced);
    assert!(report.exchanges.is_empty());
    Ok(())
}
#[cfg(feature = "tls")]
/// Real Tokio-Rustls certificate verification and HTTP bytes, entirely over duplex IO.
async fn tls_handshakes() -> Result<()> {
    use rustls::{
        ClientConfig, RootCertStore,
        pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName},
    };
    use stackpress_server_http::TlsConfig;
    let certificates = || {
        vec![CertificateDer::from(
            include_bytes!("fixtures/server.der").to_vec(),
        )]
    };
    let key = || {
        PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            include_bytes!("fixtures/server-key.der").to_vec(),
        ))
    };
    assert!(TlsConfig::new(Vec::new(), key()).is_err());
    for valid in [true, false] {
        let tls = TlsConfig::new(certificates(), key())?;
        let mut roots = RootCertStore::empty();
        roots
            .add(CertificateDer::from(
                include_bytes!("fixtures/ca.der").to_vec(),
            ))
            .unwrap();
        let mut config =
            ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
                .with_safe_default_protocol_versions()
                .unwrap()
                .with_root_certificates(roots)
                .with_no_client_auth();
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        let probe = Arc::new(Probe::default());
        let host = host(Arc::clone(&probe)).await?;
        let (server, client) = tokio::io::duplex(256);
        let serving = tokio::spawn(async move {
            host.serve_tls_io(server, "127.0.0.1:1234".parse().unwrap(), tls, pending())
                .await
        });
        let name = ServerName::try_from(if valid { "localhost" } else { "wrong.example" }).unwrap();
        let handshake = tokio_rustls::TlsConnector::from(Arc::new(config))
            .connect(name, client)
            .await;
        if valid {
            let mut client = handshake.unwrap();
            assert_eq!(
                client.get_ref().1.alpn_protocol(),
                Some(b"http/1.1".as_slice())
            );
            client
                .write_all(b"GET /normal HTTP/1.1\r\nHost: localhost\r\n\r\n")
                .await
                .unwrap();
            let mut wire = Vec::new();
            client.read_to_end(&mut wire).await.unwrap();
            assert!(String::from_utf8_lossy(&wire).contains("second"));
            let report = serving.await.unwrap();
            assert!(report.failure.is_none());
            assert_eq!(report.exchanges[0].delivery, HttpDelivery::Completed);
        } else {
            let error = handshake.unwrap_err();
            assert!(matches!(
                error
                    .get_ref()
                    .and_then(|e| e.downcast_ref::<rustls::Error>()),
                Some(rustls::Error::InvalidCertificate(
                    rustls::CertificateError::NotValidForNameContext { .. }
                ))
            ));
            let report = serving.await.unwrap();
            assert!(report.failure.is_some());
            assert_eq!(
                report.exchanges[0].delivery,
                HttpDelivery::RejectedBeforeHost
            );
            assert!(report.exchanges[0].host.is_none());
            assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
        }
    }
    Ok(())
}
/// Child-only suite: even blocking bugs remain killable by an independent parent process.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run only by parent watchdog"]
async fn duplex_child() -> Result<()> {
    completion_and_failure().await?;
    shutdown(true).await?;
    shutdown(false).await?;
    disconnect().await?;
    handler_cancel(true).await?;
    handler_cancel(false).await?;
    input_boundaries().await?;
    #[cfg(feature = "tls")]
    tls_handshakes().await?;
    Ok(())
}
/// A deadline is a failure; kill and wait guarantee a hung child cannot outlive the test.
#[test]
fn duplex_watchdog() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "duplex_child", "--ignored", "--nocapture"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success(), "duplex child failed: {status}");
            break;
        }
        if Instant::now() >= deadline {
            // Reap even if the process exits between the timeout check and the kill attempt.
            let _ = child.kill();
            child.wait().expect("timed-out duplex child was reaped");
            panic!("duplex child exceeded 20 seconds; killed and reaped");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
