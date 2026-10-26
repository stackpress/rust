//! Merged core builders execute on SQLite, including joins and explicit transaction affinity.

use std::error::Error;

use stackpress_sql::{
    Compare, Delete, Engine, ErrorKind, Expr, Identifier, Insert, Join, JoinKind, Order, Select,
    Selector, SqlValue, TableRef, Transaction, Update, WriteValue,
};

use super::{SqliteDatabase, TempDir, options, sql};

/// Qualify a column without interpreting a dotted string as executable syntax.
fn column(table: &str, name: &str) -> Identifier {
    Identifier::qualified([table, name])
}

/// Every DML family, compound predicates, ordering/pagination and aliases execute as rendered.
pub(super) async fn builders() {
    let root = TempDir::new().unwrap();
    let engine = Engine::new(SqliteDatabase::open(options(&root, 2)).await.unwrap());
    engine.execute(&sql("CREATE TABLE profile(id INTEGER PRIMARY KEY, name TEXT DEFAULT 'fallback', active INTEGER DEFAULT 0)")).await.unwrap();
    let hostile = "'; DELETE FROM profile; -- λ";
    let query = Insert::new("profile")
        .columns(["id", "name", "active"])
        .values([SqlValue::Integer(1), hostile.into(), false.into()])
        .values([SqlValue::Integer(2), "Ada".into(), true.into()])
        .values([SqlValue::Integer(3), "Ada".into(), true.into()])
        .returning_all();
    let original = query.clone();
    assert_eq!(engine.execute(&query).await.unwrap().rows.len(), 3);
    assert_eq!(query, original);
    let defaulted = engine
        .execute(
            &Insert::new("profile")
                .columns(["id", "name"])
                .values([WriteValue::from(4_i64), WriteValue::Default])
                .returning_all(),
        )
        .await
        .unwrap();
    assert_eq!(
        defaulted.rows[0].try_get::<String>("name").unwrap(),
        "fallback"
    );
    let select = Select::new(["id", "name"])
        .from("profile")
        .and_where(Expr::all([
            Expr::any([Expr::eq("active", true), Expr::eq("name", hostile)]),
            Expr::Not(Box::new(Expr::eq("id", 3_i64))),
            Expr::In(
                Box::new(Expr::column("id")),
                vec![1_i64.into(), 2_i64.into()],
                false,
            ),
        ]))
        .order_by("id", Order::Desc)
        .limit(1)
        .offset(1);
    let rows = engine.fetch(&select).await.unwrap();
    assert_eq!(rows[0].try_get::<i64>("id").unwrap(), 1);
    assert_eq!(rows[0].try_get::<String>("name").unwrap(), hostile);
    assert_eq!(
        engine
            .fetch(
                &Select::new(["name"])
                    .from("profile")
                    .distinct()
                    .where_eq("active", true)
            )
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        engine
            .fetch(&Select::new(["id"]).from("profile").limit(0))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        engine
            .fetch(
                &Select::new(["id"])
                    .from("profile")
                    .order_by("id", Order::Asc)
                    .offset(3)
            )
            .await
            .unwrap()[0]
            .try_get::<i64>("id")
            .unwrap(),
        4
    );
    let changed = engine
        .execute(
            &Update::new("profile")
                .set("name", "changed")
                .and_where(Expr::column("id").compare(Compare::Ge, Expr::value(2_i64)))
                .and_where(Expr::column("id").compare(Compare::Le, Expr::value(3_i64)))
                .returning(["id", "name"]),
        )
        .await
        .unwrap();
    assert_eq!(changed.affected, 2);
    assert_eq!(changed.rows.len(), 2);
    assert_eq!(changed.insert_id, None);
    let removed = engine
        .execute(
            &Delete::new("profile")
                .where_eq("id", 3_i64)
                .returning(["id"]),
        )
        .await
        .unwrap();
    assert_eq!(removed.affected, 1);
    assert_eq!(removed.rows[0].try_get::<i64>("id").unwrap(), 3);
    joins(&engine).await;
    engine.close().await.unwrap();
}

