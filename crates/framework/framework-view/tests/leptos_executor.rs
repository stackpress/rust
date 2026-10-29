//! This integration executable has its own process-global executor state.
#![cfg(feature = "leptos-ssr")]

use leptos::prelude::*;
use stackpress_framework_view::leptos::render::{LeptosRenderer, initialize_ssr};
use stackpress_framework_view::{PageId, PageSnapshot, ViewError};

/// Missing initialization is an ordinary error; repeated initialization is safe.
#[tokio::test]
async fn startup_is_explicit_idempotent_and_checked() {
    let seed = PageSnapshot {
        revision: 1,
        build: "test".into(),
        page: PageId::try_from("test".to_owned()).unwrap(),
        location: "/".into(),
        locale: "en".into(),
        theme: Some("light".into()),
        status: stackpress_framework_view::PublicStatus::ok(),
        data: (),
    };
    let result = LeptosRenderer::prepare(&seed, || view! { <p>"ready"</p> });
    assert!(matches!(result, Err(ViewError::ExecutorUnavailable)));
    initialize_ssr().unwrap();
    initialize_ssr().unwrap();
    let page = LeptosRenderer::prepare(&seed, || view! { <p>"ready"</p> })
        .unwrap()
        .finish()
        .await
        .unwrap();
    assert!(page.into_html().contains("ready"));
}
