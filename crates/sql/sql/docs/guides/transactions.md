# Execute work on one transaction

Two writes belong to one transaction only when they use the same transaction owner. Pass a
mutable guard to reusable async functions so connection affinity stays visible in the signature.

## Compose named operations

The helpers below expect an existing `Engine`. They update a profile and append an audit row
on the same session; the outer function owns commit and error cleanup.

```rust
use stackpress_sql::{Engine, Insert, Result, Transaction, Update};

async fn deactivate(transaction: &mut Transaction, id: i64) -> Result<()> {
    transaction
        .execute(
            &Update::new("profile")
                .set("active", false)
                .where_eq("id", id),
        )
        .await?;
    transaction
        .execute(&Insert::new("audit").columns(["profile_id"]).values([id]))
        .await?;
    Ok(())
}

async fn deactivate_profile(engine: &Engine, id: i64) -> Result<()> {
    let mut transaction = engine.begin().await?;
    if let Err(primary) = deactivate(&mut transaction, id).await {
        return Err(transaction.rollback_after(primary).await);
    }
    transaction.commit().await
}
```

The mutable borrow lasts across each await. If either statement fails, `rollback_after` keeps
the original error and attaches any rollback failure. Commit consumes the guard. For a runnable
orchestration demonstration without a database, use the existing
[execute_plan example](../../examples/execute_plan.rs); it uses a recording adapter.

## Decide where completion belongs

Use `Engine::execute` / `fetch` for one independent statement. Use `Engine::execute_plan` when
it should own the complete schema-plan transaction. Use `Transaction::execute_plan` to combine
a plan with other work; it leaves commit or rollback to you after success, failure, or cancellation.

Do not call another engine method and assume it joins the active transaction. Server event
dispatch does not inherit transaction context. A nested acquisition while holding the only
connection can time out; pass the guard directly when operations need the same transaction.

## Treat cancellation as lost observation

Uncommitted drop initiates adapter cleanup, but does not synchronously finish rollback. A write
whose future was cancelled may already have executed, and a cancelled commit may have an unknown
outcome. No operation retries automatically; application recovery must not blindly replay work.

You can now identify the function that owns completion and every operation sharing its session.
Continue with [Engine](../api/engine.md), [Transaction](../api/transaction.md), or the
[SQLite transaction lifecycle](../../../sql-sqlite/docs/concepts.md#transactions-keep-exclusive-ownership)
for native cancellation and shutdown.
