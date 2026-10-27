//! Concrete reduced-executor budget case motivating a read-only SQL-core accessor.
#![cfg(feature = "sqlite-maintenance")]

use stackpress_framework_sql::DataSession;
use stackpress_sql::{
    Create, Engine, ErrorKind, Expr, Field, Insert, QueryLimits, Render, Select, SqlType,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::time::Duration;

#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Bound native acquisition, transaction execution and close with a separate parent.
#[test]
fn reduced_executor_bind_budget() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_reduced_budget());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "reduced_executor_bind_budget",
        "executor-budget",
        Duration::from_secs(10),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// One distinct key needs one bind; no predicates or unrelated parameters are hidden here.
fn select(ids: &[i64]) -> Select {
    Select::new(["id"]).from("budget_items").and_where(Expr::In(
        Box::new(Expr::column("id")),
        ids.iter().copied().map(Into::into).collect(),
        false,
    ))
}

/// A default-budget chunk fails even though smaller chunks preserve the same complete result.
async fn prove_reduced_budget() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("budgets.sqlite")).create_if_missing(true),
        )
        .await
        .unwrap(),
    )
    .with_limits(QueryLimits {
        binds: 2,
        ..QueryLimits::default()
    });
    let table = Create::new("budget_items")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    engine
        .execute_plan(&table.create().render(engine.dialect()).unwrap())
        .await
        .unwrap();
    for id in 1..=3 {
        engine
            .execute(&Insert::new("budget_items").columns(["id"]).values([id]))
            .await
            .unwrap();
    }
    // The renderer default allows all three keys; the selected Engine deliberately does not.
    select(&[1, 2, 3])
        .render(engine.dialect())
        .unwrap()
        .validate(QueryLimits::default())
        .unwrap();
    assert_eq!(
        engine.fetch(&select(&[1, 2, 3])).await.unwrap_err().kind(),
        ErrorKind::Limit
    );
    let mut transaction = engine.begin().await.unwrap();
    assert_eq!(
        transaction
            .fetch(&select(&[1, 2, 3]))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    let mut session = DataSession::transaction(&mut transaction);
    let mut rows = session.fetch(&select(&[1, 2])).await.unwrap();
    rows.extend(session.fetch(&select(&[3])).await.unwrap());
    let mut ids: Vec<i64> = rows.iter().map(|row| row.try_get("id").unwrap()).collect();
    ids.sort();
    assert_eq!(ids, [1, 2, 3]);
    transaction.rollback().await.unwrap();
    engine.close().await.unwrap();
}
