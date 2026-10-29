# Select framework integrations

Select compiled functions with `ServerBuilder::plugin` or `plugin_with`. Features make an
entry point available; the application selects it before `build`. Configuration cannot
load code or enable a disabled integration. The existing dispatcher and lifecycle remain
in charge. There is no plugin trait or second dispatcher.

## Choose features and responsibilities

Keep runtime and generator dependencies in separate Cargo packages. Defaults remain unchanged.

| Capability | Explicit feature and entry point | Application supplies |
| --- | --- | --- |
| Preferences and flash | view `presentation`; `stackpress_framework_view::plugin::register` | `view_preferences`, `view_flash`, explicit Admission before Config, pages and assets |
| Preferences only / flash only | view `web-preferences` / `web-flash`; each module's `register` | The corresponding typed section and, for preferences, Admission |
| Native SQL operation families | framework-sql `server`; `stackpress_framework_sql::plugin::register` | An existing Engine, Policy, one Contributions service, compiled operation selection |
| Generated native admin pages | Explicit native include of `ui/pages_host.rs`; `crate::generated::pages_host::register` | Nonempty generated model selection, Mount, SQL plugin, handler config, policy and transport admission |
| Build-host transforms | schema/SQL/admin `codegen`; their `codegen::register` helpers | Normalization, definitions, diagnostics, emitters, selected models/operations and output publication |

The SQL registrar is native-only even when `server` is enabled on a browser target.
The generated page host belongs in the native app, outside the shared/browser module tree.
Pure schema/language contracts, generic shared admin and `stackpress-leptos` remain support
libraries with no runtime plugin. Native preference persistence and Tauri operation adapters
remain explicit application choices. A presentation feature selects no renderer.

## Migrate SQL Listen wiring

A host that called each generated operation registrar from its own Listen task can supply
those same functions as one static inventory. The functions still receive the existing
`StartupContext` and one shared `operations::Registration` marker. Each callback chooses its
own generated model, event prefix and operation list; no configuration string selects code.

This function accepts resources already owned by the application. It performs no database IO:

```rust
use std::sync::Arc;
use stackpress_framework_sql::{
    operations::Policy,
    plugin::{self, Contributions, RegisterOperations},
};
use stackpress_lib::{Flow, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::Engine;

pub fn selected_host(
    engine: Arc<Engine>,
    policy: Arc<Policy>,
    operations: &'static [RegisterOperations],
) -> Result<ServerBuilder> {
    let mut app = ServerBuilder::new();
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        let policy = policy.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(policy)?;
            start.register(Arc::new(Contributions { operations }))?;
            Ok(Flow::Continue)
        })
    }))?;
    app.plugin("data", plugin::register)?;
    Ok(app)
}
```

Use 1–128 callbacks. Config verifies Engine and Policy and publishes an opaque SQL Service.
Listen validates the complete count before running callbacks in inventory order. It never
opens an Engine, installs/migrates/seeds a database, adds routes/config sections, or starts a
listener. Applications own database setup and shutdown separately from plugin startup.

## Continue using direct APIs

Direct generated registrars remain first-class. Keep a single `Registration` across the
families selected for that host. This alternative requires no SQL plugin, SQL Service or
Contributions service; it deliberately retains the caller's existing inventory rules:

```rust
use std::sync::Arc;
use stackpress_framework_sql::{
    operations::{Policy, Registration},
    plugin::RegisterOperations,
};
use stackpress_lib::{Flow, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::Engine;

pub fn direct_host(
    engine: Arc<Engine>,
    policy: Arc<Policy>,
    operations: &'static [RegisterOperations],
) -> Result<ServerBuilder> {
    let mut app = ServerBuilder::new();
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        let policy = policy.clone();
        Box::pin(async move {
            start.register(engine)?;
            start.register(policy)?;
            Ok(Flow::Continue)
        })
    }))?;
    app.on_listen(task(move |start: &mut StartupContext| {
        Box::pin(async move {
            let mut registration = Registration::default();
            for register in operations {
                register(start, &mut registration)?;
            }
            Ok(Flow::Continue)
        })
    }))?;
    Ok(app)
}
```

Direct stores, public DTOs, view page registration and pure preferences/flash APIs need no
runtime plugin. The callback type alias above is only a function signature, not a selection.
See [direct generated operations](register-operations.md) for operation validation and policy.

## Compose and order integrations

A composite calls the ordinary registration functions and queues its application callbacks.
Select only the outer composite. Calling `app.plugin` inside its registration function fails
because selection has already been sealed. All nested contributions retain the composite's
plugin identity. See the [presentation example and custom composite](../../../framework-view/docs/presentation-host.md).
To migrate old preference-plus-flash wiring, replace both selected callbacks with that
composite, retaining the same typed sections, Admission, pages, assets, locale and theme setup.

Config, Listen, then Route run before publication. Required services must exist before their
consumer's Config callback; select prerequisite plugins first. `PluginSpec::requires` checks
selected identities and order before callbacks, while typed service checks verify actual
resources. The generated admin Config requires SQL's opaque Service: Engine and Policy alone
are insufficient. See [generated page-host registration](../../../framework-admin/docs/api/pages-host.md).

Duplicate typed sections, singleton services and generated operation selections retain their
existing errors. Inventory-count validation precedes SQL callbacks. A failing callback stops
later callbacks and phases, and no partial Server is published. Earlier callback effects are
not rolled back; discard a builder after registration failure and keep callbacks confined to
registration. Synchronous contribution callbacks cannot be interrupted midway by dropping an
async build; dropping a suspended build drops its retained startup state without publishing it.

Several handlers may serve the same route or event. They run by priority, then registration
order; any handler can stop the chain with `Flow::Abort` (the Rust form of returning false).
Developers own route inventories and decide how overlaps cooperate. Integrations validate
their own declared paths/selections. Every protected handler must enforce its own policy;
route ordering and public UI hints grant no authority. Preferences Admission checks current
session/CSRF facts for preference writes; native admin transport admission and SQL operation
policy remain separate application obligations.

For build tools, use the [transform registration reference](../../../framework-schema/docs/README.md).
Those helpers select functions in an existing Idea Registry; runtime plugins do not invoke them.
