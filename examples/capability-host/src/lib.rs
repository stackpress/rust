//! A compiled, in-memory profiles application with no listener or native output.
//!
//! [`build_application`] owns plugin selection and accepts explicit configuration layers.
//! The optional [`customization`] plugin can be selected by another application without
//! changing [`profiles`]. The `experimental` feature compiles and selects an additional
//! plugin; its entire module is absent from default builds. CLI and HTTP adapters can
//! share this builder once their implementations are integrated.
//!
//! ```
//! use stackpress_example_capability_host::build_application;
//! use stackpress_server::ConfigLayers;
//! use serde_json::json;
//!
//! #[tokio::main(flavor = "current_thread")]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // The application chooses compiled plugins; callers supply explicit config layers.
//!     let app = build_application(ConfigLayers::new()).await?;
//!     let reply = app.call("profile-search", json!({"id": "abc123"})).await?;
//!     assert_eq!(reply.response.try_to_json()?["results"][0]["name"], "Ada Lovelace");
//!     Ok(())
//! }
//! ```

#![cfg_attr(
    not(feature = "experimental"),
    doc = r#"
Default builds expose no experimental module, in addition to excluding its implementation:

```compile_fail,E0432
use stackpress_example_capability_host::experimental;
```
"#
)]

use stackpress_server::{BuildResult, ConfigLayers, Server, ServerBuilder};

#[cfg(feature = "experimental")]
use stackpress_server::PluginSpec;

pub mod customization;
pub mod profiles;

#[cfg(feature = "experimental")]
pub mod experimental;

/// Build the application with defaults followed by the caller's ordered config layers.
///
/// This performs no environment/file reads and starts no background work. Customization
/// is deliberately selected by callers composing their own builder, rather than a runtime flag.
///
/// # Errors
/// Returns configuration, plugin-selection, or startup failures without publishing a partial host.
pub async fn build_application(layers: ConfigLayers) -> BuildResult<Server> {
    // Install external layers before selecting the plugins that own their typed sections.
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("profiles", profiles::register)?;

    // Cargo removes both this selection and the module when the feature is disabled.
    #[cfg(feature = "experimental")]
    app.plugin_with(
        PluginSpec::new("experimental").requires(&["profiles"]),
        experimental::register,
    )?;

    // Consume startup once; callers receive only a fully constructed immutable host.
    app.build().await
}
