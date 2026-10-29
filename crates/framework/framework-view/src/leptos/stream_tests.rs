//! Hang-class regression matrix in a child with an independent kill/reap deadline.
use super::*;
use crate::{BasePath, PageId, PageLimits, PublicLocation, PublicStatus};
use futures::{FutureExt, StreamExt};
use stackpress_leptos::resources::cancel_on_cleanup;
use std::{
    sync::atomic::{AtomicUsize, Ordering},
    time::{Duration, Instant},
};

/// Keep watchdog child diagnostics under the same configurable proof artifact directory.
fn proof_output(name: &str) -> std::path::PathBuf {
    let directory = std::env::var_os("UI_STREAM_OUTPUT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/private/tmp/s8-stream".into());
    std::fs::create_dir_all(&directory).unwrap();
    directory.join(name)
}

/// Owned public seed used by both successful and failed renders.
fn seed() -> PageSnapshot<()> {
    PageSnapshot {
        revision: 1,
        build: "stream-test".into(),
        page: PageId::try_from("stream-test".to_owned()).unwrap(),
        location: "/demo?q=1&x=2".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: PublicStatus::ok(),
        data: (),
    }
}
/// Deterministic assets never create a listener or a live browser dependency.
fn assets() -> PageAssets {
    PageAssets {
        theme_bootstrap: None,
        build: "stream-test".into(),
        base: BasePath::default(),
        module: PublicLocation::new("/start.js").unwrap(),
        stylesheet: Some(PublicLocation::new("/style.css").unwrap()),
    }
}

/// A real Resource retains one capture until its owning runtime is fully reaped.
#[component]
fn Section(
    index: usize,
    gate: Arc<tokio::sync::Semaphore>,
    marker: Arc<()>,
    renders: Arc<AtomicUsize>,
    mode: &'static str,
) -> impl IntoView {
    let resource = Resource::new(
        || (),
        move |()| {
            let gate = gate.clone();
            let marker = marker.clone();
            cancel_on_cleanup(async move {
                gate.acquire().await.unwrap().forget();
                drop(marker);
                if mode == "replay-limit" {
                    "x".repeat(1000)
                } else {
                    format!("section-{index}: </script><script>bad()</script>&\u{2028}\u{2029}")
                }
            })
        },
    );
    view! { <Suspense fallback=||"waiting">{move || resource.get().map(|value| {
        renders.fetch_add(1, Ordering::SeqCst);
        if mode == "late-panic" { panic!("private streamed panic sentinel"); }
        let text = if mode == "chunk-limit" { "x".repeat(3000) } else if mode == "total-limit" { "x".repeat(2000) } else { value.unwrap() };
        view! { <p>{text}</p> }
    })}</Suspense> }
}

