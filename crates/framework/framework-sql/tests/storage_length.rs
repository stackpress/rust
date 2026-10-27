//! G38 pure planning and real SQLite resizes; every native scenario has a parent watchdog.
#![cfg(feature = "sqlite-maintenance")]
use stackpress_framework_sql::migrations::*;
use stackpress_sql::{
    Create, Delete, Engine, Field, ForeignKey, Insert, Length, Select, Sql, SqlType, SqlValue,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::time::Duration;
#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Supervise the complete resize and artifact paths independently of SQLite's worker runtime.
#[test]
fn supervised_g37_lengths() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match scenario.as_str() {
                    "legacy" => legacy(),
                    "resize" => resize().await,
                    "refusal" => refusal().await,
                    "budgets" => budgets().await,
                    "bytes" => bytes().await,
                    "lock" => lock().await,
                    "group" => group().await,
                    _ => panic!("unknown length scenario"),
                }
            });
        return;
    }
    for scenario in [
        "legacy", "resize", "refusal", "budgets", "bytes", "lock", "group",
    ] {
        let (ok, expired, output) =
            watchdog::run("supervised_g37_lengths", scenario, Duration::from_secs(20));
        assert!(ok && !expired, "{scenario}: timeout={expired}\n{output}");
    }
}

/// Exact pre-G38 bytes remain readable, including their original reverse-plan hash.
fn legacy() {
    let bytes = include_str!("fixtures/g37-v2/plan.json");
    let plan = MigrationPlan::from_artifact(bytes).unwrap();
    assert_eq!(
        plan.hash().as_str(),
        "8316a5c3327c57e0043a0598dd6b3884a2638baa4de9ce6fd74fbb2057aef3f8"
    );
    assert_eq!(plan.artifact().unwrap(), bytes);
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    for (name, bytes) in [
        ("one", include_str!("fixtures/g37-v2/one.json")),
        ("two", include_str!("fixtures/g37-v2/two.json")),
    ] {
        let path = root.path().join("revisions").join(name);
        std::fs::create_dir_all(&path).unwrap();
        std::fs::write(path.join("schema.json"), bytes).unwrap();
    }
    let history = RevisionStore::new(root.path()).check().unwrap();
    assert_eq!(
        history[0].hash(),
        "c7d587d0871bb6ed0d223787b7b746701ab47d40b62fa0856b13b39ef5a4c353"
    );
    assert_eq!(
        history[1].hash(),
        "dd5f71f7dfc3747af67086a282ae817f8bfe57fe1c919814de8d5b7cc876a464"
    );
    let store = RevisionStore::new(root.path());
    store.write_plan(&plan).unwrap();
    assert_eq!(store.plan(plan.hash()).unwrap().artifact().unwrap(), bytes);
    assert_eq!(store.check().unwrap().len(), 2);
    // A v2 reader refuses new authority even if a caller inserts it before hashing.
    let mut value: serde_json::Value = serde_json::from_str(bytes).unwrap();
    value["recipe"]["length_checks"] = serde_json::json!([]);
    assert!(MigrationPlan::from_artifact(&value.to_string()).is_err());
    let plan = MigrationPlanner::between(
        &history[0],
        &history[1],
        RenameMap::new(),
        DialectId::Sqlite,
    )
    .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&plan.artifact().unwrap()).unwrap()["recipe"]["format"],
        3
    );
}

/// Explicit integer allocation plus declared text, with no width inference.
fn snapshot(revision: &str, length: Option<i32>) -> RevisionSnapshot {
    let text = match length {
        None => Field::new("text", SqlType::Text),
        Some(n) => Field::new("text", SqlType::Varchar).length(Length::Size(n)),
    };
    let mut table = Create::new("note")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(text)
        .primary_key(["id"]);
    if length.is_some() {
        table = table.require_mysql_target();
    }
    RevisionSnapshot::new("lengths", revision, vec![table.definition().unwrap()]).unwrap()
}
async fn open(options: &SqliteOptions) -> Engine {
    Engine::new(SqliteDatabase::open(options.clone()).await.unwrap())
}
async fn apply(
    options: &SqliteOptions,
    plan: &MigrationPlan,
) -> stackpress_framework_sql::DataResult<ApplyOutcome> {
    let approval = LossApproval::new(plan.hash().clone(), "test").unwrap();
    MigrationRunner::apply_to(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        plan,
        plan.hash(),
        "test",
        Some(&approval),
    )
    .await
}
async fn install(options: &SqliteOptions, snapshot: &RevisionSnapshot) {
    apply(
        options,
        &MigrationPlanner::baseline(snapshot, DialectId::Sqlite).unwrap(),
    )
    .await
    .unwrap();
}

