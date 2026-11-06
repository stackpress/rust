//! File SQLite ownership and finite commands for the Stackpress blog boilerplate.
//!
//! Installation, revisions and population are explicit operations. Startup opens an existing
//! file and publishes immutable scripts; it never changes schema/data or reads revision files.

pub mod client;
pub mod config;
pub mod history;
pub mod http;
pub mod plugins;
pub mod render;
pub mod seed;

/// The generator owns this complete source/store tree; regenerate instead of patching it.
#[allow(missing_docs)]
pub mod generated;

use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

/// Build the finite SQL/CRUD command host; installation is a separate explicit operation.
///
/// # Errors
/// Preserves configuration, connection and event registration failures.
pub async fn build_storage(layers: ConfigLayers, database_seed: &str) -> BuildResult<Server> {
    build_storage_with(
        layers,
        database_seed,
        stackpress_framework::config::Environment::default(),
    )
    .await
}

/// Build the same finite host with explicit native argv/force inputs for trusted commands.
/// # Errors
/// Preserves the same configuration/startup failures as [`build_storage`].
pub async fn build_storage_with(
    layers: ConfigLayers,
    database_seed: &str,
    environment: stackpress_framework::config::Environment,
) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(config::shared(database_seed)?)?;
    app.config_layers(layers)?;
    stackpress_framework::config::register(&mut app)?;
    plugins::store::register(&mut app)?;
    history::register(&mut app)?;
    app.on_config(stackpress_lib::queue::task(
        move |start: &mut stackpress_server::StartupContext| {
            let environment = environment.clone();
            Box::pin(async move {
                start.register(std::sync::Arc::new(environment))?;
                Ok(stackpress_lib::Flow::Continue)
            })
        },
    ))?;
    plugins::commands::register(&mut app)?;
    // These are the generated registrars, with the complete application model inventory.
    generated::stores::profile::register(&mut app)?;
    generated::stores::category::register(&mut app)?;
    generated::stores::catalog::register(&mut app)?;
    generated::stores::article::register(&mut app)?;
    generated::stores::comment::register(&mut app)?;
    generated::stores::application::register(&mut app)?;
    generated::stores::session::register(&mut app)?;
    generated::stores::auth::register(&mut app)?;
    app.build().await
}
