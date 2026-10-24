//! Generated application; inspect src/generated and regenerate before compilation.

pub mod generated;
#[path = "../../selection.rs"]
pub mod selection;

use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

/// Build only the generated registry; removing the model leaves no hard Profile imports.
///
/// # Errors
/// Returns normal configuration, plugin selection or startup failures.
pub async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    generated::register(&mut app)?;
    app.build().await
}
