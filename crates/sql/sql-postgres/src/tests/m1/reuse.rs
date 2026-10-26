//! Observable reuse/retirement identity and clean session state on the next lease.
use super::*;

/// Reuse is permitted only after acknowledgement plus reset; dirty/error leases disappear.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;
    let first = integer(&database, "SELECT pg_backend_pid()").await?;
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, first);
    // Full row completion precedes return-to-pool; no old stream can enter the next query.
    assert_eq!(
        super::run(&database, "SELECT 1 UNION ALL SELECT 2", [])
            .await?
            .rows
            .len(),
        2
    );
    assert_eq!(integer(&database, "SELECT 9").await?, 9);
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, first);

    // DISCARD ALL removes owned temp schema objects without changing native identity.
    super::run(
        &database,
        "CREATE TEMPORARY TABLE m1_temporary (id BIGINT)",
        [],
    )
    .await?;
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, first);
    assert!(
        super::run(
            &database,
            "SELECT to_regclass('pg_temp.m1_temporary') IS NULL",
            []
        )
        .await?
        .rows[0]
            .try_get_at::<bool>(0)?
    );
    super::run(
        &database,
        "SELECT set_config('m1.marker', 'dirty', false)",
        [],
    )
    .await?;
    assert!(
        super::run(
            &database,
            "SELECT current_setting('m1.marker', true) IS DISTINCT FROM 'dirty'",
            []
        )
        .await?
        .rows[0]
            .try_get_at::<bool>(0)?
    );
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, first);
    // Session-level advisory locks from an otherwise valid SELECT also get discarded.
    super::run(&database, "SELECT pg_advisory_lock(10001) IS NULL", []).await?;
    assert!(
        !super::run(&database, "SELECT pg_advisory_unlock(10001)", [])
            .await?
            .rows[0]
            .try_get_at::<bool>(0)?
    );

    super::run(
        &database,
        "CREATE TABLE m1_reuse (id BIGINT PRIMARY KEY)",
        [],
    )
    .await?;
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    let held: i64 = transaction
        .run(query("SELECT pg_backend_pid()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at(0)?;
    transaction
        .run(
            query("INSERT INTO m1_reuse VALUES (1)", []),
            QueryLimits::default(),
        )
        .await?;
    transaction.commit().await?;
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, held);
    // A new ordinary write on the recycled TX connection must autocommit independently.
    super::run(&database, "INSERT INTO m1_reuse VALUES (2)", []).await?;
    let observer = open(name, 1).await?;
    assert_eq!(
        integer(&observer, "SELECT COUNT(*) FROM m1_reuse").await?,
        2
    );
    observer.close().await?;
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query("INSERT INTO m1_reuse VALUES (3)", []),
            QueryLimits::default(),
        )
        .await?;
    transaction.rollback().await?;
    assert_eq!(integer(&database, "SELECT pg_backend_pid()").await?, held);
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_reuse").await?,
        2
    );
    // Every public/native error must retire even when no write was possible.
    let before_error = integer(&database, "SELECT pg_backend_pid()").await?;
    assert!(
        database
            .run(
                query("SELECT bad_column FROM m1_reuse", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    assert_ne!(
        integer(&database, "SELECT pg_backend_pid()").await?,
        before_error
    );
    database.close().await?;
    Ok(())
}
