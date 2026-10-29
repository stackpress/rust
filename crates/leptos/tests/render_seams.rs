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
fn send<T: Send>(_: &T) {}
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
    let mut body = StreamRenderer::render(
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
    )
    .await
    .unwrap();
    send(&body);
    let mut bytes = Vec::new();
    while let Some(chunk) = body.next().await {
        bytes.extend(chunk.unwrap());
    }
    drop(body);
    assert!(weak.upgrade().is_none());
    assert!(String::from_utf8(bytes).unwrap().contains("local-view"));
}
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
