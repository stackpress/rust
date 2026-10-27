# Complete or cancel a transaction

Use one mutable transaction guard for work that must share a connection. These examples
expect an existing Engine and a `profile(id, name)` table. Install schema explicitly using
the [core schema guide](../../../sql/docs/guides/schema-changes.md).

## Keep completion in the outer function

The helper updates one row; the outer function decides commit or rollback:

```rust
use stackpress_sql::{Engine, Result, Transaction, Update};

async fn rename(transaction: &mut Transaction, id: i64, name: &str) -> Result<()> {
    transaction
        .execute(&Update::new("profile").set("name", name).where_eq("id", id))
        .await?;
    Ok(())
}

async fn rename_profile(engine: &Engine, id: i64, name: &str) -> Result<()> {
    let mut transaction = engine.begin().await?;
    if let Err(primary) = rename(&mut transaction, id, name).await {
        return Err(transaction.rollback_after(primary).await);
    }
    // Completion consumes the guard; an error must not trigger an automatic replay.
    transaction.commit().await
}
```

`rollback_after` preserves the primary error and attaches rollback/retirement failure through
`cleanup_error`. Commit and rollback consume the guard, including ownership captured by an
unpolled completion future. Dropping that future retires the lease.

PostgreSQL supports transactional DDL through supported core plans.

## Cancel a query and explicitly roll back

Add Tokio's `time` feature for this application deadline:

```rust
use stackpress_sql::{Engine, ErrorKind, Result, Select, SqlError};
use std::time::Duration;

async fn inspect_with_deadline(engine: &Engine) -> Result<()> {
    let mut transaction = engine.begin().await?;
    let query = Select::new(["name"]).from("profile").where_eq("id", 1);
    match tokio::time::timeout(Duration::from_millis(100), transaction.fetch(&query)).await {
        Ok(Ok(_)) => transaction.commit().await,
        Ok(Err(primary)) => Err(transaction.rollback_after(primary).await),
        Err(cause) => {
            // Preserve lost observation as the primary error, even if rollback also fails.
            let primary = SqlError::with_source(
                ErrorKind::Driver,
                "application query deadline exceeded; outcome uncertain",
                cause,
            );
            Err(transaction.rollback_after(primary).await)
        }
    }
}
```

Dropping a polled query future cannot undo its native effects. The session becomes
rollback-only; explicit rollback is permitted, but the uncertain connection is retired.
The example preserves the timeout cause without formatting it into public text.
A render/validation failure before Engine enters the session can leave the transaction
usable; always decide cleanup in the function that owns the guard.

## Handle an unknown commit outcome

BEGIN, statement execution and transaction completion use the statement deadline. Cancelling
BEGIN retires the unpublished lease. Cancelling COMMIT, losing its acknowledgement or failing
cleanup after COMMIT leaves the write outcome uncertain. A commit error is not a guarantee
that rollback happened.

Do not replay the write automatically. Use application identifiers and an independent read
or another application-specific reconciliation protocol to establish what happened.
Cancellation of rollback or dropping an unfinished transaction also retires its connection.
Drop cleanup is bounded while the runtime lives; drain it through `Engine::close`.
