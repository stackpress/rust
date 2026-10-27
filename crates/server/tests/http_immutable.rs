//! Shared immutable representations retain one-shot output authority and exact native lengths.
#![cfg(feature = "http-stream")]
use futures_core::Stream;
use stackpress_lib::{
    Body, Error, Flow, Invocation, Request, Response, ResponseView, Result, queue::task,
};
use stackpress_server::{
    ServerBuilder, ServerInvocationExt, StartupContext,
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
    sync::{Arc, Mutex},
};

/// The byte allocation and retained views live independently from the final Response value.
struct Fixture {
    bytes: Arc<[u8]>,
    views: Mutex<Vec<ResponseView>>,
}
/// Real lifecycle operation selects checked immutable bytes or attempts to reuse an old view.
async fn output(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let fixture = inv.service::<Arc<Fixture>>()?;
    if req.url.path() == "/reuse" {
        res.set(
            "application/octet-stream",
            fixture.views.lock().unwrap()[0].body.clone(),
        )?;
        return Ok(Flow::Continue);
    }
    let limits = StreamLimits {
        max_chunk_bytes: 4096,
        max_bytes: fixture.bytes.len().max(1),
        ..Default::default()
    };
    res.immutable_bytes(
        "application/octet-stream",
        Arc::clone(&fixture.bytes),
        limits,
    )?;
    match req.url.path() {
        "/views" => {
            let view = res.view();
            fixture.views.lock().unwrap().extend([view.clone(), view]);
        }
        "/replace" => {
            *res = Response::new();
            res.html("replacement")?;
        }
        "/error" => {
            return Err(Error::new("private failure after immutable output"));
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
        _ => {}
    }
    Ok(Flow::Continue)
}
/// Build only the value adapter: no IO, spawned producer or hang-class operation exists.
async fn setup(bytes: Vec<u8>) -> Result<(stackpress_server::Server, Arc<Fixture>)> {
    let fixture = Arc::new(Fixture {
        bytes: bytes.into(),
        views: Mutex::new(Vec::new()),
    });
    let shared = Arc::clone(&fixture);
    let mut builder = ServerBuilder::new();
    builder.on_config(task(move |ctx: &mut StartupContext| {
        let fixture = Arc::clone(&shared);
        Box::pin(async move {
            ctx.register(Arc::new(fixture))?;
            Ok(Flow::Continue)
        })
    }))?;
    builder.all("/:case", output)?;
    Ok((builder.build().await.map_err(|error| error.error)?, fixture))
}
/// Advance exactly one demand poll of an owned native body.
async fn next(body: &mut HttpBody) -> Option<Result<Vec<u8>>> {
    poll_fn(|cx| Pin::new(&mut *body).poll_next(cx)).await
}
/// Exact length and finite demand chunks require no whole-representation copy at installation.
#[tokio::test]
async fn known_length_and_one_shot_views() -> Result<()> {
    let bytes = vec![7; 10_001];
    let (app, fixture) = setup(bytes.clone()).await?;
    let adapter = StreamingHttpAdapter::new(HttpOptions::default())?;
    let input = http::Request::builder()
        .uri("/views")
        .body(())
        .unwrap()
        .into_parts()
        .0;
    let reply = adapter
        .handle_body(&app, input, Default::default(), || async { Ok(Vec::new()) })
        .await?;
    let StreamingHttpOutput::Response(mut response) = reply.output else {
        panic!("missing immutable output")
    };
    assert_eq!(response.headers()["content-length"], "10001");
    assert_eq!(Arc::strong_count(&fixture.bytes), 2);
    let mut delivered = Vec::new();
    while let Some(chunk) = next(response.body_mut()).await {
        let chunk = chunk?;
        assert!(chunk.len() <= 4096);
        delivered.extend(chunk);
    }
    assert_eq!(delivered, bytes);
    assert!(response.body().is_finished());
    let input = http::Request::builder()
        .uri("/reuse")
        .body(())
        .unwrap()
        .into_parts()
        .0;
    let reply = adapter
        .handle_body(&app, input, Default::default(), || async { Ok(Vec::new()) })
        .await?;
    let StreamingHttpOutput::Response(response) = reply.output else {
        panic!("missing fallback")
    };
    assert_eq!(response.status(), 500);
    Ok(())
}
/// Bodyless/HEAD lengths and error/response replacement never expose the old immutable bytes.
#[tokio::test]
async fn disposition_and_buffered_rejection() -> Result<()> {
    let (app, _) = setup(b"old-private-asset".to_vec()).await?;
    let adapter = StreamingHttpAdapter::new(HttpOptions::default())?;
    for (method, path, status, length) in [
        ("HEAD", "/head", 200, Some("17")),
        ("GET", "/204", 204, None),
        ("GET", "/205", 205, Some("0")),
        ("GET", "/304", 304, None),
        ("GET", "/replace", 200, Some("11")),
        ("GET", "/error", 500, None),
    ] {
        let input = http::Request::builder()
            .method(method)
            .uri(path)
            .body(())
            .unwrap()
            .into_parts()
            .0;
        let reply = adapter
            .handle_body(&app, input, Default::default(), || async { Ok(Vec::new()) })
            .await?;
        let StreamingHttpOutput::Response(mut response) = reply.output else {
            panic!("missing output")
        };
        assert_eq!(response.status(), status);
        if status != 500 {
            assert_eq!(
                response
                    .headers()
                    .get("content-length")
                    .and_then(|v| v.to_str().ok()),
                length
            );
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = next(response.body_mut()).await {
            bytes.extend(chunk?);
        }
        assert!(!String::from_utf8_lossy(&bytes).contains("old-private-asset"));
    }
    let reply = HttpAdapter::new(HttpOptions::default())?
        .handle(
            &app,
            http::Request::builder()
                .uri("/asset")
                .body(Vec::new())
                .unwrap(),
        )
        .await?;
    let HttpOutput::Response(response) = reply.output else {
        panic!("missing buffered fallback")
    };
    assert_eq!(response.status(), 500);
    Ok(())
}
/// Empty immutable representations finish without a poll, and invalid limits never attach bytes.
#[test]
fn immutable_policy_is_checked_before_installation() -> Result<()> {
    let mut response = Response::new();
    let bytes: Arc<[u8]> = Arc::from(b"abc".as_slice());
    assert!(
        response
            .immutable_bytes(
                "text/plain",
                Arc::clone(&bytes),
                StreamLimits {
                    max_bytes: 2,
                    ..Default::default()
                }
            )
            .is_err()
    );
    assert!(matches!(response.body(), Body::Empty));
    assert!(
        response
            .immutable_bytes(
                "text/plain",
                bytes,
                StreamLimits {
                    max_chunk_bytes: 0,
                    ..Default::default()
                }
            )
            .is_err()
    );
    response.stop()?;
    assert!(
        response
            .immutable_bytes("text/plain", Arc::from([]), StreamLimits::default())
            .is_err()
    );
    Ok(())
}
