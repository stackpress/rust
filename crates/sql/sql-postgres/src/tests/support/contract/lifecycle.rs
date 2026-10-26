//! Shared public-trait lifecycle assertions; native syntax differences stay in the suite inputs.
use super::{
    Database, DialectId, ErrorKind, QueryLimits, Result, TransactionOptions, integer, query, run,
};
use std::sync::Arc;
use std::time::Duration;

/// Deliberately long native work uses only the fixture's private database or SQLite VM.
fn slow(dialect: DialectId) -> &'static str {
    match dialect {
        DialectId::Postgres => "SELECT 1::BIGINT FROM pg_sleep(0.2)",
        DialectId::Mysql => "SELECT SLEEP(0.2)",
        DialectId::Sqlite => {
            "WITH RECURSIVE n(x) AS (VALUES(0) UNION ALL SELECT x+1 FROM n WHERE x<2000000) SELECT SUM(x) FROM n"
        }
    }
}
/// Cancel native work and queued acquisitions; retain rollback-only and unknown-commit semantics.
pub(super) async fn cancellation(database: &dyn Database) -> Result<()> {
    let dialect = database.dialect();
    // The runner configures exactly one native connection, making this queue deterministic.
    let held = database.begin(TransactionOptions::default()).await?;
    assert!(
        tokio::time::timeout(
            Duration::from_millis(15),
            database.run(query(dialect, "SELECT 1", []), QueryLimits::default())
        )
        .await
        .is_err()
    );
    assert!(
        tokio::time::timeout(
            Duration::from_millis(15),
            database.begin(TransactionOptions::default())
        )
        .await
        .is_err()
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
            assert!(
                tokio::time::timeout(
                    Duration::from_millis(10),
                    transaction.run(query(dialect, slow(dialect), []), QueryLimits::default())
                )
                .await
                .is_err()
            );
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
            assert!(
                tokio::time::timeout(
                    Duration::from_millis(10),
                    database.run(query(dialect, slow(dialect), []), QueryLimits::default())
                )
                .await
                .is_err()
            );
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
    let polled = std::future::poll_fn(|cx| std::task::Poll::Ready(pending.as_mut().poll(cx))).await;
    if let std::task::Poll::Ready(result) = polled {
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
/// Close wakes waiters, rejects new work and remains re-awaitable after interruption.
pub(super) async fn close(database: Arc<dyn Database>) -> Result<()> {
    let dialect = database.dialect();
    let held = database.begin(TransactionOptions::default()).await?;
    let shared = database.clone();
    let waiter = tokio::spawn(async move {
        shared
            .run(query(dialect, "SELECT 1", []), QueryLimits::default())
            .await
    });
    // Poll until the native acquisition is definitely pending while the transaction owns capacity.
    tokio::task::yield_now().await;
    assert!(
        tokio::time::timeout(Duration::from_millis(5), database.close())
            .await
            .is_err()
    );
    assert_eq!(waiter.await.unwrap().unwrap_err().kind(), ErrorKind::Closed);
    held.rollback().await?;
    database.close().await?;
    database.close().await?;
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
