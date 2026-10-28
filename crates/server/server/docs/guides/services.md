# Sharing services

Create a resource once at startup when several operations need it. Register an Arc during Config,
then borrow it through Invocation when handling a call.

## Initialize before dispatch

This small service owns a greeting prefix. Config inserts it, and a capability reads it without
capturing the Server or using caller-controlled request data:

```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use std::sync::Arc;

struct Greetings(String);

async fn configure(start: &mut StartupContext) -> Result<Flow> {
    start.register(Arc::new(Greetings("Hello".into())))?;
    Ok(Flow::Continue)
}

async fn greet(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    res.results(&format!("{}, Ada", inv.service::<Greetings>()?.0))?;
    Ok(Flow::Continue)
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let mut app = ServerBuilder::new();
    app.on_config(configure)?;
    app.on("greet", greet)?;
    let server = app.build().await?;
    let reply = server.call("greet", serde_json::json!({})).await?;
    assert_eq!(
        reply.response.result().and_then(|value| value.as_str()),
        Some("Hello, Ada")
    );
    let retained = server.host().shared_service::<Greetings>()?;
    assert_eq!(retained.0, "Hello");
    Ok(())
}
```

`register` takes the exact Arc allocation and rejects duplicate concrete types. `service` borrows
it; `shared_service` explicitly retains ownership past the host borrow. No payload Clone or
framework registry lock is involved.

## Customize deliberately

Use `register_named` and `service_named` for several instances, choosing a unique name across all
named types. Singleton and named slots are independent. During Config, `replace_service` replaces an
existing singleton and retains original/replacement sites; there is no named replacement method.
Listen and Route can read services but cannot insert or replace them.

Services must be Send + Sync + 'static. Put synchronization inside the resource when needed and keep
its locks away from awaits. Failed or cancelled build drops owned Arcs; asynchronous cleanup and
external effects remain your responsibility.

You now have a startup-owned resource available to operations. The canonical [profiles
service](../../../../../examples/capability-host/src/profiles.rs) combines this with settings. Use
[StartupContext](../api/startup.md) and [host lookups](../api/context.md) for lookup details.
