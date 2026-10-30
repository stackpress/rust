# Register and configure CSRF

Select the optional server integration when an event handler should retrieve the service from its invocation. Direct users can instead pass `CsrfConfig` to `Service::new` without a server.

## Publish a configured service

The [event consumer manifest](../../../framework-email/consumers/event/Cargo.toml) shows the server feature and host dependencies. Supply string overrides through the application's config layers, then call the ordinary registrar.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_csrf::plugin;
use stackpress_lib::Result;
use stackpress_server::{ConfigLayers, ServerBuilder};

pub fn configure(app: &mut ServerBuilder) -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.value(
        "application",
        json!({
            "csrf": {"name": "form_token", "error": "Please submit this form again."}
        }),
    )?;
    app.config_layers(layers)?;
    app.plugin("csrf", plugin::register)?;
    Ok(())
}
```

Build the host after configuration; the Config phase publishes `Service`. Removing the CSRF section still publishes a default service. Registration adds no global guard, so protected routes call the service themselves as in [the submit handler](protect-a-process.md).

## Choose overrides directly

`CsrfConfig` has optional `name` and `error` strings. Omission selects the source defaults; explicit empty strings remain explicit values. Invalid cookie names or response/carrier state can therefore still return existing mechanical errors.

| Config section | Result |
| --- | --- |
| Missing, null, false, zero, empty string or other nonobject | Default CSRF service |
| Object with omitted fields | Defaults for those fields |
| Object with string name/error | Exact supplied strings |
| Object with incompatible typed overrides | Startup decoding error |

Duplicate typed section/service registration follows the server's existing error contract. See [plugin reference](../api/plugin.md) for `CsrfSection` and registration errors.
