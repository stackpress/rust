//! Generic renderer boundaries independent of Stackpress page contracts.
#![cfg(all(feature = "stream", not(target_arch = "wasm32")))]
use futures::StreamExt;
use leptos::prelude::*;
use stackpress_leptos::{
    async_render::AsyncRenderer,
    render::{RenderError, RenderLimits, Renderer, initialize_ssr},
    stream::{StreamLimits, StreamRenderer},
};
use std::{
    convert::Infallible,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};
#[path = "support/async_watchdog.rs"]
mod watchdog;

/// Measure producer lookahead through public work/framing callbacks, never queue internals.
#[test]
fn stream_backpressure_has_exact_capacity() {
    if std::env::var("S8_ASYNC_CHILD").as_deref() != Ok("backpressure") {
        watchdog::run("backpressure", "stream_backpressure_has_exact_capacity");
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        initialize_ssr().unwrap();
        // Repeat both exits to exercise scheduling without a timed negative assertion.
        for _ in 0..16 {
            for disconnect in [false, true] {
                let produced = Arc::new(AtomicUsize::new(0));
                let source_count = produced.clone();
                let observation_count = produced.clone();
                let marker = Arc::new(());
                let weak = Arc::downgrade(&marker);
                let (observe, observed) = tokio::sync::oneshot::channel();
                let mut body = StreamRenderer::render(
                    RenderLimits::default(),
                    StreamLimits::default(),
                    "failure".into(),
                    move |writer| async move {
                        on_cleanup(move || drop(marker));
                        let (meta, head) = leptos_meta::ServerMetaContext::new();
                        provide_context(meta);
                        leptos_meta::provide_meta_context();
                        // The caller owns Head construction; pinned upstream Title needs escaping.
                        let _ = view! {<leptos_meta::Title text="</title><script>hostile</script>"
                        formatter=|text: String| stackpress_leptos::safety::escape_html(&text)/>}
                        .to_html();
                        let chunks = futures::stream::iter(0..6).map(move |index| {
                            source_count.fetch_add(1, Ordering::SeqCst);
                            format!("<p>fragment-{index}</p>")
                        });
                        let mut observe = Some(observe);
                        writer
                            .render(
                                chunks,
                                head,
                                "<html><head><!--HEAD--></head><body>".into(),
                                move |content, replay| {
                                    if let Some(observe) = observe.take() {
                                        let counter = observation_count.clone();
                                        // On this current-thread LocalSet, the observation runs only
                                        // after the producer poll yields. All source/frame work is
                                        // immediately ready, so it fills the bridge and parks first.
                                        tokio::task::spawn_local(async move {
                                            tokio::task::yield_now().await;
                                            let _ = observe.send(counter.load(Ordering::SeqCst));
                                        });
                                    }
                                    Ok::<_, Infallible>(content + &replay)
                                },
                                "</body></html>".into(),
                            )
                            .await
                    },
                )
                .await
                .unwrap();
                assert_eq!(
                    produced.load(Ordering::SeqCst),
                    0,
                    "no production before demand"
                );
                let prefix = String::from_utf8(body.next().await.unwrap().unwrap()).unwrap();
                assert!(prefix.contains("&lt;/title&gt;&lt;script&gt;hostile&lt;/script&gt;"));
                assert!(!prefix.contains("<script>hostile"));
                // This observes exactly two source polls while the receiver consumes no slots.
                assert_eq!(observed.await.unwrap(), 2, "bridge lookahead changed");
                assert_eq!(produced.load(Ordering::SeqCst), 2);
                if !disconnect {
                    let mut html = String::new();
                    while let Some(chunk) = body.next().await {
                        html.push_str(&String::from_utf8(chunk.unwrap()).unwrap());
                    }
                    assert!(html.contains("fragment-5"));
                    assert_eq!(produced.load(Ordering::SeqCst), 6);
                    assert!(body.next().await.is_none());
                }
                // Drop while full must cancel/reap as surely as dropping a completed body.
                drop(body);
                assert!(weak.upgrade().is_none());
            }
        }
    });
    println!("S8_ASYNC_COMPLETE backpressure");
}

