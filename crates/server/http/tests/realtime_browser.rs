//! Explicit finite browser proof; the test process owns the ephemeral production HTTP listener.
#![cfg(all(feature = "sse", feature = "websocket"))]
mod support;
use futures_core::Stream;
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
    http::sse::{SseEvent, SseOptions, SseResponseExt, last_event_id},
};
use stackpress_server_http::{
    HttpServer, RealtimeStats, Shutdown, SseHostLimits, WebSocketHostLimits,
    websocket::{WebSocketOptions, WebSocketResponseExt, WsClose, WsMessage, origin_allowed},
};
use std::{
    collections::VecDeque,
    pin::Pin,
    process::{Child, Command, Stdio},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    task::{Context, Poll},
    time::{Duration, Instant},
};
#[derive(Default)]
struct Probe {
    origin: Mutex<String>,
    stats: Mutex<Option<RealtimeStats>>,
    denied: AtomicUsize,
    opens: AtomicUsize,
    messages: AtomicUsize,
    events: AtomicUsize,
    resumes: AtomicUsize,
    cookies: AtomicUsize,
    expired: AtomicBool,
}
struct Events {
    items: VecDeque<SseEvent>,
    stay: bool,
}
impl Stream for Events {
    type Item = Result<SseEvent>;
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.items.pop_front().map_or(
            if self.stay {
                Poll::Pending
            } else {
                Poll::Ready(None)
            },
            |e| Poll::Ready(Some(Ok(e))),
        )
    }
}
async fn events(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    probe.events.fetch_add(1, Ordering::SeqCst);
    let resumed = last_event_id(req, 1024)? == Some("1");
    if resumed {
        probe.resumes.fetch_add(1, Ordering::SeqCst);
    }
    let items = if resumed {
        vec![SseEvent::data("resumed")?.id("2")?.event("ready")?]
    } else {
        vec![
            SseEvent::data("default")?.id("1")?.retry(20)?,
            SseEvent::data("first")?.id("1")?.event("ready")?,
        ]
    };
    res.sse(
        Events {
            items: items.into(),
            stay: resumed,
        },
        SseOptions::default(),
    )?;
    Ok(Flow::Continue)
}
async fn websocket(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = Arc::clone(inv.service::<Arc<Probe>>()?);
    let expected = probe.origin.lock().unwrap().clone();
    if !origin_allowed(req, &[&expected], false, false)? {
        probe.denied.fetch_add(1, Ordering::SeqCst);
        res.status_code(403, None)?;
        return Ok(Flow::Abort);
    }
    if req.session.get("session").and_then(|v| v.as_str()) == Some("fixture") {
        probe.cookies.fetch_add(1, Ordering::SeqCst);
        // Admission consults current app policy, independently of the transport's snapshot.
        if probe.expired.load(Ordering::SeqCst) {
            res.status_code(401, None)?;
            return Ok(Flow::Abort);
        }
    }
    res.headers_mut()?
        .append("set-cookie", "proof=ok; Path=/; SameSite=Lax")?;
    res.websocket(
        req,
        WebSocketOptions::default().with_snapshot(&["origin"], &["session"])?,
        move |mut connection| async move {
            probe.opens.fetch_add(1, Ordering::SeqCst);
            connection.send(WsMessage::Text("welcome".into())).await?;
            while let Some(message) = connection.recv().await? {
                // Revocation after upgrade must be checked by the application per operation.
                if probe.expired.load(Ordering::SeqCst) {
                    connection
                        .close(WsClose::new(1008, "session expired")?)
                        .await?;
                    return Ok(());
                }
                probe.messages.fetch_add(1, Ordering::SeqCst);
                connection.send(message).await?;
            }
            Ok(())
        },
    )?;
    Ok(Flow::Continue)
}
async fn expire(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // This test-only endpoint deterministically advances the app's authorization state.
    inv.service::<Arc<Probe>>()?
        .expired
        .store(true, Ordering::SeqCst);
    res.html("expired")?;
    Ok(Flow::Continue)
}
async fn stats(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    let active = probe.stats.lock().unwrap().as_ref().unwrap().active();
    res.json(&serde_json::json!({"active":active,"denied":probe.denied.load(Ordering::SeqCst),"opens":probe.opens.load(Ordering::SeqCst),"messages":probe.messages.load(Ordering::SeqCst),"events":probe.events.load(Ordering::SeqCst),"resumes":probe.resumes.load(Ordering::SeqCst),"cookies":probe.cookies.load(Ordering::SeqCst)}))?;
    Ok(Flow::Continue)
}
struct OwnedChild(Option<Child>);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        if let Some(child) = &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "invoked by the explicit browser parent watchdog"]
async fn browser_child() -> Result<()> {
    let artifacts =
        std::env::var("REALTIME_ARTIFACTS").expect("set REALTIME_ARTIFACTS to built proof assets");
    assert!(
        std::path::Path::new(&artifacts).is_dir(),
        "build tools/realtime/proof.py first"
    );
    let probe = Arc::new(Probe::default());
    let mut builder = ServerBuilder::new();
    let shared = Arc::clone(&probe);
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = Arc::clone(&shared);
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.get("/events", events)?;
    builder.get("/ws", websocket)?;
    builder.get("/stats", stats)?;
    builder.post("/expire", expire)?;
    let host = HttpServer::new(
        Arc::new(builder.build().await.map_err(|e| e.error)?),
        Default::default(),
    )?
    .with_sse_limits(SseHostLimits::default())?
    .with_websocket_limits(WebSocketHostLimits::default())?
    .with_shutdown_grace(Duration::from_millis(500))?;
    let native_stats = host.realtime_stats();
    *probe.stats.lock().unwrap() = Some(native_stats.clone());
    let bound = host.bind_prepared("127.0.0.1:0".parse().unwrap()).await?;
    let address = bound.local_addr()?;
    *probe.origin.lock().unwrap() = format!("http://{address}");
    let stop = Shutdown::new();
    let stopping = stop.clone();
    let server = tokio::spawn(bound.serve_until(async move { stopping.cancelled().await }));
    let script = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/realtime-browser.cjs");
    let child = Command::new("node")
        .arg(script)
        .env("REALTIME_ORIGIN", format!("http://{address}"))
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let mut child = OwnedChild(Some(child));
    let deadline = Instant::now() + Duration::from_secs(50);
    let outcome = loop {
        if let Some(status) = child.0.as_mut().unwrap().try_wait().unwrap() {
            child.0 = None;
            break status;
        }
        assert!(
            Instant::now() < deadline,
            "browser subprocess exceeded its finite deadline"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    };
    stop.stop();
    let report = server.await.unwrap();
    assert!(outcome.success(), "browser proof failed");
    assert_eq!(report.aborted, 0);
    assert_eq!(native_stats.active(), (0, 0));
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
    Ok(())
}
#[test]
#[ignore = "requires explicit cached Playwright and built independent artifacts"]
fn browser_watchdog() {
    support::watchdog::run("browser_child", Duration::from_secs(60), &[]);
}
