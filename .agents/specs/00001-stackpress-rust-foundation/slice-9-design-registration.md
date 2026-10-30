# Slice 9 registration DX with source activation gates

Status: Revised design-only sketch. [G43](slice-9-design-decisions.md) governs activation; #14 [SQL](../../../crates/framework/framework-sql/docs/guides/select-plugins.md) and [presentation](../../../crates/framework/framework-view/docs/presentation-host.md) govern ordinary function selection/composition. Proposed new APIs are not compiled or released.

## Select the source session composite

The original session plugin groups auth, email and identity in that order. The Rust composite does the same with ordinary constituent functions; the application selects only the outer function.

```rust
use stackpress_lib::Result;
use stackpress_server::{ConfigLayers, ServerBuilder};
use stackpress_framework_session as session;

fn application(layers: ConfigLayers) -> Result<ServerBuilder> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("session", session::plugin::register)?;
    Ok(app)
}

// Proposed implementation responsibility, not production code at this checkpoint.
fn session_composite(app: &mut ServerBuilder) -> Result<()> {
    session::auth::register(app)?;
    stackpress_framework_email::plugin::register(app)?;
    session::session::register(app)?;
    Ok(())
}
```

No nested app.plugin call occurs in the selected callback. The package exposes the equivalent composite as plugin::register and independent constituent registrars/direct mechanisms. Features select compiled availability, configuration activates source contributions; config does not dynamically import code. Selecting root session does not require a nonempty section or install a new global policy.

## Example data follows source defaults

```json
{
  "session": {
    "key": "session",
    "seed": "application-seed",
    "access": {
      "GUEST": [{"method": "GET", "route": "/auth/**"}],
      "USER": ["profile-*", {"method": "ALL", "route": "/account/**"}]
    }
  },
  "auth": {
    "base": "/auth", "redirect": "/", "roles": ["USER"],
    "menu": [], "password": {}
  }
}
```

The seed above is illustrative source config, not a deployed credential. No email config means the email plugin is inactive. Remove session entirely to deactivate its identity subplugin; remove only access to omit the global request guard; set access={} to install a guard whose authorize call returns true. Remove auth to omit auth/account routes, while root auth's Listen events remain. No ttl_seconds, empty_access, auth.methods or required Origin is added.

The source CSRF plugin is selected independently earlier in aggregate order; it exposes defaults even without a csrf section. Configure csrf.name/error only when overriding those defaults. Configure email with the original transport variant/options and auth.email sender when the application wants the email flow to send. Missing sender/transport produces the original runtime no-op, not a selected-but-missing startup error.

## Explicit native resources without a new activation policy

The application supplies an Engine when operations need it; the default session JWT path needs none. The sketch follows existing captured-resource startup style, without changing when a disabled/unused source process resolves dependencies.

```rust
use std::sync::Arc;
use stackpress_lib::{Flow, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::Engine;

fn database_resources(app: &mut ServerBuilder, engine: Arc<Engine>) -> Result<()> {
    app.on_config(task(move |start: &mut StartupContext| {
        let engine = engine.clone();
        Box::pin(async move {
            start.register(engine)?;
            Ok(Flow::Continue)
        })
    }))?;
    Ok(())
}
```

Real SQL adapter construction/setup is explicit and in full scope; registrar startup never opens a listener. Do not require Engine/SMTP/strong secret/renderer at startup solely because a function was selected if the source process would remain disabled or resolve it only on invocation. Existing builder duplicate singleton/section rules are Rust registration mechanics; they are not global route arbitration or a new per-request protection model.

## Phase and source-gate table

| Constituent | Config / registration | Listen | Route |
| --- | --- | --- | --- |
| root session composite | Call auth→email→session functions | Existing callbacks in registration order | Existing callbacks in registration order |
| session subplugin | If no truthy session config: contribute nothing. Otherwise configure Service with key/seed/access defaults | me, authorize; request guard only if access field is present/truthy as source | Account handlers/views only if auth config exists |
| auth subplugin | No new mandatory security/service precondition | auth-signup, auth-signin, auth-signout regardless of auth Route config | All original auth routes/views only if auth config exists |
| CSRF | Selected plugin registers default/configured mechanism | No automatic guard | No routes automatically protected |
| email | No email config: no contribution | Only truthy configured email installs email-send emitter/listener | None |

Typed optional config is decoded before callbacks, but adapters retain missing-versus-present-empty semantics and extension maps. No dynamic import becomes an arbitrary runtime code loader; compiled functions occupy those source import sites. Standalone direct APIs can be used with no plugin or ServerBuilder.

Several handlers can serve one route/event. Preserve priorities/FIFO and any explicit Abort; a normal source return remains Continue. Validate a contribution's own inventory only where existing registration mechanics require it, never reserve routes or inject authorization into another process. App authors can compose additional guards/policies using the same mechanisms. G43 does not ask the framework to decide which routes must be protected.
