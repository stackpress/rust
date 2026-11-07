//! Owned-file connection, startup and configuration boundaries without generated models.
use serde_json::json;
use stackpress_server::{ConfigLayers, ServerBuilder};
use stackpress_sql::Engine;

/// Acquisition never creates a missing file unless the explicit setup caller requests it.
#[tokio::test(flavor = "current_thread")]
async fn explicit_creation_then_connection_only_startup_preserves_file() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = root.path().join("store.sqlite");
    assert!(blog_store::open(&path, false).await.is_err());
    assert!(!path.exists());
    // Setup's explicit creation is followed by the plugin's ordinary existing-file startup.
    blog_store::open(&path, true)
        .await
        .unwrap()
        .close()
        .await
        .unwrap();
    let before = std::fs::read(&path).unwrap();
    let mut builder = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers
        .value(
            "owned store fixture",
            json!({"sqlite":{"path":path},"database":{"seed":"fixture-seed"}}),
        )
        .unwrap();
    builder.config_layers(layers).unwrap();
    // This minimal host supplies only the aggregate section consumed by the connection.
    builder
        .config_section(
            "database",
            stackpress_framework::config::DatabaseOptions::default(),
            |_| Ok(()),
        )
        .unwrap();
    blog_store::register(&mut builder).unwrap();
    let server = builder.build().await.unwrap();
    assert_eq!(
        server
            .host()
            .service::<stackpress_framework_sql::source::SourceEnvironment>()
            .unwrap()
            .seed,
        "fixture-seed"
    );
    // Reap this fixture's connection before comparing bytes and retiring its directory.
    server
        .host()
        .service::<Engine>()
        .unwrap()
        .close()
        .await
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

/// Typed configuration rejects an invalid file before connection acquisition.
#[tokio::test(flavor = "current_thread")]
async fn invalid_path_configuration_fails_without_a_connection() {
    assert_eq!(
        blog_store::DatabaseConfig::default().path,
        std::path::Path::new(".build/blog.sqlite")
    );
    for path in [json!(42), json!("")] {
        let mut builder = ServerBuilder::new();
        let mut layers = ConfigLayers::new();
        layers
            .value("invalid fixture", json!({"sqlite":{"path":path}}))
            .unwrap();
        builder.config_layers(layers).unwrap();
        builder
            .config_section(
                "database",
                stackpress_framework::config::DatabaseOptions::default(),
                |_| Ok(()),
            )
            .unwrap();
        blog_store::register(&mut builder).unwrap();
        assert!(builder.build().await.is_err());
    }
}
