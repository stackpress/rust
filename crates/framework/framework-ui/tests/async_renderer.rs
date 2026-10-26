//! Real buffered renderer lifecycle/replay tests under an independent process watchdog.
#![cfg(feature = "ssr")]
use futures::FutureExt;
use leptos::prelude::*;
use stackpress_framework_ui::{
    async_render::AsyncLeptosRenderer, render::initialize_ssr, resources::cancel_on_cleanup,
};
use stackpress_framework_view::{
    AsyncPageRenderer, BasePath, PageAssets, PageContract, PageId, PageLimits, PageSnapshot,
    PublicLocation, PublicStatus, ViewError,
};
use std::sync::{
    Arc, Weak,
    atomic::{AtomicBool, Ordering},
};
#[path = "support/async_watchdog.rs"]
mod watchdog;

/// A real guarded loader owns its capture across a yielding pending await.
#[component]
fn Content(marker: Weak<()>, started: Arc<AtomicBool>, mode: &'static str) -> impl IntoView {
    let retained = marker.upgrade().unwrap();
    let _state = RwSignal::new(retained.clone());
    let resource = Resource::new(
        || (),
        move |()| {
            let retained = retained.clone();
            let started = started.clone();
            cancel_on_cleanup(async move {
                tokio::task::yield_now().await;
                started.store(true, Ordering::SeqCst);
                if mode == "loader-panic" {
                    panic!("private loader sentinel");
                }
                if mode == "cancel" {
                    std::future::pending::<()>().await;
                }
                drop(retained);
                if mode == "replay-limit" {
                    "x".repeat(1000)
                } else {
                    "</script><script>bad()</script>&\u{2028}".to_owned()
                }
            })
        },
    );
    let title = RwSignal::new("Reactive".to_owned());
    view! {
        <stackpress_framework_ui::leptos_meta::Title text=move||{if mode=="head-panic" {panic!("trusted Head fixture");}title.get()}/>
        <Suspense fallback=||"pending">{move||resource.get().map(move |value| {
                if mode == "loader-panic" {
                    let error = value.as_ref().unwrap_err();
                    assert!(std::error::Error::source(error).unwrap().to_string().contains("private loader sentinel"));
                }
                view! { <p>{value.unwrap_or_else(|error| error.to_string())}</p> }
            })}</Suspense>
    }
}

/// Minimal explicit public seed; no request or developer data enters replay.
fn seed() -> PageSnapshot<()> {
    PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("test".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en".into(),
        theme: "light".into(),
        status: PublicStatus::ok(),
        data: (),
    }
}

/// Exercise all exits in a single isolated, externally bounded executor.
fn matrix() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    runtime.block_on(run_matrix());
    drop(runtime);
    println!("S8_ASYNC_COMPLETE renderer-matrix");
}

/// Alternate independent render roots while preserving the outer reactive context.
async fn run_matrix() {
    initialize_ssr().unwrap();
    let caller = Owner::new();
    caller.set();
    provide_context("outer");
    for mode in [
        "success",
        "cancel",
        "factory-error",
        "factory-panic",
        "head-panic",
        "loader-panic",
        "html-limit",
        "replay-limit",
    ] {
        check_case(mode, &caller).await;
    }
    caller.unset_with_forced_cleanup();
}

/// Admit real work, finish/drop/unwind the renderer, then check capture/context release.
async fn check_case(mode: &'static str, caller: &Owner) {
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let capture = weak.clone();
    let started = Arc::new(AtomicBool::new(false));
    let started_capture = started.clone();
    let mut contract = PageContract::new(1, "test").unwrap();
    if mode == "html-limit" {
        contract.limits = PageLimits {
            html_bytes: 1,
            ..PageLimits::default()
        };
    }
    if mode == "replay-limit" {
        contract.limits = PageLimits {
            snapshot_bytes: 500,
            ..PageLimits::default()
        };
    }
    let assets = PageAssets {
        build: "test".into(),
        base: BasePath::default(),
        module: PublicLocation::new("/start.js").unwrap(),
        stylesheet: Some(PublicLocation::new("/style.css").unwrap()),
    };
    let renderer = AsyncLeptosRenderer::new(contract, assets, move |_: &PageSnapshot<()>| {
        // Arena state also has to drop when the synchronous factory fails.
        let _marker = RwSignal::new(capture.upgrade().unwrap());
        if mode == "factory-panic" {
            panic!("trusted factory fixture");
        }
        if mode == "factory-error" {
            return Err(ViewError::InvalidSnapshot);
        }
        Ok(
            view! { <Content marker=capture.clone() started=started_capture.clone() mode/> }
                .into_any(),
        )
    })
    .unwrap();
    if mode == "factory-panic" {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            renderer.render_async(seed())
        }));
        assert!(result.is_err());
        drop(marker);
    } else {
        let mut future = renderer.render_async(seed());
        drop(marker);
        assert_eq!(Owner::current(), Some(caller.clone()));
        if mode == "cancel" {
            // Eager evaluation spawned the loader; wait for actual admission before disposal.
            assert!(future.as_mut().now_or_never().is_none());
            for _ in 0..16 {
                tokio::task::yield_now().await;
            }
            assert!(started.load(Ordering::SeqCst));
            drop(future);
        } else {
            check_result(mode, future.await);
        }
    }
    // Idle task loops as well as aborted loaders get a bounded scheduler drain.
    for _ in 0..16 {
        tokio::task::yield_now().await;
    }
    assert!(
        weak.upgrade().is_none(),
        "{mode} retained render/loader capture"
    );
    assert_eq!(Owner::current(), Some(caller.clone()), "{mode}");
    assert_eq!(use_context::<&'static str>(), Some("outer"));
}

