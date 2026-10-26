# Adapter contracts and driver futures

`Database` and `TransactionSession` are public extension contracts. Most applications consume
an existing adapter rather than implement them. Source: [engine.rs](../../src/engine.rs).

```rust
use stackpress_sql::{Database, Query, QueryLimits, QueryResult, Result, TransactionOptions};

async fn run_direct(database: &dyn Database, query: Query) -> Result<QueryResult> {
    let limits = QueryLimits::default();
    query.validate(limits)?;
    database.run(query, limits).await
}

async fn begin_and_rollback(database: &dyn Database) -> Result<()> {
    let session = database.begin(TransactionOptions::default()).await?;
    session.rollback().await
}
```

Direct trait use does not remove adapter validation duties. Engine adds an application-facing
rendering boundary; a correct adapter also validates callers that bypass Engine.

Contents: [Database](#1-database) · [Session](#2-transactionsession) · [Types](#3-sqlfuture-and-transactionoptions)

## 1. Database

`Database: Send + Sync + 'static` has these required methods. Its dialect is stable for its
entire lifetime. No native resource may escape owned outcomes.

| Signature | Parameters / responsibility |
| --- | --- |
| `dialect(&self) -> DialectId` | No parameters; return stable renderer identity. |
| `run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>` | Validate identity/input, acquire/prepare/check arity, bind, collect bounded owned outcomes, then release normal lease. |
| `begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>>` | Acquire one exclusive native session; cancellation cleans up or retires it. |
| `close(&self) -> SqlFuture<'_, ()>` | Reject new work and wait for held leases; cancellation may be re-awaited. |

**Returns** dialect returns identity immediately. Other methods return boxed Send futures with
Result outputs. Native/admission/closed/arity/decode/limit failures retain typed kinds and causes.
Adapters must enforce output budgets during collection before publishing a partial result and
must never retry operations automatically.

## 2. TransactionSession

`TransactionSession: Send + 'static` owns one connection until completion/drop. It exposes:

| Signature | Parameters / responsibility |
| --- | --- |
| `run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult>` | Exclusive borrow through await; same validation/binding/result budgets as Database::run. |
| `commit(self: Box<Self>) -> SqlFuture<'static, ()>` | Future owns session until commit acknowledgement/error. |
| `rollback(self: Box<Self>) -> SqlFuture<'static, ()>` | Future owns session until rollback acknowledgement/error. |

**Returns** Owned Result outcomes through Send futures. Rows must own their contents. Uncommitted
Drop must initiate rollback or retire the connection so unrelated borrowers cannot observe its
pending transaction. Never publish an uncertain lease as clean after cancellation. Lost commit
acknowledgement is an unknown outcome and must not trigger automatic replay.

The existing [recording support](../../examples/support/mod.rs) illustrates trait wiring only;
it does not interpret SQL or establish native storage/cancellation correctness. The
[SQLite adapter docs](../../../sql-sqlite/docs/README.md) own the concrete implementation.

## 3. SqlFuture and TransactionOptions

`SqlFuture<'a, T>` aliases `Pin<Box<dyn Future<Output = Result<T>> + Send + 'a>>`.
It preserves the borrow lifetime rather than widening it to static; consuming completion methods
can return static futures because they own the session.

TransactionOptions is Copy/Clone/Debug/Default and currently an empty struct. It selects one
ordinary deferred transaction. There is no configurable isolation, callback, or savepoint
surface. Engine::begin supplies its default value.

Related: [Engine](engine.md), [Transaction](transaction.md), [errors](errors.md).
