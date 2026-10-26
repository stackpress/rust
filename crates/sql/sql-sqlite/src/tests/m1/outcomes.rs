//! Real CRUD, RETURNING, ordered rows and statement-local native outcomes.

use std::error::Error;
use std::sync::Arc;

use sqlx::Executor;
use stackpress_sql::{Engine, ErrorKind, SqlValue};
use tempfile::TempDir;

use crate::SqliteDatabase;
use crate::tests::{options, sql};

/// Native equivalents of SQLite inventory rows 1/2/4/5/6/7/12, using M0 SQL syntax.
pub(super) async fn crud() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    for _ in 0..2 {
        for statement in [
            sql(
                "CREATE TABLE IF NOT EXISTS profile(id INTEGER PRIMARY KEY, name TEXT NOT NULL, active INTEGER DEFAULT 0, age INTEGER DEFAULT 0, tags TEXT, refs TEXT, created TEXT DEFAULT '2026-10-26T00:00:00Z')",
            ),
            sql("CREATE UNIQUE INDEX IF NOT EXISTS profile_name ON profile(name)"),
            sql("CREATE INDEX IF NOT EXISTS profile_age ON profile(age)"),
        ] {
            let result = engine.execute(&statement).await.unwrap();
            assert!(result.rows.is_empty());
            assert_eq!(result.affected, 0);
            assert_eq!(result.insert_id, None);
        }
    }
    assert_eq!(engine.fetch(&sql("SELECT count(*) AS n FROM sqlite_master WHERE type = 'index' AND tbl_name = 'profile'")).await.unwrap()[0].try_get::<i64>("n").unwrap(), 2);
    let hostile = "Jane'; DELETE FROM profile; --🦀";
    let query = sql("INSERT INTO profile(name, age, active, tags, refs) VALUES (?, ?, ?, ?, ?), (?, ?, ?, ?, ?) RETURNING id, name, active, created")
        .bind("John").bind(30_i64).bind(false).bind("[\"foo\",\"bar\"]").bind(SqlValue::Null)
        .bind(hostile).bind(25_i64).bind(true).bind("[]").bind("{\"foo\":\"bar\"}");
    let created = engine.execute(&query).await.unwrap();
    assert_eq!(created.affected, 2);
    assert_eq!(created.insert_id, Some(2));
    assert_eq!(created.rows.len(), 2);
    assert!(!created.rows[0].try_get::<bool>("active").unwrap());
    assert!(created.rows[1].try_get::<bool>("active").unwrap());
    assert_eq!(created.rows[1].try_get::<String>("name").unwrap(), hostile);
    assert_eq!(
        created.rows[1].try_get::<String>("created").unwrap(),
        "2026-10-26T00:00:00Z"
    );
    let updated = engine.execute(&sql("-- leading ?\nWITH target AS (SELECT id FROM profile WHERE name = ?) UPDATE profile SET age = ?, active = ? WHERE id IN (SELECT id FROM target) RETURNING id, age, active")
        .bind(hostile).bind(31_i64).bind(false)).await.unwrap();
    assert_eq!(updated.affected, 1);
    assert_eq!(updated.insert_id, None);
    assert_eq!(updated.rows[0].try_get::<i64>("age").unwrap(), 31);
    assert!(!updated.rows[0].try_get::<bool>("active").unwrap());
    let selected = engine.execute(&sql("WITH target AS (SELECT * FROM profile) SELECT id, name, age, active, tags, refs FROM target ORDER BY id")).await.unwrap();
    assert_eq!(selected.rows.len(), 2);
    assert_eq!(selected.affected, 0);
    assert_eq!(selected.insert_id, None);
    assert_eq!(selected.rows[0].cells()[5].1, SqlValue::Null);
    assert_eq!(
        selected.rows[1].try_get::<String>("refs").unwrap(),
        "{\"foo\":\"bar\"}"
    );
    let failed = engine
        .execute(&sql("INSERT INTO profile(name) VALUES (?)").bind(hostile))
        .await
        .unwrap_err();
    assert!(failed.source().is_some());
    assert_eq!(failed.kind(), ErrorKind::Driver);
    assert!(!format!("{failed:?}").contains(hostile));
    assert!(
        engine
            .execute(&sql("SELECT * FROM missing_table"))
            .await
            .is_err()
    );
    let deleted = engine.execute(&sql("WITH target AS (SELECT id FROM profile WHERE name = ?) DELETE FROM profile WHERE id IN (SELECT id FROM target) RETURNING id, name").bind(hostile)).await.unwrap();
    assert_eq!(deleted.affected, 1);
    assert_eq!(deleted.insert_id, None);
    assert_eq!(deleted.rows[0].try_get::<i64>("id").unwrap(), 2);
    let empty = engine
        .execute(&sql("DELETE FROM profile WHERE id = 999 RETURNING id"))
        .await
        .unwrap();
    assert!(empty.rows.is_empty());
    assert_eq!(empty.affected, 0);
    assert_eq!(empty.insert_id, None);
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    let rows = reopened
        .fetch(&sql("SELECT name FROM profile"))
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].try_get::<String>("name").unwrap(), "John");
    reopened.close().await.unwrap();
}

