//! One-shot ownership and lifecycle boundaries, without a network or executor-owned producer.
#![cfg(feature = "http-stream")]
use futures_core::Stream;
use stackpress_lib::queue::task;
use stackpress_lib::{Body, Error, Flow, Invocation, Request, Response, ResponseView, Result};
use stackpress_server::{
    HandleOptions, Server, ServerBuilder, ServerInvocationExt, StartupContext,
    http::{
        HttpAdapter, HttpOptions, HttpOutput,
        stream::{
            HttpBody, HttpResponseExt, StreamLimits, StreamingHttpAdapter, StreamingHttpOutput,
        },
    },
};
use std::{
    future::poll_fn,
    pin::Pin,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    task::{Context, Poll},
};

/// Per-test counters and retained clones, avoiding shared mutable global fixtures.
#[derive(Default)]
struct Probe {
    polls: Arc<AtomicUsize>,
    drops: Arc<AtomicUsize>,
    views: Mutex<Vec<ResponseView>>,
    hooks: AtomicUsize,
}
/// Demand-driven two-chunk producer, with observable destruction and polling.
struct Chunks {
    probe: Arc<Probe>,
    next: u8,
    failure: bool,
}
impl Stream for Chunks {
    type Item = Result<Vec<u8>>;
    /// Yield empty/binary chunks and then EOF or one original producer error.
    fn poll_next(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.probe.polls.fetch_add(1, Ordering::SeqCst);
        let next = self.next;
        self.next += 1;
        Poll::Ready(match next {
            0 => Some(Ok(vec![])),
            1 => Some(Ok(vec![0, 255])),
            2 if self.failure => Some(Err(Error::new("private producer"))),
            _ => None,
        })
    }
}
impl Drop for Chunks {
    /// Producer destruction is independent of retained views sharing only a slot identity.
    fn drop(&mut self) {
        self.probe.drops.fetch_add(1, Ordering::SeqCst);
    }
}
/// Install one stream; paths select observable replacement/failure/bodyless scenarios.
async fn output(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    if req.url.path() == "/reuse" {
        let view = probe.views.lock().unwrap()[0].clone();
        res.set("text/plain", view.body)?;
        return Ok(Flow::Continue);
    }
    let limits = match req.url.path() {
        "/chunk-limit" => StreamLimits {
            max_chunk_bytes: 1,
            ..Default::default()
        },
        "/total-limit" => StreamLimits {
            max_bytes: 1,
            ..Default::default()
        },
        "/limit-equal" => StreamLimits {
            max_bytes: 2,
            max_chunk_bytes: 2,
            ..Default::default()
        },
        _ => StreamLimits::default(),
    };
    res.stream(
        "application/octet-stream",
        Chunks {
            probe: Arc::clone(probe),
            next: 0,
            failure: req.url.path() == "/stream-error",
        },
        limits,
    )?;
    match req.url.path() {
        "/clone" => {
            let view = res.view();
            probe.views.lock().unwrap().extend([view.clone(), view]);
        }
        "/replace" => {
            *res = Response::new();
            res.html("replacement")?;
        }
        "/body-replace" => {
            res.html("replacement")?;
        }
        "/error" => {
            res.headers_mut()?.set("x-old", "old")?;
            res.session_mut()?.set("old", "old", None)?;
            return Err(Error::new("private operation"));
        }
        "/204" => {
            res.status_code(204, None)?;
        }
        "/205" => {
            res.status_code(205, None)?;
        }
        "/304" => {
            res.status_code(304, None)?;
        }
        "/bad-status" => {
            res.status_code(700, None)?;
        }
        "/bad-header" => {
            res.headers_mut()?.set("transfer-encoding", "chunked")?;
        }
        "/headers" => {
            res.headers_mut()?
                .set("content-length", "999")?
                .set("connection", "x-remove")?
                .set("x-remove", "secret")?
                .append("set-cookie", "one=1")?
                .append("set-cookie", "two=2")?;
        }
        "/stop" => {
            res.stop()?;
        }
        "/foreign" => {
            res.dispatch().await?;
        }
        _ => {}
    }
    Ok(Flow::Continue)
}
/// Verify producer is unpolled even when the final response hook observes the installed body.
async fn response_hook(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let probe = inv.service::<Arc<Probe>>()?;
    assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    probe.hooks.fetch_add(1, Ordering::SeqCst);
    Ok(Flow::Continue)
}
/// Publish a fresh app owning one fixture so parallel tests cannot leak counters or snapshots.
async fn app(probe: Arc<Probe>) -> Result<Server> {
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let probe = Arc::clone(&probe);
        Box::pin(async move {
            ctx.register(Arc::new(probe))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.all("/:case", output)?;
    builder.on("response", response_hook)?;
    builder.build().await.map_err(|error| error.error)
}
/// Convert a native head through the real normalized Input runner.
async fn call(
    app: &Server,
    method: &str,
    path: &str,
) -> Result<stackpress_server::http::stream::StreamingHttpReply> {
    let adapter = StreamingHttpAdapter::new(HttpOptions::default())?;
    let (parts, ()) = http::Request::builder()
        .method(method)
        .uri(path)
        .body(())
        .unwrap()
        .into_parts();
    adapter
        .handle_body(app, parts, HandleOptions::default(), || async {
            Ok(Vec::new())
        })
        .await
}
/// Drive an exclusive body and expose the original producer error instead of collecting silently.
async fn collect(mut body: HttpBody) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    while let Some(chunk) = poll_fn(|cx| Pin::new(&mut body).poll_next(cx)).await {
        bytes.extend(chunk?);
    }
    Ok(bytes)
}
/// Snapshots cannot produce a second output or second poll, including a repeated lifecycle.
#[tokio::test(flavor = "multi_thread")]
async fn cloned_views_share_only_one_shot_identity() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let app = app(Arc::clone(&probe)).await?;
    let reply = call(&app, "GET", "/clone").await?;
    assert_eq!(probe.views.lock().unwrap().len(), 2);
    assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    let StreamingHttpOutput::Response(native) = reply.output else {
        panic!("native output missing")
    };
    // A second response can reuse the native identity, but cannot take its producer again.
    let duplicate = call(&app, "GET", "/reuse").await?;
    assert_eq!(duplicate.report.failures.len(), 1);
    assert_eq!(
        duplicate.report.failures[0].stage,
        stackpress_server::LifecycleStage::Output
    );
    let StreamingHttpOutput::Response(fallback) = duplicate.output else {
        panic!()
    };
    assert_eq!(fallback.status(), 500);
    assert!(collect(fallback.into_body()).await?.is_empty());
    assert_eq!(collect(native.into_body()).await?, [0, 255]);
    assert_eq!(probe.polls.load(Ordering::SeqCst), 3);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    Ok(())
}
/// Replacing the whole response/body and failing after success never polls or leaks old bytes.
#[tokio::test]
async fn replacement_and_error_detach_success_stream() -> Result<()> {
    for path in ["/replace", "/body-replace", "/error"] {
        let probe = Arc::new(Probe::default());
        let app = app(Arc::clone(&probe)).await?;
        let reply = call(&app, "GET", path).await?;
        let StreamingHttpOutput::Response(native) = reply.output else {
            panic!()
        };
        assert!(native.headers().get("x-old").is_none());
        assert!(native.headers().get("set-cookie").is_none());
        let bytes = collect(native.into_body()).await?;
        if path == "/error" {
            assert!(!String::from_utf8_lossy(&bytes).contains("private operation"));
        } else {
            assert_eq!(bytes, b"replacement");
        }
        assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
        assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    }
    Ok(())
}
/// HEAD and body-forbidden statuses discard producers without executing their side effects.
#[tokio::test]
async fn head_and_bodyless_are_unpolled_and_framed() -> Result<()> {
    for (method, path, length) in [
        ("HEAD", "/clone", None),
        ("GET", "/204", None),
        ("GET", "/205", Some("0")),
        ("GET", "/304", None),
    ] {
        let probe = Arc::new(Probe::default());
        let app = app(Arc::clone(&probe)).await?;
        let reply = call(&app, method, path).await?;
        assert!(reply.report.failures.is_empty());
        let StreamingHttpOutput::Response(native) = reply.output else {
            panic!()
        };
        assert_eq!(
            native
                .headers()
                .get("content-length")
                .map(|v| v.to_str().unwrap()),
            length
        );
        assert!(collect(native.into_body()).await?.is_empty());
        assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
        assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    }
    Ok(())
}
/// The ordinary buffered adapter must reject a typed producer without collecting it.
#[tokio::test]
async fn buffered_adapter_rejects_stream_without_polling() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let app = app(Arc::clone(&probe)).await?;
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle(
            &app,
            http::Request::builder()
                .uri("/normal")
                .body(vec![])
                .unwrap(),
        )
        .await?;
    let HttpOutput::Response(native) = reply.output else {
        panic!()
    };
    assert_eq!(native.status(), 500);
    assert!(native.body().is_empty());
    assert_eq!(
        reply.report.failures[0].stage,
        stackpress_server::LifecycleStage::Output
    );
    assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    Ok(())
}
/// Protocol preflight rejects invalid status/framing before a producer can be taken or polled.
#[tokio::test]
async fn preflight_headers_failures_suppression_and_foreign_output() -> Result<()> {
    for path in [
        "/bad-status",
        "/bad-header",
        "/headers",
        "/stop",
        "/foreign",
    ] {
        let probe = Arc::new(Probe::default());
        let app = app(Arc::clone(&probe)).await?;
        let reply = call(&app, "GET", path).await?;
        match reply.output {
            StreamingHttpOutput::Response(native) => {
                if path == "/headers" {
                    assert!(native.headers().get("content-length").is_none());
                    assert!(native.headers().get("connection").is_none());
                    assert!(native.headers().get("x-remove").is_none());
                    assert_eq!(native.headers().get_all("set-cookie").iter().count(), 2);
                } else {
                    assert_eq!(native.status(), 500);
                }
                drop(native);
            }
            StreamingHttpOutput::Suppressed => assert_eq!(path, "/stop"),
            StreamingHttpOutput::Unavailable { .. } => assert_eq!(path, "/foreign"),
        }
        assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    }
    Ok(())
}
/// Loader failures are offered once after normalization; malformed input never invokes it.
#[tokio::test]
async fn normalization_and_loader_failures_preserve_g12_g17() -> Result<()> {
    let app = ServerBuilder::new()
        .build()
        .await
        .map_err(|error| error.error)?;
    let adapter = StreamingHttpAdapter::new(HttpOptions::default())?;
    for path in ["*", "/valid"] {
        let (parts, ()) = http::Request::builder()
            .uri(path)
            .body(())
            .unwrap()
            .into_parts();
        let called = AtomicUsize::new(0);
        let reply = adapter
            .handle_body(&app, parts, HandleOptions::default(), || async {
                called.fetch_add(1, Ordering::SeqCst);
                Err(Error::new("private loader").with_cause(std::io::Error::other("typed IO")))
            })
            .await?;
        assert_eq!(called.load(Ordering::SeqCst), usize::from(path == "/valid"));
        assert_eq!(
            reply.report.error_offers.len(),
            usize::from(path == "/valid")
        );
        assert_eq!(
            reply.report.failures[0].stage,
            stackpress_server::LifecycleStage::Input
        );
    }
    Ok(())
}
/// Stream policy is checked before installation; errors fuse rather than polling the producer again.
#[tokio::test]
async fn producer_errors_fuse_and_byte_limits_are_exact() -> Result<()> {
    let probe = Arc::new(Probe::default());
    let app = app(Arc::clone(&probe)).await?;
    let reply = call(&app, "GET", "/stream-error").await?;
    let StreamingHttpOutput::Response(native) = reply.output else {
        panic!()
    };
    let mut body = native.into_body();
    assert_eq!(collect_chunks(&mut body).await, 2);
    assert!(
        poll_fn(|cx| Pin::new(&mut body).poll_next(cx))
            .await
            .is_none()
    );
    assert_eq!(probe.polls.load(Ordering::SeqCst), 3);
    let mut response = Response::new();
    let limits = StreamLimits {
        max_bytes: 0,
        ..Default::default()
    };
    assert!(
        response
            .stream(
                "text/plain",
                Chunks {
                    probe,
                    next: 0,
                    failure: false
                },
                limits
            )
            .is_err()
    );
    assert!(matches!(response.body(), Body::Empty));
    Ok(())
}
/// Read exactly through one producer error, retaining its unmodified identity and prior chunks.
async fn collect_chunks(body: &mut HttpBody) -> usize {
    let mut count = 0;
    while let Some(item) = poll_fn(|cx| Pin::new(&mut *body).poll_next(cx)).await {
        match item {
            Ok(_) => count += 1,
            Err(error) => {
                assert_eq!(error.message(), "private producer");
                break;
            }
        }
    }
    count
}

