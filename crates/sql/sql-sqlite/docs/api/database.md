# SqliteDatabase API

SqliteDatabase owns a bounded native pool and implements `stackpress_sql::Database`.
It is Send + Sync + 'static and is not Clone. Most callers wrap it in Engine; direct trait
examples below show the adapter boundary. Source: [lib.rs](../../src/lib.rs),
[execution](../../src/execute.rs), [collection](../../src/collect.rs) and
[session](../../src/session.rs).

Contents: [1. Open](#1-sqlitedatabaseopen) · [2. Dialect](#2-databasedialect) ·
[3. Run](#3-databaserun) · [4. Begin](#4-databasebegin) · [5. Close](#5-databaseclose) ·
[6. Errors](#6-error-categories).

## 1. SqliteDatabase::open

`async fn open(options: SqliteOptions) -> stackpress_sql::Result<Self>`

Validate configuration and eagerly open a real file connection. This does not install
schema or seed data. It requires a live Tokio runtime.

```rust
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("open.sqlite")).create_if_missing(true),
    )
    .await?;
    let engine = Engine::new(database);
    engine.close().await?;
    Ok(())
}
```

The example creates only a file and closes it explicitly. Engine takes ownership of the
adapter; use `Arc<Engine>` to share that ownership with several callers.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `SqliteOptions` | Owned file policy and finite pool/admission budgets. |

**Returns** The initialized adapter, or configuration/native-open errors. Missing files fail
unless creation was enabled. Cancellation retires the unpublished pool, but creation may
already have left an empty file. Foreign keys and FULL synchronous policy apply to each
connection; writable open requests WAL, read-only open preserves the journal mode.

## 2. Database::dialect

`fn dialect(&self) -> DialectId`

Read the adapter's stable renderer identity. Import the Database trait to call its methods.

```rust
use stackpress_sql::{Database, DialectId};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("dialect.sqlite")).create_if_missing(true),
    )
    .await?;
    assert_eq!(database.dialect(), DialectId::Sqlite);
    database.close().await?;
    Ok(())
}
```

The identity never switches to a network backend or in-memory fallback.

**Returns** `DialectId::Sqlite` without IO.

## 3. Database::run

`fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>`

Execute one validated, prepared statement on a lease. Engine normally performs rendering
for you; direct callers supply an owned Query.

```rust
use stackpress_sql::{Database, DialectId, QueryLimits, Render, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("run.sqlite")).create_if_missing(true),
    )
    .await?;
    let query = Sql::literal(DialectId::Sqlite, "SELECT ? AS name")
        .bind("O'Reilly")
        .render(DialectId::Sqlite)?;
    let result = database.run(query, QueryLimits::default()).await?;
    assert_eq!(result.rows[0].try_get::<String>("name")?, "O'Reilly");
    assert_eq!(result.affected, 0);
    assert_eq!(result.insert_id, None);
    database.close().await?;
    Ok(())
}
```

Native preparation checks bind arity on the same connection before binding and execution.
The returned rows are owned; no cursor or native connection escapes.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `query` | `Query` | SQLite-tagged statement with separate binds. |
| `limits` | `QueryLimits` | Finite input and retained-output budgets. |

**Returns** A Send borrowing future yielding the complete QueryResult or typed failure.
Limits error instead of truncating. Only acknowledged success reuses an ordinary connection;
failure/cancellation retires it. Cancellation can lose observation of persisted effects.
`insert_id` records a native rowid transition and can be None for a successful insert;
use RETURNING for an authoritative key. See [limits and outcomes](../concepts.md#rows-and-sizes-are-bounded).

## 4. Database::begin

`fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>>`

Acquire one lease and acknowledge ordinary deferred `BEGIN` before returning the session.
Engine exposes this through its caller-facing Transaction guard.

```rust
use stackpress_sql::{Database, DialectId, QueryLimits, Render, Sql, TransactionOptions};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("transaction.sqlite")).create_if_missing(true),
    )
    .await?;
    let mut session = database.begin(TransactionOptions::default()).await?;
    let query = Sql::literal(DialectId::Sqlite, "SELECT ? AS id")
        .bind(7)
        .render(DialectId::Sqlite)?;
    let result = session.run(query, QueryLimits::default()).await?;
    assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
    session.commit().await?;
    let session = database.begin(TransactionOptions::default()).await?;
    session.rollback().await?;
    database.close().await?;
    Ok(())
}
```

Session run borrows exclusively across await; commit/rollback consume the session.
Completed transactions retire their connections rather than returning them for reuse.

| Parameter | Type | Meaning |
| --- | --- | --- |
| `options` | `TransactionOptions` | Empty core options; deferred BEGIN, no savepoints/isolation settings. |

**Returns** A Send borrowing future yielding an owned Send transaction session or an
admission/acquisition/native error. Dropping an uncommitted session initiates rollback
through native close. Failed or cancelled session work becomes rollback-only; explicit
rollback remains allowed. Cancellation of commit has an unknown outcome and is never
retried. Core [transaction contracts](../../../sql/docs/concepts.md) own the shared abstraction;
[SQLite lifecycle concepts](../concepts.md#transactions-keep-exclusive-ownership) explain cleanup.

## 5. Database::close

`fn close(&self) -> SqlFuture<'_, ()>`

On polling, close admission and await pool shutdown. Engine forwards close to this method.

```rust
use stackpress_sql::{Database, DialectId, ErrorKind, QueryLimits, Render, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("close.sqlite")).create_if_missing(true),
    )
    .await?;
    database.close().await?;
    database.close().await?;
    let query = Sql::literal(DialectId::Sqlite, "SELECT 1").render(DialectId::Sqlite)?;
    let error = database
        .run(query, QueryLimits::default())
        .await
        .unwrap_err();
    assert_eq!(error.kind(), ErrorKind::Closed);
    Ok(())
}
```

Repeated close is supported, and valid new work is rejected after shutdown. A cancelled
close can be re-awaited; cancellation does not reopen admission.

**Returns** A Send borrowing future yielding unit after pool shutdown. It can remain pending
while a transaction is held; finish/drop guards first and choose an application shutdown
deadline. Drop without a live runtime is best effort and does not acknowledge worker
completion. See [runtime shutdown](../concepts.md#closing-and-runtime-shutdown).

## 6. Error categories

All operations use core SqlError and Result. Adapter-produced Display/Debug keep SQL, paths
and native causes out of default formatting; explicit source inspection retains diagnostic
detail. Native-source errors and first-party validation failures differ:

| ErrorKind | Typical trigger |
| --- | --- |
| `Configuration` | Invalid options or native configuration failure. |
| `Limit` | Immediate admission rejection or input/retained-output budget exceeded. |
| `PoolTimedOut` | Admitted work cannot acquire a connection in time. |
| `Busy` | SQLite busy/locked, including extended codes with those primary codes. |
| `Closed` | Closed admission/pool or worker loss. |
| `Unsupported` | Query tagged for a different dialect. |
| `Arity { expected, actual }` | Prepared bind count differs from supplied count. |
| `InvalidQuery` | Invalid native argument/encoding or invalid input value. |
| `Decode` | Native row/type/UTF-8 decode failure or nonfinite computed real. |
| `RollbackOnly` | Session cannot run/commit after failed or cancelled native work. |
| `Driver` | Other native failures, including constraints or missing tables. |

An original SQLx error is retained as source when one exists. Options/arity/limit/rollback-only
errors are first-party failures and need not have a native source. Explicit rollback errors
remain observable; `Transaction::rollback_after` keeps both primary and cleanup errors.
Detached Drop cleanup cannot return an async close diagnostic.

Related: [options](options.md), [Engine service guide](../guides/server-service.md),
[core API](../../../sql/docs/api/README.md), [error disclosure](../concepts.md#outcomes-and-error-disclosure).