/// Native joins retain duplicate names; only explicit aliases make named lookup unambiguous.
async fn joins(engine: &Engine) {
    engine
        .execute(&sql("CREATE TABLE labels(id INTEGER, label TEXT)"))
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("labels")
                .columns(["id", "label"])
                .values([SqlValue::Integer(2), "label".into()]),
        )
        .await
        .unwrap();
    let join = Join {
        kind: JoinKind::Left,
        table: TableRef::new("labels").alias("l"),
        on: Some(
            Expr::column(column("p", "id")).compare(Compare::Eq, Expr::column(column("l", "id"))),
        ),
    };
    let select = Select::new([column("p", "id"), column("l", "id")])
        .from(TableRef::new("profile").alias("p"))
        .join(join.clone())
        .order_by(column("p", "id"), Order::Asc);
    let rows = engine.fetch(&select).await.unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(
        rows[0].try_get::<i64>("id").unwrap_err().kind(),
        ErrorKind::Decode
    );
    assert_eq!(rows[0].try_get_at::<Option<i64>>(1).unwrap(), None);
    assert_eq!(rows[1].try_get_at::<i64>(1).unwrap(), 2);
    let metadata = rows[1].columns().collect::<Vec<_>>();
    assert_eq!((metadata[0].name, metadata[1].name), ("id", "id"));
    assert_eq!((metadata[0].ordinal, metadata[1].ordinal), (0, 1));
    let alias = "joined.\"; SELECT -- λ";
    let rows = engine
        .fetch(
            &Select::new([
                Selector::expression(Expr::column(column("p", "id"))).alias("profile_id"),
                Selector::expression(Expr::column(column("l", "label"))).alias(alias),
            ])
            .from(TableRef::new("profile").alias("p"))
            .join(join)
            .where_eq(column("p", "id"), 2_i64),
        )
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>(alias).unwrap(), "label");
}

/// A reusable application action accepts a mutable guard, never a pool captured in a closure.
async fn rename(transaction: &mut Transaction, id: i64, name: &str) {
    transaction
        .execute(&Update::new("profile").set("name", name).where_eq("id", id))
        .await
        .unwrap();
}

/// Builders see their own writes on one lease; outside reads and rollback remain isolated.
pub(super) async fn transactions() {
    let root = TempDir::new().unwrap();
    let engine = super::super::engine(&root, 2).await;
    engine
        .execute(
            &Insert::new("profile")
                .columns(["id", "name"])
                .values([SqlValue::Integer(1), "original".into()]),
        )
        .await
        .unwrap();
    let mut transaction = engine.begin().await.unwrap();
    rename(&mut transaction, 1, "inside").await;
    tokio::task::yield_now().await;
    let lookup = Select::new(["name"]).from("profile").where_eq("id", 1_i64);
    assert_eq!(
        transaction.fetch(&lookup).await.unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "inside"
    );
    assert_eq!(
        engine.fetch(&lookup).await.unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "original"
    );
    transaction.rollback().await.unwrap();
    assert_eq!(
        engine.fetch(&lookup).await.unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "original"
    );
    let mut transaction = engine.begin().await.unwrap();
    rename(&mut transaction, 1, "committed").await;
    transaction.commit().await.unwrap();
    let mut transaction = engine.begin().await.unwrap();
    rename(&mut transaction, 1, "rolled back on constraint").await;
    let primary = transaction
        .execute(
            &Insert::new("profile")
                .columns(["id", "name"])
                .values([SqlValue::Integer(1), "duplicate".into()]),
        )
        .await
        .unwrap_err();
    assert!(primary.source().is_some());
    assert_eq!(
        transaction.fetch(&lookup).await.unwrap_err().kind(),
        ErrorKind::RollbackOnly
    );
    assert!(
        transaction
            .rollback_after(primary)
            .await
            .cleanup_error()
            .is_none()
    );
    // Dropping a completion future before its first poll never queues COMMIT or ROLLBACK.
    // The captured session must still retire and roll back its uncommitted native changes.
    for commit in [false, true] {
        let mut transaction = engine.begin().await.unwrap();
        rename(&mut transaction, 1, "unpolled completion").await;
        if commit {
            drop(transaction.commit());
        } else {
            drop(transaction.rollback());
        }
        assert_eq!(
            engine.fetch(&lookup).await.unwrap()[0]
                .try_get::<String>("name")
                .unwrap(),
            "committed"
        );
    }
    engine.close().await.unwrap();
    let reopened = Engine::new(SqliteDatabase::open(options(&root, 1)).await.unwrap());
    assert_eq!(
        reopened.fetch(&lookup).await.unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "committed"
    );
    reopened.close().await.unwrap();
}
