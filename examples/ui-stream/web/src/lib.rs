//! Listener-free real server-http exchanges and bounded HTTP chunk decoding.
use futures::StreamExt;
use leptos::prelude::*;
use stackpress_example_ui_stream_shared::{DemoData, StreamDemo};

use stackpress_framework_view::leptos::{
    render::initialize_ssr,
    stream::{StreamLeptosRenderer, StreamLimits as RenderStreamLimits},
};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageSnapshot, PublicLocation, PublicStatus,
    ViewError,
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
    http::{
        HttpOptions,
        stream::{HttpResponseExt, StreamLimits},
    },
};
use stackpress_server_http::{ConnectionReport, HttpServer, NetworkLimits};
use std::{
    future::Future,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Independent lifecycle and ownership observations retained by each exchange.
#[derive(Default)]
pub struct Probe {
    /// Exactly one normal response hook per page.
    pub hooks: AtomicUsize,
    /// Precommit errors alone enter the error hook.
    pub errors: AtomicUsize,
    /// Root cleanup happens exactly once on every admitted render.
    pub cleanups: AtomicUsize,
    /// Root and Resource share this capture; teardown must release both.
    marker: Mutex<Option<Weak<()>>>,
}
impl Probe {
    /// Check capture release after the real Hyper host has dropped the body.
    pub fn released(&self) -> bool {
        self.marker
            .lock()
            .unwrap()
            .as_ref()
            .is_none_or(|marker| marker.upgrade().is_none())
    }
}
/// One immutable per-host scenario; no shared mutable global state.
struct Scenario(String);

/// Errors keep a native source and a fixed safe public message.
fn view_error(error: ViewError) -> Error {
    Error::new("UI render failed")
        .with_public_message("Page unavailable")
        .with_cause(error)
}

/// One normal Send handler stages the local renderer before installing its body.
async fn page(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let mode = inv.service::<Scenario>()?.0.clone();
    let base = if req.url.path().starts_with("/demo/") {
        "/demo"
    } else {
        ""
    };
    render_page(req, res, inv, base.to_owned(), mode).await
}
/// Render one scenario while retaining the existing owned stream/cancellation path.
async fn render_page(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
    base: String,
    mode: String,
) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?.clone();
    let assets = PageAssets {
        theme_bootstrap: None,
        build: "ui-stream".into(),
        base: BasePath::new(&base).map_err(view_error)?,
        module: PublicLocation::new("/assets/start.js").map_err(view_error)?,
        stylesheet: None,
    };
    let factory_mode = mode.clone();
    let renderer = StreamLeptosRenderer::new(
        PageContract::new(1, "ui-stream").map_err(view_error)?,
        assets,
        RenderStreamLimits::default(),
        move |snapshot: &PageSnapshot<DemoData>| {
            if factory_mode == "precommit" {
                return Err(ViewError::InvalidSnapshot);
            }
            let marker = Arc::new(());
            *probe.marker.lock().unwrap() = Some(Arc::downgrade(&marker));
            provide_context(marker);
            let cleanup = probe.clone();
            on_cleanup(move || {
                cleanup.cleanups.fetch_add(1, Ordering::SeqCst);
            });
            Ok(view! { <StreamDemo data=snapshot.data.clone()/> })
        },
    )
    .map_err(view_error)?
    .with_status_script(PublicLocation::new("/assets/stream-status.js").map_err(view_error)?)
    .map_err(view_error)?;
    let mode = mode.as_str();
    let seed = PageSnapshot {
        revision: 1,
        build: "ui-stream".into(),
        page: PageId::try_from("stream".to_owned()).map_err(view_error)?,
        location: req.url.path().to_owned(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::ok(),
        data: DemoData {
            delay_ms: 800,
            fail: mode == "late",
            pending: matches!(mode, "disconnect" | "deadline" | "shutdown" | "pending"),
        },
    };
    let rendered = renderer.render(seed).await.map_err(view_error)?;
    let limits = StreamLimits {
        total_timeout: Duration::from_secs(4),
        idle_timeout: if mode == "deadline" {
            Duration::from_millis(80)
        } else {
            Duration::from_secs(2)
        },
        ..Default::default()
    };
    res.stream(
        "text/html; charset=utf-8",
        rendered.body.map(|chunk| chunk.map_err(view_error)),
        limits,
    )?;
    res.status_code(rendered.status, None)?;
    Ok(Flow::Continue)
}
/// Precommit failures use the ordinary fresh response, with no discarded stream.
async fn error(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Arc<Probe>>()?
        .errors
        .fetch_add(1, Ordering::SeqCst);
    res.html("<h1>Page unavailable</h1>")?;
    res.status_code(500, None)?;
    Ok(Flow::Continue)
}
/// There is one hook pass before the server publishes any producer bytes.
async fn response(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<Arc<Probe>>()?
        .hooks
        .fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}
/// Build a real host with no call to bind/listen.
pub async fn host(mode: &str, probe: Arc<Probe>) -> Result<Arc<HttpServer>> {
    initialize_ssr().map_err(view_error)?;
    let mode = mode.to_owned();
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = probe.clone();
        let mode = mode.clone();
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            ctx.register(Arc::new(Scenario(mode)))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.get("/stream", page)?;
    builder.get("/demo/stream", page)?;
    builder.on("error", error)?;
    builder.on("response", response)?;
    let app = Arc::new(builder.build().await.map_err(|error| error.error)?);
    let limits = NetworkLimits {
        shutdown_grace: Duration::from_millis(50),
        ..Default::default()
    };
    Ok(Arc::new(
        HttpServer::new(
            app,
            HttpOptions {
                base_url: "https://stream.invalid/".into(),
                ..Default::default()
            },
        )?
        .with_limits(limits)?,
    ))
}

/// Parsed host bytes plus final ownership/lifecycle evidence.
pub struct Exchange {
    /// Actual received HTTP status and headers, never reconstructed by the demo.
    pub head: String,
    /// Actual decoded HTTP body fragments in received order.
    pub chunks: Vec<String>,
    /// Whether the native peer actually delivered the terminating HTTP chunk.
    pub terminal_chunk: bool,
    /// Host's original completion/failure/cancellation diagnostics.
    pub report: ConnectionReport,
    /// Shared independent counters.
    pub probe: Arc<Probe>,
}

/// Run an exchange over bounded real Hyper IO and report each actual chunk.
///
/// `stop` owns cancellation input. Disconnect stops after useful shell delivery;
/// shutdown uses the host's own stop path. The producer and native host are
/// joined before returning. No socket, external request or detached task exists.
pub async fn exchange(
    mode: &str,
    base: &str,
    mut emit: impl FnMut(&str),
    stop: impl Future<Output = ()> + Send + 'static,
) -> Result<Exchange> {
    let probe = Arc::new(Probe::default());
    let host = host(mode, probe.clone()).await?;
    let (server, mut client) = tokio::io::duplex(128);
    let (shutdown, stopped) = tokio::sync::oneshot::channel::<()>();
    let host_future = async move {
        host.serve_io(server, "127.0.0.1:4321".parse().unwrap(), async {
            let _ = stopped.await;
        })
        .await
    };
    let client_future = async move {
        let read = async {
            client
        .write_all(
            format!(
                "GET {base}/stream HTTP/1.1\r\nHost: stream.invalid\r\nConnection: close\r\n\r\n"
            )
            .as_bytes(),
        )
        .await
        .map_err(Error::from_source)?;
            let mut bytes = Vec::new();
            let mut head = String::new();
            let mut chunks = Vec::new();
            let mut terminal_chunk = false;
            let mut buffer = [0; 256];
            loop {
                let count = client.read(&mut buffer).await.map_err(Error::from_source)?;
                if count == 0 {
                    break;
                }
                bytes.extend_from_slice(&buffer[..count]);
                if head.is_empty() {
                    if let Some(index) = bytes.windows(4).position(|part| part == b"\r\n\r\n") {
                        head = String::from_utf8(bytes.drain(..index + 4).collect())
                            .map_err(Error::from_source)?;
                    } else {
                        continue;
                    }
                }
                if !head
                    .to_ascii_lowercase()
                    .contains("transfer-encoding: chunked")
                {
                    continue;
                }
                while let Some(index) = bytes.windows(2).position(|part| part == b"\r\n") {
                    let size = usize::from_str_radix(
                        std::str::from_utf8(&bytes[..index]).map_err(Error::from_source)?,
                        16,
                    )
                    .map_err(Error::from_source)?;
                    // EOF alone is not successful delivery: retain actual HTTP framing.
                    if size == 0 {
                        terminal_chunk = bytes.starts_with(b"0\r\n\r\n");
                        break;
                    }
                    if bytes.len() < index + 2 + size + 2 {
                        break;
                    }
                    bytes.drain(..index + 2);
                    let chunk = String::from_utf8(bytes.drain(..size).collect())
                        .map_err(Error::from_source)?;
                    bytes.drain(..2);
                    emit(&chunk);
                    chunks.push(chunk);
                    if matches!(mode, "disconnect" | "shutdown")
                        && chunks
                            .last()
                            .is_some_and(|chunk| chunk.contains("id=\"shell\""))
                    {
                        return Ok((head, chunks, terminal_chunk));
                    }
                }
            }
            // Buffered precommit errors have no transfer encoding and retain their exact body.
            if !head
                .to_ascii_lowercase()
                .contains("transfer-encoding: chunked")
                && !bytes.is_empty()
            {
                let chunk = String::from_utf8(bytes).map_err(Error::from_source)?;
                emit(&chunk);
                chunks.push(chunk);
            }
            Ok::<_, Error>((head, chunks, terminal_chunk))
        };
        let received = tokio::select! { result = read => result, _ = stop => Ok((String::new(), Vec::new(), false)) };
        if mode == "shutdown" {
            let _ = shutdown.send(());
            // Keep the peer connected while the host's grace timer cancels the
            // pending render. Disconnect cannot race and mask this proof.
            let mut remainder = Vec::new();
            let _ = client.read_to_end(&mut remainder).await;
        } else {
            drop(shutdown);
        }
        drop(client);
        received
    };
    // Joint ownership means dropping this exchange drops host and client work;
    // no spawned host JoinHandle can be silently detached by cancellation.
    let (report, received) = tokio::join!(host_future, client_future);
    let (head, chunks, terminal_chunk) = received?;
    Ok(Exchange {
        head,
        chunks,
        terminal_chunk,
        report,
        probe,
    })
}

/// Register the same streamed component at three explicit routes without opening IO.
/// # Errors
/// Returns route/service registration errors before the caller builds its host.
pub fn configure(builder: &mut ServerBuilder, base: &str, probe: Arc<Probe>) -> Result<()> {
    use stackpress_lib::handler;
    builder.on_config(task(move |start: &mut StartupContext| {
        let probe = probe.clone();
        Box::pin(async move {
            start.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    for (suffix, mode) in [
        ("stream", "normal"),
        ("late", "late"),
        ("pending", "pending"),
    ] {
        let base = base.to_owned();
        builder.get(
            &format!("{base}/{suffix}"),
            handler(move |req, res, inv| {
                Box::pin(render_page(req, res, inv, base.clone(), mode.to_owned()))
            }),
        )?;
    }
    Ok(())
}
