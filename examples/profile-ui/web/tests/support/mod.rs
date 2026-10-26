//! Temporary application fixture shared by default and selected-backend regressions.

use stackpress_framework_view::BasePath;
use stackpress_server::{ConfigLayers, Server};

/// Own the database directory until after the server and its read-only service drop.
pub async fn application(
    base: BasePath,
) -> stackpress_server::BuildResult<(tempfile::TempDir, Server)> {
    let directory = tempfile::tempdir().unwrap();
    let layers = layers(directory.path()).await;
    let app = stackpress_example_profile_ui_web::application_with_config(base, layers).await?;
    Ok((directory, app))
}

/// Install only in fixture setup; application startup always receives an existing file.
async fn layers(_directory: &std::path::Path) -> ConfigLayers {
    #[cfg(feature = "sqlite")]
    {
        let path = _directory.join("profiles.sqlite");
        stackpress_example_sql_profile::setup::install_and_seed(&path)
            .await
            .unwrap();
        let mut layers = ConfigLayers::new();
        layers
            .value("fixture", serde_json::json!({"sqlite":{"path":path}}))
            .unwrap();
        layers
    }
    #[cfg(not(feature = "sqlite"))]
    ConfigLayers::new()
}
