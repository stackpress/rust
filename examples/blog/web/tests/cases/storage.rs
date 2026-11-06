//! Real file SQLite setup, ordered population, joins and restart preservation.

use serde_json::{Value, json};
use stackpress_lib::{Request, Response};
use stackpress_server::{ConfigLayers, Server};
use stackpress_sql::Engine;
use std::path::Path;

/// Use exactly one app-owned file layer above shared defaults.
fn layers(path: &Path) -> ConfigLayers {
    let mut layers = ConfigLayers::new();
    layers
        .value("test file", json!({"sqlite":{"path":path}}))
        .unwrap();
    layers
}

/// Dispatch the real compiled source events without opening a listener.
async fn emit(server: &Server, event: &str, data: Value) -> Value {
    let mut request = Request::builder().data(data).build().unwrap();
    let mut response = Response::new();
    server
        .emit(event, &mut request, &mut response)
        .await
        .unwrap();
    response.try_to_json().unwrap()
}

/// Assert the actual stored counts through the generated model search processes.
async fn counts(server: &Server) {
    for (name, count) in [
        ("profile", 1),
        ("auth", 2),
        ("application", 2),
        ("session", 0),
        ("category", 2),
        ("article", 2),
        ("catalog", 2),
        ("comment", 4),
    ] {
        let reply = emit(server, &format!("{name}-search"), json!({})).await;
        assert_eq!(reply["code"], 200, "{name}: {reply}");
        assert_eq!(reply["total"], count, "{name}: {reply}");
    }
}

#[tokio::test(flavor = "current_thread")]
async fn setup_populate_join_defaults_composite_key_and_restart() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = fixture.path().join("blog.sqlite");
    blog_web::plugins::store::setup(&path).await.unwrap();
    let server = blog_web::build_storage(layers(&path), "abc123")
        .await
        .unwrap();
    assert_eq!(emit(&server, "article-search", json!({})).await["total"], 0);
    assert_eq!(emit(&server, "populate", json!({})).await["code"], 200);
    counts(&server).await;
    let catalog = emit(&server, "catalog-search", json!({})).await;
    assert_eq!(
        catalog["results"][0]["article"]["profile"]["id"],
        "developer"
    );
    let duplicate = emit(
        &server,
        "catalog-create",
        json!({"categoryId":"anime","articleId":"pokemon"}),
    )
    .await;
    // Composite primary-key collisions keep the source database-error status; no app rewrite.
    assert_eq!(duplicate["code"], 500);
    assert_eq!(emit(&server, "catalog-search", json!({})).await["total"], 2);
    let draft = emit(
        &server,
        "article-create",
        json!({"profileId":"developer","title":"Draft","slug":"draft"}),
    )
    .await;
    assert_eq!(draft["code"], 200, "{draft}");
    assert_eq!(draft["results"]["status"], "DRAFT");
    assert_eq!(draft["results"]["active"], true);
    server
        .host()
        .shared_service::<Engine>()
        .unwrap()
        .close()
        .await
        .unwrap();
    drop(server);

    // Reopening is connection-only; it cannot install or re-run the population list.
    let server = blog_web::build_storage(layers(&path), "abc123")
        .await
        .unwrap();
    assert_eq!(emit(&server, "article-search", json!({})).await["total"], 3);
    assert_eq!(emit(&server, "comment-search", json!({})).await["total"], 4);
    server
        .host()
        .shared_service::<Engine>()
        .unwrap()
        .close()
        .await
        .unwrap();
}

#[tokio::test(flavor = "current_thread")]
async fn missing_file_and_invalid_config_fail_without_setup() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let missing = fixture.path().join("missing.sqlite");
    assert!(
        blog_web::build_storage(layers(&missing), "abc123")
            .await
            .is_err()
    );
    assert!(!missing.exists());
    let mut invalid = ConfigLayers::new();
    invalid
        .value("invalid file", json!({"sqlite":{"path":42}}))
        .unwrap();
    assert!(blog_web::build_storage(invalid, "abc123").await.is_err());
    assert_eq!(
        blog_web::seed::events().unwrap().as_array().unwrap().len(),
        15
    );
}

#[tokio::test(flavor = "current_thread")]
async fn cli_b_file_overrides_shared_path_and_invalid_values_fail() {
    let fixture = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = fixture.path().join("cli.sqlite");
    blog_web::plugins::store::setup(&path).await.unwrap();
    let config = fixture.path().join("develop.json");
    std::fs::write(
        &config,
        serde_json::to_vec(&json!({"sqlite":{"path":path}})).unwrap(),
    )
    .unwrap();
    let invoke = |event: &str| {
        std::process::Command::new(env!("CARGO_BIN_EXE_blog-web"))
            .args(["-b"])
            .arg(&config)
            .args(["emit", event])
            .output()
            .unwrap()
    };
    let populated = invoke("populate");
    assert!(
        populated.status.success(),
        "{}",
        String::from_utf8_lossy(&populated.stderr)
    );
    let searched = invoke("article-search");
    assert!(searched.status.success());
    let reply: Value = serde_json::from_slice(&searched.stdout).unwrap();
    assert_eq!(reply["total"], 2, "{reply}");
    std::fs::write(&config, r#"{"sqlite":{"path":42}}"#).unwrap();
    assert!(!invoke("article-search").status.success());
}
