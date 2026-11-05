# Registration and compiled catalog

Native registration is available when a runtime stage is selected on a non-WASM target. The default crate has no registrar/native dependency graph. See [feature lookup](features.md).

## Registrar functions

Use `register(&mut ServerBuilder) -> Result<()>` for an absent generated client. Use `register_with(&mut ServerBuilder, &'static CompiledClient) -> Result<()>` for a statically linked catalog. `Register` aliases the ordinary `fn(&mut ServerBuilder) -> Result<()>` callback shape.

| Parameter | Meaning |
| --- | --- |
| app | Mutable builder; selected stage contributions are queued in source order. |
| client | Static compiled catalog; configuration strings never load executable code. |

**Returns** `Ok(())` after registration, or the first existing constituent/callback error. Earlier builder mutations are not rolled back. Startup callback errors surface when building the host. `ServerBuilder::build` returns its distinct BuildError, so application examples use a boxed error result rather than assuming it converts into the handler's stackpress-lib Error. No implicit Engine, client generation or global guard is supplied.

This headless-graph example proves nullable and required catalog lookup remain distinct.

<!-- example: headless -->
```rust
use stackpress_framework::client::Client;
use stackpress_server::ServerBuilder;

pub async fn absent_client() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = ServerBuilder::new();
    stackpress_framework::register(&mut app)?;
    let server = app.build().await?;
    let client = server.host().service::<Client>()?;
    assert_eq!(client.module(), "stackpress-client");
    assert!(client.get(true)?.is_none());
    assert!(client.get(false).is_err());
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

An empty supplied catalog instead makes `get(false)` succeed with `Some`; it does not mean missing. The [compiled catalog guide](../guides/registration.md) shows a present catalog and a named plugin wrapper.

## Catalog fields and invocation

Callbacks are invoked once per model entry per aggregate registration when their owning feature is selected. Repeated catalog entries remain repeated contributions; no hidden deduplication or route arbitration is added.

| Type / field | Contract |
| --- | --- |
| CompiledClient.bindings | FrameworkBindings with optional session/API Register callbacks; Default contains None. Empty callbacks retain existing default registrars. |
| CompiledClient.models | Static ordered slice of CompiledModel entries. |
| CompiledClient.build | Static build identity for inspection, not an import path. |
| CompiledClient.describe | fn() -> Vec<framework_schema::Model>; evaluated explicitly, not during startup. |
| CompiledModel.name | Static application model identity. |
| CompiledModel.listen | Optional Register, called during SQL composition in catalog order; queue actual listeners for Listen. |
| CompiledModel.admin | Optional Register, called in the final admin stage in catalog order. |

Account's `account::events::register_listen` queues the same seven handlers as its immediate `register`. This preserves SQL command → model events → session/API event ordering. Missing callbacks stay `None`; the aggregate supplies no replacement admin implementation.

## Client service and direct constituents

`client::register(app, Option<&'static CompiledClient>)` publishes Client in Config at priority 10 and queues schema command registration. Call `config::register` first in direct composition so its typed roots exist. `Client::module() -> &str` defaults to `stackpress-client`; `get(nullable: bool) -> Result<Option<&'static CompiledClient>>` returns the supplied catalog or the required-lookup error shown above.

The [direct consumer](../../consumers/support/native.rs) demonstrates all ordinary contribution functions. Registering the aggregate and the same constituent twice preserves native duplicate-section errors; G42's multiple-handler allowance is not a promise that duplicate configuration registrations succeed. Evidence: [catalog tests](../../tests/cases/catalog.rs), [phase tests](../../tests/cases/mod.rs), [duplicate test](../../tests/cases/closeout.rs).