/// Check public outcomes and exact inert replay, keeping private causes off the wire.
fn check_result(mode: &str, result: Result<stackpress_framework_view::RenderedPage, ViewError>) {
    match mode {
        "factory-error" => assert_eq!(result.unwrap_err(), ViewError::InvalidSnapshot),
        "head-panic" => {
            let error = result.unwrap_err();
            assert_eq!(error.to_string(), "Could not render page");
            assert!(
                std::error::Error::source(&error)
                    .unwrap()
                    .to_string()
                    .contains("trusted Head fixture")
            );
        }
        "html-limit" => assert_eq!(result.unwrap_err(), ViewError::HtmlLimit),
        "replay-limit" => assert_eq!(result.unwrap_err(), ViewError::SnapshotLimit),
        _ => {
            let html = result.unwrap().into_html();
            assert!(html.contains("href=\"/style.css\""));
            assert!(html.contains("<title>Reactive</title>"));
            assert!(!html.contains("private loader sentinel"));
            let replay = html
                .split("id=\"stackpress-resources\">")
                .nth(1)
                .unwrap()
                .split("</script>")
                .next()
                .unwrap();
            assert!(!replay.contains('<'));
            let replay: Vec<(usize, String)> = serde_json::from_str(replay).unwrap();
            let data: Result<String, stackpress_framework_ui::resources::ResourceFailure> =
                serde_json::from_str(&replay[0].1).unwrap();
            if mode == "loader-panic" {
                assert_eq!(
                    data.unwrap_err().kind(),
                    stackpress_framework_ui::resources::ResourceFailureKind::Panicked
                );
            } else {
                assert_eq!(data.unwrap(), "</script><script>bad()</script>&\u{2028}");
            }
        }
    }
}

/// The parent watchdog bounds every hang-class renderer scenario and requires child completion.
#[test]
fn owned_async_renderer_exit_matrix() {
    if std::env::var("S8_ASYNC_CHILD").as_deref() == Ok("renderer-matrix") {
        matrix();
    } else {
        watchdog::run("renderer-matrix", "owned_async_renderer_exit_matrix");
    }
}

/// A Send render can migrate threads while each poll restores that thread's caller.
#[test]
fn owned_render_migrates_between_threads() {
    if std::env::var("S8_ASYNC_CHILD").as_deref() != Ok("renderer-migration") {
        watchdog::run(
            "renderer-migration",
            "owned_render_migrates_between_threads",
        );
        return;
    }
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    runtime.block_on(async {
        initialize_ssr().unwrap();
        let first=Owner::new();first.set();provide_context("first caller");
        let marker=Arc::new(());let weak=Arc::downgrade(&marker);let capture=weak.clone();
        let renderer=AsyncLeptosRenderer::new(PageContract::new(1,"test").unwrap(),PageAssets{build:"test".into(),base:BasePath::default(),module:PublicLocation::new("/start.js").unwrap(),stylesheet:None},move|_:&PageSnapshot<()>|{
            let _state=RwSignal::new(capture.upgrade().unwrap());
            let resource=Resource::new(||(),|()|cancel_on_cleanup(async{tokio::task::yield_now().await;"migrated".to_owned()}));
            Ok(view!{<Suspense fallback=||"pending">{move||resource.get().map(|value|view!{<p>{value.unwrap()}</p>})}</Suspense>})
        }).unwrap();
        let mut future=renderer.render_async(seed());drop(marker);
        assert!(future.as_mut().now_or_never().is_none());
        assert_eq!(Owner::current(),Some(first.clone()));
        let handle=runtime.handle().clone();
        std::thread::scope(|scope|{
            scope.spawn(move||{
                let _entered=handle.enter();
                let second=Owner::new();second.set();provide_context("second caller");
                let html=futures::executor::block_on(future).unwrap().into_html();
                assert!(html.contains("migrated"));
                assert_eq!(Owner::current(),Some(second.clone()));
                assert_eq!(use_context::<&'static str>(),Some("second caller"));
                second.unset_with_forced_cleanup();
            }).join().unwrap();
        });
        assert!(weak.upgrade().is_none());
        assert_eq!(use_context::<&'static str>(),Some("first caller"));
        first.unset_with_forced_cleanup();
    });
    drop(runtime);
    println!("S8_ASYNC_COMPLETE renderer-migration");
}
