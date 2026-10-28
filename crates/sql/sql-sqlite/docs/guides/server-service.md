# Register an Engine service

When several operations need SQLite, open one Engine during server Config and register it
as an `Arc`. Handlers borrow that service through Invocation. This keeps file selection at
startup and gives the application one explicit resource to close.

## Open during Config

The following example creates an empty temporary file as explicit setup, then opens it
read-only during server startup. Its named handler returns a bound greeting without creating
schema or exposing a generic SQL operation.

```rust
use std::sync::Arc;

use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{ServerBuilder, ServerInvocationExt, StartupContext};
use stackpress_sql::{DialectId, Engine, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn greet(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let name = req.data.path_as::<String>("name")?.unwrap_or_default();
    let rows = inv
        .service::<Engine>()?
        .fetch(&Sql::literal(DialectId::Sqlite, "SELECT ? AS name").bind(name))
        .await
        .map_err(Error::from_source)?;
    let name = rows[0]
        .try_get::<String>("name")
        .map_err(Error::from_source)?;
    res.results(&name)?;
    Ok(Flow::Continue)
}

async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let options = start.config::<SqliteOptions>()?.clone();
    let native = SqliteDatabase::open(options)
        .await
        .map_err(Error::from_source)?;
    start.register(Arc::new(Engine::new(native)))?;
    Ok(Flow::Continue)
}

fn validate(options: &SqliteOptions) -> Result<()> {
    options.validate().map_err(Error::from_source)
}

async fn example() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let path = root.path().join("greeting.sqlite");
    // This is explicit setup, before startup. Ordinary Config never creates the file.
    let setup = Engine::new(
        SqliteDatabase::open(SqliteOptions::file(&path).create_if_missing(true)).await?,
    );
    setup.close().await?;

    let mut app = ServerBuilder::new();
    app.config_section(
        "sqlite",
        SqliteOptions::file(&path).read_only(true),
        validate,
    )?;
    app.on_config(configure)?;
    app.on("greet", greet)?;
    let server = app.build().await?;
    let reply = server
        .call("greet", serde_json::json!({"name": "Ada"}))
        .await?;
    assert_eq!(
        reply.response.result().and_then(|value| value.as_str()),
        Some("Ada")
    );

    // Stop scheduling calls, then await resource cleanup before runtime shutdown.
    server.host().shared_service::<Engine>()?.close().await?;
    Ok(())
}
```

`SqliteOptions::clone()` copies configuration, not an open pool. Config publishes the exact
`Arc<Engine>` allocation, and `service::<Engine>()` borrows it without cloning the Engine.
`shared_service` retains ownership explicitly for shutdown. This example requires
`stackpress-lib`, `stackpress-server` and `serde_json` in addition to the
[quick-start dependencies](../getting-started.md#prepare-a-consumer).

## Keep policy in the application

Services are Send + Sync + 'static and registration is permitted during Config. For several
databases, choose application-owned names with `register_named` and `service_named`.
Registration conflicts are startup errors; read-only policy should be fixed by application
code when configuration must not enable writes.

SqliteOptions validates finite settings, but read-only open alone does not prove the required
schema exists. The [profile database plugin](../../../../../examples/sql-profile/src/database.rs)
performs a qualified `LIMIT 0` projection before publishing its service and explicitly closes
on verification failure. It rejects missing schema without installing it.

## Compose transactions explicitly

A handler that starts a transaction must pass `&mut Transaction` to participating helpers.
Nested dispatch and service lookup do not inherit it. Calling the shared Engine while a
transaction holds the only pool slot can time out; adding connections does not make those
separate calls atomic.

You now have a startup-owned Engine and a named operation that borrows it. Use the
[server services guide](../../../../server/server/docs/guides/services.md) for registry rules and
the [SQL core transaction guide](../../../sql/docs/guides/transactions.md) for transaction methods. The
[profile walkthrough](sql-profile.md) adds explicit installation, schema verification and
public DTOs to this pattern.
