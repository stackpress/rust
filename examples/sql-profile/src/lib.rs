//! Application-owned SQLite profiles with the original in-memory capability's wire contract.
//!
//! Run the separate setup executable once before startup. [`build_application`] opens only
//! an existing file read-only, registers an Engine and exposes one read capability/route.
//! No import, startup callback or search installs/seeds data or starts a listener.
//!
//! ```no_run
//! use serde_json::json;
//! use stackpress_example_sql_profile::{build_application, setup};
//! use stackpress_server::ConfigLayers;
//! use stackpress_sql::Engine;
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! setup::install_and_seed("profiles.sqlite").await?;
//! let app = build_application(ConfigLayers::new()).await?;
//! let reply = app.call("profile-search", json!({"id": "abc"})).await?;
//! assert_eq!(reply.response.try_to_json()?["total"], 3);
//! // Await close while the runtime is alive; callers choose their shutdown deadline.
//! app.host().shared_service::<Engine>()?.close().await?;
//! # Ok(())
//! # }
//! ```

pub mod database;
pub mod profiles;
pub mod setup;

use stackpress_server::{BuildResult, ConfigLayers, PluginSpec, Server, ServerBuilder};

/// Select the database prerequisite and profiles plugin with caller-supplied ordered layers.
///
/// Startup never creates a missing file or installs schema/data. Use [`setup::install_and_seed`]
/// explicitly first. The resulting Server has no network listener; HTTP stays buffered/in-process.
///
/// # Errors
/// Returns configuration, plugin ordering, database-open or missing-schema startup failures.
pub async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    // Configuration belongs to the application; request data never selects the database file.
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("database", database::register)?;
    app.plugin_with(
        PluginSpec::new("profiles").requires(&["database"]),
        profiles::register,
    )?;
    app.build().await
}
