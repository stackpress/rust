# Slice 7 proposed API and author experience

Load with [decisions](slice-7-design-decisions.md). New names/signatures below
are proposals, not compilable APIs in the current checkout. Existing
ServerBuilder, function plugins, Engine and handler signatures are retained.
M0 must replace sketches with real production types and compile-check examples.

## Production application main

```rust
use std::{net::SocketAddr, sync::Arc};
use stackpress::lib::{Flow, Invocation, Request, Response, Result};
use stackpress::server::{ServerBuilder, ServerInvocationExt};
use stackpress::server_http::{HttpServer, NetworkOptions}; // proposed

async fn health(
    _: &mut Request, response: &mut Response, _: Invocation<'_>,
) -> Result<Flow> {
    response.json(&serde_json::json!({ "healthy": true }))?;
    Ok(Flow::Continue)
}

fn app(builder: &mut ServerBuilder) -> Result<()> {
    builder.get("/health", health)?;
    Ok(())
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Selected plugins complete configuration/startup before any socket bind.
    let mut builder = ServerBuilder::new();
    builder.plugin("app", app)?;
    let server = Arc::new(builder.build().await.map_err(|error| error.error)?);
    let options = NetworkOptions::http(
        "127.0.0.1:8080".parse::<SocketAddr>()?,
        "https://profiles.example/",
    );
    // This explicit operation binds in production; never run it directly for dev.
    let listener = HttpServer::bind(server, options).await?;
    // Library neither installs process signal handlers nor owns the runtime.
    let report = listener.serve_until(async { tokio::signal::ctrl_c().await }).await?;
    eprintln!("HTTP drain completed: {} exchanges", report.exchanges);
    Ok(())
}
```

`bind` validates config/assets/TLS and binds once; error retains native cause.
`serve_until` stops admission when the caller future resolves, drains within the
configured grace period, cancels/reaps overdue owned tasks and returns a drain
report. Fatal accept/stop errors are returned after cleanup. Process services
must be closed explicitly after this await. Linux production can select SIGTERM
in the application stop future; Ctrl-C is the portable illustration.

For repository development, the app reads its supplied port and is run only via
`devmetrics start --summary "production host review" -- '<app> --port {port}'`.
This is an example command, not a request to start a listener at design stage.

## Network and stream surfaces

| Proposed surface | Parameters and return |
| --- | --- |
| `NetworkOptions::http(address, public_origin)` / `https(address, origin, tls)` | Validated finite policies; trusted origin, allowed authorities, optional proxy policy, connection/request/body/deadline controls. Builder setters return must_use values. |
| `HttpServer::bind(Arc<Server>, NetworkOptions)` | Async Result of bound owner; default loopback, explicit HTTPS selection. Binding implies no automatic serve/background task. |
| `HttpServer::serve_io(io, peer, stop)` | Supplied Send AsyncRead+AsyncWrite IO, trusted peer metadata and stop future; duplex test seam, no bind. Same policy as real accepts. |
| `HttpServer::serve_until(stop)` | Own accept/supervision until shutdown/drain; returns `DrainReport`, not a detached JoinHandle. |
| `HttpRequestContext` via `Invocation::adapter_context` | Read-only peer/public-origin/cancellation metadata; no native writer. |
| `HttpResponseExt::stream(mime, stream, StreamLimits)` | Owned Send stream of Result<Vec<u8>>, guarded installation on an Open Response; returns Result<&mut Response>. Explicit byte/chunk/time limits. |
| `HttpExchangeReport` / observer | Final host report if started, delivery result, timing and bytes; no payload/secret auto logging. |

Server-owned additive seam, to be sequenced under G23:

```rust
// Proposed signatures: loader runs only after bounded head normalization.
async fn handle_with_options(
    &self, app: &Server, input: BufferedRequest, options: HandleOptions,
) -> Result<HttpReply>;

async fn handle_body<F, Fut>(
    &self, app: &Server, head: http::request::Parts,
    options: HandleOptions, load: F,
) -> Result<StreamingHttpReply>
where
    F: FnOnce() -> Fut + Send,
    Fut: Future<Output = Result<Vec<u8>>> + Send;
```

`handle_body` is on proposed StreamingHttpAdapter under `http-stream`; it
normalizes before calling `load`, stages decoded bytes through the existing
Input runner, then returns the one-shot native body plus report to the network
owner. It does not itself perform network IO. Loader errors retain real causes.
Aggregate head+body budgets are rechecked after load without invoking a second
lifecycle. `StreamingHttpReply` distinguishes response/suppressed/unavailable
like today's HttpOutput. A value-only caller drives the resulting body itself.

## Slice 8 streamed SSR example

```rust
use stackpress::server::http::stream::{HttpResponseExt, StreamLimits}; // proposed

async fn page(
    req: &mut Request, res: &mut Response, inv: Invocation<'_>,
) -> Result<Flow> {
    let model = load_public_page(req, &inv).await?;
    // Page module returns an owned Send stream, with no borrowed Invocation.
    let chunks = render_owned_page(model)?;
    res.stream("text/html; charset=utf-8", chunks, StreamLimits::default())?;
    Ok(Flow::Continue)
}
```

The page module is app/Slice 8 code. No fictitious generic Leptos stream adapter
is promised here: local render owners may be !Send. Slice 8 must either produce
an owned Send stream or supervise a local renderer behind a two-chunk bounded
bridge, with cancellation/join and escaped hydration data. Response hooks run
before first polling/publication. Subsequent body failure cannot replace an
already published document with an error envelope.

## Config-only SQLite to PostgreSQL switch

Compile both adapters into this application, e.g. facade features
`sql-sqlite,sql-postgres` (the latter proposed). A configuration change can select
only an already compiled backend, consistent with compile-time plugin selection.

```json
{
  "database": {
    "adapter": "sqlite",
    "options": { "path": "data/profiles.sqlite", "create_if_missing": false }
  }
}
```

Change the database section, leaving routes/generated stores/operation source
unchanged:

```json
{
  "database": {
    "adapter": "postgres",
    "options": {
      "host": "db.internal", "port": 5432, "database": "profiles",
      "user": "profiles_app", "tls": { "mode": "verify_full" }
    }
  }
}
```

Package-owned config applies validated defaults for omitted finite budgets. The
application resolves passwords explicitly from its secret source before open;
unknown/uncompiled backend fails startup. No password belongs in printed config.
An application-owned config enum selects a normal Engine:

```rust
use stackpress::sql::Engine;
use stackpress::sql_sqlite::{SqliteDatabase, SqliteOptions};
use stackpress::sql_postgres::{PostgresDatabase, PostgresOptions}; // proposed

async fn open_database(config: DatabaseConfig) -> stackpress::sql::Result<Engine> {
    match config {
        DatabaseConfig::Sqlite(options) => {
            Ok(Engine::new(SqliteDatabase::open(options).await?))
        }
        DatabaseConfig::Postgres(options) => {
            Ok(Engine::new(PostgresDatabase::open(options).await?))
        }
    }
}
```

Register `Arc<Engine>` in the existing config startup callback; generated stores
take that Arc and render for engine.dialect(). A MySQL variant selects
`MysqlDatabase::open(MysqlOptions)` identically. The application-owned enum is
illustrative, not a new framework config parser or SQL trait requirement.

This selects a backend; it does not copy SQLite data, install schema, migrate
revisions or guarantee identical backend capabilities. Prepare the matching
PostgreSQL schema/data explicitly through Slice 6's reviewed migration workflow.
MySQL transactional-DDL limitations still apply. The production proof must show
the same generated CRUD source running against both selected backends, and
failure before service publication when the selected database is unavailable.