/// Different exits share the real renderer, not a fake chunk producer.
async fn case(mode: &'static str) {
    eprintln!("CASE {mode}");
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let renders = Arc::new(AtomicUsize::new(0));
    let mut contract = PageContract::new(1, "stream-test").unwrap();
    if mode == "replay-limit" {
        contract.limits.snapshot_bytes = 400;
    }
    if mode == "total-limit" {
        contract.limits.html_bytes = 2500;
    }
    let policy = StreamLimits {
        render_timeout: if mode == "late-panic" {
            Duration::from_millis(100)
        } else {
            Duration::from_secs(3)
        },
        max_chunk_bytes: if mode == "chunk-limit" {
            1500
        } else {
            contract.limits.html_bytes.min(64 * 1024)
        },
    };
    let captures = (marker.clone(), gate.clone(), renders.clone());
    let renderer = StreamLeptosRenderer::new(contract, assets(), policy, move |_: &PageSnapshot<()>| {
        if mode == "factory-error" { return Err(ViewError::InvalidSnapshot); }
        if mode == "factory-panic" { panic!("private factory sentinel"); }
        let title = if mode == "head-panic" { panic!("private Head sentinel"); } else { "Frozen </title>&" };
        Ok(view! {
            <leptos_meta::Title text=title/>
            <h1>"Useful shell"</h1>
            <Section index=0 gate=captures.1.clone() marker=captures.0.clone() renders=captures.2.clone() mode/>
            {(mode == "body-panic").then(|| Suspend::new(async {
                tokio::task::yield_now().await;
                panic!("private body-poll sentinel");
                #[allow(unreachable_code)] ()
            }))}
        })
    }).unwrap();
    let result = renderer.render(seed()).await;
    drop(renderer);
    drop(marker);
    if matches!(mode, "factory-error" | "factory-panic" | "head-panic") {
        assert!(result.is_err());
    } else {
        let mut page = result.unwrap();
        assert_eq!(page.status, 200);
        // Staging cannot render beyond the component's synchronous shell.
        assert_eq!(renders.load(Ordering::SeqCst), 0);
        let prefix = String::from_utf8(page.body.next().await.unwrap().unwrap()).unwrap();
        assert!(prefix.contains("Frozen &lt;/title&gt;&amp;"));
        assert!(prefix.contains("/style.css"));
        assert!(prefix.contains("stackpress-stream-recovery"));
        let shell = String::from_utf8(page.body.next().await.unwrap().unwrap()).unwrap();
        assert!(shell.contains("Useful shell"));
        assert!(!shell.contains("section-0"));
        if mode != "disconnect" {
            gate.add_permits(6);
        }
        if mode == "disconnect" {
            drop(page);
        } else {
            let settled_at = Instant::now();
            let mut text = String::new();
            let mut failed = false;
            while let Some(chunk) = page.body.next().await {
                match chunk {
                    Ok(chunk) => text.push_str(&String::from_utf8(chunk).unwrap()),
                    Err(error) => {
                        failed = true;
                        assert!(!error.to_string().contains("sentinel"));
                        if mode == "late-panic" {
                            assert!(settled_at.elapsed() < Duration::from_millis(600));
                            let cause = std::error::Error::source(&error).unwrap();
                            assert_eq!(
                                cause.downcast_ref::<std::io::Error>().unwrap().kind(),
                                std::io::ErrorKind::TimedOut
                            );
                        }
                        if mode == "body-panic" {
                            assert!(settled_at.elapsed() < Duration::from_millis(600));
                            assert!(
                                std::error::Error::source(&error)
                                    .unwrap()
                                    .to_string()
                                    .contains("private body-poll sentinel")
                            );
                        }
                    }
                }
            }
            assert_eq!(failed, mode != "success", "{mode}: {text}");
            if failed {
                assert!(text.contains("stackpress-stream-failed"));
                assert!(!text.contains("stackpress-stream-complete"));
            } else {
                assert!(text.contains("stackpress-stream-complete"));
                assert!(text.contains("data-stackpress-replay"));
                assert!(!text.contains("<script>bad()"));
                assert!(text.contains("\\u003c"));
            }
            drop(page);
        }
    }
    eprintln!("DONE {mode}");
    assert!(
        weak.upgrade().is_none(),
        "render/loader capture leaked after {mode}"
    );
}

/// Queue capacity must stop render polling before another section is evaluated.
async fn backpressure() {
    eprintln!("CASE backpressure");
    let gate = Arc::new(tokio::sync::Semaphore::new(0));
    let renders = Arc::new(AtomicUsize::new(0));
    let capture = renders.clone();
    let loading = gate.clone();
    let renderer = StreamLeptosRenderer::new(
        PageContract::new(1, "stream-test").unwrap(),
        assets(),
        StreamLimits::default(),
        move |_: &PageSnapshot<()>| {
            Ok(view! { <h1>"shell"</h1>{(0..6).map(|index| {
                let loading = loading.clone();
                let capture = capture.clone();
                Suspend::new(async move {
                    loading.acquire().await.unwrap().forget();
                    capture.fetch_add(1, Ordering::SeqCst);
                    view! { <p>{format!("fragment-{index}")}</p> }
                })
            }).collect_view()} })
        },
    )
    .unwrap();
    let mut page = renderer.render(seed()).await.unwrap();
    page.body.next().await.unwrap().unwrap();
    gate.add_permits(6);
    let deadline = Instant::now() + Duration::from_secs(2);
    while renders.load(Ordering::SeqCst) < 1 {
        assert!(Instant::now() < deadline);
        tokio::task::yield_now().await;
    }
    let before = renders.load(Ordering::SeqCst);
    // The independent worker gets ample scheduler time while the consumer is idle.
    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(renders.load(Ordering::SeqCst), before);
    assert!(
        before < 6,
        "producer rendered every section despite a full bridge"
    );
    while let Some(chunk) = page.body.next().await {
        chunk.unwrap();
    }
    assert_eq!(renders.load(Ordering::SeqCst), 6);
}

