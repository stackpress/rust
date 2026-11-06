# Register compiled models

Generated operations need to join the same startup sequence as the framework. Supply a static catalog to `register_with`; wrap it in a named registrar when using `ServerBuilder::plugin`. The following full-consumer example contributes real account events and a small application-owned admin route.

## Supply a catalog

The model callback queues account handlers for Listen. The admin callback queues a Route callback, so its route participates in the normal handler chain.

<!-- example: full -->
```rust
use stackpress_framework::{CompiledClient, CompiledModel};
use stackpress_framework_schema::Model;
use stackpress_framework_session::columns::SourceSchema;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};

async fn admin_ready(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"admin ready")?;
    Ok(Flow::Continue)
}

async fn admin_routes(start: &mut StartupContext) -> Result<Flow> {
    start.get("/docs-admin", admin_ready)?;
    Ok(Flow::Continue)
}

fn admin(app: &mut ServerBuilder) -> Result<()> {
    app.on_route(admin_routes)
}

fn describe() -> Vec<Model> {
    vec![stackpress_framework_session::profile::ProfileSchema::describe()]
}

static CLIENT: CompiledClient = CompiledClient {
    bindings: stackpress_framework::FrameworkBindings {
        session: None,
        api: None,
    },
    build: "docs-account-v1",
    describe,
    models: &[CompiledModel {
        name: "account",
        listen: Some(stackpress_framework_session::account::events::register_listen),
        admin: Some(admin),
    }],
};

pub fn application(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework::register_with(app, &CLIENT)
}

pub async fn catalog_host() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = ServerBuilder::new();
    app.plugin("framework", application)?;
    let server = app.build().await?;
    let patterns: Vec<_> = server
        .listeners()
        .map(|item| item.pattern.as_str())
        .collect();
    assert!(
        patterns.iter().position(|p| *p == "upgrade")
            < patterns.iter().position(|p| *p == "auth-search")
    );
    assert!(patterns.contains(&"GET /docs-admin"));
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

This is a finite registration example, not a storage implementation: it never executes an account SQL action. Replace the small admin registrar with your generated `pages_host` registrar when integrating a real schema. The [full consumer](../../consumers/support/native.rs) demonstrates that generated registrar with actual SQLite and application policy.

## Use direct constituents

Call ordinary registrars directly when you need explicit composition. Keep the source order when parity is the goal, including session's composite and admin last. The [direct consumer implementation](../../consumers/support/native.rs) calls config, server commands, client, language, CSRF, SQL/model callbacks, view, session, API and generated admin individually; its inventory is compared with the aggregate.

Direct source adapters for server/schema/language/view live in this crate, so that consumer still imports the aggregate crate while bypassing `register_with`. Independent CSRF/session/API registrars remain in their own crates. Do not select the aggregate and then re-register those same constituents: existing duplicate-section errors are preserved, with no rollback of earlier builder mutations. See [registration lookup](../api/registration.md).

## Generate an extended source client

The build-host-only `commands::source_client::emit` accepts SourceGraph, source UI
definitions, Options and OutputPlan. One transform owns source/stores/ui/client. Options
selects compiled session/API registrar paths and an application admin callback; missing
bindings remain absent in emitted code. A supplied CompiledClient's empty FrameworkBindings
retains the aggregate's generated-default session/API registrars.

The [maintained source input](../../consumers/build-host/source.idea) imports
stackpress/stackpress.idea and appends Profile.nickname. [Build-host](../../consumers/build-host/src/main.rs)
emits its native schemas/stores/client separately from portable UI. Its generated family
reaches [real auth/CRUD/joins/admin/export handlers](../../consumers/support/source_family.rs).
Schema generation opens no database; the application explicitly installs or migrates any
extension DDL. Catalog order follows G19 composition; SQL model listeners precede session,
API and final generated admin callbacks. Existing Typed generation is unchanged.
