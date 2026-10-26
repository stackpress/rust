//! Statement-local native outcomes, concurrent exclusive leases and TX DDL policy.
use super::*;
use std::sync::Arc;

/// Outcomes never inherit an ID/count from a prior statement or another active native lease.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let database = Arc::new(open(name, 2).await?);
    super::run(
        &database,
        "CREATE TABLE m1_outcomes (id BIGSERIAL PRIMARY KEY, value BIGINT)",
        [],
    )
    .await?;
    let inserted = super::run(
        &database,
        "INSERT INTO m1_outcomes(value) VALUES (10) RETURNING id",
        [],
    )
    .await?;
    assert_eq!(inserted.affected, 1);
    assert_eq!(inserted.insert_id, None);
    assert_eq!(inserted.rows[0].try_get_at::<i64>(0)?, 1);
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
        .run(query("SELECT pg_backend_pid()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at(0)?;
    let right_id: i64 = right
        .run(query("SELECT pg_backend_pid()", []), QueryLimits::default())
        .await?
        .rows[0]
        .try_get_at(0)?;
    assert_ne!(left_id, right_id);
    let (left_result, right_result) = tokio::join!(
        left.run(
            query(
                "INSERT INTO m1_outcomes(value) VALUES (20) RETURNING id",
                []
            ),
            QueryLimits::default()
        ),
        right.run(
            query(
                "INSERT INTO m1_outcomes(value) VALUES (30) RETURNING id",
                []
            ),
            QueryLimits::default()
        )
    );
    let (left_result, right_result) = (left_result?, right_result?);
    assert_eq!(left_result.affected, 1);
    assert_eq!(right_result.affected, 1);
    assert_eq!(left_result.insert_id, None);
    assert_eq!(right_result.insert_id, None);
    assert_ne!(
        left_result.rows[0].try_get_at::<i64>(0)?,
        right_result.rows[0].try_get_at::<i64>(0)?
    );
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

    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query("CREATE TABLE m1_transactional_ddl (id BIGINT)", []),
            QueryLimits::default(),
        )
        .await?;
    transaction.rollback().await?;
    assert!(
        database
            .run(
                query("SELECT id FROM m1_transactional_ddl", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );

    database.close().await?;
    Ok(())
}
