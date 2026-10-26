# Transaction

A Transaction owns one exclusive adapter session. Pass it by `&mut` across composed async
functions and consume it exactly once for completion. Source: [engine.rs](../../src/engine.rs).

```rust
use stackpress_sql::{QueryPlan, QueryResult, Result, SqlError, Transaction, Update};

async fn change_profile(transaction: &mut Transaction, id: i64) -> Result<()> {
    transaction
        .execute(
            &Update::new("profile")
                .set("active", true)
                .where_eq("id", id),
        )
        .await?;
    Ok(())
}

async fn apply_plan(transaction: &mut Transaction, plan: &QueryPlan) -> Result<Vec<QueryResult>> {
    transaction.execute_plan(plan).await
}

async fn fail_and_rollback(transaction: Transaction, primary: SqlError) -> SqlError {
    transaction.rollback_after(primary).await
}
```

These helpers retain the caller's completion responsibility. The
[transaction guide](../guides/transactions.md) shows the outer begin/commit/error path;
the [recording example](../../examples/execute_plan.rs) runs it without storage.

## 1. Ownership and operations

Only Engine::begin constructs a Transaction. It is Send, non-Sync, non-Clone, has no public
fields, and copies
the engine's dialect/limits at begin. One mutable session borrow lasts across await.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `execute(&mut self, query)` | `&(impl Render + Sync)` | Render/validate/run on this session. |
| `fetch(&mut self, query)` | `&(impl Render + Sync)` | Return owned rows from this session. |
| `execute_plan(&mut self, plan)` | `&QueryPlan` | Run ordered plan statements without completing the transaction. |
| `commit(self)` | none | Consume the session and await commit acknowledgement. |
| `rollback(self)` | none | Consume and await explicit rollback. |
| `rollback_after(self, primary)` | `SqlError` | Consume, rollback, and preserve both errors if needed. |

**Returns** Awaited execute returns `Result<QueryResult>`; fetch returns `Result<Vec<Row>>`;
execute_plan returns `Result<Vec<QueryResult>>`; commit/rollback return `Result<()>`.
rollback_after returns the primary SqlError, attaching a cleanup error if rollback fails.
Query/plan validation errors happen before session run; adapters supply native errors.

## 2. Plan failures and cancellation

Transaction::execute_plan validates every statement and requires transactional DDL before
running members. It stops at the first failure and returns no partial result vector. It neither
commits nor automatically rolls back: the caller retains the guard after failure/cancellation
and must decide completion. MySQL is rejected before plan statements execute.

A cancelled statement may already have effects; the adapter can mark an uncertain session
rollback-only. Uncommitted Drop initiates rollback or retires the connection and does not
synchronously finish cleanup. Completion futures own their sessions; commit cancellation can
leave an unknown outcome. There are no implicit retries, nested transactions, or savepoints.

Engine operations and nested server dispatch do not inherit this transaction. Reacquiring
another lease while retaining one can time out or contend on native locks. See the
[SQLite adapter](../../../sql-sqlite/docs/README.md) for native transitions and shutdown.

Related: [Engine](engine.md), [TransactionSession](database.md#2-transactionsession),
[errors](errors.md), [QueryPlan](query-plan.md).