/// Optional ID observations remain accurate for rowid reuse/zero, triggers and concurrent inserts.
pub(super) async fn metadata() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    // A native fixture installs a trigger outside the intentionally restricted raw API.
    // The trigger tests metadata isolation; no production native-resource API is exposed.
    let mut connection = db.pool.acquire().await.unwrap();
    connection
        .execute("CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT UNIQUE)")
        .await
        .unwrap();
    connection
        .execute("CREATE TABLE audit(id INTEGER PRIMARY KEY)")
        .await
        .unwrap();
    connection.execute("CREATE TRIGGER audited AFTER UPDATE ON profile BEGIN INSERT INTO audit VALUES (999); END").await.unwrap();
    drop(connection);
    let engine = Engine::new(db);
    assert_eq!(
        engine
            .execute(&sql("INSERT INTO profile VALUES (1, 'first')"))
            .await
            .unwrap()
            .insert_id,
        Some(1)
    );
    let updated = engine
        .execute(&sql(
            "UPDATE profile SET name = 'second' WHERE id = 1 RETURNING id",
        ))
        .await
        .unwrap();
    assert_eq!(updated.affected, 1);
    assert_eq!(updated.insert_id, None);
    assert_eq!(
        engine.fetch(&sql("SELECT id FROM audit")).await.unwrap()[0]
            .try_get::<i64>("id")
            .unwrap(),
        999
    );
    assert_eq!(
        engine
            .execute(&sql("DELETE FROM profile WHERE id = 1"))
            .await
            .unwrap()
            .insert_id,
        None
    );
    // Native last-rowid is still 1. Reusing it cannot be distinguished by its transition;
    // None is honest, and RETURNING still carries the exact inserted key.
    let reused = engine
        .execute(&sql(
            "INSERT INTO profile VALUES (1, 'reused') RETURNING id",
        ))
        .await
        .unwrap();
    assert_eq!(reused.insert_id, None);
    assert_eq!(reused.rows[0].try_get::<i64>("id").unwrap(), 1);
    let zero = engine
        .execute(&sql("INSERT INTO profile VALUES (0, 'zero') RETURNING id"))
        .await
        .unwrap();
    assert_eq!(zero.insert_id, Some(0));
    assert_eq!(zero.rows[0].try_get::<i64>("id").unwrap(), 0);
    let ignored = engine
        .execute(&sql("INSERT OR IGNORE INTO profile VALUES (1, 'ignored')"))
        .await
        .unwrap();
    assert_eq!(ignored.affected, 0);
    assert_eq!(ignored.insert_id, None);
    engine
        .execute(&sql(
            "CREATE TABLE no_rowid(name TEXT PRIMARY KEY) WITHOUT ROWID",
        ))
        .await
        .unwrap();
    let no_rowid = engine
        .execute(&sql("INSERT INTO no_rowid VALUES ('key') RETURNING name"))
        .await
        .unwrap();
    assert_eq!(no_rowid.affected, 1);
    assert_eq!(no_rowid.insert_id, None);
    for statement in [
        sql("CREATE TABLE copied AS SELECT * FROM profile"),
        sql("EXPLAIN UPDATE profile SET name = 'never'"),
    ] {
        let outcome = engine.execute(&statement).await.unwrap();
        assert_eq!(outcome.affected, 0);
        assert_eq!(outcome.insert_id, None);
    }
    let joined = engine
        .fetch(&sql(
            "SELECT p.id, p.id, p.name AS \"long 'quoted' alias λ\" FROM profile p WHERE p.id = 1",
        ))
        .await
        .unwrap();
    assert_eq!(joined[0].cells().len(), 3);
    assert!(joined[0].try_get::<i64>("id").is_err());
    assert_eq!(
        joined[0]
            .try_get::<String>("long 'quoted' alias λ")
            .unwrap(),
        "reused"
    );
    engine.close().await.unwrap();

    let concurrent_root = TempDir::new().unwrap();
    let native = SqliteDatabase::open(options(&concurrent_root, 4))
        .await
        .unwrap();
    let shared = Arc::new(Engine::new(native));
    shared
        .execute(&sql(
            "CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT UNIQUE)",
        ))
        .await
        .unwrap();
    let ready = Arc::new(tokio::sync::Barrier::new(4));
    let mut tasks = Vec::new();
    for id in 1..=4_i64 {
        let engine = Arc::clone(&shared);
        let ready = Arc::clone(&ready);
        tasks.push(tokio::spawn(async move {
            ready.wait().await;
            let result = engine
                .execute(
                    &sql("INSERT INTO profile VALUES (?, ?) RETURNING id")
                        .bind(id)
                        .bind(format!("profile-{id}")),
                )
                .await
                .unwrap();
            assert_eq!(result.affected, 1);
            assert_eq!(result.rows[0].try_get::<i64>("id").unwrap(), id);
            assert_eq!(result.insert_id, Some(id));
        }));
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert_eq!(
        shared
            .fetch(&sql("SELECT count(*) AS n FROM profile"))
            .await
            .unwrap()[0]
            .try_get::<i64>("n")
            .unwrap(),
        4
    );
    shared.close().await.unwrap();
}
