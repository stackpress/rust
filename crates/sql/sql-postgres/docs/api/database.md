# PostgresDatabase

A non-Clone owner implementing the core `Database` trait. `Engine::new(database)` takes
ownership. Share the Engine through Arc; native connections and pool handles stay private.

## Eager open

`PostgresDatabase::open(options: PostgresOptions).await -> Result<Self>` validates explicit
options and opens an existing database on the caller's Tokio runtime. It creates no database
or schema, performs no migration and does not retry application operations. Invalid options
return `Configuration`; connection/TLS/authentication failures retain a typed native cause;
the whole open deadline returns `PoolTimedOut`.

Open checks one connection, with minimum pool size zero and the configured maximum.
No partially opened owner is published on failure.

## Database trait methods

Import `stackpress_sql::Database` to call these methods directly:

| Method | Result and contract |
| --- | --- |
| `dialect()` | `DialectId::Postgres`; synchronous, no IO |
| `run(query, limits).await` | `Result<QueryResult>` for one validated statement and bounded owned rows |
| `begin(TransactionOptions::default()).await` | `Result<Box<dyn TransactionSession>>` after acknowledged BEGIN |
| `close().await` | `Result<()>` after admission closure and owned connection/cleanup drain |

`run` requires this adapter's dialect. It validates limits before acquisition, prepares and
checks native arity on the same exclusive lease, decodes all retained rows, then awaits reset
or retirement before returning. For example:

```rust
use stackpress_sql::{Database, QueryLimits, QueryResult, Render, Result, Sql};
use stackpress_sql_postgres::PostgresDatabase;

async fn fetch_native(database: &PostgresDatabase) -> Result<QueryResult> {
    let dialect = database.dialect();
    let query = Sql::literal(dialect, "SELECT $1::INT8 AS id")
        .bind(7_i64)
        .render(dialect)?;
    database.run(query, QueryLimits::default()).await
}
```

The result carries owned rows plus this statement's outcome. Prefer `Engine::fetch` and
`Engine::execute` for application builders; direct trait use does not bypass query validation.

## Session and completion

TransactionSession supplies exclusive `run(&mut self, query, limits)` and consuming
`commit(self: Box<Self>)` / `rollback(self: Box<Self>)` futures. Runs use the inherited
statement deadline; failed or cancelled polled runs become rollback-only. No isolation-level
or nested/savepoint options are exposed by the empty core TransactionOptions.

A completion future owns the session from construction. Unpolled drop retires its lease.
See [transactions and cancellation](../guides/transactions.md) before handling completion
errors or adding caller deadlines.

## Lease reset

Only fully acknowledged success qualifies for reuse. The adapter clears SQLx statement
bookkeeping, executes DISCARD ALL outside a transaction, restores its fixed startup settings
and waits for acknowledgement. Failure or cancellation of any phase retires the connection.
Prepared caching is disabled. Session variables, temp tables and advisory locks cannot
persist across independent Engine calls.

## Close and drop

Polling close shuts admission, wakes pool waiters with `Closed` and drains held/retiring
leases. Finish or drop transactions first: close has no overall deadline and may wait on
caller-held work. An interrupted close can be re-awaited; it never reopens the owner.
Tracked background cleanup reports its first failure once.

Drop while Tokio lives schedules bounded retirement; without a runtime it disposes native
sockets by RAII. Neither path promises immediate remote rollback acknowledgement.
