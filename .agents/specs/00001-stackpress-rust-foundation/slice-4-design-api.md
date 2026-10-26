# Slice 4 proposed API and DX

Companion to [decisions](slice-4-design-decisions.md). Load for public shape, standalone execution, service integration and generated-store direction. Every SQL symbol below is a **design sketch**, not an existing or compile-verified Stackpress SQL API. Existing ServerBuilder/StartupContext/Invocation calls match current source and the disposable host probe. Freeze names/signatures in M0 with real types, doctests and compile failures.

## Crates and dependency direction

`stackpress-sql` owns typed query/schema data, expressions, dialect rendering, errors/rows, driver contracts and Engine. It depends on no host/Idea/driver/runtime. `stackpress-sql-sqlite` depends on core and SQLx/Tokio, owns native options/pools/decoding and implements core contracts. Applications choose registration and operation exposure. The default root facade remains driver-free; opt-in `sql` and `sql-sqlite` features expose `stackpress::sql` and `stackpress::sql_sqlite`.

Under recommended S4-D02, no framework-sql manifest is created yet. S4-F will own normalized metadata→stores/actions/events, with build-time generator dependencies isolated from runtime. A standalone query must not require ServerBuilder or an Idea schema.

## Core shape for implementers

The following signatures show the object boundary. The complete public types require Rustdoc and real M0 compilation; error aliases/argument types are omitted only where evident.

```rust
pub type SqlFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, SqlError>> + Send + 'a>>;

pub trait Database: Send + Sync + 'static {
    fn dialect(&self) -> DialectId;
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    fn begin(&self, options: TransactionOptions)
        -> SqlFuture<'_, Box<dyn TransactionSession>>;
    fn close(&self) -> SqlFuture<'_, ()>;
}

pub trait TransactionSession: Send + 'static {
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>;
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()>;
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()>;
}
```

Engine owns a Box<dyn Database>, limits and immutable observation policy. It is Send+Sync, not Clone. `Arc<Engine>` is deliberate shared pool ownership. `Transaction` wraps a boxed session and the same query policy; it is Send, not Clone, only exposes query work through `&mut self` and consumes itself on commit/rollback. No unsafe lifetime widening or reference-to-static conversion is permitted. No Sync guarantee is needed for the session.

`QueryResult` contains owned ordered rows, affected count and optional statement-local insert ID. A row retains column metadata/order; named lookup errors on duplicate names instead of silently overwriting a join column. `try_get<T>` is fallible and validates NULL/range/type; DTO decoding is explicit. Core SqlValue is not lib Value and carries no SharedNative or arbitrary native payload.

Detached builders offer fluent owned methods and `build()` inspection. `render(DialectId)` returns a validated `QueryPlan`, containing one DML query or an ordered DDL batch. Rendering never performs IO. `Engine::fetch` returns owned rows for a single statement; `execute` returns its outcome; `execute_plan` executes a whole validated transactional plan. Arbitrary SQL batches and mixed dialects are rejected at these boundaries.

## Build, inspect and execute standalone

This proposed native program creates its own temporary/local database explicitly. Creating a file is a caller option, never a fallback after a failed network connection.

```rust
use stackpress_sql::{DialectId, Engine, Order, Select};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn search() -> Result<(), Box<dyn std::error::Error>> {
    let native = SqliteDatabase::open(
        SqliteOptions::file("profiles.sqlite").create_if_missing(true),
    ).await?;
    let database = Engine::new(native);

    // An owned query can be inspected independently of any connection.
    let query = Select::new(["id", "name"])
        .from("profile")
        .where_eq("name", "O'Reilly")
        .order_by("id", Order::Asc)
        .limit(20);
    let plan = query.render(DialectId::Sqlite)?;
    println!("{}", plan.queries()[0].sql());
    // SQL contains a placeholder; the separate bind contains O'Reilly.
    let rows = database.fetch(&query).await?;
    for row in rows {
        println!("{}", row.try_get::<String>("name")?);
    }
    database.close().await?;
    Ok(())
}
```

Table installation is a separate explicit operation using Create + execute_plan, not a side effect of search/open. Trusted raw literals use `Sql::literal(DialectId::Sqlite, "SELECT name FROM profile WHERE id = ?").bind("abc123")`; dynamic trusted SQL requires a visibly named `TrustedSql::audited` constructor. These are not request-data constructors. The adapter prepares/checks argument arity on the same lease before execution.

## Function plugin and typed host service

A database is a service, a function plugin constructs/registers it, and a capability performs business work. These are complementary roles. SQL configuration cannot activate a compiled-out backend. No ServerBuilder changes or ambient transactions are needed.

