//! Shared public-trait transactions assertions; native syntax differences stay in the suite inputs.
use super::{Database, ErrorKind, QueryLimits, Result, TransactionOptions, integer, query, run};

/// Exercise affinity, explicit commit/rollback, poison, rejected runs and owned Drop completion.
pub(super) async fn check(database: &dyn Database) -> Result<()> {
    let dialect = database.dialect();
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query(dialect, "INSERT INTO contract_items(value) VALUES (20)", []),
            QueryLimits::default(),
        )
        .await?;
    let own = transaction
        .run(
            query(
                dialect,
                "SELECT COUNT(*) FROM contract_items WHERE value=20",
                [],
            ),
            QueryLimits::default(),
        )
        .await?;
    assert_eq!(own.rows[0].try_get_at::<i64>(0)?, 1);
    transaction.commit().await?;
    assert_eq!(
        integer(
            database,
            "SELECT COUNT(*) FROM contract_items WHERE value=20"
        )
        .await?,
        1
    );
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query(dialect, "INSERT INTO contract_items(value) VALUES (30)", []),
            QueryLimits::default(),
        )
        .await?;
    transaction.rollback().await?;
    assert_eq!(
        integer(
            database,
            "SELECT COUNT(*) FROM contract_items WHERE value=30"
        )
        .await?,
        0
    );
    // Native error and pre-effect validation both poison the same portable session contract.
    for invalid in [false, true] {
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query(dialect, "INSERT INTO contract_items(value) VALUES (40)", []),
                QueryLimits::default(),
            )
            .await?;
        let (sql, limits) = if invalid {
            (
                "SELECT 1",
                QueryLimits {
                    sql_bytes: 0,
                    ..QueryLimits::default()
                },
            )
        } else {
            (
                "SELECT missing_column FROM contract_items",
                QueryLimits::default(),
            )
        };
        assert!(
            transaction
                .run(query(dialect, sql, []), limits)
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
        assert_eq!(
            integer(
                database,
                "SELECT COUNT(*) FROM contract_items WHERE value=40"
            )
            .await?,
            0
        );
    }
    for completion in [0, 1, 2] {
        let mut transaction = database.begin(TransactionOptions::default()).await?;
        transaction
            .run(
                query(dialect, "INSERT INTO contract_items(value) VALUES (50)", []),
                QueryLimits::default(),
            )
            .await?;
        match completion {
            0 => drop(transaction),
            1 => drop(transaction.commit()),
            _ => drop(transaction.rollback()),
        }
        assert_eq!(
            integer(
                database,
                "SELECT COUNT(*) FROM contract_items WHERE value=50"
            )
            .await?,
            0
        );
    }
    // The same capability branch proves transactional DDL or rejection before an implicit commit.
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query(dialect, "INSERT INTO contract_items(value) VALUES (65)", []),
            QueryLimits::default(),
        )
        .await?;
    let ddl = transaction
        .run(
            query(dialect, "CREATE TABLE contract_tx_ddl (id BIGINT)", []),
            QueryLimits::default(),
        )
        .await;
    if dialect.capabilities().transactional_ddl {
        ddl?;
    } else {
        assert_eq!(ddl.unwrap_err().kind(), ErrorKind::Unsupported);
    }
    transaction.rollback().await?;
    assert_eq!(
        integer(
            database,
            "SELECT COUNT(*) FROM contract_items WHERE value=65"
        )
        .await?,
        0
    );
    assert!(
        run(database, "SELECT id FROM contract_tx_ddl", [])
            .await
            .is_err()
    );
    // A poisoned commit consumes ownership and never commits the preceding write.
    let mut transaction = database.begin(TransactionOptions::default()).await?;
    transaction
        .run(
            query(dialect, "INSERT INTO contract_items(value) VALUES (60)", []),
            QueryLimits::default(),
        )
        .await?;
    assert!(
        transaction
            .run(
                query(dialect, "SELECT missing_column FROM contract_items", []),
                QueryLimits::default()
            )
            .await
            .is_err()
    );
    assert_eq!(
        transaction.commit().await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    assert_eq!(
        integer(
            database,
            "SELECT COUNT(*) FROM contract_items WHERE value=60"
        )
        .await?,
        0
    );
    Ok(())
}
