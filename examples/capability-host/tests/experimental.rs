//! Separate default-off and enabled builds exercise actual Cargo compile-time selection.

use serde_json::json;
use stackpress_example_capability_host::build_application;
use stackpress_lib::Result;
use stackpress_server::ConfigLayers;

/// A default build excludes the module (enforced by its compile_error guard) and the operation.
#[cfg(not(feature = "experimental"))]
#[tokio::test]
async fn disabled_plugin_is_excluded_from_the_build() -> Result<()> {
    // Successful compilation itself proves experimental.rs was not compiled with its guard active.
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    assert_eq!(
        app.plugins()
            .map(|plugin| plugin.id.as_str())
            .collect::<Vec<_>>(),
        ["profiles"]
    );

    // The runtime check supplements compile-time exclusion; it is not its sole evidence.
    let reply = app.call("profile-preview", json!({"id": "abc123"})).await?;
    assert_eq!(reply.response.try_to_json()?["code"], 404);
    assert_eq!(
        reply.report.operation,
        Some(stackpress_lib::DispatchOutcome::NoListener)
    );
    Ok(())
}

/// Enabling the feature makes both the module API and its delegated operation available.
#[cfg(feature = "experimental")]
#[tokio::test]
async fn enabled_plugin_compiles_and_delegates_to_profiles() -> Result<()> {
    // This direct function reference also proves the feature publishes a real module API.
    let _: fn(&mut stackpress_server::ServerBuilder) -> Result<()> =
        stackpress_example_capability_host::experimental::register;
    let app = build_application(ConfigLayers::new())
        .await
        .map_err(|error| error.error)?;
    assert_eq!(
        app.plugins()
            .map(|plugin| plugin.id.as_str())
            .collect::<Vec<_>>(),
        ["profiles", "experimental"]
    );

    // Alias delegation produces the same complete public result without another lifecycle.
    let preview = app.call("profile-preview", json!({"id": "abc123"})).await?;
    let ordinary = app.call("profile-search", json!({"id": "abc123"})).await?;
    assert_eq!(
        preview.response.try_to_json()?,
        ordinary.response.try_to_json()?
    );
    assert!(preview.report.is_success(&preview.response));
    assert_eq!(preview.report.stages.len(), ordinary.report.stages.len());
    Ok(())
}
