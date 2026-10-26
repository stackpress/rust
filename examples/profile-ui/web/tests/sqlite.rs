//! Compare actual in-memory and SQLite hosts through the same SSR and HTTP boundary.
#![cfg(feature = "sqlite")]

use serde_json::json;
use stackpress_example_profile_ui_web::{
    application_with_config, register_pages, routes, search_api,
};
use stackpress_framework_view::BasePath;
use stackpress_server::{
    ConfigLayers, Server, ServerBuilder,
    http::{HttpAdapter, HttpOutput},
};

/// Supply only app-owned path and policy; no request can select either setting.
fn layers(path: &std::path::Path, limit: usize) -> ConfigLayers {
    let mut layers = ConfigLayers::new();
    layers
        .value(
            "fixture",
            json!({"sqlite":{"path":path},"profiles":{"limit":limit}}),
        )
        .unwrap();
    layers
}

/// Build the actual generated in-memory reference with identical presentation routes.
async fn memory(base: BasePath, limit: usize) -> Server {
    let mut app = ServerBuilder::new();
    let mut config = ConfigLayers::new();
    config
        .value("reference", json!({"profiles":{"limit":limit}}))
        .unwrap();
    app.config_layers(config).unwrap();
    stackpress_example_generated_profile::generated::register(&mut app).unwrap();
    #[cfg(feature = "demo-customization")]
    app.plugin(
        "profile-ui-labels",
        stackpress_example_profile_ui_web::customization::register,
    )
    .unwrap();
    register_pages(&mut app, base.clone()).unwrap();
    app.post(routes::search_api(&base).unwrap().as_str(), search_api)
        .unwrap();
    app.build().await.unwrap()
}

/// Preserve exact status, headers and bytes rather than comparing only decoded rows.
async fn response(app: &Server, uri: &str, body: Option<Vec<u8>>) -> http::Response<Vec<u8>> {
    let request = http::Request::builder()
        .uri(uri)
        .method(if body.is_some() { "POST" } else { "GET" })
        .header(
            "content-type",
            if body.is_some() {
                "application/json"
            } else {
                "text/plain"
            },
        )
        .body(body.unwrap_or_default())
        .unwrap();
    let HttpOutput::Response(response) = HttpAdapter::new(Default::default())
        .unwrap()
        .handle(app, request)
        .await
        .unwrap()
        .output
    else {
        panic!("missing response")
    };
    response
}

/// Every visible page/seed, error status and typed envelope agrees at both mounts.
#[tokio::test]
async fn complete_ssr_and_typed_http_parity() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    for base in ["", "/demo/v1"] {
        let base = BasePath::new(base).unwrap();
        for limit in [1, 3] {
            let sqlite = application_with_config(base.clone(), layers(&path, limit))
                .await
                .unwrap();
            let reference = memory(base.clone(), limit).await;
            for prefix in [
                "abc",
                "abc4",
                "ABC",
                "missing",
                "%",
                "_",
                "'",
                "\"",
                "\\",
                "雪",
                "abc' OR 1=1 --",
                "abc\0",
                "",
            ] {
                // Encode even invalid public input so the actual HTTP boundary rejects it.
                let encoded = prefix
                    .bytes()
                    .map(|byte| format!("%{byte:02X}"))
                    .collect::<String>();
                let uri = format!("{}/ui/profiles?id={encoded}", base.as_str());
                for body in [
                    None,
                    Some(serde_json::to_vec(&json!({"revision":1,"id":prefix})).unwrap()),
                ] {
                    let uri = if body.is_some() {
                        routes::search_api(&base).unwrap().as_str().to_owned()
                    } else {
                        uri.clone()
                    };
                    let actual = response(&sqlite, uri.as_str(), body.clone()).await;
                    let expected = response(&reference, uri.as_str(), body).await;
                    assert_eq!(actual.status(), expected.status(), "{prefix:?}");
                    assert_eq!(actual.headers(), expected.headers(), "{prefix:?}");
                    assert_eq!(actual.body(), expected.body(), "{prefix:?}");
                }
            }
            for suffix in [
                "/ui/profiles/abc456?return=abc",
                "/ui/profiles/missing?return=abc456",
            ] {
                let uri = base.join(suffix).unwrap();
                let actual = response(&sqlite, uri.as_str(), None).await;
                let expected = response(&reference, uri.as_str(), None).await;
                assert_eq!(actual.status(), expected.status());
                assert_eq!(actual.body(), expected.body());
            }
        }
    }
}

/// Missing/uninstalled files fail startup without installation, seed or file changes.
#[tokio::test]
async fn startup_has_no_setup_effects() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("missing.sqlite");
    let error = application_with_config(BasePath::default(), layers(&path, 3))
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("sql-profile-setup"));
    assert_eq!(
        error.error.to_string(),
        "SQLite profile UI requires an installed sqlite.path; run sql-profile-setup explicitly"
    );
    let cause = std::error::Error::source(&error.error).unwrap();
    assert!(cause.downcast_ref::<stackpress_lib::Error>().is_some());
    assert!(std::error::Error::source(cause).is_some());

    assert!(!path.exists());
    std::fs::write(&path, []).unwrap();
    let error = application_with_config(BasePath::default(), layers(&path, 3))
        .await
        .err()
        .unwrap();
    assert!(error.to_string().contains("sql-profile-setup"));
    assert!(std::fs::read(&path).unwrap().is_empty());
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    assert!(
        application_with_config(BasePath::default(), layers(&path, 0))
            .await
            .is_err()
    );
    let app = application_with_config(BasePath::default(), layers(&path, 3))
        .await
        .unwrap();
    assert_eq!(
        response(&app, "/ui/profiles?id=abc", None).await.status(),
        200
    );
    assert_eq!(std::fs::read(&path).unwrap(), before);
}
