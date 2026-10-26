//! Observable reuse/retirement identity and clean session state on the next lease.
use super::*;

/// Reuse is permitted only after acknowledgement plus reset; dirty/error leases disappear.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = open(name, 1).await?;
    let first = integer(&database, "SELECT CONNECTION_ID()").await?;
    assert_eq!(integer(&database, "SELECT CONNECTION_ID()").await?, first);
    // Full row completion precedes return-to-pool; no old stream can enter the next query.
    assert_eq!(
        super::run(&database, "SELECT 1 UNION ALL SELECT 2", [])
            .await?
            .rows
            .len(),
        2
    );
    assert_eq!(integer(&database, "SELECT 9").await?, 9);
    assert_eq!(integer(&database, "SELECT CONNECTION_ID()").await?, first);

    // Public core refuses raw SET/variables before a lease can be acquired.
    assert!(
        Sql::literal(DIALECT, "SET @m1_marker = 1")
            .render(DIALECT)
            .is_err()
    );
    assert!(!crate::reuse::allowed_sql("SET @m1_marker = 1"));
    // Private native fixture injection proves the same dirty branch even if trusted
    // caller code could install such state; no production native handle is exposed.
    let mut dirty = database.lease().await?;
    dirty.connection()?.execute("SET @m1_marker = 1").await?;
    dirty.finish(Ok(()), false).await?;
    let mut fresh = database.lease().await?;
    let (id, empty): (u64, bool) = sqlx::query_as("SELECT CONNECTION_ID(), @m1_marker IS NULL")
        .fetch_one(fresh.connection()?)
        .await?;
    assert_ne!(i64::try_from(id)?, first);
    assert!(empty);
    fresh.finish(Ok(()), true).await?;
    let before_temp = integer(&database, "SELECT CONNECTION_ID()").await?;
    super::run(
        &database,
        "CREATE TEMPORARY TABLE m1_temporary (id BIGINT)",
        [],
    )
    .await?;
    assert_ne!(
        integer(&database, "SELECT CONNECTION_ID()").await?,
        before_temp
    );
    assert!(
        database
            .run(
                query("SELECT id FROM m1_temporary", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    let before_function = integer(&database, "SELECT CONNECTION_ID()").await?;
    super::run(
        &database,
        "SELECT GET_LOCK(?,0)",
        [SqlValue::Text(name.into())],
    )
    .await?;
    let mut fresh = database.lease().await?;
    let free: i64 = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
        .bind(name)
        .fetch_one(fresh.connection()?)
        .await?;
    assert_eq!(free, 1);
    fresh.finish(Ok(()), false).await?;
    assert_ne!(
        integer(&database, "SELECT CONNECTION_ID()").await?,
        before_function
    );

    super::run(
        &database,
        "CREATE TABLE m1_reuse (id BIGINT PRIMARY KEY) ENGINE=InnoDB",
        [],
    )
    .await?;
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    let held: i64 = transaction
        .run(query("SELECT CONNECTION_ID()", []), QueryLimits::default())
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
    assert_eq!(integer(&database, "SELECT CONNECTION_ID()").await?, held);
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
    assert_eq!(integer(&database, "SELECT CONNECTION_ID()").await?, held);
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_reuse").await?,
        2
    );
    // A stateful function disqualifies the entire transaction, for both completions.
    for commit in [false, true] {
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        let held: i64 = transaction
            .run(query("SELECT CONNECTION_ID()", []), QueryLimits::default())
            .await?
            .rows[0]
            .try_get_at(0)?;
        transaction
            .run(
                query("SELECT GET_LOCK(?,0)", [SqlValue::Text(name.into())]),
                QueryLimits::default(),
            )
            .await?;
        if commit {
            transaction.commit().await?;
        } else {
            transaction.rollback().await?;
        }
        assert_ne!(integer(&database, "SELECT CONNECTION_ID()").await?, held);
        let mut fresh = database.lease().await?;
        let free: i64 = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
            .bind(name)
            .fetch_one(fresh.connection()?)
            .await?;
        assert_eq!(free, 1);
        fresh.finish(Ok(()), false).await?;
    }
    // Every public/native error must retire even when no write was possible.
    let before_error = integer(&database, "SELECT CONNECTION_ID()").await?;
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
        integer(&database, "SELECT CONNECTION_ID()").await?,
        before_error
    );
    database.close().await?;
    Ok(())
}
