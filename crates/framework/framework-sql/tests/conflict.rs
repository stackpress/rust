//! Native targeted conflict semantics, including two writers admitted before either insert.
#![cfg(feature = "sqlite-maintenance")]

use stackpress_sql::{
    Create, Engine, ErrorKind, Field, Insert, InsertConflict, Render, Select, SqlType,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{sync::Arc, time::Duration};

#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// A parent process bounds the entire concurrency proof, including worker cleanup.
#[test]
fn sqlite_targeted_conflicts() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_conflicts());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "sqlite_targeted_conflicts",
        "conflicts",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Keep the actual insert statement identical for ordinary, updating and no-change paths.
fn insert(id: &str, email: &str, title: &str) -> Insert {
    Insert::new("conflict_items")
        .columns(["id", "email", "title"])
        .values([id, email, title])
        .returning(["id", "email", "title"])
}

/// SQLite chooses only the declared target; an unrelated unique key remains an error.
async fn prove_conflicts() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("conflict.sqlite")).create_if_missing(true),
        )
        .await
        .unwrap(),
    ));
    let table = Create::new("conflict_items")
        .add_field(Field::new("id", SqlType::Text).not_null())
        .add_field(Field::new("email", SqlType::Text).not_null())
        .add_field(Field::new("title", SqlType::Text).not_null())
        .primary_key(["id"])
        .add_index(stackpress_sql::Index::new("unique_email", ["email"]).unique())
        .definition()
        .unwrap();
    engine
        .execute_plan(&table.create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    let initial = insert("a", "a@example", "original");
    assert_eq!(engine.fetch(&initial).await.unwrap().len(), 1);
    let updated = insert("a", "ignored@example", "ignored").on_conflict(InsertConflict::DoUpdate {
        target: vec!["id".into()],
        assignments: vec![("title".into(), "updated".into())],
    });
    let rows = engine.fetch(&updated).await.unwrap();
    assert_eq!(rows[0].try_get::<String>("title").unwrap(), "updated");
    assert_eq!(rows[0].try_get::<String>("email").unwrap(), "a@example");
    let nothing =
        insert("a", "ignored@example", "ignored").on_conflict(InsertConflict::DoNothing {
            target: vec!["id".into()],
        });
    assert!(engine.fetch(&nothing).await.unwrap().is_empty());
    // The MySQL renderer refuses before execution; no broader conflict policy is substituted.
    let refusal = nothing
        .render(stackpress_sql::DialectId::Mysql)
        .unwrap_err();
    assert_eq!(refusal.kind(), ErrorKind::Unsupported);
    assert!(
        engine
            .fetch(
                &insert("b", "a@example", "wrong").on_conflict(InsertConflict::DoUpdate {
                    target: vec!["id".into()],
                    assignments: vec![("title".into(), "wrong".into())],
                })
            )
            .await
            .is_err()
    );
    assert_eq!(
        engine
            .fetch(&Select::default().from("conflict_items"))
            .await
            .unwrap()
            .len(),
        1
    );
    engine
        .fetch(&insert("other", "other@example", "Other"))
        .await
        .unwrap();
    // A constraint failure inside the update branch rolls back that whole statement.
    assert!(
        engine
            .fetch(&insert("a", "ignored@example", "Ignored").on_conflict(
                InsertConflict::DoUpdate {
                    target: vec!["id".into()],
                    assignments: vec![
                        ("email".into(), "other@example".into()),
                        ("title".into(), "Should roll back".into())
                    ],
                }
            ))
            .await
            .is_err()
    );
    let rows = engine
        .fetch(&Select::default().from("conflict_items").where_eq("id", "a"))
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("email").unwrap(), "a@example");
    assert_eq!(rows[0].try_get::<String>("title").unwrap(), "updated");
    // Explicit caller rollback also discards an acknowledged RETURNING update.
    let mut transaction = engine.begin().await.unwrap();
    transaction.fetch(&updated).await.unwrap();
    transaction
        .fetch(&insert("temporary", "temp@example", "Temporary"))
        .await
        .unwrap();
    transaction.rollback().await.unwrap();
    assert!(
        engine
            .fetch(
                &Select::default()
                    .from("conflict_items")
                    .where_eq("id", "temporary")
            )
            .await
            .unwrap()
            .is_empty()
    );
    // Both writers wait at the same barrier with no preliminary existence SELECT.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut workers = Vec::new();
    for title in ["writer one", "writer two"] {
        let db = engine.clone();
        let barrier = barrier.clone();
        workers.push(tokio::spawn(async move {
            barrier.wait().await;
            db.fetch(&insert("race", "race@example", title).on_conflict(
                InsertConflict::DoNothing {
                    target: vec!["id".into()],
                },
            ))
            .await
            .unwrap()
        }));
    }
    let mut inserted = 0;
    for worker in workers {
        inserted += worker.await.unwrap().len();
    }
    assert_eq!(inserted, 1);
    let rows = engine
        .fetch(
            &Select::default()
                .from("conflict_items")
                .where_eq("id", "race"),
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert!(
        ["writer one", "writer two"]
            .contains(&rows[0].try_get::<String>("title").unwrap().as_str())
    );
    engine.close().await.unwrap();
}

/// A MySQL identity with no executable backend proves refusal occurs above the adapter seam.
struct NoMysqlEffects;
impl stackpress_sql::Database for NoMysqlEffects {
    /// Select the dialect whose conflict policy cannot preserve a restricted target.
    fn dialect(&self) -> stackpress_sql::DialectId {
        stackpress_sql::DialectId::Mysql
    }
    /// Reaching native execution would be a regression even if the driver later refused.
    fn run(
        &self,
        _: stackpress_sql::Query,
        _: stackpress_sql::QueryLimits,
    ) -> stackpress_sql::SqlFuture<'_, stackpress_sql::QueryResult> {
        panic!("MySQL conflict reached native execution")
    }
    /// A single-statement refusal must not acquire a hidden transaction either.
    fn begin(
        &self,
        _: stackpress_sql::TransactionOptions,
    ) -> stackpress_sql::SqlFuture<'_, Box<dyn stackpress_sql::TransactionSession>> {
        panic!("MySQL conflict acquired a transaction")
    }
    /// No native resource exists in this refusal-only fixture.
    fn close(&self) -> stackpress_sql::SqlFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

/// Neither conflict action can substitute MySQL's broader any-unique-key behavior.
#[tokio::test]
async fn mysql_conflict_refuses_before_effects() {
    let engine = Engine::new(NoMysqlEffects);
    for action in [
        InsertConflict::DoNothing {
            target: vec!["id".into()],
        },
        InsertConflict::DoUpdate {
            target: vec!["id".into()],
            assignments: vec![("title".into(), "Changed".into())],
        },
    ] {
        let query = Insert::new("items")
            .columns(["id"])
            .values(["a"])
            .on_conflict(action);
        let refusal = engine.execute(&query).await.unwrap_err();
        assert_eq!(refusal.kind(), ErrorKind::Unsupported);
        assert!(refusal.to_string().contains("cannot be restricted"));
    }
    engine.close().await.unwrap();
}
