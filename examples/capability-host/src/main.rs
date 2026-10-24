//! Application-owned executable: compiled plugins and one reusable CLI entry pipeline.

use std::process::ExitCode;

use stackpress_example_capability_host::profiles;
use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

#[cfg(feature = "experimental")]
use stackpress_server::PluginSpec;

/// Install CLI-supplied config layers and select this executable's compiled plugins.
async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    // External file layers are already parsed; the app owns applying them before startup.
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("profiles", profiles::register)?;

    // The feature excludes both the experimental implementation and its selection when off.
    #[cfg(feature = "experimental")]
    app.plugin_with(
        PluginSpec::new("experimental").requires(&["profiles"]),
        stackpress_example_capability_host::experimental::register,
    )?;

    // Consume startup once; customization remains an independently test-selected plugin.
    app.build().await
}

/// Run native argv/config/output handling on one application-owned runtime.
#[tokio::main(flavor = "current_thread")]
async fn main() -> ExitCode {
    // main_with owns usage, bootstrap loading, one lifecycle, JSON stdout and process exit policy.
    stackpress_cli::main_with(build_application).await
}