/// Compile-time evidence that only owned Send buffers cross the generic boundary.
fn send<T: Send>(_: &T) {}
/// Factories preserve typed errors and release root state across all three kernels.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn native_seams_preserve_errors_owner_and_send_boundaries() {
    initialize_ssr().unwrap();
    let cleaned = Arc::new(AtomicUsize::new(0));
    let capture = cleaned.clone();
    let staged = Renderer::try_prepare(
        RenderLimits::default(),
        move || {
            on_cleanup(move || {
                capture.fetch_add(1, Ordering::SeqCst);
            });
            Ok::<_, &str>(view! {<p>"safe <&雪"</p>})
        },
        |body| Ok(format!("<html><head></head><body>{body}</body></html>")),
    )
    .unwrap();
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
    send(&staged);
    let html = staged.finish().await.unwrap();
    assert!(html.contains("safe &lt;&amp;雪"));
    let error = Renderer::try_prepare::<AnyView, _>(RenderLimits::default(), || Err("factory"), Ok)
        .err()
        .unwrap();
    assert!(matches!(error, RenderError::Component("factory")));
    let owned = AsyncRenderer::try_prepare(
        RenderLimits::default(),
        || Ok::<_, Infallible>(view! {<p>"async"</p>}),
        || Ok(()),
        |body, replay| {
            Ok(format!(
                "<html><head></head><body>{body}{replay}</body></html>"
            ))
        },
    )
    .unwrap();
    send(&owned);
    assert!(
        tokio::spawn(owned)
            .await
            .unwrap()
            .unwrap()
            .contains("async")
    );
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let staging = StreamRenderer::render(
        RenderLimits::default(),
        StreamLimits::default(),
        "failure".into(),
        move |writer| async move {
            let retained = marker;
            on_cleanup(move || drop(retained));
            let (meta, head) = leptos_meta::ServerMetaContext::new();
            provide_context(meta);
            leptos_meta::provide_meta_context();
            let local = std::rc::Rc::new("local-view");
            let body = view! {<p>{local.to_string()}</p>}.to_html_stream_in_order();
            writer
                .render(
                    body,
                    head,
                    "<html><head></head><body>".into(),
                    |body, replay| Ok::<_, Infallible>(body + &replay),
                    "</body></html>".into(),
                )
                .await
        },
    );
    // The staging future is also Send even though its constructed worker view uses Rc.
    send(&staging);
    let mut body = staging.await.unwrap();
    send(&body);
    let mut bytes = Vec::new();
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk.unwrap());
    }
    drop(body);
    assert!(weak.upgrade().is_none());
    assert!(String::from_utf8(bytes).unwrap().contains("local-view"));
}
/// Cancelling before first demand still synchronously joins and cleans the local root.
#[tokio::test]
async fn stream_drop_before_demand_joins_local_owner() {
    initialize_ssr().unwrap();
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let body = StreamRenderer::render(
        RenderLimits::default(),
        StreamLimits::default(),
        "failure".into(),
        move |writer| async move {
            on_cleanup(move || drop(marker));
            let (meta, head) = leptos_meta::ServerMetaContext::new();
            provide_context(meta);
            writer
                .render(
                    futures::stream::iter(["body".into()]),
                    head,
                    "<html><head></head><body>".into(),
                    |a, b| Ok::<_, Infallible>(a + &b),
                    "</body></html>".into(),
                )
                .await
        },
    )
    .await
    .unwrap();
    drop(body);
    assert!(weak.upgrade().is_none());
}

/// A zero fragment budget rejects before admitting the worker, even with no failure bytes.
#[tokio::test]
async fn zero_chunk_budget_is_never_unlimited() {
    initialize_ssr().unwrap();
    let result = StreamRenderer::render(
        RenderLimits::default(),
        StreamLimits {
            max_chunk_bytes: 0,
            ..Default::default()
        },
        String::new(),
        |_| async { Ok::<_, RenderError<Infallible>>(()) },
    )
    .await;
    assert!(matches!(result, Err(RenderError::InvalidPolicy)));
}
