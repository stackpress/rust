//! Rendering cannot retain the reactive Owner after staging or unwinding.
#![cfg(feature = "ssr")]

use leptos::prelude::*;
use stackpress_framework_ui::render::{LeptosRenderer, initialize_ssr};
use stackpress_framework_view::{
    BasePath, PageAssets, PageContract, PageId, PageLimits, PageRenderer, PageSnapshot,
    PublicLocation, ViewError,
};
use std::sync::Arc;

/// Construct explicit safe metadata for ownership tests.
fn seed() -> PageSnapshot<()> {
    PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("test".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: stackpress_framework_view::PublicStatus::ok(),
        data: (),
    }
}

/// Owner cleanup releases captured state before owned Head assembly is awaited.
#[tokio::test]
async fn staged_render_releases_owner_state() {
    initialize_ssr().unwrap();
    let marker = Arc::new(());
    let weak = Arc::downgrade(&marker);
    let staged = LeptosRenderer::prepare(&seed(), move || {
        let _state = RwSignal::new(marker);
        view! { <p>"owned"</p> }
    })
    .unwrap();
    assert!(weak.upgrade().is_none());
    staged.finish().await.unwrap();
}

/// Deferred reactive Head fails safely; independent eager Head still renders afterward.
#[tokio::test]
async fn reactive_title_survives_owner_cleanup() {
    initialize_ssr().unwrap();
    let retained = Arc::new(());
    let weak = Arc::downgrade(&retained);
    let staged = LeptosRenderer::prepare(&seed(), move || {
        let title = RwSignal::new("Reactive title".to_owned());
        view! { <stackpress_framework_ui::leptos_meta::Title text=move || {
            assert_eq!(Arc::strong_count(&retained), 1);
            title.get()
        }/> }
    })
    .unwrap();
    assert!(weak.upgrade().is_some());
    let error = staged.finish().await.unwrap_err();
    assert!(
        weak.upgrade().is_none(),
        "failed Head assembly must drop captured state"
    );
    assert!(matches!(error, ViewError::Failure(_)));
    assert_eq!(error.to_string(), "Could not render page");
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .to_string()
            .contains("eager owned")
    );
    let html = LeptosRenderer::prepare(&seed(), || {
        let title = RwSignal::new("Owned title".to_owned());
        view! { <stackpress_framework_ui::leptos_meta::Title text=title.get()/> }
    })
    .unwrap()
    .finish()
    .await
    .unwrap()
    .into_html();
    assert!(html.contains("<title>Owned title</title>"));
}

/// Errors and discarded metadata futures release signals and preserve the outer Owner.
#[tokio::test]
async fn every_render_exit_releases_reactive_state() {
    initialize_ssr().unwrap();
    let outer = Owner::new();
    outer.set();
    for mode in [
        "component-error",
        "html-limit",
        "discard",
        "cancel",
        "success",
    ] {
        let marker = Arc::new(());
        let weak = Arc::downgrade(&marker);
        let capture = weak.clone();
        let mut contract = PageContract::new(1, "test").unwrap();
        if mode == "html-limit" {
            contract.limits = PageLimits {
                html_bytes: 1,
                ..PageLimits::default()
            };
        }
        let renderer = LeptosRenderer::new(
            contract,
            PageAssets {
                theme_bootstrap: None,
                build: "test".into(),
                base: BasePath::default(),
                module: PublicLocation::new("/start.js").unwrap(),
                stylesheet: None,
            },
            move |_: &PageSnapshot<()>| {
                let _state = RwSignal::new(capture.upgrade().unwrap());
                if mode == "component-error" {
                    Err(ViewError::InvalidSnapshot)
                } else {
                    Ok("owned".to_owned())
                }
            },
        )
        .unwrap();
        // render() stages synchronously; the future must already own no reactive state.
        let mut rendered = renderer.render(seed());
        drop(marker);
        assert!(weak.upgrade().is_none(), "{mode}");
        assert!(Owner::current() == Some(outer.clone()), "{mode}");
        match mode {
            "component-error" => assert!(matches!(rendered.await, Err(ViewError::InvalidSnapshot))),
            "html-limit" => assert!(matches!(rendered.await, Err(ViewError::HtmlLimit))),
            "discard" => drop(rendered),
            "cancel" => {
                std::future::poll_fn(|context| {
                    assert!(rendered.as_mut().poll(context).is_pending());
                    std::task::Poll::Ready(())
                })
                .await;
                drop(rendered);
            }
            _ => {
                rendered.await.unwrap();
            }
        }
        assert!(Owner::current() == Some(outer.clone()), "{mode}");
    }
    outer.unset_with_forced_cleanup();
}

/// Leaving the runtime is also a typed error, even after successful startup.
#[test]
fn no_runtime_never_panics() {
    assert_eq!(initialize_ssr(), Err(ViewError::ExecutorUnavailable));
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let staged = runtime.block_on(async {
        initialize_ssr().unwrap();
        LeptosRenderer::prepare(&seed(), || view! { <p>"owned"</p> }).unwrap()
    });
    drop(runtime);
    assert!(matches!(
        futures::executor::block_on(staged.finish()),
        Err(ViewError::ExecutorUnavailable)
    ));
}

#[path = "../../../lib/tests/support/watchdog.rs"]
mod watchdog;

/// Panicking trusted render work runs in a killed/reaped child if it stops progressing.
#[test]
fn unwind_child_releases_render_state() {
    if std::env::var_os("PROFILE_UI_RENDER_CHILD").is_some() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            initialize_ssr().unwrap();
            let outer = Owner::new();
            outer.set();
            provide_context("outer context");
            let marker = Arc::new(());
            let weak = Arc::downgrade(&marker);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let _ = LeptosRenderer::prepare(&seed(), move || {
                    let _state = RwSignal::new(marker);
                    view! { <p>{panicking_text()}</p> }
                });
            }));
            assert!(result.is_err());
            assert!(weak.upgrade().is_none());
            assert!(
                Owner::current() == Some(outer.clone()),
                "unwinding a render must restore its caller's reactive Owner"
            );
            assert_eq!(use_context::<&'static str>(), Some("outer context"));
            // A later independent render remains isolated after the unwind.
            LeptosRenderer::prepare(&seed(), || view! { <p>"next"</p> })
                .unwrap()
                .finish()
                .await
                .unwrap();
            outer.unset_with_forced_cleanup();
        });
        return;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", "unwind_child_releases_render_state"])
        .env("PROFILE_UI_RENDER_CHILD", "1");
    let result = watchdog::run(
        &mut command,
        "render-unwind",
        std::time::Duration::from_secs(5),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}

/// Fail during trusted synchronous component evaluation.
fn panicking_text() -> String {
    panic!("intentional trusted renderer panic")
}
