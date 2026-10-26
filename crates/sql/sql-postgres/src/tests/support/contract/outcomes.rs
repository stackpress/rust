//! Shared public-trait outcomes assertions; native syntax differences stay in the suite inputs.
use super::{Database, DialectId, Result, run};

/// Verify native statement-local counts/IDs and supported RETURNING without sticky state.
pub(super) async fn check(database: &dyn Database) -> Result<()> {
    let create = match database.dialect() {
        DialectId::Postgres => {
            "CREATE TABLE contract_items (id BIGSERIAL PRIMARY KEY, value BIGINT)"
        }
        DialectId::Mysql => {
            "CREATE TABLE contract_items (id BIGINT AUTO_INCREMENT PRIMARY KEY, value BIGINT) ENGINE=InnoDB"
        }
        DialectId::Sqlite => "CREATE TABLE contract_items (id INTEGER PRIMARY KEY, value BIGINT)",
    };
    assert_eq!(run(database, create, []).await?.affected, 0);
    let insert = run(
        database,
        "INSERT INTO contract_items(value) VALUES (10)",
        [],
    )
    .await?;
    assert_eq!(insert.affected, 1);
    assert_eq!(
        insert.insert_id,
        if database.dialect() == DialectId::Postgres {
            None
        } else {
            Some(1)
        }
    );
    let update = run(
        database,
        "UPDATE contract_items SET value=11 WHERE id=1",
        [],
    )
    .await?;
    assert_eq!((update.affected, update.insert_id), (1, None));
    let select = run(database, "SELECT value FROM contract_items WHERE id=1", []).await?;
    assert_eq!(select.insert_id, None);
    assert_eq!(select.rows[0].try_get_at::<i64>(0)?, 11);
    // Capabilities encode actual syntax differences; generated values remain observable.
    if database.dialect().capabilities().returning {
        let returned = run(
            database,
            "INSERT INTO contract_items(value) VALUES (12) RETURNING id",
            [],
        )
        .await?;
        assert_eq!(returned.affected, 1);
        assert_eq!(returned.rows[0].try_get_at::<i64>(0)?, 2);
    } else {
        assert_eq!(
            run(
                database,
                "INSERT INTO contract_items(value) VALUES (12)",
                []
            )
            .await?
            .insert_id,
            Some(2)
        );
    }
    let ddl = run(database, "CREATE TABLE contract_ddl (id BIGINT)", []).await?;
    assert_eq!((ddl.affected, ddl.insert_id), (0, None));
    let delete = run(database, "DELETE FROM contract_items WHERE value=12", []).await?;
    assert_eq!((delete.affected, delete.insert_id), (1, None));
    Ok(())
}
