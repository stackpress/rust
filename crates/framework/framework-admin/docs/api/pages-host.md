# Generated native page host

`codegen::emit_pages` emits a separate `ui/pages_host.rs` for an application's explicit model
selection. Include it as `crate::generated::pages_host` only in the native host. Pure page/role
adapters stay in shared UI. Generic `stackpress-framework-admin` and `stackpress-leptos` have
no runtime plugin.

## Generated entry points

| Entry point | Contract |
| --- | --- |
| `Mount::new(value: &str) -> stackpress_lib::Result<Mount>` | Validates the native page mount before registration; `as_str(&self) -> &str` borrows it |
| `register(app: &mut ServerBuilder) -> stackpress_lib::Result<()>` | Config claims the singleton and supplies SQL Contributions; Route mounts the existing selected mappings |
| `operations(start: &mut StartupContext, marker: &mut operations::Registration) -> stackpress_lib::Result<()>` | Direct compiled generated operation families, sharing the caller's marker |
| `routes(app: &mut ServerBuilder, mount: &str) -> stackpress_lib::Result<()>` | Direct fixed mappings beneath the caller's previously validated mount |

The generator checks at most 128 unique selected model names. Removed models contribute no
mappings; an empty effective selection emits no Mount or registrar. Applications choose the
models and operation capabilities at generation time. Config/path strings cannot import code.

Config requires Mount and `stackpress_framework_sql::plugin::Service`, claims its own singleton,
and publishes SQL Contributions containing the direct generated operations function. SQL must
precede it. Listen registers operations through SQL; Route then adds the same existing page/API
mappings as `routes`. No database setup or listener starts. Missing Mount/SQL selection, wrong
order and duplicate claims fail through startup errors before a Server is published.

## Migrate a native application

Replace the old operations Listen task and direct `pages_host::routes` call with SQL selection
followed by generated-page selection. Register the validated Mount in Config and use that same
value in the application's page/asset configuration. Retain the existing handlers, operation
policy, transport/session/CSRF admission, error lifecycle and asset bundle.

The checked-in finite example provides that application-specific configuration. Its public
helper accepts an already opened Engine and explicit policy; it selects both plugins:

```rust
use std::sync::Arc;
use generated_admin_web::host;
use stackpress_framework_sql::operations::OperationPolicy;
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;
use stackpress_sql::Engine;

pub fn configured_app(
    engine: Arc<Engine>,
    policy: Arc<dyn OperationPolicy>,
    config: host::Config,
    origin: String,
) -> Result<ServerBuilder> {
    let mut app = ServerBuilder::new();
    host::configure(&mut app, engine, config, policy, origin)?;
    Ok(app)
}
```

`host::configure` validates origin, mount, token and transfer bounds before contributing
startup tasks. Its fixture principal/token demonstrate finite proofs; supply production
session and authentication integration in a real app. SQL policy still runs for each action,
including reads and transfer. Protected handlers enforce their own authority.

## Compose operation families directly

The generated registrar owns the singleton Contributions service. For a custom operation
composite, supply Engine and Policy in Config, then compose SQL with one Contributions list.
Select the following function once with `app.plugin("application-data", application_data)`.
It registers real generated operations; add other compiled family callbacks to the same list:

```rust
use generated_admin_web::generated::pages_host;
use stackpress_framework_sql::plugin::{self, Contributions};
use stackpress_lib::{Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

async fn operation_families(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(Contributions {
        // Add other compiled family functions here; SQL shares one marker across them.
        operations: &[pages_host::operations],
    }))?;
    Ok(Flow::Continue)
}

pub fn application_data(app: &mut ServerBuilder) -> Result<()> {
    plugin::register(app)?;
    app.on_config(operation_families)?;
    Ok(())
}
```

To mount pages directly, the native application calls its own generated
`pages_host::routes(app, mount.as_str())` after constructing Mount and supplying its handler
configuration. Derive that configuration and asset mount from the same checked value. Those
routes call the application's `crate::host` handlers, whose setup remains application-owned.
The finite example's public `host::configure` supplies its private host state and selects the
default SQL/pages pair. Direct operation registration needs only its Engine and Policy.

Several handlers may share these routes/events, ordered by priority then registration order.
A handler can stop the chain with `Flow::Abort`. Developers decide how overlaps cooperate
and keep protected-handler policy explicit.

See [runtime selection and direct alternatives](../../../framework-sql/docs/guides/select-plugins.md)
and [build-time helpers](../../../framework-schema/docs/README.md). Generate from the template;
never hand-edit the emitted host. Public response, locale/theme, mount/assets and lifecycle
contracts remain with the existing page and transport implementations.
