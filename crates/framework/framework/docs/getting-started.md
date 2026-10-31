# Handle your first aggregate event

An application can register the framework before it has a database, generated client or command provider. This example builds a finite event host and resolves one application event. It opens no listener.

## Build a small host

Use the dependencies and locked graph of the [full consumer](../consumers/full/Cargo.toml): the nine aggregate leaves, lib, server, session, SQL and Tokio. Put the following code in your application module and call `first_aggregate().await` from its async entry point.

<!-- example: full -->
```rust
use stackpress_lib::{Flow, Invocation, Request, Response, Result};
use stackpress_server::ServerBuilder;

async fn health(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"ready")?;
    Ok(Flow::Continue)
}

pub async fn first_aggregate() -> std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let mut app = ServerBuilder::new();
    app.plugin("framework", stackpress_framework::register)?;
    app.on("health", health)?;
    let server = app.build().await?;
    let reply = server
        .resolve("health", stackpress_lib::Nest::new())
        .await?;
    assert_eq!(reply.response.results.unwrap().as_str(), Some("ready"));
    assert!(server.host().service::<stackpress_sql::Engine>().is_err());
    server
        .host()
        .service::<stackpress_framework_session::auth::Hooks>()?
        .shutdown()
        .await;
    Ok(())
}
```

The assertion is your first success: the application event returns `ready` even though no Engine exists. `plugin` accepts the ordinary registrar, while the named async function handles the event. The last call closes the session hook lifecycle owned by this host; no authentication work was submitted.

## What the registration did

`register` selected the compiled stages and queued their startup callbacks with an absent client catalog. The schema service's nullable lookup remains absent; registration does not invent a generated client or execute a command. Missing session/access/auth configuration does not add a global guard or auth routes.

You now have a host that can resolve application events. Add [compiled model callbacks](guides/registration.md) when generated operations are available, or bind a [finite command provider](guides/commands.md) before emitting command events. The [concepts page](concepts.md) explains the ordering those additions retain.
