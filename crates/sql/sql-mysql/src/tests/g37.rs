//! Opt-in G38 DDL proof: unchanged generated SQL, own metadata and bound-value round trips.
use super::*;
use stackpress_sql::{Engine, Insert};

/// The permanent generator proof checks these bytes against freshly emitted all-dialect SQL.
const DDL: &str = include_str!("../../../../framework/framework-sql/tests/fixtures/g37-ddl.json");

/// Borrow unescaped MySQL statements directly from the static checked fixture.
#[derive(serde::Deserialize)]
struct FixtureSql<'a> {
    #[serde(borrow)]
    mysql: Vec<&'a str>,
}

/// Close the owned pool on every ordinary result before the parent exact-name DROP runs.
pub(super) async fn run(name: &str) -> TestResult<()> {
    let engine = Engine::new(open_config(name, stress::Config::contract()).await?);
    let result = prove(&engine).await;
    let close = engine.close().await;
    result?;
    close?;
    Ok(())
}

/// Exercise only the fresh annotated fixture; this is not generated CRUD or migration apply.
async fn prove(engine: &Engine) -> TestResult<()> {
    let ddl: FixtureSql<'static> = serde_json::from_str(DDL)?;
    let statements = ddl.mysql;
    assert_eq!(statements.len(), 3);
    for statement in statements {
        // No replacements, width overrides or wrapper DDL: execute each emitted statement verbatim.
        engine.execute(&Sql::literal(DIALECT, statement)).await?;
    }
    metadata(engine).await?;
    text_values(engine).await?;
    boundary(engine).await?;
    println!(
        "G38: 3 unchanged generated tables, columns/indexes/FKs, exact text and key boundary passed"
    );
    Ok(())
}

/// Read metadata only from DATABASE(), the parent-created throwaway database.
async fn metadata(engine: &Engine) -> TestResult<()> {
    let columns = engine.execute(&Sql::literal(DIALECT,
        "SELECT TABLE_NAME, COLUMN_NAME, DATA_TYPE, CHARACTER_MAXIMUM_LENGTH, CHARACTER_SET_NAME, COLLATION_NAME FROM INFORMATION_SCHEMA.COLUMNS WHERE TABLE_SCHEMA=DATABASE() ORDER BY TABLE_NAME, ORDINAL_POSITION")).await?.rows;
    let expected = [
        ("profile", "id", 2),
        ("profile", "name", 2),
        ("profile", "secret", 5),
        ("profile", "token", 7),
        ("project", "id", 2),
        ("project", "owner_id", 2),
        ("project", "title", 2),
        ("task", "id", 2),
        ("task", "project_id", 2),
        ("task", "title", 2),
    ];
    assert_eq!(columns.len(), 11);
    for (table, field, width) in expected {
        let row = columns
            .iter()
            .find(|row| {
                row.try_get::<String>("TABLE_NAME").ok().as_deref() == Some(table)
                    && row.try_get::<String>("COLUMN_NAME").ok().as_deref() == Some(field)
            })
            .ok_or("missing own column")?;
        assert_eq!(row.try_get::<String>("DATA_TYPE")?, "varchar");
        assert_eq!(row.try_get::<i64>("CHARACTER_MAXIMUM_LENGTH")?, width);
        assert_eq!(row.try_get::<String>("CHARACTER_SET_NAME")?, "utf8mb4");
        assert_eq!(row.try_get::<String>("COLLATION_NAME")?, "utf8mb4_bin");
    }
    let tables = engine.execute(&Sql::literal(DIALECT,
        "SELECT ENGINE, ROW_FORMAT, TABLE_COLLATION FROM INFORMATION_SCHEMA.TABLES WHERE TABLE_SCHEMA=DATABASE()")).await?.rows;
    assert_eq!(tables.len(), 3);
    for row in tables {
        assert_eq!(row.try_get::<String>("ENGINE")?, "InnoDB");
        assert_eq!(row.try_get::<String>("ROW_FORMAT")?, "Dynamic");
        assert_eq!(row.try_get::<String>("TABLE_COLLATION")?, "utf8mb4_bin");
    }
    let indexes = engine.execute(&Sql::literal(DIALECT,
        "SELECT TABLE_NAME, INDEX_NAME, COLUMN_NAME, SUB_PART FROM INFORMATION_SCHEMA.STATISTICS WHERE TABLE_SCHEMA=DATABASE() ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX")).await?.rows;
    assert_eq!(indexes.len(), 7); // Three PKs, two explicit indexes, two full FK support indexes.
    for row in &indexes {
        assert_eq!(row.try_get::<Option<i64>>("SUB_PART")?, None);
    }
    for (table, index, column) in [
        ("profile", "PRIMARY", "id"),
        ("project", "PRIMARY", "id"),
        ("task", "PRIMARY", "id"),
        ("profile", "profile_name_index", "name"),
        ("project", "project_title_index", "title"),
    ] {
        assert!(
            indexes
                .iter()
                .any(
                    |row| row.try_get::<String>("TABLE_NAME").ok().as_deref() == Some(table)
                        && row.try_get::<String>("INDEX_NAME").ok().as_deref() == Some(index)
                        && row.try_get::<String>("COLUMN_NAME").ok().as_deref() == Some(column)
                )
        );
    }
    let foreign = engine.execute(&Sql::literal(DIALECT,
        "SELECT TABLE_NAME, COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME FROM INFORMATION_SCHEMA.KEY_COLUMN_USAGE WHERE TABLE_SCHEMA=DATABASE() AND REFERENCED_TABLE_SCHEMA=DATABASE() ORDER BY TABLE_NAME")).await?.rows;
    assert_eq!(foreign.len(), 2);
    for (row, (table, column, target)) in foreign.iter().zip([
        ("project", "owner_id", "profile"),
        ("task", "project_id", "project"),
    ]) {
        assert_eq!(row.try_get::<String>("TABLE_NAME")?, table);
        assert_eq!(row.try_get::<String>("COLUMN_NAME")?, column);
        assert_eq!(row.try_get::<String>("REFERENCED_TABLE_NAME")?, target);
        assert_eq!(row.try_get::<String>("REFERENCED_COLUMN_NAME")?, "id");
    }
    Ok(())
}

