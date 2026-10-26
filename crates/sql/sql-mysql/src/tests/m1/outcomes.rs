//! Statement-local native outcomes, concurrent exclusive leases and TX DDL policy.
use super::*;
use std::sync::Arc;

/// Outcomes never inherit an ID/count from a prior statement or another active native lease.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = Arc::new(open(name, 2).await?);
    super::run(&database, "CREATE TABLE m1_outcomes (id BIGINT AUTO_INCREMENT PRIMARY KEY, value BIGINT) ENGINE=InnoDB", []).await?;
    let inserted = super::run(&database, "INSERT INTO m1_outcomes(value) VALUES (10)", []).await?;
    assert_eq!(inserted.affected, 1);
    assert_eq!(inserted.insert_id, Some(1));
    assert!(inserted.rows.is_empty());
    let updated = super::run(
        &database,
        "UPDATE m1_outcomes SET value=11 WHERE value=10",
        [],
    )
    .await?;
    assert_eq!(updated.affected, 1);
    assert_eq!(updated.insert_id, None);
    let selected = super::run(
        &database,
        "SELECT value FROM m1_outcomes WHERE value=11",
        [],
    )
    .await?;
    assert_eq!(selected.rows[0].try_get_at::<i64>(0)?, 11);
    assert_eq!(selected.insert_id, None);
    let ddl = super::run(&database, "CREATE TABLE m1_outcome_ddl (id BIGINT)", []).await?;
    assert_eq!(ddl.affected, 0);
    assert_eq!(ddl.insert_id, None);
    let deleted = super::run(&database, "DELETE FROM m1_outcomes WHERE value=11", []).await?;
    assert_eq!(deleted.affected, 1);
    assert_eq!(deleted.insert_id, None);
    // Both owned TX leases remain held until each native outcome has been observed.
    let mut left = database.begin(TransactionOptions::default()).await?;
    let mut right = database.begin(TransactionOptions::default()).await?;
    let left_id: i64 = left
        .run(query("SELECT CONNECTION_ID()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at(0)?;
    let right_id: i64 = right
        .run(query("SELECT CONNECTION_ID()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at(0)?;
    assert_ne!(left_id, right_id);
    let (left_result, right_result) = tokio::join!(
        left.run(
            query("INSERT INTO m1_outcomes(value) VALUES (20)", []),
            QueryLimits::default()
        ),
        right.run(
            query("INSERT INTO m1_outcomes(value) VALUES (30)", []),
            QueryLimits::default()
        )
    );
    let (left_result, right_result) = (left_result?, right_result?);
    assert_eq!(left_result.affected, 1);
    assert_eq!(right_result.affected, 1);
    assert_ne!(left_result.insert_id, right_result.insert_id);
    assert!(left_result.insert_id.is_some());
    assert!(right_result.insert_id.is_some());
    let (left_done, right_done) = tokio::join!(left.commit(), right.commit());
    left_done?;
    right_done?;
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_outcomes").await?,
        2
    );
    // A failed statement leaves only rollback available, regardless of native TX permissiveness.
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query("INSERT INTO m1_outcomes(value) VALUES (40)", []),
            QueryLimits::default(),
        )
        .await?;
    assert!(
        transaction
            .run(
                query("SELECT bad_column FROM m1_outcomes", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    assert_eq!(
        transaction
            .run(query("SELECT 1", []), QueryLimits::default())
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::RollbackOnly
    );
    transaction.rollback().await?;
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_outcomes WHERE value=40").await?,
        0
    );

    // ID overflow is an observation failure after the native write, never a retry.
    super::run(&database,"CREATE TABLE m1_overflow (id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY) ENGINE=InnoDB AUTO_INCREMENT=9223372036854775808",[]).await?;
    assert_eq!(
        database
            .run(
                query("INSERT INTO m1_overflow () VALUES ()", []),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Decode
    );
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_overflow").await?,
        1
    );
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query("INSERT INTO m1_outcomes(value) VALUES (50)", []),
            QueryLimits::default(),
        )
        .await?;
    assert_eq!(
        transaction
            .run(
                query("/* lead */ CREATE TABLE m1_forbidden (id BIGINT)", []),
                QueryLimits::default()
            )
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    transaction.rollback().await?;
    assert_eq!(
        integer(&database, "SELECT COUNT(*) FROM m1_outcomes WHERE value=50").await?,
        0
    );
    assert!(
        database
            .run(
                query("SELECT id FROM m1_forbidden", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );

    database.close().await?;
    Ok(())
}
