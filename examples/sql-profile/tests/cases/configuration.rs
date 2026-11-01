//! G47's shared, environment-file and explicit app layers use the real application builder.

use std::path::Path;

use serde_json::json;
use stackpress_example_sql_profile::{
    build_application, database::DatabaseConfig, profiles::ProfilesConfig,
};
use stackpress_server::{ConfigLayers, Server, StartupStage};
use tempfile::TempDir;

use super::close;

/// Read the actual app-owned environment file without changing process working directory.
fn file(path: &Path) -> ConfigLayers {
    let mut layers = ConfigLayers::new();
    layers
        .json("environment-file", &std::fs::read_to_string(path).unwrap())
        .unwrap();
    layers
}

/// Pin both the validated typed policy and its observable result from the same built host.
async fn assert_limit(app: Server, expected: usize) {
    assert_eq!(
        app.host().config::<ProfilesConfig>().unwrap().limit,
        expected
    );
    let reply = app
        .call("profile-search", json!({"id":"abc","limit":100}))
        .await
        .unwrap();
    assert_eq!(reply.response.try_to_json().unwrap()["total"], expected);
    close(app).await;
}

/// Plugin defaults < shared src/config.rs < environment file < explicit app code overrides.
pub(crate) async fn configuration() {
    let root = TempDir::new().unwrap();
    let path = root.path().join("profiles.sqlite");
    stackpress_example_sql_profile::setup::install_and_seed(&path)
        .await
        .unwrap();
    assert_eq!(ProfilesConfig::default().limit, 3);

    // Supply only the isolated database path: the real builder's shared policy must beat 3.
    let mut layers = ConfigLayers::new();
    layers
        .value("fixture-database", json!({"sqlite":{"path":path}}))
        .unwrap();
    assert_limit(build_application(layers).await.unwrap(), 2).await;

    for (name, configured_path) in [
        ("develop", "target/sql-profile-develop.sqlite"),
        ("build", "target/sql-profile-build.sqlite"),
    ] {
        let config = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("config/{name}.json"));
        let mut layers = file(&config);
        assert_eq!(
            layers.entries().next().unwrap().1["sqlite"]["path"],
            configured_path
        );
        // Replace only storage location for this finite fixture; leave the file policy intact.
        layers
            .value("fixture-database", json!({"sqlite":{"path":path}}))
            .unwrap();
        let app = build_application(layers).await.unwrap();
        assert_eq!(app.host().config::<DatabaseConfig>().unwrap().path, path);
        assert_limit(app, 3).await;

        // App-authored data appended after the same file must take precedence, including limit.
        let mut layers = file(&config);
        layers
            .value(
                "explicit-app-override",
                json!({"sqlite":{"path":path},"profiles":{"limit":1}}),
            )
            .unwrap();
        assert_limit(build_application(layers).await.unwrap(), 1).await;
    }

    // A syntactically valid file with an invalid typed value must survive neither by fallback
    // nor by a misplaced shared layer. Validation still precedes native Config callbacks.
    let bad = root.path().join("bad.json");
    for limit in [json!(0), json!("three")] {
        std::fs::write(
            &bad,
            serde_json::to_vec(&json!({"sqlite":{"path":path},"profiles":{"limit":limit}}))
                .unwrap(),
        )
        .unwrap();
        let error = build_application(file(&bad))
            .await
            .err()
            .expect("bad file must reject build");
        assert_eq!(error.stage, StartupStage::Config);
    }
}
