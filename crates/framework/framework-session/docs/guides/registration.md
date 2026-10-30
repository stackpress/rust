# Register identity or the session group

Choose the smallest registrar that supplies your processes. Identity alone adds the configured cookie service and `me`/`authorize`; the composite includes auth and email in the source package order. Both remain ordinary `fn(&mut ServerBuilder) -> Result<()>` functions.

## Select identity

The [plugin consumer manifest](../../consumers/plugin/Cargo.toml) supplies server dependencies for these examples. A host needing only identity can select `stackpress-framework-session/server` instead of `auth-server`. Add a session section, then select the identity function.

<!-- example: server -->
```rust
use serde_json::json;
use stackpress_framework_session::{Service, plugin};
use stackpress_lib::Result;
use stackpress_server::{ConfigLayers, ServerBuilder};

pub async fn identity_host() -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.value("application", json!({"session": {"seed": "example-seed"}}))?;
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("identity", plugin::register)?;
    let server = app.build().await.map_err(|failure| failure.error)?;
    assert_eq!(server.host().service::<Service>()?.config().key, "session");
    assert!(server.listeners().any(|entry| entry.pattern == "me"));
    assert!(!server.listeners().any(|entry| entry.pattern == "request"));
    Ok(())
}
```

This builds only an in-process application. The omitted access map means no session request guard, as pinned by [host tests](../../tests/main/host.rs). It does not open a listener or database.

## Select the source group

Use `composite::register` for auth → email → identity. You can compose those same functions directly when your application needs to name the constituents.

<!-- example: server -->
```rust
use stackpress_framework_session::composite;
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

pub fn grouped(app: &mut ServerBuilder) -> Result<()> {
    app.plugin("session", composite::register)?;
    Ok(())
}
```

The [compiled composite implementation](../../src/composite.rs) shows the three standalone calls: auth::register, the email plugin registrar, then identity plugin::register. Calling them directly requires an explicit email dependency; the existing plugin consumer needs only the composite. Choose one form for a builder, preserving constituent order without nested plugin selection. [Composite tests](../../tests/main/composite.rs) compare missing-config modes, listener order, real JSON email and multiple-handler Abort behavior.

## Place it in an application

The source main aggregate places session after views and before API/admin; CSRF is selected earlier. The [finite plugin consumer](../../consumers/plugin/src/main.rs) demonstrates that order with real registrars and SQLite. It installs tables, supplies native services, binds Hooks and shuts them down explicitly. Those actions are not side effects of registration.

Register `account::events::register` separately when using the included generated-style SQL event adapters, or provide your own event handlers. Add handlers to the same route/event with ordinary priorities; registration grants no ownership or conflict arbitration. Continue with [configuration](configuration.md), [hooks](hooks.md) and [API caller integration](../api/identity.md#caller-interface-for-api-and-other-handlers).
