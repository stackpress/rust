//! Native shared view boundaries: empty content, optional data and source error presentation.
use blog_shared::{Kind, Page};
use serde_json::json;

/// Empty home/article states and production trace visibility retain original source fallbacks.
#[tokio::test(flavor = "current_thread")]
async fn native_shared_views_keep_empty_fallbacks_and_error_trace_visibility() {
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    let base = Page {
        kind: Kind::Home,
        title: "The Blog".into(),
        data: json!([]),
        code: 200,
        path: "/".into(),
        locale: "en_US".into(),
        mode: "production".into(),
        brand: json!({"name":"Stackpress"}),
        ..Default::default()
    };
    let empty = blog_web::render::document(base.clone()).await.unwrap();
    assert!(empty.contains("No published articles yet"));
    assert!(empty.contains("Browse Writing"));
    let article = Page {
        kind: Kind::Article,
        title: "Optional fields".into(),
        data: json!({"title":"Optional fields","comments":[{}]}),
        path: "/articles/optional".into(),
        ..base.clone()
    };
    let html = blog_web::render::document(article).await.unwrap();
    assert!(html.contains("Unscheduled"));
    assert!(html.contains("No comment text provided."));
    assert!(html.contains("1 comment"));
    let error = Page {
        kind: Kind::Error,
        title: "Oops...".into(),
        code: 500,
        error: "Source failure".into(),
        stack: json!([{"file":"owned.rs","line":2,"column":1,"snippet":{"before":"before","main":"main","location":"^","after":"after"}}]),
        ..base.clone()
    };
    let production = blog_web::render::document(error.clone()).await.unwrap();
    assert!(!production.contains("class=\"error-trace\""));
    let development = blog_web::render::document(Page {
        mode: "develop".into(),
        ..error
    })
    .await
    .unwrap();
    assert!(development.contains("class=\"error-trace\""));
    assert!(development.contains("<pre>main</pre>"));
    let invalid = Page {
        kind: Kind::Article,
        data: json!({"published":"invalid"}),
        ..base
    };
    assert!(blog_web::render::document(invalid).await.is_err());
}
