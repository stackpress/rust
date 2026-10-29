//! Independent public-API proofs, externally bounded so deadlocks fail and reap.
#![cfg(feature = "ssr")]
#[cfg(feature = "stream")]
use futures::StreamExt;
use leptos::prelude::*;
use leptos_independent::Initial;
#[cfg(feature = "stream")]
use stackpress_leptos::leptos_meta::ServerMetaContext;
#[cfg(feature = "async")]
use stackpress_leptos::leptos_meta::Title;
use stackpress_leptos::render::{RenderError, RenderLimits, Renderer, initialize_ssr};
#[cfg(any(feature = "async", feature = "stream"))]
use std::convert::Infallible;
#[cfg(feature = "async")]
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

/// Execute the potentially blocking lifecycle matrix with an exact completion marker.
#[test]
fn independent_lifecycle_watchdog() {
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--ignored", "--exact", "lifecycle_child", "--nocapture"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(20);
    loop {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("independent lifecycle child timed out and was reaped");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .any(|line| line == "INDEPENDENT_LIFECYCLE_COMPLETE")
    );
}

/// The parent is the only caller; direct harness invocation keeps this child ignored.
#[test]
#[ignore = "executed by independent_lifecycle_watchdog"]
fn lifecycle_child() {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(matrix());
    println!("INDEPENDENT_LIFECYCLE_COMPLETE");
}

/// Prove concrete documents and independent root values before testing exceptional exits.
async fn matrix() {
    initialize_ssr().unwrap();
    let light = Initial::new("light", "en", "<draft>&雪".into()).unwrap();
    let dark = Initial::new("dark", "fr", "Second root".into()).unwrap();
    let first = leptos_independent::native::render(light, false, RenderLimits::default())
        .await
        .unwrap();
    let second = leptos_independent::native::render(dark, false, RenderLimits::default())
        .await
        .unwrap();
    assert!(first.contains("Welcome") && first.contains("data-theme=\"light\""));
    assert!(second.contains("Bienvenue") && second.contains("data-theme=\"dark\""));
    assert!(!first.contains("Bienvenue") && !second.contains("&lt;draft&gt;"));
    assert!(first.contains("&lt;draft&gt;&amp;雪"));
    assert!(first.contains("<title>Independent &lt; &amp; &gt;</title>"));
    let limited = leptos_independent::native::render(
        Initial::new("light", "en", String::new()).unwrap(),
        false,
        RenderLimits {
            html_bytes: 64,
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(limited, Err(RenderError::HtmlLimit)));
    synchronous_cleanup().await;
    #[cfg(feature = "async")]
    for mode in ["success", "cancel", "panic", "head-panic"] {
        buffered_cleanup(mode).await;
    }
    #[cfg(feature = "stream")]
    streaming_cleanup().await;
}

/// Success, typed error and trusted unwind all release captured state and restore the caller.
async fn synchronous_cleanup() {
    let caller = Owner::new();
    caller.set();
    provide_context("caller");
    for exit in ["success", "error", "panic"] {
        let marker = Arc::new(());
        let weak = Arc::downgrade(&marker);
        let staged = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Renderer::try_prepare(
                RenderLimits::default(),
                move || {
                    on_cleanup(move || drop(marker));
                    if exit == "panic" {
                        panic!("trusted fixture factory");
                    }
                    if exit == "error" {
                        return Err("factory rejected");
                    }
                    Ok(view! {<p>"owned body"</p>})
                },
                |body| Ok(format!("<html><head></head><body>{body}</body></html>")),
            )
        }));
        match exit {
            "success" => {
                assert!(
                    staged
                        .unwrap()
                        .unwrap()
                        .finish()
                        .await
                        .unwrap()
                        .contains("owned body")
                );
            }
            "error" => assert!(matches!(
                staged.unwrap(),
                Err(RenderError::Component("factory rejected"))
            )),
            _ => assert!(staged.is_err()),
        }
        assert!(weak.upgrade().is_none(), "cleanup {exit}");
        assert_eq!(use_context::<&'static str>(), Some("caller"));
    }
    caller.unset_with_forced_cleanup();
}

