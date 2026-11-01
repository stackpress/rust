# Bind command execution

Command names are registered during Listen, but registration performs no command IO. Supply compiled handlers through `commands::Bindings` for the native actions you want to execute. This example deliberately returns serve options instead of opening a socket.

## Publish a finite binding

Use the full consumer graph. Publish the binding as a shared service in Config, before an explicit command invocation.

<!-- example: full -->
```rust
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

pub async fn finite_serve() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut app = ServerBuilder::new();
    stackpress_framework::register(&mut app)?;
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