/// Exact equality passes; oversize chunks and aggregate totals fail before yielding offending bytes.
#[tokio::test]
async fn stream_byte_boundaries_and_drop_release() -> Result<()> {
    for path in ["/limit-equal", "/chunk-limit", "/total-limit"] {
        let probe = Arc::new(Probe::default());
        let app = app(Arc::clone(&probe)).await?;
        let reply = call(&app, "GET", path).await?;
        let StreamingHttpOutput::Response(native) = reply.output else {
            panic!()
        };
        let result = collect(native.into_body()).await;
        if path == "/limit-equal" {
            assert_eq!(result?, [0, 255]);
        } else {
            assert_eq!(result.unwrap_err().kind(), stackpress_lib::ErrorKind::Limit);
        }
        assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    }
    let probe = Arc::new(Probe::default());
    let app = app(Arc::clone(&probe)).await?;
    let reply = call(&app, "GET", "/normal").await?;
    drop(reply);
    assert_eq!(probe.polls.load(Ordering::SeqCst), 0);
    assert_eq!(probe.drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Both adapters propagate trusted metadata into the same host context without public input authority.
#[tokio::test]
async fn trusted_options_reach_both_entrypoints() -> Result<()> {
    async fn metadata(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        res.html(inv.adapter_context::<String>().unwrap())?;
        Ok(Flow::Continue)
    }
    let mut builder = ServerBuilder::new();
    builder.get("/metadata", metadata)?;
    let app = builder.build().await.map_err(|error| error.error)?;
    let options = || HandleOptions {
        adapter_context: Some(Arc::new("trusted".to_owned())),
    };
    let input = || {
        http::Request::builder()
            .uri("/metadata")
            .body(Vec::new())
            .unwrap()
    };
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle_with_options(&app, input(), options())
        .await?;
    let HttpOutput::Response(native) = reply.output else {
        panic!()
    };
    assert_eq!(native.body(), b"trusted");
    let (parts, _) = input().into_parts();
    let reply = StreamingHttpAdapter::new(HttpOptions::default())?
        .handle_body(&app, parts, options(), || async { Ok(Vec::new()) })
        .await?;
    let StreamingHttpOutput::Response(native) = reply.output else {
        panic!()
    };
    assert_eq!(collect(native.into_body()).await?, b"trusted");
    Ok(())
}
