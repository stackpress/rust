//! No-listener API host composition. The application owns Engine lifetime and config layers.
//! JWT identity, API model events and process routes are selected independently; consent
//! rendering and installed/browser hosts belong to the later presentation milestone.
use stackpress_framework_api::pages;
use stackpress_lib::{Flow, Result, queue::task};
use stackpress_server::{ConfigLayers, Server, ServerBuilder, StartupContext};
use stackpress_sql::Engine;
use std::sync::Arc;

/// Ordinary #14 composite: no required plugin IDs, ambient caller slot or startup SQL.
/// # Errors
/// Preserves each constituent's registration errors.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_session::plugin::register(app)?;
    stackpress_framework_api::sql::register(app)?;
    stackpress_framework_api::register(app)
}

/// Build a host without binding a port. Install/seed stores explicitly before requesting pages.
///
/// Layers supply source api/session configuration; presentation maps the app's view/brand
/// objects into the API page environment without claiming other plugins' typed config keys.
/// Keep the Engine alive until requests finish, then close it explicitly in the application.
/// # Errors
/// Returns ordinary startup/config errors; no listener or detached worker is created.
pub async fn build(
    layers: ConfigLayers,
    engine: Arc<Engine>,
    presentation: pages::Environment,
) -> Result<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        let presentation = presentation.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(Arc::new(presentation))?;
            Ok(Flow::Continue)
        })
    }))?;
    app.plugin("application", register)?;
    app.build().await.map_err(|failure| failure.error)
}