/// Forward/reverse preserve exact scalar values, row identities and deleted AUTOINCREMENT history.
async fn resize() {
    for maximum in [Some(4), None] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let options =
            SqliteOptions::file(root.path().join("resize.sqlite")).create_if_missing(true);
        let source = snapshot("one", Some(2));
        let target = snapshot("two", maximum);
        install(&options, &source).await;
        let engine = open(&options).await;
        for text in ["😀😀", "e\u{301}", "a\0", "a "] {
            engine
                .execute(&Insert::new("note").columns(["text"]).values([text]))
                .await
                .unwrap();
        }
        engine
            .execute(
                &Insert::new("note")
                    .columns(["id", "text"])
                    .values([SqlValue::Integer(900), "ok".into()]),
            )
            .await
            .unwrap();
        engine
            .execute(&Delete::new("note").where_eq("id", 900))
            .await
            .unwrap();
        engine.close().await.unwrap();
        let plan = MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite)
            .unwrap();
        assert!(plan.blocked_reasons().is_empty());
        assert!(
            plan.findings()
                .iter()
                .any(|r| r.kind == "storage_length_widening" && !r.destructive)
        );
        assert_eq!(
            MigrationPlan::from_artifact(&plan.artifact().unwrap())
                .unwrap()
                .hash(),
            plan.hash()
        );
        let mut tampered: serde_json::Value =
            serde_json::from_str(&plan.artifact().unwrap()).unwrap();
        tampered["recipe"]["length_checks"] = if maximum.is_some() {
            serde_json::json!([])
        } else {
            serde_json::json!([{"table":"note","column":"text","maximum":2}])
        };
        assert!(MigrationPlan::from_artifact(&tampered.to_string()).is_err());
        apply(&options, &plan).await.unwrap();
        let reverse = plan.reverse().unwrap();
        apply(&options, reverse).await.unwrap();
        let engine = open(&options).await;
        let rows = engine
            .fetch(
                &Select::new(["text"])
                    .from("note")
                    .order_by("id", stackpress_sql::Order::Asc),
            )
            .await
            .unwrap();
        assert_eq!(
            rows.into_iter()
                .map(|row| row.try_get::<Option<String>>("text").unwrap().unwrap())
                .collect::<Vec<_>>(),
            ["😀😀", "e\u{301}", "a\0", "a "]
        );
        engine
            .execute(&Insert::new("note").columns(["text"]).values(["ab"]))
            .await
            .unwrap();
        assert_eq!(
            engine
                .fetch(&Select::new(["id"]).from("note").where_eq("text", "ab"))
                .await
                .unwrap()[0]
                .try_get::<i64>("id")
                .unwrap(),
            901
        );
        // SQLite's declaration is metadata only: raw SQL can insert a violating value.
        engine
            .execute(&Insert::new("note").columns(["text"]).values(["abc"]))
            .await
            .unwrap();
        engine.close().await.unwrap();
        // Reapply the widening, then its fresh narrowing refuses a now-long value.
        apply(&options, &plan).await.unwrap();
        assert!(apply(&options, reverse).await.is_err());
        assert!(
            MigrationRunner::status(options, Some(&target))
                .await
                .unwrap()
                .schema_matches
        );
    }
}

/// Narrowing approval cannot waive NUL-aware count/type checks; failed scans leave the head intact.
async fn refusal() {
    for value in [
        SqlValue::Text("a\0x".into()),
        SqlValue::Text("a  ".into()),
        SqlValue::Text("😀😀😀".into()),
        SqlValue::Blob(vec![1, 2, 3]),
    ] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let options =
            SqliteOptions::file(root.path().join("refuse.sqlite")).create_if_missing(true);
        let source = snapshot("one", None);
        let target = snapshot("two", Some(2));
        install(&options, &source).await;
        let engine = open(&options).await;
        engine
            .execute(&Insert::new("note").columns(["text"]).values([value]))
            .await
            .unwrap();
        engine.close().await.unwrap();
        let plan = MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite)
            .unwrap();
        assert!(
            plan.findings()
                .iter()
                .any(|r| r.kind == "storage_length_narrowing" && r.destructive)
        );
        assert!(apply(&options, &plan).await.is_err());
        assert!(
            MigrationRunner::status(options, Some(&source))
                .await
                .unwrap()
                .schema_matches
        );
    }
}