/// A later Resource may resolve first without overtaking an earlier Suspense.
async fn ordered_resources() {
    let first = Arc::new(tokio::sync::Semaphore::new(0));
    let second = Arc::new(tokio::sync::Semaphore::new(0));
    let rendered = Arc::new(AtomicUsize::new(0));
    let captures = (first.clone(), second.clone(), rendered.clone());
    let renderer = StreamLeptosRenderer::new(PageContract::new(1, "stream-test").unwrap(), assets(), StreamLimits::default(), move |_: &PageSnapshot<()>| {
        Ok(view! {
            <h1>"shell"</h1>
            <Section index=0 gate=captures.0.clone() marker=Arc::new(()) renders=captures.2.clone() mode="success"/>
            <Section index=1 gate=captures.1.clone() marker=Arc::new(()) renders=captures.2.clone() mode="success"/>
        })
    }).unwrap();
    let mut page = renderer.render(seed()).await.unwrap();
    page.body.next().await.unwrap().unwrap();
    page.body.next().await.unwrap().unwrap();
    second.add_permits(1);
    while rendered.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    assert!(
        page.body.next().now_or_never().is_none(),
        "later section overtook the pending first section"
    );
    first.add_permits(1);
    let mut output = String::new();
    while let Some(chunk) = page.body.next().await {
        output.push_str(&String::from_utf8(chunk.unwrap()).unwrap());
    }
    assert!(output.find("<p>section-0:").unwrap() < output.find("<p>section-1:").unwrap());
}

/// Cancelling the staging future owns and joins even a blocking Resource.
async fn precommit_cancellation() {
    let capture = Arc::new(());
    let weak = Arc::downgrade(&capture);
    let entered = Arc::new(AtomicUsize::new(0));
    let signal = entered.clone();
    let renderer = StreamLeptosRenderer::new(
        PageContract::new(1, "stream-test").unwrap(),
        assets(),
        StreamLimits::default(),
        move |_: &PageSnapshot<()>| {
            let capture = capture.clone();
            signal.store(1, Ordering::SeqCst);
            let _resource = Resource::new_blocking(
                || (),
                move |()| {
                    let capture = capture.clone();
                    cancel_on_cleanup(async move {
                        let _capture = capture;
                        std::future::pending::<String>().await
                    })
                },
            );
            Ok(view! { <h1>"blocked"</h1> })
        },
    )
    .unwrap();
    let mut staging = Box::pin(renderer.render(seed()));
    assert!(staging.as_mut().now_or_never().is_none());
    while entered.load(Ordering::SeqCst) == 0 {
        tokio::task::yield_now().await;
    }
    drop(staging);
    drop(renderer);
    assert!(
        weak.upgrade().is_none(),
        "precommit cancellation left a loader/root alive"
    );
}

/// Public seed and opening failures must happen before any stream is returned.
async fn precommit_validation() {
    let mut contract = PageContract::new(1, "stream-test").unwrap();
    contract.limits.html_bytes = 600;
    let renderer = StreamLeptosRenderer::new(
        contract,
        assets(),
        StreamLimits {
            max_chunk_bytes: 600,
            ..Default::default()
        },
        |_: &PageSnapshot<()>| Ok(()),
    )
    .unwrap();
    assert!(matches!(
        renderer.render(seed()).await,
        Err(ViewError::HtmlLimit)
    ));
    let mut wrong = seed();
    wrong.locale = "fr".into();
    assert!(matches!(
        renderer.render(wrong).await,
        Err(ViewError::ContractMismatch)
    ));
    let mut wrong = seed();
    wrong.build = "other".into();
    assert!(matches!(
        renderer.render(wrong).await,
        Err(ViewError::ContractMismatch)
    ));
}

