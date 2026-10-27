//! Shared public-trait lifecycle assertions; native syntax differences stay in the suite inputs.
use super::{
    Database, DialectId, ErrorKind, QueryLimits, Result, TransactionOptions, integer, query, run,
};
use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;

/// Bound harmless retries when a probe completes before its first suspension.
const CANCELLATION_PROBES: usize = 64;

/// Harmless native probes make real suspension points observable without timing a completion.
fn cancellation_sql(dialect: DialectId) -> &'static str {
    match dialect {
        DialectId::Postgres => "SELECT 1::BIGINT FROM pg_sleep(0.2)",
        DialectId::Mysql => "SELECT SLEEP(0.2)",
        DialectId::Sqlite => {
            "WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<2000) SELECT SUM(x) FROM n"
        }
    }
}
/// Observe one actual poll without turning Pending into an elapsed-time expectation.
async fn poll_once<F: Future + ?Sized>(mut future: Pin<&mut F>) -> Poll<F::Output> {
    poll_fn(|cx| Poll::Ready(future.as_mut().poll(cx))).await
}

/// Drop the owned/borrowed operation immediately after its observed poll, before returning.
async fn cut<F: Future>(future: F) -> Poll<F::Output> {
    let mut future = Box::pin(future);
    let result = poll_once(future.as_mut()).await;
    drop(future);
    result
}

/// Cancel queued and active public futures; retain rollback-only and unknown-commit semantics.
pub(super) async fn cancellation(database: &dyn Database) -> Result<()> {
    let dialect = database.dialect();
    // This suite observes public future ownership; exact native phases remain covered
    // by the adapters' transition-gated tests rather than inferred from a short sleep.
    // The runner configures exactly one native connection, making this queue deterministic.
    let held = database.begin(TransactionOptions::default()).await?;
    assert!(
        cut(database.run(query(dialect, "SELECT 1", []), QueryLimits::default()))
            .await
            .is_pending()
    );
    assert!(
        cut(database.begin(TransactionOptions::default()))
            .await
            .is_pending()
    );
    held.rollback().await?;
    for in_transaction in [false, true] {
        if in_transaction {
            let mut transaction = database.begin(TransactionOptions::default()).await?;
            transaction
                .run(
                    query(dialect, "INSERT INTO contract_items(value) VALUES (70)", []),
                    QueryLimits::default(),
                )
                .await?;
            let mut cancelled = false;
            for _ in 0..CANCELLATION_PROBES {
                // A probe may finish before its first suspension. Accept that harmless
                // SELECT and retry; only an observed Pending counts as cancellation.
                match cut(transaction.run(
                    query(dialect, cancellation_sql(dialect), []),
                    QueryLimits::default(),
                ))
                .await
                {
                    Poll::Ready(result) => {
                        result?;
                    }
                    Poll::Pending => {
                        cancelled = true;
                        break;
                    }
                }
            }
            assert!(cancelled, "transaction probes never suspended");
            assert_eq!(
                transaction
                    .run(query(dialect, "SELECT 1", []), QueryLimits::default())
                    .await
                    .unwrap_err()
                    .kind(),
                ErrorKind::RollbackOnly
            );
            transaction.rollback().await?;
        } else {
            let mut cancelled = false;
            for _ in 0..CANCELLATION_PROBES {
                match cut(database.run(
                    query(dialect, cancellation_sql(dialect), []),
                    QueryLimits::default(),
                ))
                .await
                {
                    Poll::Ready(result) => {
                        result?;
                    }
                    Poll::Pending => {
                        cancelled = true;
                        break;
                    }
                }
            }
            assert!(cancelled, "ordinary probes never suspended");
        }
        assert_eq!(
            integer(
                database,
                "SELECT COUNT(*) FROM contract_items WHERE value=70"
            )
            .await?,
            0
        );
    }
    run(database, "CREATE TABLE contract_counter (value BIGINT)", []).await?;
    run(database, "INSERT INTO contract_counter VALUES (0)", []).await?;
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query(dialect, "UPDATE contract_counter SET value=value+1", []),
            QueryLimits::default(),
        )
        .await?;
    let mut pending = transaction.commit();
    if let Poll::Ready(result) = poll_once(pending.as_mut()).await {
        result?;
    }
    drop(pending);
    let value = integer(database, "SELECT value FROM contract_counter").await?;
    assert!(
        value == 0 || value == 1,
        "unknown commit must not duplicate the increment"
    );
    Ok(())
}
/// Validation stays pre-effect even when acquisition cannot succeed or the owner is closed.
async fn pre_effect_validation(database: &dyn Database) {
    let dialect = database.dialect();
    let mut cases = vec![(
        query(dialect, "SELECT 1", []),
        QueryLimits {
            sql_bytes: 0,
            ..QueryLimits::default()
        },
        ErrorKind::Limit,
    )];
    // Network owners promise dialect rejection before acquisition. SQLite's existing
    // dialect check belongs to execute after acquisition; do not invent a stronger contract.
    if dialect != DialectId::Sqlite {
        cases.push((
            query(DialectId::Sqlite, "SELECT 1", []),
            QueryLimits::default(),
            ErrorKind::Unsupported,
        ));
    }
    for (query, limits, kind) in cases {
        // Ready on the first poll proves no exhausted-pool wait can mask a pure refusal.
        let Poll::Ready(result) = cut(database.run(query, limits)).await else {
            panic!("pre-effect validation waited for native acquisition");
        };
        assert_eq!(result.unwrap_err().kind(), kind);
    }
}
/// Close wakes waiters, rejects new work and remains re-awaitable after interruption.
pub(super) async fn close(database: Arc<dyn Database>) -> Result<()> {
    let dialect = database.dialect();
    let held = database.begin(TransactionOptions::default()).await?;
    pre_effect_validation(database.as_ref()).await;
    let mut waiter = database.run(query(dialect, "SELECT 1", []), QueryLimits::default());
    // The held transaction forces the waiter to queue, and forces close to remain pending.
    assert!(poll_once(waiter.as_mut()).await.is_pending());
    let mut closing = database.close();
    assert!(poll_once(closing.as_mut()).await.is_pending());
    drop(closing);
    assert_eq!(waiter.await.unwrap_err().kind(), ErrorKind::Closed);
    held.rollback().await?;
    database.close().await?;
    database.close().await?;
    pre_effect_validation(database.as_ref()).await;
    assert_eq!(
        database
            .run(query(dialect, "SELECT 1", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Closed
    );
    assert_eq!(
        database
            .begin(TransactionOptions::default())
            .await
            .err()
            .unwrap()
            .kind(),
        ErrorKind::Closed
    );
    Ok(())
}