/// Real guarded loaders expose completion, cancellation and native-only panic diagnostics.
#[cfg(feature = "async")]
#[component]
fn Guarded(marker: Arc<()>, started: Arc<AtomicBool>, mode: &'static str) -> impl IntoView {
    let resource = Resource::new(
        || (),
        move |_| {
            let retained = marker.clone();
            let started = started.clone();
            stackpress_leptos::resources::cancel_on_cleanup(async move {
                started.store(true, Ordering::SeqCst);
                tokio::task::yield_now().await;
                if mode == "cancel" {
                    std::future::pending::<()>().await;
                }
                if mode == "panic" {
                    panic!("private independent loader sentinel");
                }
                drop(retained);
                "Ready".to_owned()
            })
        },
    );
    view! {
        <Title text=move || {
            if mode == "head-panic" { panic!("trusted independent Head sentinel"); }
            resource.get().map(|_| "Resolved title").unwrap_or("Pending title")
        }/>
        <Suspense fallback=|| "waiting">{move || resource.get().map(|result| result.unwrap_or_else(|e| e.to_string()))}</Suspense>
    }
}

/// The future may migrate threads, but dropping it still releases its root and loader.
#[cfg(feature = "async")]
async fn buffered_cleanup(mode: &'static str) {
    use stackpress_leptos::async_render::AsyncRenderer;
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let started = Arc::new(AtomicBool::new(false));
    let observed = started.clone();
    let mut render = AsyncRenderer::try_prepare(RenderLimits::default(), move || {
        Ok::<_, Infallible>(view! {<Guarded marker started mode/>})
    }, || Ok(()), |body, replay| Ok(format!("<html><head></head><body>{body}<script type=\"application/json\">{replay}</script></body></html>"))).unwrap();
    if mode == "cancel" {
        // A short timeout polls the owned future, then leaves it available to drop.
        assert!(
            tokio::time::timeout(Duration::from_millis(10), &mut render)
                .await
                .is_err()
        );
        assert!(observed.load(Ordering::SeqCst));
        drop(render);
    } else {
        let result = tokio::spawn(render).await.unwrap();
        if mode == "head-panic" {
            assert!(matches!(result, Err(RenderError::Failure(_))));
        } else {
            let html = result.unwrap();
            assert!(html.contains("Resolved title"));
            assert!(html.contains(if mode == "panic" {
                "Resource failed"
            } else {
                "Ready"
            }));
            assert!(!html.contains("private independent loader sentinel"));
        }
    }
    for _ in 0..100 {
        if weak.upgrade().is_none() {
            break;
        }
        tokio::task::yield_now().await;
    }
    assert!(weak.upgrade().is_none(), "buffered cleanup {mode}");
}

/// Generic streams support local views, slow demand, drop, deadline and framed-byte limits.
#[cfg(feature = "stream")]
async fn streaming_cleanup() {
    use stackpress_leptos::stream::{StreamLimits, StreamRenderer};
    // Every case owns one worker and uses only generic framing, never page/status types.
    for mode in ["slow", "drop", "deadline", "limit"] {
        let marker = Arc::new(());
        let weak = Arc::downgrade(&marker);
        let body = StreamRenderer::render(
            RenderLimits {
                html_bytes: if mode == "limit" { 180 } else { 4096 },
                replay_bytes: 1024,
            },
            StreamLimits {
                max_chunk_bytes: 128,
                render_timeout: Duration::from_millis(100),
            },
            "<p>Fixture stream failed</p>".into(),
            move |writer| async move {
                on_cleanup(move || drop(marker));
                let (meta, head) = ServerMetaContext::new();
                provide_context(meta);
                let local = std::rc::Rc::new("local");
                let stream = futures::stream::unfold((0, local), move |(n, local)| async move {
                    if mode == "deadline" {
                        std::future::pending::<()>().await;
                    }
                    (n < 6).then(|| {
                        (
                            format!("<p>{local}:{n} {}</p>", "x".repeat(25)),
                            (n + 1, local),
                        )
                    })
                });
                writer
                    .render(
                        stream,
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
        if mode == "drop" {
            drop(body);
        } else {
            let mut body = Box::pin(body);
            let mut bytes = Vec::new();
            let mut failed = false;
            assert!(body.next().await.unwrap().is_ok());
            if mode == "slow" {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            while let Some(chunk) = body.next().await {
                match chunk {
                    Ok(chunk) => bytes.extend(chunk),
                    Err(_) => failed = true,
                }
            }
            assert_eq!(failed, mode != "slow");
            let text = String::from_utf8(bytes).unwrap();
            assert!(text.contains(if mode == "slow" {
                "local:5"
            } else {
                "Fixture stream failed"
            }));
            assert!(body.next().await.is_none());
            drop(body);
        }
        assert!(weak.upgrade().is_none(), "stream cleanup {mode}");
    }
}