```rust
use std::sync::Arc;
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result};
use stackpress_server::{
    BuildResult, ConfigLayers, PluginSpec, Server, ServerBuilder,
    ServerInvocationExt, StartupContext,
};
use stackpress_sql::{Engine, Order, Select};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

fn database_plugin(app: &mut ServerBuilder) -> Result<()> {
    app.config_section(
        "sqlite",
        SqliteOptions::file("profiles.sqlite"),
        validate_database,
    )?;
    app.on_config(open_database)?;
    Ok(())
}

fn validate_database(options: &SqliteOptions) -> Result<()> {
    options.validate().map_err(Error::from_source)
}

async fn open_database(start: &mut StartupContext) -> Result<Flow> {
    // The config type is ordinary owned data; this clone never clones a pool.
    let options = start.config::<SqliteOptions>()?.clone();
    let native = SqliteDatabase::open(options).await.map_err(Error::from_source)?;
    start.register(Arc::new(Engine::new(native)))?;
    Ok(Flow::Continue)
}

#[derive(serde::Serialize)]
struct Profile { id: String, name: String }

async fn profile_search(
    req: &mut Request,
    res: &mut Response,
    inv: Invocation<'_>,
) -> Result<Flow> {
    let id = req.data.path_as::<String>("id")?
        .filter(|id| !id.is_empty())
        .ok_or_else(|| Error::invalid("profile search requires an id"))?;
    let query = Select::new(["id", "name"]).from("profile")
        .where_eq("id", id).order_by("id", Order::Asc).limit(100);
    let rows = inv.service::<Engine>()?.fetch(&query).await.map_err(Error::from_source)?;
    let profiles = rows.into_iter().map(|row| -> stackpress_sql::Result<Profile> {
        Ok(Profile { id: row.try_get("id")?, name: row.try_get("name")? })
    }).collect::<stackpress_sql::Result<Vec<_>>>().map_err(Error::from_source)?;
    res.rows(&profiles)?;
    Ok(Flow::Continue)
}

fn profiles_plugin(app: &mut ServerBuilder) -> Result<()> {
    app.on("profile-search", profile_search)?;
    Ok(())
}

async fn application(layers: ConfigLayers) -> BuildResult<Server> {
    let mut app = ServerBuilder::new();
    app.config_layers(layers)?;
    app.plugin("database", database_plugin)?;
    app.plugin_with(PluginSpec::new("profiles").requires(&["database"]), profiles_plugin)?;
    app.build().await
}
```

SqliteOptions is proposed as serde config data with unknown-field rejection and finite budgets; opening/validation are adapter-owned. The file above intentionally does not seed/install on startup. Tests/operator setup own those explicit effects. Missing file/create permission produces BuildError, not an empty in-memory database. Named instances can use existing `register_named`/`service_named` with app-owned names/types.

HTTP adds an app-owned route delegating to profile-search through the existing buffered adapter. CLI calls the same capability through `stackpress_cli::main_with(application)`. The new example should preserve the prior proof's exact case-sensitive id-prefix/insertion-order semantics using an escaped LIKE or explicit prefix expression plus stable ordinal, rather than silently treating this sketch's equality query as parity. The coordinator chooses that example's final contract before implementation.

Explicit shutdown is app-owned: retain `server.host().shared_service::<Engine>()?`, stop new app work, await `database.close()` within the caller's deadline, then drop owners. Current main_with does not expose an async resource-finalization hook; the ordinary CLI path gets documented RAII fallback. An application requiring awaited shutdown uses `run`/its own entry wrapper. A universal server shutdown facility is a separate G23 request, not required here.

## Transaction use and future generated Profile store

These are proposed call shapes. The store is a sketch for S4-F, not a planned general generator in this slice.

```rust
async fn rename(database: &Engine) -> stackpress_sql::Result<()> {
    let mut tx = database.begin().await?;
    // &mut tx keeps all statements on this transaction's one leased connection.
    let store = ProfileStore;
    let update = store.rename("abc123", "Ada");
    tx.execute(&update).await?;
    tokio::task::yield_now().await;
    let rows = tx.fetch(&store.by_id("abc123")).await?;
    assert_eq!(rows.len(), 1);
    tx.commit().await
}

struct ProfileStore;
impl ProfileStore {
    fn by_id(&self, id: &str) -> Select {
        Select::new(["id", "name"]).from("profile").where_eq("id", id)
    }
    fn rename(&self, id: &str, name: &str) -> stackpress_sql::Update {
        stackpress_sql::Update::new("profile").set("name", name).where_eq("id", id)
    }
}
```

Future generated stores build queries and decode model values; executable actions accept `&Engine` or `&mut Transaction` explicitly. They never close over the pool and silently leave the transaction. Future schema-generated `create()` supplies an inspectable TableDefinition; generic filters/relations/validation/soft-delete/migrations are the S4-F/M decisions in the inventory. No authorization, retry, implicit commit on handler success, or rollback on Flow::Abort is inferred: application code chooses when to commit, and cancellation/drop is documented separately.
