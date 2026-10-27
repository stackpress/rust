//! Public core plan APIs execute real SQLite DDL with explicit transaction ownership.

use std::sync::{Arc, Mutex, mpsc};

use stackpress_sql::{
    Alter, Create, DialectId, Engine, ErrorKind, Field, Index, Insert, Order, Select, SqlType,
};
use tokio::sync::oneshot;

use super::{SqliteDatabase, TempDir, options, schema, sql};

/// Cross-dialect plans fail before acquiring a lease, including inside a healthy caller guard.
pub(super) async fn validation() {
    let root = TempDir::new().unwrap();
    let engine = super::super::engine(&root, 1).await;
    engine
        .execute(
            &Insert::new("profile")
                .columns(["name"])
                .values(["preserved"]),
        )
        .await
        .unwrap();
    let before = schema(&engine).await;
    let rows = engine
        .fetch(&Select::new(["*"]).from("profile"))
        .await
        .unwrap();
    let create = Create::new("wrong_dialect")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("wrong_index", ["id"]));
    let mut transaction = engine.begin().await.unwrap();
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        let plan = create.render(dialect).unwrap();
        // Holding the only pool lease distinguishes validation from acquisition timeout.
        assert_eq!(
            engine.execute_plan(&plan).await.unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        assert_eq!(
            transaction.execute_plan(&plan).await.unwrap_err().kind(),
            ErrorKind::Unsupported
        );
        // A rejected plan never ran a statement or poisoned this caller-owned transaction.
        assert_eq!(
            transaction
                .fetch(&Select::new(["*"]).from("profile"))
                .await
                .unwrap(),
            rows
        );
    }
    transaction.rollback().await.unwrap();
    assert_eq!(schema(&engine).await, before);
    assert_eq!(
        engine
            .fetch(&Select::new(["*"]).from("profile"))
            .await
            .unwrap(),
        rows
    );
    engine.close().await.unwrap();
}

/// Cancel the public owned-batch future inside native statement two, after statement one's ALTER.
pub(super) async fn cancellation() {
    let root = TempDir::new().unwrap();
    let db = SqliteDatabase::open(options(&root, 1)).await.unwrap();
    let (ready, entered) = oneshot::channel();
    let (release, proceed) = mpsc::channel();
    let gate = Mutex::new(Some((ready, proceed)));
    let mut connection = db.pool.acquire().await.unwrap();
    connection
        .lock_handle()
        .await
        .unwrap()
        .create_collation("plan_gate", move |left, right| {
            // SQLx requires a Sync/Fn comparator. Take its one-shot state and release
            // the test-only lock before signaling or waiting on the native worker.
            let pause = gate.lock().unwrap().take();
            if let Some((ready, proceed)) = pause {
                ready.send(()).unwrap();
                // Only the later text index compares these values; setup and ADD do not sort them.
                // The external process watchdog bounds this native handshake independently.
                proceed.recv().unwrap();
            }
            left.cmp(right)
        })
        .unwrap();
    drop(connection);
    let engine = Arc::new(Engine::new(db));
    engine
        .execute(&sql(
            "CREATE TABLE original(id INTEGER PRIMARY KEY, value TEXT COLLATE plan_gate)",
        ))
        .await
        .unwrap();
    engine
        .execute(&sql(
            "INSERT INTO original VALUES (1, 'z'), (2, 'a'), (3, 'm')",
        ))
        .await
        .unwrap();
    let before = schema(&engine).await;
    let query = Select::new(["id", "value"])
        .from("original")
        .order_by("id", Order::Asc);
    let rows = engine.fetch(&query).await.unwrap();
    let plan = Alter::new("original")
        .add_field(Field::new("partial", SqlType::Integer))
        .add_index(Index::new("partial_index", ["value"]))
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(plan.queries().len(), 2);
    let shared = Arc::clone(&engine);
    let task = tokio::spawn(async move { shared.execute_plan(&plan).await });
    entered.await.unwrap();
    // Native sorting proves statement two is executing, so the first ADD is already acknowledged.
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    release.send(()).unwrap();
    assert_eq!(schema(&engine).await, before);
    assert_eq!(engine.fetch(&query).await.unwrap(), rows);
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(schema(&reopened).await, before);
    assert_eq!(reopened.fetch(&query).await.unwrap(), rows);
    reopened.close().await.unwrap();
}

/// A borrowed batch never commits its caller's work; rollback/commit remain explicit afterward.
pub(super) async fn caller_transaction() {
    let root = TempDir::new().unwrap();
    let engine = super::super::engine(&root, 2).await;
    engine
        .execute(
            &Insert::new("profile")
                .columns(["name"])
                .values(["committed baseline"]),
        )
        .await
        .unwrap();
    let before = schema(&engine).await;
    let plan = Create::new("caller_table")
        .add_field(Field::new("id", SqlType::Integer))
        .add_field(Field::new("value", SqlType::Text))
        .add_index(Index::new("caller_index", ["value"]))
        .render(DialectId::Sqlite)
        .unwrap();
    let query = Select::new(["value"]).from("caller_table");
    for commit in [false, true] {
        let mut transaction = engine.begin().await.unwrap();
        transaction
            .execute(
                &Insert::new("profile")
                    .columns(["name"])
                    .values(["caller work"]),
            )
            .await
            .unwrap();
        let results = transaction.execute_plan(&plan).await.unwrap();
        assert_eq!(results.len(), 2);
        // This ordinary query after execute_plan proves ownership and the same session remain live.
        transaction
            .execute(
                &Insert::new("caller_table")
                    .columns(["id", "value"])
                    .values([stackpress_sql::SqlValue::Integer(1), "inside".into()]),
            )
            .await
            .unwrap();
        tokio::task::yield_now().await;
        assert_eq!(
            transaction.fetch(&query).await.unwrap()[0]
                .try_get::<String>("value")
                .unwrap(),
            "inside"
        );
        assert_eq!(schema(&engine).await, before);
        assert_eq!(super::super::count(&engine).await, 1);
        if commit {
            transaction.commit().await.unwrap();
        } else {
            transaction.rollback().await.unwrap();
            assert_eq!(schema(&engine).await, before);
            assert_eq!(super::super::count(&engine).await, 1);
        }
    }
    assert_eq!(super::super::count(&engine).await, 2);
    assert_eq!(
        engine.fetch(&query).await.unwrap()[0]
            .try_get::<String>("value")
            .unwrap(),
        "inside"
    );
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(super::super::count(&reopened).await, 2);
    assert_eq!(
        reopened.fetch(&query).await.unwrap()[0]
            .try_get::<String>("value")
            .unwrap(),
        "inside"
    );
    reopened.close().await.unwrap();
}
