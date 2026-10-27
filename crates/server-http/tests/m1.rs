//! Native policy proofs over duplex IO, each scenario isolated by a kill-and-reap watchdog.
mod support;

use futures_core::Stream;
use stackpress_lib::{Body, Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
    http::{
        HttpOptions,
        stream::{HttpResponseExt, StreamLimits},
    },
};
use stackpress_server_http::{
    AcceptedIo, Acceptor, Cancellation, ConnectionReport, HttpDelivery, HttpRequestContext,
    HttpServer, NetworkLimits, Shutdown,
};
use std::{
    future::pending,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, DuplexStream},
    sync::{Notify, mpsc},
};

/// Per-scenario signals and counters; no mutable globals coordinate native work.
#[derive(Default)]
struct Probe {
    calls: AtomicUsize,
    errors: AtomicUsize,
    drops: AtomicUsize,
    polls: AtomicUsize,
    entered: Notify,
    polled: Notify,
    released: AtomicBool,
    waker: Mutex<Option<Waker>>,
    cancellations: Mutex<Vec<Cancellation>>,
}
/// Lifetime marker proves actual active-future destruction, independently of metadata.
struct Dropped(Arc<Probe>);
impl Drop for Dropped {
    fn drop(&mut self) {
        self.0.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// Owned demand-driven prefix then pending stream, or an allocation-observable stalled writer.
struct Producer {
    probe: Arc<Probe>,
    step: usize,
    flood: bool,
    trickle: Option<Pin<Box<tokio::time::Sleep>>>,
}
impl Stream for Producer {
    type Item = Result<Vec<u8>>;
    /// Allocation/poll counters distinguish backpressure from unbounded eager collection.
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.probe.polls.fetch_add(1, Ordering::SeqCst);
        if let Some(timer) = self.trickle.as_mut() {
            if std::future::Future::poll(timer.as_mut(), cx).is_pending() {
                return Poll::Pending;
            }
            timer
                .as_mut()
                .reset(tokio::time::Instant::now() + Duration::from_millis(15));
            return Poll::Ready(Some(Ok(vec![b't'])));
        }
        if self.flood {
            return Poll::Ready(Some(Ok(vec![b'x'; 64 * 1024])));
        }
        if self.step == 1 && !self.probe.released.load(Ordering::Acquire) {
            *self.probe.waker.lock().unwrap() = Some(cx.waker().clone());
            self.probe.polled.notify_one();
            return Poll::Pending;
        }
        let step = self.step;
        self.step += 1;
        Poll::Ready(match step {
            0 => Some(Ok(b"first".to_vec())),
            1 => Some(Ok(b"second".to_vec())),
            _ => None,
        })
    }
}
impl Drop for Producer {
    fn drop(&mut self) {
        self.probe.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// Real Send operations access trusted cancellation context and the once-loaded body.
async fn route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    probe.cancellations.lock().unwrap().push(
        inv.adapter_context::<HttpRequestContext>()
            .unwrap()
            .cancellation
            .clone(),
    );
    probe.calls.fetch_add(1, Ordering::SeqCst);
    probe.entered.notify_one();
    match req.url.path() {
        "/pending" => {
            let _guard = Dropped(Arc::clone(probe));
            pending::<()>().await;
        }
        "/panic" => {
            let _guard = Dropped(Arc::clone(probe));
            panic!("intentional M1 task panic");
        }
        "/stream" | "/flood" | "/trickle" => {
            let limits = StreamLimits {
                total_timeout: Duration::from_millis(90),
                idle_timeout: Duration::from_millis(40),
                ..Default::default()
            };
            res.stream(
                "text/plain",
                Producer {
                    probe: Arc::clone(probe),
                    step: 0,
                    flood: req.url.path() == "/flood",
                    trickle: (req.url.path() == "/trickle")
                        .then(|| Box::pin(tokio::time::sleep(Duration::from_millis(1)))),
                },
                limits,
            )?;
        }
        "/large" => {
            res.set(
                "application/octet-stream",
                Body::Bytes(vec![b'x'; 64 * 1024]),
            )?;
        }
        "/body" => {
            res.set("application/octet-stream", req.body().clone())?;
        }
        _ => {
            res.html("buf")?;
        }
    }
    Ok(Flow::Continue)
}
/// Input failure still enters exactly one ordinary error offer, never the operation.
async fn error(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    probe.errors.fetch_add(1, Ordering::SeqCst);
    probe.cancellations.lock().unwrap().push(
        inv.adapter_context::<HttpRequestContext>()
            .unwrap()
            .cancellation
            .clone(),
    );
    Ok(Flow::Continue)
}
/// Short finite policies keep watchdog scenarios deterministic without virtual IO clocks.
fn limits() -> NetworkLimits {
    NetworkLimits {
        max_connections: 4,
        max_handshakes: 2,
        max_requests: 4,
        head_timeout: Duration::from_millis(120),
        head_idle_timeout: Duration::from_millis(50),
        body_timeout: Duration::from_millis(120),
        body_idle_timeout: Duration::from_millis(50),
        lifecycle_timeout: Duration::from_millis(100),
        response_timeout: Duration::from_millis(200),
        write_idle_timeout: Duration::from_millis(70),
        keep_alive_idle: Duration::from_millis(40),
        handshake_timeout: Duration::from_millis(80),
        connection_lifetime: Duration::from_secs(2),
        shutdown_grace: Duration::from_millis(60),
        ..Default::default()
    }
}
/// Build the actual Stackpress lifecycle and IO host; no socket or external service exists.
async fn build_host(probe: Arc<Probe>, limits: NetworkLimits) -> Result<Arc<HttpServer>> {
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = Arc::clone(&probe);
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.all("/:case", route)?;
    builder.on("error", error)?;
    let app = builder.build().await.map_err(|e| e.error)?;
    Ok(Arc::new(
        HttpServer::new(Arc::new(app), HttpOptions::default())?.with_limits(limits)?,
    ))
}
/// Spawn only test-owned IO driving; production serve_io itself spawns no task.
fn serve(host: Arc<HttpServer>, io: DuplexStream) -> tokio::task::JoinHandle<ConnectionReport> {
    tokio::spawn(async move {
        host.serve_io(io, "127.0.0.1:1234".parse().unwrap(), pending())
            .await
    })
}
/// Read until native close, retaining an optional read failure for disconnect tests.
async fn read(mut peer: DuplexStream) -> Vec<u8> {
    let mut bytes = Vec::new();
    let _ = peer.read_to_end(&mut bytes).await;
    bytes
}
/// Send raw bytes through real Hyper, never a test-side HTTP request normalizer.
async fn raw(
    bytes: &[u8],
    policy: NetworkLimits,
) -> Result<(Vec<u8>, ConnectionReport, Arc<Probe>)> {
    let probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&probe), policy).await?;
    let (io, mut peer) = tokio::io::duplex(512);
    let task = serve(host, io);
    let _ = peer.write_all(bytes).await;
    let wire = read(peer).await;
    Ok((wire, task.await.unwrap(), probe))
}
/// All accepted/rejected framing combinations are tested in their actual raw occurrence order.
async fn framing() -> Result<()> {
    let bad:&[&[u8]]=&[
        b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 3\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",
        b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 3\r\nContent-Length: 4\r\n\r\nabc",
        b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: nope\r\n\r\n",
        b"POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked, gzip\r\n\r\n",
        b"POST /body HTTP/1.0\r\nTransfer-Encoding: chunked\r\n\r\n",
        b"POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\nz\r\n",
        b"GET /buf HTTP/1.1\r\nHost: x\r\nX-Thing: good\r\n folded\r\n\r\n",
        b"GET /buf HTTP/1.1\r\nHost: x\r\nX-Thing: bad\x01value\r\n\r\n",
        b"GET /buf HTTP/1.1\r\nHost: x\r\nX-Thing: \xff\r\n\r\n",
        b"POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: gzip, chunked\r\n\r\n0\r\n\r\n",
        b"POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n0\r\nX-Trailer: no\r\n\r\n",
    ];
    for bytes in bad {
        let (wire, report, probe) = raw(bytes, limits()).await?;
        assert_eq!(
            probe.calls.load(Ordering::SeqCst),
            0,
            "{:?}",
            String::from_utf8_lossy(bytes)
        );
        assert!(
            String::from_utf8_lossy(&wire).contains("400"),
            "wire={:?}, report={report:?}",
            String::from_utf8_lossy(&wire)
        );
    }
    // Equal CL is normalized to one length; an ordinary length-delimited successor still reuses IO.
    let input=b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 3\r\nContent-Length: 3\r\n\r\nabcGET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n";
    let (wire, report, probe) = raw(input, limits()).await?;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 2);
    assert_eq!(report.exchanges.len(), 2);
    assert!(
        report
            .exchanges
            .iter()
            .all(|r| r.delivery == HttpDelivery::Completed)
    );
    let text = String::from_utf8_lossy(&wire);
    assert_eq!(text.matches("HTTP/1.1 200").count(), 2);
    assert!(text.contains("abcHTTP/1.1"));
    // TE precedes CL: Hyper drops CL; TE wins, and the entire connection closes after this response.
    for headers in [
        "Transfer-Encoding: chunked\r\nContent-Length: 999\r\n",
        "Transfer-Encoding: chunked\r\n",
    ] {
        let payload = b"GET /fake HTTP/1.1\r\nHost: fake\r\n\r\n";
        let input = format!(
            "POST /body HTTP/1.1\r\nHost: x\r\n{headers}\r\n{:x}\r\n{}\r\n0\r\n\r\nGET /buf HTTP/1.1\r\nHost: x\r\n\r\n",
            payload.len(),
            String::from_utf8_lossy(payload)
        );
        let (wire, report, probe) = raw(input.as_bytes(), limits()).await?;
        assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
        assert_eq!(report.exchanges.len(), 1);
        let text = String::from_utf8_lossy(&wire);
        assert_eq!(text.matches("HTTP/1.1 200").count(), 1);
        assert!(text.contains("connection: close"));
        assert!(text.ends_with(String::from_utf8_lossy(payload).as_ref()));
    }
    Ok(())
}
/// Byte/count boundaries retain the native status while normalization itself keeps G17's 400.
async fn budgets() -> Result<()> {
    let mut policy = limits();
    policy.max_body_bytes = 3;
    for (body, status, calls) in [("abc", 200, 1), ("abcd", 413, 0)] {
        for chunked in [false, true] {
            let input = if chunked {
                format!(
                    "POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n{:x}\r\n{body}\r\n0\r\n\r\n",
                    body.len()
                )
            } else {
                format!(
                    "POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                )
            };
            let (wire, report, probe) = raw(input.as_bytes(), policy.clone()).await?;
            assert!(
                String::from_utf8_lossy(&wire).contains(&format!("HTTP/1.1 {status}")),
                "{report:?}"
            );
            assert_eq!(probe.calls.load(Ordering::SeqCst), calls);
            if status == 413 {
                assert_eq!(probe.errors.load(Ordering::SeqCst), 1);
                assert!(report.exchanges[0].host.is_some());
            }
        }
    }
    let cases = [
        (
            1,
            16 * 1024,
            8192,
            b"GET /buf HTTP/1.1\r\nHost: x\r\nExtra: y\r\n\r\n".as_slice(),
        ),
        (64, 5, 8192, b"GET /buf HTTP/1.1\r\nHost: x\r\nX: y\r\n\r\n"),
        (64, 16384, 3, b"GET /buf HTTP/1.1\r\nHost: x\r\n\r\n"),
    ];
    for (count, headers, target, input) in cases {
        let mut policy = limits();
        policy.max_headers = count;
        policy.max_header_bytes = headers;
        policy.max_target_bytes = target;
        let (wire, _, probe) = raw(input, policy).await?;
        assert!(String::from_utf8_lossy(&wire).contains("431"));
        assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    }
    // Header/target equality is legal, and malformed canonicalization remains G17 with a real report.
    let mut policy = limits();
    policy.max_header_bytes = 5;
    policy.max_target_bytes = 4;
    let (wire, _, probe) = raw(
        b"GET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
        policy.clone(),
    )
    .await?;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    assert!(String::from_utf8_lossy(&wire).contains("431"));
    let (wire, _, probe) = raw(b"GET /buf HTTP/1.1\r\nHost: x\r\n\r\n", policy).await?;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert!(String::from_utf8_lossy(&wire).contains("200"));
    let (wire, report, probe) = raw(
        b"GET /a/../buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n",
        limits(),
    )
    .await?;
    assert!(String::from_utf8_lossy(&wire).contains("400"));
    assert!(report.exchanges[0].host.is_some());
    assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    assert_eq!(probe.errors.load(Ordering::SeqCst), 0);
    // This exact head is 44 decoded method/target/header bytes; three payload bytes total 47.
    for (cap, status) in [(47, 200), (46, 413)] {
        let mut policy = limits();
        policy.max_request_bytes = cap;
        let (wire, _, probe) = raw(
            b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 3\r\nConnection: close\r\n\r\nabc",
            policy,
        )
        .await?;
        assert!(String::from_utf8_lossy(&wire).contains(&format!("HTTP/1.1 {status}")));
        assert_eq!(
            probe.calls.load(Ordering::SeqCst),
            usize::from(status == 200)
        );
    }
    // The parser's independent byte buffer is also bounded, before normalized allocation.
    let mut policy = limits();
    policy.parser_buffer_bytes = 8192;
    let input = format!(
        "GET /buf HTTP/1.1\r\nHost: x\r\nX: {}\r\n\r\n",
        "x".repeat(9000)
    );
    let (wire, _, probe) = raw(input.as_bytes(), policy).await?;
    assert!(String::from_utf8_lossy(&wire).contains("431"));
    assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    Ok(())
}
/// Absolute head/body clocks cannot be extended by one-byte or tiny-chunk progress.
async fn input_deadlines() -> Result<()> {
    for body in [false, true] {
        let probe = Arc::new(Probe::default());
        let mut policy = limits();
        policy.head_timeout = Duration::from_millis(70);
        policy.body_timeout = Duration::from_millis(70);
        let host = build_host(Arc::clone(&probe), policy).await?;
        let (io, mut peer) = tokio::io::duplex(512);
        let task = serve(host, io);
        if body {
            peer.write_all(b"POST /body HTTP/1.1\r\nHost: x\r\nTransfer-Encoding: chunked\r\n\r\n")
                .await
                .unwrap();
        }
        let start = Instant::now();
        for _ in 0..10 {
            if peer
                .write_all(if body { b"1\r\na\r\n" } else { b"G" })
                .await
                .is_err()
            {
                break;
            }
            tokio::time::sleep(Duration::from_millis(15)).await;
        }
        let wire = read(peer).await;
        let report = task.await.unwrap();
        assert!(start.elapsed() < Duration::from_millis(300));
        assert!(String::from_utf8_lossy(&wire).contains("408"), "{report:?}");
        assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
        assert_eq!(probe.errors.load(Ordering::SeqCst), usize::from(body));
        if body {
            let host = report.exchanges[0].host.as_ref().unwrap();
            assert_eq!(host.error_offers.len(), 1);
            assert!(
                std::error::Error::source(host.failures[0].error.as_ref())
                    .unwrap()
                    .downcast_ref::<tokio::time::error::Elapsed>()
                    .is_some()
            );
        }
    }
    // No progress at all is bounded separately from total input time.
    for input in [
        b"G".as_slice(),
        b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 4\r\n\r\na",
    ] {
        let (wire, _, probe) = raw(input, limits()).await?;
        assert!(String::from_utf8_lossy(&wire).contains("408"));
        assert_eq!(probe.calls.load(Ordering::SeqCst), 0);
    }
    Ok(())
}
/// An unfinished handler gets cancellation and one empty 408, without re-running error hooks.
async fn lifecycle_deadline() -> Result<()> {
    let (wire, report, probe) = raw(b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n", limits()).await?;
    assert!(String::from_utf8_lossy(&wire).contains("408"));
    assert_eq!(probe.errors.load(Ordering::SeqCst), 0);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert!(probe.cancellations.lock().unwrap()[0].is_cancelled());
    assert_eq!(report.exchanges[0].delivery, HttpDelivery::Cancelled);
    assert!(report.exchanges[0].host.is_none());
    Ok(())
}
/// Producer idle, total output time, and blocked native writes all release actual resources.
async fn output_deadlines() -> Result<()> {
    for path in ["/stream", "/flood", "/large"] {
        let probe = Arc::new(Probe::default());
        let host = build_host(Arc::clone(&probe), limits()).await?;
        let (io, mut peer) = tokio::io::duplex(128);
        let task = serve(Arc::clone(&host), io);
        peer.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
            .await
            .unwrap();
        if path == "/stream" {
            let wire = read(peer).await;
            assert!(String::from_utf8_lossy(&wire).contains("first"));
            assert!(!String::from_utf8_lossy(&wire).contains("408"));
        } else {
            // A stalled first exchange does not prevent a second independently admitted connection.
            let (io, mut second) = tokio::io::duplex(512);
            let independent = serve(Arc::clone(&host), io);
            second
                .write_all(b"GET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
                .await
                .unwrap();
            assert!(String::from_utf8_lossy(&read(second).await).ends_with("buf"));
            assert!(independent.await.unwrap().failure.is_none());
            // Keep the first peer connected and unread until the native write deadline fires.
            probe.entered.notified().await;
            let report = task.await.unwrap();
            assert!(report.failure.is_some());
            drop(peer);
            if path == "/flood" {
                assert!(probe.polls.load(Ordering::SeqCst) <= 4);
                assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
            }
            continue;
        }
        let report = task.await.unwrap();
        assert!(report.failure.is_some());
        assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    }
    // Producer and write progress continue, but the stream's own total_timeout still wins.
    let start = Instant::now();
    let (wire, report, probe) = raw(b"GET /trickle HTTP/1.1\r\nHost: x\r\n\r\n", limits()).await?;
    assert!(
        start.elapsed() >= Duration::from_millis(80)
            && start.elapsed() < Duration::from_millis(300)
    );
    assert!(String::from_utf8_lossy(&wire).starts_with("HTTP/1.1 200"));
    assert_eq!(report.exchanges[0].delivery, HttpDelivery::Cancelled);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    // A write trickle resets progress but never extends the absolute buffered response budget.
    let probe = Arc::new(Probe::default());
    let mut policy = limits();
    policy.response_timeout = Duration::from_millis(60);
    policy.write_idle_timeout = Duration::from_millis(200);
    let host = build_host(probe, policy).await?;
    let (io, mut peer) = tokio::io::duplex(32);
    let task = serve(host, io);
    peer.write_all(b"GET /large HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let start = Instant::now();
    for _ in 0..12 {
        let mut one = [0; 1];
        if peer.read(&mut one).await.unwrap_or(0) == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(task.await.unwrap().failure.is_some());
    assert!(start.elapsed() < Duration::from_millis(300));
    Ok(())
}
/// Reuse is sequential across producer and flush, bounded by count, idle and connection age.
async fn reuse() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&probe), limits()).await?;
    let (io, mut peer) = tokio::io::duplex(512);
    let task = serve(host, io);
    peer.write_all(b"GET /stream HTTP/1.1\r\nHost: x\r\n\r\nGET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n").await.unwrap();
    probe.polled.notified().await;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    probe.released.store(true, Ordering::Release);
    if let Some(w) = probe.waker.lock().unwrap().take() {
        w.wake();
    }
    let wire = read(peer).await;
    let report = task.await.unwrap();
    let text = String::from_utf8_lossy(&wire);
    assert_eq!(probe.calls.load(Ordering::SeqCst), 2);
    assert_eq!(text.matches("HTTP/1.1 200").count(), 2);
    assert!(text.find("second").unwrap() < text.rfind("buf").unwrap());
    assert!(
        report
            .exchanges
            .iter()
            .all(|e| e.delivery == HttpDelivery::Completed)
    );
    // Hyper's own head timer also bounds a partial successor already in its read-ahead buffer.
    let mut partial = limits();
    partial.head_timeout = Duration::from_millis(300);
    partial.head_idle_timeout = Duration::from_millis(30);
    partial.keep_alive_idle = Duration::from_millis(250);
    let start = Instant::now();
    let (wire, report, probe) = raw(b"GET /buf HTTP/1.1\r\nHost: x\r\n\r\nG", partial).await?;
    assert!(start.elapsed() < Duration::from_millis(200));
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert_eq!(
        String::from_utf8_lossy(&wire)
            .matches("HTTP/1.1 200")
            .count(),
        1
    );
    assert_eq!(report.exchanges.len(), 2);
    assert_eq!(report.exchanges[0].delivery, HttpDelivery::Completed);
    assert_eq!(report.exchanges[1].status, Some(408));
    assert!(report.exchanges[1].host.is_none());
    let mut policy = limits();
    policy.max_requests = 1;
    let (_, report, probe) = raw(
        b"GET /buf HTTP/1.1\r\nHost: x\r\n\r\nGET /buf HTTP/1.1\r\nHost: x\r\n\r\n",
        policy,
    )
    .await?;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert_eq!(report.exchanges.len(), 1);
    let (_, report, probe) = raw(b"GET /buf HTTP/1.1\r\nHost: x\r\n\r\n", limits()).await?;
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert!(report.failure.is_none());
    let mut policy = limits();
    policy.connection_lifetime = Duration::from_millis(30);
    policy.lifecycle_timeout = Duration::from_secs(1);
    let (_, report, probe) = raw(b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n", policy).await?;
    assert!(report.forced);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    Ok(())
}
/// Generic channel-backed acceptor: queue ownership belongs to this test, never the host.
struct Source {
    rx: mpsc::Receiver<AcceptedIo<DuplexStream>>,
}
impl Acceptor for Source {
    type Io = DuplexStream;
    async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
        Ok(self.rx.recv().await)
    }
}
/// Wait for observable ownership rather than racing fixed sleeps with admission.
async fn active(host: &HttpServer, count: usize) {
    tokio::time::timeout(Duration::from_secs(1), async {
        while host.active().0 != count {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
}
/// Overload closes IO promptly; global stop drains all admitted work and returns every lease.
async fn admission_and_drain() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let mut policy = limits();
    policy.max_connections = 2;
    policy.max_handshakes = 1;
    policy.lifecycle_timeout = Duration::from_secs(1);
    let host = build_host(Arc::clone(&probe), policy).await?;
    let stop = Shutdown::new();
    let (tx, rx) = mpsc::channel(4);
    let owner = Arc::clone(&host);
    let stopping = stop.clone();
    let task = tokio::spawn(async move { owner.serve_acceptor(Source { rx }, stopping).await });
    let mut peers = Vec::new();
    for _ in 0..2 {
        let (io, mut peer) = tokio::io::duplex(512);
        tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
            .await
            .unwrap();
        peer.write_all(b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n")
            .await
            .unwrap();
        peers.push(peer);
    }
    active(&host, 2).await;
    tokio::time::timeout(Duration::from_secs(1), async {
        while probe.calls.load(Ordering::SeqCst) != 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let (io, peer) = tokio::io::duplex(128);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
        .await
        .unwrap();
    assert!(read(peer).await.is_empty());
    assert_eq!(host.active().0, 2);
    stop.stop();
    stop.stop();
    let report = task.await.unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(report.rejected, 1);
    assert_eq!(report.completed, 2);
    assert_eq!(host.active(), (0, 0));
    assert_eq!(probe.drops.load(Ordering::SeqCst), 2);
    assert!(
        probe
            .cancellations
            .lock()
            .unwrap()
            .iter()
            .all(Cancellation::is_cancelled)
    );
    drop(peers);
    // Direct supplied IO uses the same immediate admission budget without host-side spawning.
    let probe = Arc::new(Probe::default());
    let mut policy = limits();
    policy.max_connections = 1;
    policy.max_handshakes = 1;
    let host = build_host(probe, policy).await?;
    let (io, _peer) = tokio::io::duplex(128);
    let first = serve(Arc::clone(&host), io);
    active(&host, 1).await;
    let (io, peer) = tokio::io::duplex(128);
    let report = host
        .serve_io(io, "127.0.0.1:1234".parse().unwrap(), pending())
        .await;
    assert_eq!(report.exchanges[0].status, Some(503));
    assert!(read(peer).await.is_empty());
    first.abort();
    let _ = first.await;
    assert_eq!(host.active(), (0, 0));
    Ok(())
}
/// Stop inside accept races no admission: the same mutex linearizes the last spawn boundary.
async fn stop_boundary() -> Result<()> {
    struct StopSource {
        stop: Shutdown,
        io: Option<DuplexStream>,
        polls: Arc<AtomicUsize>,
    }
    impl Acceptor for StopSource {
        type Io = DuplexStream;
        async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
            self.polls.fetch_add(1, Ordering::SeqCst);
            self.stop.stop();
            Ok(self
                .io
                .take()
                .map(|io| AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap())))
        }
    }
    for before in [false, true] {
        let host = build_host(Arc::new(Probe::default()), limits()).await?;
        let stop = Shutdown::new();
        let (io, _peer) = tokio::io::duplex(128);
        let polls = Arc::new(AtomicUsize::new(0));
        if before {
            stop.stop();
        }
        let report = host
            .serve_acceptor(
                StopSource {
                    stop: stop.clone(),
                    io: Some(io),
                    polls: Arc::clone(&polls),
                },
                stop,
            )
            .await;
        assert_eq!(report.admitted, 0);
        assert_eq!(polls.load(Ordering::SeqCst), usize::from(!before));
    }
    Ok(())
}
/// Accept failure drains owned work; task panic is reaped and retains its typed cause.
async fn accept_and_task_errors() -> Result<()> {
    struct Broken {
        io: Option<DuplexStream>,
        probe: Arc<Probe>,
    }
    impl Acceptor for Broken {
        type Io = DuplexStream;
        async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
            if let Some(io) = self.io.take() {
                return Ok(Some(AcceptedIo::plain(
                    io,
                    "127.0.0.1:1234".parse().unwrap(),
                )));
            }
            self.probe.entered.notified().await;
            Err(Error::new("accept failed").with_cause(std::io::Error::other("typed accept")))
        }
    }
    let failed_probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&failed_probe), limits()).await?;
    let (io, mut peer) = tokio::io::duplex(256);
    peer.write_all(b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let owner = Arc::clone(&host);
    let report = owner
        .serve_acceptor(
            Broken {
                io: Some(io),
                probe: Arc::clone(&failed_probe),
            },
            Shutdown::new(),
        )
        .await;
    assert_eq!(report.admitted, 1);
    assert_eq!(report.completed, 1);
    assert_eq!(host.active(), (0, 0));
    assert_eq!(failed_probe.drops.load(Ordering::SeqCst), 1);
    assert!(
        std::error::Error::source(report.failure.as_ref().unwrap().as_ref())
            .unwrap()
            .downcast_ref::<std::io::Error>()
            .is_some()
    );
    let probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&probe), limits()).await?;
    let (tx, rx) = mpsc::channel(1);
    let (io, mut peer) = tokio::io::duplex(256);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
        .await
        .unwrap();
    peer.write_all(b"GET /panic HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    let owner = Arc::clone(&host);
    let stop = Shutdown::new();
    let stopping = stop.clone();
    let task = tokio::spawn(async move { owner.serve_acceptor(Source { rx }, stopping).await });
    probe.entered.notified().await;
    active(&host, 0).await;
    stop.stop();
    let report = task.await.unwrap();
    assert_eq!(report.task_failures, 1);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    Ok(())
}
/// Disconnects during head, loader, handler and output do not strand active connection permits.
async fn disconnects() -> Result<()> {
    for input in [
        b"G".as_slice(),
        b"POST /body HTTP/1.1\r\nHost: x\r\nContent-Length: 20\r\n\r\na",
        b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n",
        b"GET /stream HTTP/1.1\r\nHost: x\r\n\r\n",
    ] {
        let probe = Arc::new(Probe::default());
        let host = build_host(Arc::clone(&probe), limits()).await?;
        let (io, mut peer) = tokio::io::duplex(512);
        let task = serve(Arc::clone(&host), io);
        peer.write_all(input).await.unwrap();
        if input.starts_with(b"GET") {
            probe.entered.notified().await;
        }
        drop(peer);
        let _ = task.await.unwrap();
        assert_eq!(host.active(), (0, 0));
        assert!(
            probe
                .cancellations
                .lock()
                .unwrap()
                .iter()
                .all(Cancellation::is_cancelled)
        );
    }
    Ok(())
}
#[cfg(feature = "tls")]
/// The connection and handshake budgets are both acquired before any TLS task starts.
async fn tls_admission() -> Result<()> {
    use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
    let tls = || {
        stackpress_server_http::TlsConfig::new(
            vec![CertificateDer::from(
                include_bytes!("fixtures/server.der").to_vec(),
            )],
            PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
                include_bytes!("fixtures/server-key.der").to_vec(),
            )),
        )
        .unwrap()
    };
    let mut policy = limits();
    policy.max_handshakes = 1;
    policy.handshake_timeout = Duration::from_millis(60);
    let host = build_host(Arc::new(Probe::default()), policy).await?;
    let (io, _peer) = tokio::io::duplex(128);
    let owner = Arc::clone(&host);
    let config = tls();
    let first = tokio::spawn(async move {
        owner
            .serve_tls_io(io, "127.0.0.1:1234".parse().unwrap(), config, pending())
            .await
    });
    tokio::time::timeout(Duration::from_secs(1), async {
        while host.active() != (1, 1) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let (io, peer) = tokio::io::duplex(128);
    let report = host
        .serve_tls_io(io, "127.0.0.1:1234".parse().unwrap(), tls(), pending())
        .await;
    assert_eq!(report.exchanges[0].status, Some(503));
    assert!(read(peer).await.is_empty());
    assert_eq!(host.active(), (1, 1));
    let report = first.await.unwrap();
    assert!(
        report
            .failure
            .as_ref()
            .unwrap()
            .message()
            .contains("deadline")
    );
    assert_eq!(host.active(), (0, 0));
    let (io, _peer) = tokio::io::duplex(128);
    let report = host
        .serve_tls_io(io, "127.0.0.1:1234".parse().unwrap(), tls(), async {})
        .await;
    assert!(report.shutdown);
    assert_eq!(host.active(), (0, 0));
    // Connection age includes handshaking even when its own handshake budget is longer.
    let mut policy = limits();
    policy.handshake_timeout = Duration::from_millis(400);
    policy.connection_lifetime = Duration::from_millis(30);
    let aged = build_host(Arc::new(Probe::default()), policy).await?;
    let (io, _peer) = tokio::io::duplex(128);
    let started = Instant::now();
    let report = aged
        .serve_tls_io(io, "127.0.0.1:1234".parse().unwrap(), tls(), pending())
        .await;
    assert_eq!(report.exchanges[0].status, Some(408));
    assert!(started.elapsed() < Duration::from_millis(250));
    assert_eq!(aged.active(), (0, 0));
    // The generic supervisor also checks TLS admission before spawning; plaintext stays independent.
    let probe = Arc::new(Probe::default());
    let mut policy = limits();
    policy.max_handshakes = 1;
    policy.handshake_timeout = Duration::from_secs(3);
    let host = build_host(Arc::clone(&probe), policy).await?;
    let stop = Shutdown::new();
    let (tx, rx) = mpsc::channel(3);
    let owner = Arc::clone(&host);
    let stopping = stop.clone();
    let task = tokio::spawn(async move { owner.serve_acceptor(Source { rx }, stopping).await });
    let (io, _tls_peer) = tokio::io::duplex(128);
    tx.send(AcceptedIo::tls(
        io,
        "127.0.0.1:1234".parse().unwrap(),
        tls(),
    ))
    .await
    .unwrap();
    tokio::time::timeout(Duration::from_secs(1), async {
        while host.active() != (1, 1) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    let (io, peer) = tokio::io::duplex(128);
    tx.send(AcceptedIo::tls(
        io,
        "127.0.0.1:1234".parse().unwrap(),
        tls(),
    ))
    .await
    .unwrap();
    assert!(read(peer).await.is_empty());
    assert_eq!(host.active(), (1, 1));
    let (io, mut peer) = tokio::io::duplex(256);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
        .await
        .unwrap();
    peer.write_all(b"GET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
        .await
        .unwrap();
    assert!(String::from_utf8_lossy(&read(peer).await).ends_with("buf"));
    stop.stop();
    let report = task.await.unwrap();
    assert_eq!(report.admitted, 2);
    assert_eq!(report.rejected, 1);
    assert_eq!(report.completed, 2);
    assert_eq!(probe.calls.load(Ordering::SeqCst), 1);
    assert_eq!(host.active(), (0, 0));
    Ok(())
}
/// Initiating graceful drain never extends an existing stream/lifecycle deadline.
async fn deadlines_during_drain() -> Result<()> {
    for path in ["/stream", "/pending"] {
        let probe = Arc::new(Probe::default());
        let mut policy = limits();
        policy.shutdown_grace = Duration::from_secs(1);
        let host = build_host(Arc::clone(&probe), policy).await?;
        let (io, mut peer) = tokio::io::duplex(512);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            host.serve_io(io, "127.0.0.1:1234".parse().unwrap(), async {
                let _ = stopped.await;
            })
            .await
        });
        peer.write_all(format!("GET {path} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes())
            .await
            .unwrap();
        probe.entered.notified().await;
        let started = Instant::now();
        stop.send(()).unwrap();
        let reading = tokio::spawn(read(peer));
        let report = task.await.unwrap();
        let _ = reading.await.unwrap();
        assert!(report.shutdown);
        assert!(!report.forced);
        assert!(started.elapsed() < Duration::from_millis(300));
        assert_eq!(report.exchanges[0].delivery, HttpDelivery::Cancelled);
        assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    }
    // A buffered producer can finish before a stalled writer; forced drain must not claim completion.
    let probe = Arc::new(Probe::default());
    let mut policy = limits();
    policy.response_timeout = Duration::from_secs(2);
    policy.write_idle_timeout = Duration::from_secs(2);
    let host = build_host(Arc::clone(&probe), policy).await?;
    let (io, mut peer) = tokio::io::duplex(128);
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        host.serve_io(io, "127.0.0.1:1234".parse().unwrap(), async {
            let _ = stopped.await;
        })
        .await
    });
    peer.write_all(b"GET /large HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    // Consume some response bytes, then retain an unread peer beyond the shorter drain budget.
    let mut prefix = [0; 128];
    peer.read_exact(&mut prefix).await.unwrap();
    stop.send(()).unwrap();
    let report = task.await.unwrap();
    assert!(report.forced);
    assert_eq!(report.exchanges[0].delivery, HttpDelivery::Cancelled);
    assert!(probe.cancellations.lock().unwrap()[0].is_cancelled());
    drop(peer);
    Ok(())
}
/// Returned history stays bounded while counters cover every admitted and reaped task.
async fn bounded_history() -> Result<()> {
    let mut policy = limits();
    policy.max_connections = 1;
    policy.max_handshakes = 1;
    let host = build_host(Arc::new(Probe::default()), policy).await?;
    let (tx, rx) = mpsc::channel(1);
    let owner = Arc::clone(&host);
    let task =
        tokio::spawn(async move { owner.serve_acceptor(Source { rx }, Shutdown::new()).await });
    for _ in 0..4 {
        active(&host, 0).await;
        let (io, mut peer) = tokio::io::duplex(256);
        tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
            .await
            .unwrap();
        peer.write_all(b"GET /buf HTTP/1.1\r\nHost: x\r\nConnection: close\r\n\r\n")
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&read(peer).await).ends_with("buf"));
    }
    drop(tx);
    let report = task.await.unwrap();
    assert_eq!(report.admitted, 4);
    assert_eq!(report.completed, 4);
    assert_eq!(report.rejected, 0);
    assert_eq!(report.reports.len(), 1);
    assert_eq!(report.discarded_reports, 3);
    assert_eq!(host.active(), (0, 0));
    Ok(())
}
/// An always-ready source cannot starve owned tasks or stop even on a single local executor.
async fn ready_acceptor() -> Result<()> {
    struct Ready;
    impl Acceptor for Ready {
        type Io = DuplexStream;
        async fn accept(&mut self) -> Result<Option<AcceptedIo<Self::Io>>> {
            let (io, peer) = tokio::io::duplex(16);
            drop(peer);
            Ok(Some(AcceptedIo::plain(
                io,
                "127.0.0.1:1234".parse().unwrap(),
            )))
        }
    }
    // LocalSet keeps both futures on one thread so a second worker cannot hide starvation.
    tokio::task::LocalSet::new()
        .run_until(async {
            let host = build_host(Arc::new(Probe::default()), limits()).await?;
            let owner = Arc::clone(&host);
            let stop = Shutdown::new();
            let stopping = stop.clone();
            let started = Instant::now();
            let task =
                tokio::task::spawn_local(
                    async move { owner.serve_acceptor(Ready, stopping).await },
                );
            tokio::time::sleep(Duration::from_millis(30)).await;
            stop.stop();
            let report = task.await.unwrap();
            assert!(started.elapsed() < Duration::from_millis(500));
            assert!(report.admitted > 0);
            assert_eq!(report.completed, report.admitted);
            assert_eq!(host.active(), (0, 0));
            Ok(())
        })
        .await
}
/// Cancelling the supervisor also aborts its owned child tasks rather than detaching them.
async fn supervisor_drop() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let host = build_host(Arc::clone(&probe), limits()).await?;
    let (tx, rx) = mpsc::channel(1);
    let owner = Arc::clone(&host);
    let task =
        tokio::spawn(async move { owner.serve_acceptor(Source { rx }, Shutdown::new()).await });
    let (io, mut peer) = tokio::io::duplex(256);
    tx.send(AcceptedIo::plain(io, "127.0.0.1:1234".parse().unwrap()))
        .await
        .unwrap();
    peer.write_all(b"GET /pending HTTP/1.1\r\nHost: x\r\n\r\n")
        .await
        .unwrap();
    probe.entered.notified().await;
    task.abort();
    assert!(task.await.err().unwrap().is_cancelled());
    active(&host, 0).await;
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    assert!(probe.cancellations.lock().unwrap()[0].is_cancelled());
    Ok(())
}
/// Invalid controls cannot become unlimited sentinels or overflow runtime timer/semaphore state.
async fn invalid_options() -> Result<()> {
    let mut invalid = limits();
    invalid.max_connections = 0;
    assert!(
        build_host(Arc::new(Probe::default()), invalid)
            .await
            .is_err()
    );
    let mut invalid = limits();
    invalid.head_timeout = Duration::ZERO;
    assert!(
        build_host(Arc::new(Probe::default()), invalid)
            .await
            .is_err()
    );
    let mut invalid = limits();
    invalid.parser_buffer_bytes = 8191;
    assert!(
        build_host(Arc::new(Probe::default()), invalid)
            .await
            .is_err()
    );
    Ok(())
}
/// The child selects a single bounded scenario so a hang is independently killable and attributable.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run only by external M1 watchdog"]
async fn scenario_child() -> Result<()> {
    match std::env::var("S7_SCENARIO").unwrap().as_str() {
        "framing" => framing().await,
        "budgets" => budgets().await,
        "input_deadlines" => input_deadlines().await,
        "lifecycle_deadline" => lifecycle_deadline().await,
        "output_deadlines" => output_deadlines().await,
        "reuse" => reuse().await,
        "admission_and_drain" => admission_and_drain().await,
        "stop_boundary" => stop_boundary().await,
        "accept_and_task_errors" => accept_and_task_errors().await,
        "disconnects" => disconnects().await,
        "bounded_history" => bounded_history().await,
        "ready_acceptor" => ready_acceptor().await,
        "deadlines_during_drain" => deadlines_during_drain().await,
        "supervisor_drop" => supervisor_drop().await,
        "invalid_options" => invalid_options().await,
        #[cfg(feature = "tls")]
        "tls_admission" => tls_admission().await,
        _ => panic!("unknown M1 scenario"),
    }
}
/// Every hang-class scenario has a parent process that kills, reaps and fails after ten seconds.
#[test]
fn m1_watchdog() {
    let scenarios = [
        "framing",
        "budgets",
        "input_deadlines",
        "lifecycle_deadline",
        "output_deadlines",
        "reuse",
        "admission_and_drain",
        "stop_boundary",
        "accept_and_task_errors",
        "disconnects",
        "invalid_options",
        "bounded_history",
        "ready_acceptor",
        "deadlines_during_drain",
        "supervisor_drop",
        #[cfg(feature = "tls")]
        "tls_admission",
    ];
    for scenario in scenarios {
        support::watchdog::run(
            "scenario_child",
            Duration::from_secs(10),
            &[("S7_SCENARIO", scenario)],
        );
    }
}
