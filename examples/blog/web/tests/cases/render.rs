//! Native shared view boundaries: empty content, optional data and source error presentation.
use blog_app::{Kind, Page};
use leptos::prelude::*;
use serde_json::json;

/// Prototype the actual macro's markers, rather than inventing DOM anchors for the patcher.
#[test]
fn hot_reload_macro_marker_prototype() {
    let html = view! { <p class="before">"Before"</p> }.to_html();
    assert_eq!(
        html.contains("<!--hot-reload|"),
        option_env!("LEPTOS_WATCH").is_some()
    );
    if std::env::var_os("BLOG_MARKER_PROTOTYPE").is_some() {
        println!("MARKER_PROTOTYPE={}", serde_json::to_string(&html).unwrap());
    }
}

/// Subprocess fixture avoids mutating process-wide environment in parallel Rust tests.
#[tokio::test(flavor = "current_thread")]
async fn dev_document_fixture() {
    if std::env::var_os("BLOG_DOCUMENT_FIXTURE").is_none() {
        return;
    }
    stackpress_framework_view::leptos::render::initialize_ssr().unwrap();
    // Compare the same public snapshot, without database timestamps or random auth CSRF data.
    let page = Page {
        kind: Kind::Home,
        title: "The Blog".into(),
        data: json!([]),
        code: 200,
        path: "/".into(),
        locale: "en_US".into(),
        ..Default::default()
    };
    let html = blog_web::render::document(page).await.unwrap();
    let html = blog_web::dev::document(&html).unwrap_or(html);
    println!("DEV_DOCUMENT={}", serde_json::to_string(&html).unwrap());
    assert!(blog_web::dev::document("fragment without head").is_none());
    // Separately prove that the registered response hook decorates both actual document families.
    let app = super::support::App::new().await;
    for path in ["/", "/auth/signin/username"] {
        let response = app.route("GET", path, json!({}), None, None).await;
        let stackpress_lib::Body::Text(body) = response.body() else {
            panic!("HTML required");
        };
        assert_eq!(body.contains("/live_reload"), html.contains("/live_reload"));
    }
    app.close().await;
}

/// Runtime-only injection leaves vanilla HTML identical when the dev block is removed.
#[test]
fn autoreload_is_only_in_the_explicit_dev_environment() {
    let render = |port: Option<&str>, watch: bool| {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "render::dev_document_fixture", "--nocapture"])
            .env("BLOG_DOCUMENT_FIXTURE", "1")
            .env_remove("BLOG_DEV_RELOAD_PORT")
            .env_remove("LEPTOS_WATCH");
        if let Some(port) = port {
            command.env("BLOG_DEV_RELOAD_PORT", port);
        }
        if watch {
            command.env("LEPTOS_WATCH", "1");
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let output = String::from_utf8(output.stdout).unwrap();
        serde_json::from_str::<String>(
            output
                .lines()
                .find_map(|s| s.strip_prefix("DEV_DOCUMENT="))
                .unwrap(),
        )
        .unwrap()
    };
    let normal = render(None, false);
    assert!(!normal.contains("/live_reload"));
    assert_eq!(render(None, true), normal);
    assert_eq!(render(Some("bad"), true), normal);
    assert_eq!(render(Some("0"), true), normal);
    assert_eq!(render(Some("3101"), false), normal);
    let dev = render(Some("3101"), true);
    let (before, tail) = dev.split_once("<!--blog-dev-reload-->").unwrap();
    let (script, after) = tail.split_once("<!--/blog-dev-reload-->").unwrap();
    assert!(script.contains("/live_reload"));
    assert!(script.contains("3101"));
    assert!(script.contains("if (msg.all || msg.blog_reload === blogRenderer)"));
    assert!(script.contains("if (msg.css)"));
    assert_eq!(format!("{before}{after}"), normal);
}

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
