# Bind command execution

Command names are registered during Listen, but registration performs no command IO. Supply compiled handlers through `commands::Bindings` for the native actions you want to execute. This example deliberately returns serve options instead of opening a socket.

## Publish a finite binding

Use the full consumer graph. Publish the binding as a shared service in Config, before an explicit command invocation.

<!-- example: full -->
```rust
// Generate this application module from its Idea imports before compiling.
#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "models/mod.rs"]
mod models;
use stackpress_framework::commands::Bindings;
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use std::sync::Arc;

async fn inspect_options(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&req.data.to_json()?)?;
    Ok(Flow::Continue)
}

async fn bindings(start: &mut StartupContext) -> Result<Flow> {
    let mut bindings = Bindings::default();
    bindings.bind("serve", inspect_options);
    start.register(Arc::new(bindings))?;
    Ok(Flow::Continue)
}

static CLIENT: stackpress_framework::CompiledClient = stackpress_framework::CompiledClient {
    bindings: stackpress_framework::FrameworkBindings {
        session: Some(
            stackpress_framework_session::composite::register_with::<models::stores::family::Models>,
        ),
        api: Some(stackpress_framework_api::register_with::<models::stores::family::Models>),
    },
    build: "docs-application",
    describe: Vec::new,
    models: &[],
};
fn application(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework::register_with(app, &CLIENT)
}

pub async fn finite_serve() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = ServerBuilder::new();
    application(&mut app)?;
    app.on_config(bindings)?;
    let server = app.build().await?;
    let reply = server.resolve("serve", serde_json::json!({})).await?;
    assert_eq!(
        reply.response.results.unwrap().to_json()?,
        serde_json::json!({"host":"127.0.0.1","port":3000})
    );
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

The provider receives owned command options, not a request to select executable code from configuration. `bind` stores a handler and does not call it. Missing bindings fail at invocation with `native command binding is unavailable`; there is no implicit listener or console fallback.

## Select the right provider

The event wrappers preserve source input, skip, status and error rules. Their direct functions remain independently usable.

| Work | Supply |
| --- | --- |
| serve / develop | Bindings handlers owning the actual listener/process/watcher lifecycle. The documentation fixture binds finite inspection only. |
| emit / query output | An `output` binding; emit passes pretty response text, query passes an owned native QueryResult. |
| generate | A `generate` binding, optionally the compiled Rust build-host provider. |
| SQL lifecycle | Explicit Engine, compiled Scripts and lazily required revision History. |
| populate | Configured event/data pairs; no Engine/client is needed. |
| view build | BuildProvider and Catalog; optional supervised Packaging. |

`serve`, `emit`, `generate` and SQL events skip an earlier nonzero/non-200 status. `develop` and view `build` deliberately do not use that skip. Do not add a common precheck that changes these differences. The [command reference](../api/commands.md) lists each command's exact contract; the [SQL reference](../api/sql.md) covers transactions and revision providers.
