# Typed configuration

A plugin should validate its own settings before allocating resources. Register a concrete section
with defaults, then apply explicit data layers so every Config task sees the same checked value.

## Claim and override a section

This example starts with three results and narrows the limit to one. Its validator runs during
build, before any async startup work:

```rust
use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Result};
use stackpress_server::{ConfigLayers, ServerBuilder};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfilesConfig {
    limit: usize,
}

fn validate(config: &ProfilesConfig) -> Result<()> {
    if !(1..=100).contains(&config.limit) {
        return Err(Error::invalid("limit must be in 1..=100"));
    }
    Ok(())
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut layers = ConfigLayers::new();
    layers.json("file", r#"{"profiles":{"limit":2}}"#)?;
    layers.value("code", serde_json::json!({"profiles": {"limit": 1}}))?;
    let mut app = ServerBuilder::new();
    app.config_section("profiles", ProfilesConfig { limit: 3 }, validate)?;
    app.config_layers(layers)?;
    let server = app.build().await?;
    assert_eq!(server.host().config::<ProfilesConfig>()?.limit, 1);
    Ok(())
}
```

Defaults always precede explicit layers, even when an application installs layers before selecting
the plugin that claims them. Later object keys override recursively. Arrays, scalars, and null
replace whole values; serde and your validator decide whether the result is a valid section.

## Own external input explicitly

Convert environment values or file contents in application code and append their data layers in the
intended order. ServerBuilder does not read them for you. The CLI entry point optionally reads one
bounded JSON bootstrap file, but your app builder must install the ConfigLayers it receives.

Use newtypes for two settings roles with the same underlying shape: each concrete TypeId and section
name can be claimed once. Unknown top-level names fail build; `deny_unknown_fields` handles unknown
fields inside this example's section. A rejected layer append leaves accepted layers unchanged.

You now have one validated settings type with clear override order. Use it during [service
initialization](services.md); consult [ConfigLayers](../api/config.md) for limits and
[ServerBuilder](../api/builder.md) for registration errors.