/// Bound raw fixture inserts prove storage, including NUL/trailing spaces and actual FK checks.
async fn text_values(engine: &Engine) -> TestResult<()> {
    let id = "😀😀";
    engine
        .execute(
            &Insert::new("profile")
                .columns(["id", "name", "secret", "token"])
                .values([id, "e\u{301}", "a\0b", "a  "]),
        )
        .await?;
    engine
        .execute(
            &Insert::new("project")
                .columns(["id", "owner_id", "title"])
                .values(["p1", id, "a "]),
        )
        .await?;
    engine
        .execute(
            &Insert::new("task")
                .columns(["id", "project_id", "title", "done"])
                .values([
                    SqlValue::from("t1"),
                    "p1".into(),
                    "\0 ".into(),
                    false.into(),
                ]),
        )
        .await?;
    let rows = engine.execute(&Sql::literal(DIALECT,
        "SELECT p.id,p.name,p.secret,p.token,j.title AS project_title,t.title AS task_title FROM profile p JOIN project j ON j.owner_id=p.id JOIN task t ON t.project_id=j.id")).await?.rows;
    assert_eq!(rows.len(), 1);
    for (column, value) in [
        ("id", id),
        ("name", "e\u{301}"),
        ("secret", "a\0b"),
        ("token", "a  "),
        ("project_title", "a "),
        ("task_title", "\0 "),
    ] {
        assert_eq!(rows[0].try_get::<String>(column)?, value);
    }
    assert!(
        engine
            .execute(
                &Insert::new("project")
                    .columns(["id", "owner_id", "title"])
                    .values(["p2", "xx", "ok"])
            )
            .await
            .is_err()
    );
    Ok(())
}

/// A full 3,072-byte utf8mb4 key succeeds; the next full key refuses on this local target.
async fn boundary(engine: &Engine) -> TestResult<()> {
    for (sql, accepted) in [
        (
            "CREATE TABLE key_boundary_ok (id VARCHAR(768) NOT NULL PRIMARY KEY) ENGINE=InnoDB ROW_FORMAT=DYNAMIC DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin",
            true,
        ),
        (
            "CREATE TABLE key_boundary_refused (id VARCHAR(769) NOT NULL PRIMARY KEY) ENGINE=InnoDB ROW_FORMAT=DYNAMIC DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_bin",
            false,
        ),
    ] {
        // Separate trusted boundary fixtures bypass pure planning so this tests the native limit.
        let result = engine.execute(&Sql::literal(DIALECT, sql)).await;
        assert_eq!(result.is_ok(), accepted);
        if let Err(error) = result {
            // A native key-too-long response proves this did not stop at a pure capability gate.
            let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(&error);
            let mut native_number = None;
            while let Some(error) = cause {
                if let Some(sqlx::Error::Database(database)) = error.downcast_ref::<sqlx::Error>() {
                    native_number = database
                        .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
                        .map(|error| error.number());
                    break;
                }
                cause = error.source();
            }
            assert_eq!(native_number, Some(1071));
        }
    }
    Ok(())
}
