//! Rendering cannot retain the reactive Owner after staging or unwinding.
#![cfg(feature = "ssr")]

use leptos::prelude::*;
use stackpress_framework_ui::render::{LeptosRenderer, initialize_ssr};
use stackpress_framework_view::{PageId, PageSnapshot, ViewError};
use std::sync::Arc;

/// Construct explicit safe metadata for ownership tests.
fn seed() -> PageSnapshot<()> {
    PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("test".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en".into(),
        theme: "light".into(),
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
            // A later independent render remains isolated after the unwind.
            LeptosRenderer::prepare(&seed(), || view! { <p>"next"</p> })
                .unwrap()
                .finish()
                .await
                .unwrap();
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