/// Lowered byte policies reject output at equality+1 without arithmetic wrap.
#[test]
fn budget_and_replay_contract() {
    let mut contract = PageContract::new(1, "stream-test").unwrap();
    contract.limits = PageLimits {
        html_bytes: 0,
        ..PageLimits::default()
    };
    assert!(
        StreamLeptosRenderer::new(
            contract,
            assets(),
            StreamLimits::default(),
            |_: &PageSnapshot<()>| Ok(())
        )
        .is_err()
    );
    for deadline in [Duration::ZERO, Duration::MAX] {
        assert!(
            StreamLeptosRenderer::new(
                PageContract::new(1, "stream-test").unwrap(),
                assets(),
                StreamLimits {
                    render_timeout: deadline,
                    ..Default::default()
                },
                |_: &PageSnapshot<()>| Ok(())
            )
            .is_err()
        );
    }
}

/// Run all hang-capable cases only in the externally watched child.
#[test]
#[ignore = "called by stream watchdog"]
fn child() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(async {
        crate::leptos::render::initialize_ssr().unwrap();
        for mode in [
            "success",
            "disconnect",
            "factory-error",
            "factory-panic",
            "head-panic",
            "late-panic",
            "body-panic",
            "chunk-limit",
            "total-limit",
            "replay-limit",
        ] {
            case(mode).await;
        }
        backpressure().await;
        ordered_resources().await;
        precommit_cancellation().await;
        precommit_validation().await;
        status_asset().await;
    });
    println!("S8_STREAM_COMPLETE");
}

/// Optional status assets stay mounted, escaped, external and opt-in.
async fn status_asset() {
    for enabled in [false, true] {
        let mut mounted = assets();
        mounted.base = BasePath::new("/demo").unwrap();
        let renderer = StreamLeptosRenderer::new(
            PageContract::new(1, "stream-test").unwrap(),
            mounted,
            StreamLimits::default(),
            |_: &PageSnapshot<()>| Ok(view! {<p>"Settled"</p>}),
        )
        .unwrap();
        let renderer = if enabled {
            renderer
                .with_status_script(PublicLocation::new("/status.js?x=1&y=2").unwrap())
                .unwrap()
        } else {
            renderer
        };
        let mut page = renderer.render(seed()).await.unwrap();
        let mut html = String::new();
        while let Some(chunk) = page.body.next().await {
            html.push_str(&String::from_utf8(chunk.unwrap()).unwrap());
        }
        assert_eq!(
            html.contains("<script async src=\"/demo/status.js?x=1&amp;y=2\"></script>"),
            enabled
        );
        assert!(
            !html.contains("MutationObserver"),
            "status code must remain external"
        );
        assert!(html.contains("stackpress-stream-complete"));
    }
}

/// Parent termination is independent of the renderer/executor being tested.
#[test]
fn watchdog() {
    use std::process::{Child, Command, Stdio};
    struct Reap(Child);
    impl Drop for Reap {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let path = proof_output(&format!("unit-child-{}.log", std::process::id()));
    let log = std::fs::File::create(&path).unwrap();
    let mut child = Reap(
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "leptos::stream::tests::child",
                "--ignored",
                "--nocapture",
            ])
            .stdout(Stdio::from(log.try_clone().unwrap()))
            .stderr(Stdio::from(log))
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            let output = std::fs::read_to_string(&path).unwrap();
            assert!(status.success(), "{output}");
            assert!(
                output.lines().any(|line| line == "S8_STREAM_COMPLETE"),
                "{output}"
            );
            break;
        }
        assert!(
            Instant::now() < deadline,
            "stream child timed out: {path:?}"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
