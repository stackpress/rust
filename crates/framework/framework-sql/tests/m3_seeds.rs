//! Cross-model seed and late reconstruction-constraint failures must roll back exact prior state.
#![cfg(feature = "sqlite-maintenance")]

use stackpress_framework_sql::migrations::{
    DialectId, Literal, LossApproval, MigrationPlanner, MigrationRunner, MigrationStep, RenameMap,
    RevisionSnapshot, SeedRow, SqliteMaintenance,
};
use stackpress_sql::{Create, Engine, Field, ForeignKey, Index, Insert, Select, SqlType, SqlValue};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{collections::BTreeMap, time::Duration};

#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Supervise both native failure scenarios independently of the current-thread runtime.
#[test]
fn supervised_seed_atomicity() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                second_model_failure().await;
                reconstruction_unique_failure().await;
            });
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "supervised_seed_atomicity",
        "cross-model-seeds",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Install the checked baseline with the production namespaced runner.
async fn install(options: &SqliteOptions, source: &RevisionSnapshot) {
    let plan = MigrationPlanner::baseline(source, DialectId::Sqlite).unwrap();
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
    )
    .await
    .unwrap();
}

/// A unique failure in the second model preserves the first model and the applied ledger.
async fn second_model_failure() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("seeds.sqlite")).create_if_missing(true);
    let first = Create::new("first")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let second = Create::new("second")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("parent", SqlType::Integer).not_null())
        .add_field(Field::new("code", SqlType::Text).not_null())
        .primary_key(["id"])
        .add_index(Index::new("second_code", ["code"]).unique())
        .add_foreign_key(ForeignKey::new(
            "second_parent",
            ["parent"],
            "first",
            ["id"],
        ))
        .definition()
        .unwrap();
    let source = RevisionSnapshot::new("seeds", "v1", vec![first, second]).unwrap();
    install(&options, &source).await;
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    engine
        .execute(&Insert::new("first").columns(["id"]).values([0]))
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("second")
                .columns(["id", "parent", "code"])
                .values([SqlValue::Integer(0), SqlValue::Integer(0), "taken".into()]),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
    let plan = MigrationPlanner::populate(
        &source,
        "second-model-fails",
        vec![
            SeedRow {
                table: "first".into(),
                values: BTreeMap::from([("id".into(), Literal::Integer(1))]),
                upsert: false,
            },
            SeedRow {
                table: "second".into(),
                values: BTreeMap::from([
                    ("id".into(), Literal::Integer(1)),
                    ("parent".into(), Literal::Integer(1)),
                    ("code".into(), Literal::Text("taken".into())),
                ]),
                upsert: false,
            },
        ],
        DialectId::Sqlite,
    )
    .unwrap();
    assert!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &plan,
            plan.hash()
        )
        .await
        .is_err()
    );
    let engine = Engine::new(SqliteDatabase::open(options).await.unwrap());
    for table in ["first", "second"] {
        let rows = engine
            .fetch(&Select::new(["id"]).from(table))
            .await
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].try_get::<i64>("id").unwrap(), 0);
    }
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        1
    );
    engine.close().await.unwrap();
}

/// A late index-creation failure restores copied/dropped/renamed tables even with loss approval.
async fn reconstruction_unique_failure() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("unique.sqlite")).create_if_missing(true);
    let table = Create::new("record")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("value", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let source = RevisionSnapshot::new("unique", "v1", vec![table.clone()]).unwrap();
    install(&options, &source).await;
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    for id in [1, 2] {
        engine
            .execute(
                &Insert::new("record")
                    .columns(["id", "value"])
                    .values([SqlValue::Integer(id), "duplicate".into()]),
            )
            .await
            .unwrap();
    }
    engine.close().await.unwrap();
    let mut target = table;
    target.fields[1].default = Some("fallback".into());
    target
        .indexes
        .push(Index::new("record_value", ["value"]).unique());
    let target = RevisionSnapshot::new("unique", "v2", vec![target]).unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert!(
        plan.steps()
            .iter()
            .any(|step| matches!(step, MigrationStep::Rebuild { .. }))
    );
    let approval = LossApproval::new(plan.hash().clone(), "constraint-test").unwrap();
    assert!(
        MigrationRunner::apply_to(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &plan,
            plan.hash(),
            "constraint-test",
            Some(&approval)
        )
        .await
        .is_err()
    );
    let engine = Engine::new(SqliteDatabase::open(options).await.unwrap());
    let rows = engine
        .fetch(&Select::default().from("record"))
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert!(
        rows.iter()
            .all(|row| row.try_get::<String>("value").unwrap() == "duplicate")
    );
    let head = engine
        .fetch(&Select::new(["signature"]).from("_stackpress_schema_state"))
        .await
        .unwrap();
    assert_eq!(
        head[0].try_get::<String>("signature").unwrap(),
        source.hash()
    );
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        1
    );
    engine.close().await.unwrap();
}