/// Refuse incomplete scans before reconstruction when the complete row budget is exceeded.
async fn budgets() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("budget.sqlite")).create_if_missing(true);
    let source = snapshot("one", None);
    let target = snapshot("two", Some(2));
    install(&options, &source).await;
    let engine = open(&options).await;
    engine.execute(&Sql::literal(DialectId::Sqlite, "WITH RECURSIVE n(x) AS (VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<10001) INSERT INTO note(text) SELECT 'ok' FROM n")).await.unwrap();
    engine.close().await.unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    let error = apply(&options, &plan).await.unwrap_err();
    assert_eq!(error.context(), "migration length scan row budget exceeded");
    assert!(
        MigrationRunner::status(options, Some(&source))
            .await
            .unwrap()
            .schema_matches
    );
}

/// Both FK endpoints resize in one locked group; a mismatched or absent width refuses planning.
async fn group() {
    let tables = |n| {
        let parent = Create::new("parent")
            .add_field(
                Field::new("id", SqlType::Varchar)
                    .length(Length::Size(n))
                    .not_null(),
            )
            .primary_key(["id"])
            .require_mysql_target()
            .definition()
            .unwrap();
        let child = Create::new("child")
            .add_field(Field::new("id", SqlType::Integer).not_null())
            .add_field(
                Field::new("parent", SqlType::Varchar)
                    .length(Length::Size(n))
                    .not_null(),
            )
            .primary_key(["id"])
            .add_foreign_key(ForeignKey::new(
                "child_parent",
                ["parent"],
                "parent",
                ["id"],
            ))
            .require_mysql_target()
            .definition()
            .unwrap();
        vec![parent, child]
    };
    let source = RevisionSnapshot::new("group", "one", tables(2)).unwrap();
    let target = RevisionSnapshot::new("group", "two", tables(4)).unwrap();
    let mut invalid = tables(4);
    invalid[1].fields[1].length = Some(Length::Size(2));
    assert!(RevisionSnapshot::new("group", "invalid", invalid).is_err());
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("group.sqlite")).create_if_missing(true);
    install(&options, &source).await;
    let engine = open(&options).await;
    engine
        .execute(&Insert::new("parent").columns(["id"]).values(["😀😀"]))
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("child")
                .columns(["id", "parent"])
                .values([SqlValue::Integer(1), "😀😀".into()]),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    apply(&options, &plan).await.unwrap();
    apply(&options, plan.reverse().unwrap()).await.unwrap();
    assert!(
        MigrationRunner::status(options, Some(&source))
            .await
            .unwrap()
            .schema_matches
    );
}

/// Scan aggregate bytes across bounded batches; an approval cannot waive incomplete work.
async fn bytes() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("bytes.sqlite")).create_if_missing(true);
    let source = snapshot("one", None);
    let target = snapshot("two", Some(1_048_576));
    install(&options, &source).await;
    let engine = open(&options).await;
    for _ in 0..17 {
        engine
            .execute(
                &Insert::new("note")
                    .columns(["text"])
                    .values(["a".repeat(1024 * 1024)]),
            )
            .await
            .unwrap();
    }
    engine.close().await.unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert_eq!(
        apply(&options, &plan).await.unwrap_err().context(),
        "migration length scan byte budget exceeded"
    );
    assert!(
        MigrationRunner::status(options, Some(&source))
            .await
            .unwrap()
            .schema_matches
    );
}

/// A competing writer keeps resize checks/DDL behind the exclusive maintenance lock.
async fn lock() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let mut options = SqliteOptions::file(root.path().join("lock.sqlite")).create_if_missing(true);
    options.busy_timeout_ms = 100;
    let source = snapshot("one", Some(2));
    let target = snapshot("two", Some(4));
    install(&options, &source).await;
    let owner = SqliteMaintenance::open(options.clone()).await.unwrap();
    let held = owner.begin_rebuild().await.unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert!(apply(&options, &plan).await.is_err());
    held.rollback().await.unwrap();
    assert!(
        MigrationRunner::status(options, Some(&source))
            .await
            .unwrap()
            .schema_matches
    );
}
