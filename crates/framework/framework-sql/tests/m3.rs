//! Real-file M3 history, authority, reconstruction and operator contracts under a watchdog.
#![cfg(feature = "sqlite-maintenance")]
use stackpress_framework_sql::DataErrorKind;
use stackpress_framework_sql::migrations::*;
use stackpress_sql::{
    Create, Delete, Engine, Field, ForeignKey, Index, Insert, ReferentialAction, Select, Sql,
    SqlType, SqlValue,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{collections::BTreeMap, time::Duration};
#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Supervise every filesystem/native scenario independently of its async executor.
#[test]
fn supervised_m3() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match scenario.as_str() {
                    "history" => history(),
                    "planner" => planner(),
                    "rebuild" => rebuild().await,
                    "operators" => operators().await,
                    "refusals" => refusals().await,
                    "backfill" => backfill().await,
                    "table-rename" => table_rename().await,
                    "nullable-default" => nullable_default().await,
                    _ => panic!("unknown M3 scenario"),
                }
            });
        return;
    }
    for scenario in [
        "history",
        "planner",
        "rebuild",
        "operators",
        "refusals",
        "backfill",
        "table-rename",
        "nullable-default",
    ] {
        let (ok, expired, output) =
            watchdog::run("supervised_m3", scenario, Duration::from_secs(15));
        assert!(ok && !expired, "{scenario}: timeout={expired}\n{output}");
    }
}
/// A populated explicit-ID base for pure and native planning.
fn table() -> stackpress_sql::TableDefinition {
    Create::new("profile")
        .add_field(Field::new("id", SqlType::Text).not_null())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .add_field(Field::new("note", SqlType::Text))
        .primary_key(["id"])
        .definition()
        .unwrap()
}
/// Preserve durable history across no-op recording, pending plans and checked corruption refusal.
fn history() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let path = root.path().join("schema");
    let store = RevisionStore::new(&path);
    let first = RevisionSnapshot::new("workspace", "0001", vec![table()]).unwrap();
    assert!(matches!(
        store.record(&first, None).unwrap(),
        RevisionOutcome::Recorded(_)
    ));
    let parent = RevisionId::parse("0001").unwrap();
    let same = RevisionSnapshot::new("workspace", "ignored", vec![table()]).unwrap();
    assert_eq!(
        store.record(&same, Some(&parent)).unwrap(),
        RevisionOutcome::NoChanges(parent.clone())
    );
    let mut new = table();
    new.fields.push(Field::new("summary", SqlType::Text));
    let second = RevisionSnapshot::new("workspace", "0002", vec![new]).unwrap();
    store.record(&second, Some(&parent)).unwrap();
    assert!(store.record(&second, Some(&parent)).is_err());
    let chain = store.check().unwrap();
    assert_eq!(chain.len(), 2);
    let baseline = MigrationPlanner::baseline(&chain[0], DialectId::Sqlite).unwrap();
    store.write_plan(&baseline).unwrap();
    let plan = MigrationPlanner::between(&chain[0], &chain[1], RenameMap::new(), DialectId::Sqlite)
        .unwrap();
    store.write_plan(&plan).unwrap();
    assert_eq!(store.plan(plan.hash()).unwrap().hash(), plan.hash());
    assert_eq!(store.write_plan(&plan).unwrap(), *plan.hash());
    let companion = path
        .join("migrations")
        .join(plan.hash().as_str())
        .join("sqlite-up.sql");
    let bytes = std::fs::read(&companion).unwrap();
    std::fs::write(&companion, "edited").unwrap();
    assert!(store.check().is_err());
    std::fs::write(&companion, bytes).unwrap();
    let schema = path.join("revisions/0001/schema.json");
    let original = std::fs::read_to_string(&schema).unwrap();
    std::fs::write(&schema, original.replace("storage-1", "storage-2")).unwrap();
    assert!(store.check().is_err());
    std::fs::write(&schema, &original).unwrap();
    std::fs::write(&schema, format!("{original}\n")).unwrap();
    assert!(store.check().is_err());
    std::fs::write(&schema, &original).unwrap();
    std::fs::create_dir(path.join(".pending-interrupted")).unwrap();
    assert!(store.check().is_err());
    std::fs::remove_dir(path.join(".pending-interrupted")).unwrap();
    std::fs::write(path.join(".writer.lock"), "owned elsewhere").unwrap();
    assert!(
        store
            .record(&same, Some(&RevisionId::parse("0002").unwrap()))
            .is_err()
    );
    std::fs::remove_file(path.join(".writer.lock")).unwrap();
    // Produce a second valid child through the real writer, then expose the unresolved branch.
    let branch_store = RevisionStore::new(root.path().join("branch-history"));
    branch_store.record(&first, None).unwrap();
    let mut branch_table = table();
    branch_table
        .fields
        .push(Field::new("branch_only", SqlType::Text));
    branch_store
        .record(
            &RevisionSnapshot::new("workspace", "branch", vec![branch_table]).unwrap(),
            Some(&parent),
        )
        .unwrap();
    let bytes = std::fs::read(
        root.path()
            .join("branch-history/revisions/branch/schema.json"),
    )
    .unwrap();
    let branch = path.join("revisions/branch");
    std::fs::create_dir(&branch).unwrap();
    std::fs::write(branch.join("schema.json"), &bytes).unwrap();
    assert!(store.check().is_err());
    let missing = root.path().join("missing/revisions/branch");
    std::fs::create_dir_all(&missing).unwrap();
    std::fs::write(missing.join("schema.json"), &bytes).unwrap();
    assert!(
        RevisionStore::new(root.path().join("missing"))
            .check()
            .is_err()
    );
    std::fs::remove_file(branch.join("schema.json")).unwrap();
    std::fs::remove_dir(branch).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&schema, path.join("revisions/0002/link")).unwrap();
        assert!(store.check().is_err());
        std::fs::remove_file(path.join("revisions/0002/link")).unwrap();
        // The byte limit must not open/block on a FIFO that replaced an artifact.
        let saved = path.join("saved.json");
        std::fs::rename(&schema, &saved).unwrap();
        let mut command = std::process::Command::new("mkfifo");
        command.arg(&schema);
        fixture_command(command);
        assert!(store.check().is_err());
        std::fs::remove_file(&schema).unwrap();
        std::fs::rename(&saved, &schema).unwrap();
        let link = root.path().join("linked");
        std::os::unix::fs::symlink(&path, &link).unwrap();
        assert!(RevisionStore::new(link).check().is_err());
    }
    let foreign = RevisionSnapshot::new("another-app", "foreign", vec![table()]).unwrap();
    assert!(
        store
            .record(&foreign, Some(&RevisionId::parse("0002").unwrap()))
            .is_err()
    );
    assert_eq!(store.check().unwrap().len(), 2);
    let corrupt = original.replace("\"format\": 3", "\"format\": 999");
    std::fs::write(&schema, corrupt).unwrap();
    assert!(store.check().is_err());
}
/// Explicit mapping, aggregate findings, irreversibility and render-only dialect artifacts.
fn planner() {
    rendered_snapshots();
    let source = RevisionSnapshot::new("workspace", "v1", vec![table()]).unwrap();
    let mut target = table();
    target.fields[1].name = "heading".into();
    let target = RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap();
    assert_eq!(
        MigrationPlanner::suggestions(&source, &target)
            .unwrap()
            .len(),
        1
    );
    assert!(
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).is_err()
    );
    for backend in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        let plan = MigrationPlanner::between(
            &source,
            &target,
            RenameMap::new()
                .field("profile", "name", "heading")
                .unwrap(),
            backend,
        )
        .unwrap();
        assert!(
            plan.queries()
                .iter()
                .any(|q| q.sql().contains("RENAME COLUMN"))
        );
        assert_eq!(
            MigrationPlan::from_artifact(&plan.artifact().unwrap())
                .unwrap()
                .hash(),
            plan.hash()
        );
        assert!(plan.reverse().is_some());
        let mut value: serde_json::Value = serde_json::from_str(&plan.artifact().unwrap()).unwrap();
        value["hash"] = serde_json::json!("0".repeat(64));
        assert!(MigrationPlan::from_artifact(&value.to_string()).is_err());
    }
    let deliberate = MigrationPlanner::between(
        &source,
        &target,
        RenameMap::new()
            .drop_field("profile", "name")
            .unwrap()
            .backfill("profile", "heading", Literal::Text("replacement".into()))
            .unwrap(),
        DialectId::Sqlite,
    )
    .unwrap();
    assert!(deliberate.findings().iter().any(|f| f.destructive));
    assert!(deliberate.reverse().is_none());
    let mut lossy = table();
    let _ = lossy.fields.remove(2);
    lossy.fields[1].data_type = SqlType::Integer;
    let lossy = RevisionSnapshot::new("workspace", "lossy", vec![lossy]).unwrap();
    let plan =
        MigrationPlanner::between(&source, &lossy, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert!(plan.findings().len() >= 2);
    assert!(!plan.blocked_reasons().is_empty());
    assert!(matches!(plan.reverse_plan(), ReversePlan::Refused(_)));
    let mut ordering = table();
    ordering.fields.swap(0, 2);
    let ordering = RevisionSnapshot::new("workspace", "ordered", vec![ordering]).unwrap();
    let plan =
        MigrationPlanner::between(&source, &ordering, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert!(plan.queries().is_empty());
    let mut many = Create::new("many")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .primary_key(["id"]);
    for i in 0..65 {
        many = many.add_field(Field::new(format!("old_{i}"), SqlType::Text));
    }
    let old = many.definition().unwrap();
    let mut new = old.clone();
    for (i, field) in new.fields.iter_mut().skip(1).enumerate() {
        field.name = format!("new_{i}");
    }
    assert!(
        MigrationPlanner::suggestions(
            &RevisionSnapshot::new("workspace", "many_old", vec![old]).unwrap(),
            &RevisionSnapshot::new("workspace", "many_new", vec![new]).unwrap()
        )
        .is_err()
    );
    let large = SeedRow {
        table: "profile".into(),
        values: BTreeMap::from([
            ("id".into(), Literal::Text("id".into())),
            ("name".into(), Literal::Text("x".repeat(700_000))),
        ]),
        upsert: false,
    };
    assert!(
        MigrationPlanner::populate(
            &source,
            "too-wide",
            vec![large.clone(), large],
            DialectId::Sqlite
        )
        .is_err()
    );
    assert!(
        RenameMap::new()
            .backfill("profile", "one", Literal::Text("x".repeat(700_000)))
            .unwrap()
            .backfill("profile", "two", Literal::Text("x".repeat(700_000)))
            .is_err()
    );
    let mut huge = table();
    huge.fields
        .push(Field::new("huge", SqlType::Text).default("x".repeat(1024 * 1024 + 1)));
    assert!(RevisionSnapshot::new("workspace", "huge", vec![huge]).is_err());
    assert!(PlanHash::parse("bad").is_err());
    assert!(RevisionId::parse("../bad").is_err());
    let no =
        MigrationPlanner::between(&source, &source, RenameMap::new(), DialectId::Sqlite).unwrap();
    assert!(no.queries().is_empty());
}
/// Open/close ordinary pools explicitly; maintenance never quiesces them implicitly.
async fn engine(options: &SqliteOptions) -> Engine {
    Engine::new(SqliteDatabase::open(options.clone()).await.unwrap())
}
/// Install through the actual planner and namespaced ledger.
async fn install(options: &SqliteOptions, snapshot: &RevisionSnapshot) {
    let plan = MigrationPlanner::baseline(snapshot, DialectId::Sqlite).unwrap();
    assert_eq!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &plan,
            plan.hash()
        )
        .await
        .unwrap(),
        ApplyOutcome::Applied
    );
}
/// Build inbound CASCADE/RESTRICT, self and cyclic managed references.
fn group() -> Vec<stackpress_sql::TableDefinition> {
    let mut parent = Create::new("parent")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(Field::new("name", SqlType::Text).not_null())
        .add_field(Field::new("note", SqlType::Text))
        .add_field(Field::new("ancestor", SqlType::Integer))
        .primary_key(["id"]);
    let mut own = ForeignKey::new("parent_self", ["ancestor"], "parent", ["id"]);
    own.on_delete = ReferentialAction::Cascade;
    parent = parent
        .add_foreign_key(own)
        .add_index(Index::new("parent_name", ["name"]).unique());
    let mut tables = vec![parent.definition().unwrap()];
    for (name, action) in [
        ("cascade", ReferentialAction::Cascade),
        ("restrict", ReferentialAction::Restrict),
    ] {
        let mut fk = ForeignKey::new(format!("{name}_parent"), ["parent_id"], "parent", ["id"]);
        fk.on_delete = action;
        tables.push(
            Create::new(name)
                .add_field(Field::new("id", SqlType::Integer).not_null())
                .add_field(Field::new("parent_id", SqlType::Integer).not_null())
                .primary_key(["id"])
                .add_foreign_key(fk)
                .definition()
                .unwrap(),
        );
    }
    for (name, remote) in [("cycle_a", "cycle_b"), ("cycle_b", "cycle_a")] {
        tables.push(
            Create::new(name)
                .add_field(Field::new("id", SqlType::Integer).not_null())
                .add_field(Field::new("peer", SqlType::Integer))
                .primary_key(["id"])
                .add_foreign_key(ForeignKey::new(
                    format!("{name}_fk"),
                    ["peer"],
                    remote,
                    ["id"],
                ))
                .definition()
                .unwrap(),
        );
    }
    tables
}
/// Preserve rows/defaults/NULL/keys/indexes/FKs and deleted-ID sequence history both directions.
async fn rebuild() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("matrix.sqlite")).create_if_missing(true);
    let tables = group();
    let source = RevisionSnapshot::new("workspace", "v1", tables.clone()).unwrap();
    install(&options, &source).await;
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    for id in [1, 2, 100] {
        session
            .execute(
                &Insert::new("parent")
                    .columns(["id", "name", "ancestor"])
                    .values([
                        SqlValue::Integer(id),
                        format!("row-{id}").into(),
                        if id == 2 { 1.into() } else { SqlValue::Null },
                    ]),
            )
            .await
            .unwrap();
    }
    session
        .execute(&Delete::new("parent").where_eq("id", 100))
        .await
        .unwrap();
    for table in ["cascade", "restrict"] {
        session
            .execute(
                &Insert::new(table)
                    .columns(["id", "parent_id"])
                    .values([1, 1]),
            )
            .await
            .unwrap();
    }
    for table in ["cycle_a", "cycle_b"] {
        session
            .execute(&Insert::new(table).columns(["id", "peer"]).values([1, 1]))
            .await
            .unwrap();
    }
    session.commit().await.unwrap();
    let mut target = tables;
    let parent = target.iter_mut().find(|t| t.name == "parent").unwrap();
    parent.fields[1].name = "heading".into();
    parent.fields[1].default = Some("fallback".into());
    parent.indexes[0].columns = vec!["heading".into()];
    parent.indexes.push(Index::new("parent_note", ["note"]));
    // Both members of a cycle reconstruct in one FK-disabled, integrity-checked group.
    for table in target.iter_mut().filter(|t| t.name.starts_with("cycle_")) {
        table.fields[1].default = Some(1.into());
    }
    let target = RevisionSnapshot::new("workspace", "v2", target).unwrap();
    let plan = MigrationPlanner::between(
        &source,
        &target,
        RenameMap::new().field("parent", "name", "heading").unwrap(),
        DialectId::Sqlite,
    )
    .unwrap();
    assert!(
        plan.steps()
            .iter()
            .filter(|s| matches!(s, MigrationStep::Rebuild { .. }))
            .count()
            >= 3
    );
    assert!(
        plan.findings()
            .iter()
            .any(|f| f.kind == "plain_integer_primary_key_may_reuse_deleted_ids")
    );
    let plan = MigrationPlan::from_artifact(&plan.artifact().unwrap()).unwrap();
    assert_eq!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &plan,
            plan.hash()
        )
        .await
        .unwrap(),
        ApplyOutcome::Applied
    );
    let status = MigrationRunner::status(options.clone(), Some(plan.target_snapshot()))
        .await
        .unwrap();
    assert!(status.schema_matches);
    let db = engine(&options).await;
    let rows = db
        .fetch(
            &Select::default()
                .from("parent")
                .order_by("id", stackpress_sql::Order::Asc),
        )
        .await
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].try_get::<String>("heading").unwrap(), "row-1");
    assert_eq!(rows[0].try_get::<Option<String>>("note").unwrap(), None);
    assert!(
        db.execute(&Insert::new("parent").columns(["heading"]).values(["row-1"]))
            .await
            .is_err()
    );
    assert!(
        db.execute(
            &Insert::new("cascade")
                .columns(["id", "parent_id"])
                .values([999, 999])
        )
        .await
        .is_err()
    );
    db.execute(&Insert::new("parent").columns(["heading"]).values(["new"]))
        .await
        .unwrap();
    let id = db
        .fetch(
            &Select::new(["id"])
                .from("parent")
                .where_eq("heading", "new"),
        )
        .await
        .unwrap()[0]
        .try_get::<i64>("id")
        .unwrap();
    assert_eq!(id, 101);
    db.execute(&Delete::new("parent").where_eq("id", 101))
        .await
        .unwrap();
    db.close().await.unwrap();
    let reverse = plan.reverse().unwrap();
    assert_eq!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            reverse,
            reverse.hash()
        )
        .await
        .unwrap(),
        ApplyOutcome::Applied
    );
    let db = engine(&options).await;
    db.execute(
        &Insert::new("parent")
            .columns(["name"])
            .values(["after-reverse"]),
    )
    .await
    .unwrap();
    assert_eq!(
        db.fetch(
            &Select::new(["id"])
                .from("parent")
                .where_eq("name", "after-reverse")
        )
        .await
        .unwrap()[0]
            .try_get::<i64>("id")
            .unwrap(),
        102
    );
    assert!(
        db.execute(&Delete::new("parent").where_eq("id", 1))
            .await
            .is_err()
    );
    db.execute(&Delete::new("restrict").all_rows())
        .await
        .unwrap();
    db.execute(&Delete::new("parent").where_eq("id", 1))
        .await
        .unwrap();
    assert!(
        db.fetch(&Select::default().from("cascade"))
            .await
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        db.fetch(&Select::default().from("parent"))
            .await
            .unwrap()
            .len(),
        1
    );
    db.close().await.unwrap();
}
/// Plan named seeds and destructive operations; unrelated data survives all owned operations.
async fn operators() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("operators.sqlite")).create_if_missing(true);
    let db = engine(&options).await;
    db.execute_plan(
        &Create::new("unrelated")
            .add_field(Field::new("value", SqlType::Text))
            .render(DialectId::Sqlite)
            .unwrap(),
    )
    .await
    .unwrap();
    db.execute(
        &Insert::new("unrelated")
            .columns(["value"])
            .values(["sentinel"]),
    )
    .await
    .unwrap();
    db.close().await.unwrap();
    let source = RevisionSnapshot::new("workspace", "v1", vec![table()]).unwrap();
    install(&options, &source).await;
    let rows = vec![SeedRow {
        table: "profile".into(),
        values: BTreeMap::from([
            ("id".into(), Literal::Text("p1".into())),
            ("name".into(), Literal::Text("Ada".into())),
        ]),
        upsert: true,
    }];
    let seed = MigrationPlanner::populate(&source, "seed-v1", rows, DialectId::Sqlite).unwrap();
    for expected in [ApplyOutcome::Applied, ApplyOutcome::AlreadyApplied] {
        assert_eq!(
            MigrationRunner::apply(
                SqliteMaintenance::open(options.clone()).await.unwrap(),
                &seed,
                seed.hash()
            )
            .await
            .unwrap(),
            expected
        );
    }
    let purge = MigrationPlanner::purge(&source, DialectId::Sqlite).unwrap();
    assert!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &purge,
            purge.hash()
        )
        .await
        .is_err()
    );
    let wrong = LossApproval::new(purge.hash().clone(), "other").unwrap();
    assert!(
        MigrationRunner::apply_to(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &purge,
            purge.hash(),
            "test",
            Some(&wrong)
        )
        .await
        .is_err()
    );
    let approval = LossApproval::new(purge.hash().clone(), "test").unwrap();
    MigrationRunner::apply_to(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &purge,
        purge.hash(),
        "test",
        Some(&approval),
    )
    .await
    .unwrap();
    let db = engine(&options).await;
    assert!(
        db.fetch(&Select::default().from("profile"))
            .await
            .unwrap()
            .is_empty()
    );
    db.close().await.unwrap();
    let uninstall = MigrationPlanner::uninstall(&source, DialectId::Sqlite).unwrap();
    let approval = LossApproval::new(uninstall.hash().clone(), "test").unwrap();
    MigrationRunner::apply_to(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &uninstall,
        uninstall.hash(),
        "test",
        Some(&approval),
    )
    .await
    .unwrap();
    let status = MigrationRunner::status(options.clone(), Some(&source))
        .await
        .unwrap();
    assert_eq!(status.applied.unwrap().state, "uninstalled");
    assert!(!status.schema_matches);
    let db = engine(&options).await;
    assert_eq!(
        db.fetch(&Select::default().from("unrelated"))
            .await
            .unwrap()[0]
            .try_get::<String>("value")
            .unwrap(),
        "sentinel"
    );
    assert_eq!(
        db.fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        4
    );
    db.execute_plan(&table().create().render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    db.close().await.unwrap();
    assert!(
        MigrationRunner::apply_to(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &uninstall,
            uninstall.hash(),
            "test",
            Some(&approval)
        )
        .await
        .is_err()
    );
}
/// Drift, namespace collisions, unavailable conversions, seed failure and network apply all refuse.
async fn refusals() {
    for object in [
        "index",
        "view",
        "quoted-view",
        "quoted-fk",
        "trigger",
        "ledger",
        "unmanaged",
        "seed",
        "network",
        "head",
        "ancestor",
    ] {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let options =
            SqliteOptions::file(root.path().join("refuse.sqlite")).create_if_missing(true);
        let source = RevisionSnapshot::new("workspace", "v1", vec![table()]).unwrap();
        if object == "ledger" || object == "unmanaged" {
            let db = engine(&options).await;
            let name = if object == "ledger" {
                "_stackpress_schema_state"
            } else {
                "profile"
            };
            db.execute_plan(
                &Create::new(name)
                    .add_field(Field::new("value", SqlType::Text))
                    .render(DialectId::Sqlite)
                    .unwrap(),
            )
            .await
            .unwrap();
            db.close().await.unwrap();
            let plan = MigrationPlanner::baseline(&source, DialectId::Sqlite).unwrap();
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
            continue;
        }
        install(&options, &source).await;
        let mut target = table();
        target.fields.push(Field::new("summary", SqlType::Text));
        let target = RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap();
        let plan = MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite)
            .unwrap();
        if object == "trigger" {
            let mut command = std::process::Command::new("python3");
            command.args(["-c","import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); c.execute(sys.argv[2]); c.commit(); c.close()"]).arg(root.path().join("refuse.sqlite")).arg("CREATE TRIGGER unknown_trigger AFTER INSERT ON profile BEGIN SELECT 1; END");
            fixture_command(command);
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
        } else if object == "quoted-fk" {
            let db = engine(&options).await;
            db.execute(&Sql::literal(
                DialectId::Sqlite,
                "CREATE TABLE unknown_child (remote TEXT REFERENCES 'profile' (id))",
            ))
            .await
            .unwrap();
            db.close().await.unwrap();
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
        } else if object == "quoted-view" {
            let db = engine(&options).await;
            db.execute(&Sql::literal(
                DialectId::Sqlite,
                "CREATE VIEW quoted_view AS SELECT * FROM 'profile'",
            ))
            .await
            .unwrap();
            db.close().await.unwrap();
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
        } else if object == "index" || object == "view" {
            let db = engine(&options).await;
            db.execute(&Sql::literal(
                DialectId::Sqlite,
                if object == "index" {
                    "CREATE INDEX unknown_index ON profile(name)"
                } else {
                    "CREATE VIEW café AS SELECT * FROM profile /* 🐟 */"
                },
            ))
            .await
            .unwrap();
            db.close().await.unwrap();
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
        } else if object == "head" || object == "ancestor" {
            let db = engine(&options).await;
            let update = if object == "head" {
                stackpress_sql::Update::new("_stackpress_schema_state")
                    .set("format", 999)
                    .where_eq("id", 1)
            } else {
                stackpress_sql::Update::new("_stackpress_migrations")
                    .set("parent", "wrong")
                    .where_eq("sequence", 1)
            };
            db.execute(&update).await.unwrap();
            db.close().await.unwrap();
            assert!(
                MigrationRunner::status(options.clone(), Some(&source))
                    .await
                    .is_err()
            );
            assert!(
                MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash()
                )
                .await
                .is_err()
            );
        } else if object == "network" {
            for backend in [DialectId::Postgres, DialectId::Mysql] {
                let plan =
                    MigrationPlanner::between(&source, &target, RenameMap::new(), backend).unwrap();
                let error = MigrationRunner::apply(
                    SqliteMaintenance::open(options.clone()).await.unwrap(),
                    &plan,
                    plan.hash(),
                )
                .await
                .unwrap_err();
                assert_eq!(error.kind(), DataErrorKind::Unsupported);
            }
        } else {
            let row = SeedRow {
                table: "profile".into(),
                values: BTreeMap::from([
                    ("id".into(), Literal::Text("same".into())),
                    ("name".into(), Literal::Text("Ada".into())),
                ]),
                upsert: false,
            };
            let plan = MigrationPlanner::populate(
                &source,
                "failing",
                vec![row.clone(), row],
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
            let db = engine(&options).await;
            assert!(
                db.fetch(&Select::default().from("profile"))
                    .await
                    .unwrap()
                    .is_empty()
            );
            db.close().await.unwrap();
        }
        let db = engine(&options).await;
        assert_eq!(
            db.fetch(&Select::default().from("_stackpress_migrations"))
                .await
                .unwrap()
                .len(),
            1
        );
        db.close().await.unwrap();
    }
}

/// A literal backfill touches NULLs only, then a required-column rebuild checks all constraints.
async fn backfill() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("backfill.sqlite")).create_if_missing(true);
    let source = RevisionSnapshot::new("workspace", "v1", vec![table()]).unwrap();
    install(&options, &source).await;
    let db = engine(&options).await;
    for (id, note) in [
        ("empty", SqlValue::Null),
        ("kept", SqlValue::Text("keep".into())),
    ] {
        db.execute(
            &Insert::new("profile")
                .columns(["id", "name", "note"])
                .values([id.into(), "Ada".into(), note]),
        )
        .await
        .unwrap();
    }
    db.close().await.unwrap();
    let mut target = table();
    target.fields[2].nullable = false;
    let target = RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap();
    assert!(
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).is_err()
    );
    let plan = MigrationPlanner::between(
        &source,
        &target,
        RenameMap::new()
            .backfill("profile", "note", Literal::Text("filled".into()))
            .unwrap(),
        DialectId::Sqlite,
    )
    .unwrap();
    assert!(plan.reverse().is_none());
    let loss = LossApproval::new(plan.hash().clone(), "backfill-test").unwrap();
    MigrationRunner::apply_to(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
        "backfill-test",
        Some(&loss),
    )
    .await
    .unwrap();
    let db = engine(&options).await;
    for (id, expected) in [("empty", "filled"), ("kept", "keep")] {
        assert_eq!(
            db.fetch(&Select::new(["note"]).from("profile").where_eq("id", id))
                .await
                .unwrap()[0]
                .try_get::<String>("note")
                .unwrap(),
            expected
        );
    }
    db.close().await.unwrap();
}
/// Explicit table identity mapping preserves inbound references and allocation history in reverse.
async fn table_rename() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("rename.sqlite")).create_if_missing(true);
    let mut tables = group();
    tables.retain(|t| matches!(t.name.as_str(), "parent" | "cascade"));
    let source = RevisionSnapshot::new("workspace", "v1", tables.clone()).unwrap();
    install(&options, &source).await;
    let db = engine(&options).await;
    db.execute(
        &Insert::new("parent")
            .columns(["id", "name"])
            .values([SqlValue::Integer(100), "issued".into()]),
    )
    .await
    .unwrap();
    db.execute(&Delete::new("parent").where_eq("id", 100))
        .await
        .unwrap();
    db.execute(
        &Insert::new("parent")
            .columns(["id", "name"])
            .values([SqlValue::Integer(1), "retained".into()]),
    )
    .await
    .unwrap();
    db.execute(
        &Insert::new("cascade")
            .columns(["id", "parent_id"])
            .values([1, 1]),
    )
    .await
    .unwrap();
    db.close().await.unwrap();
    for table in &mut tables {
        if table.name == "parent" {
            table.name = "person".into();
        }
        for key in &mut table.foreign_keys {
            if key.table == "parent" {
                key.table = "person".into();
            }
        }
    }
    let target = RevisionSnapshot::new("workspace", "v2", tables).unwrap();
    let plan = MigrationPlanner::between(
        &source,
        &target,
        RenameMap::new().table("parent", "person").unwrap(),
        DialectId::Sqlite,
    )
    .unwrap();
    assert!(plan.findings().iter().all(|r| !r.destructive));
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
    )
    .await
    .unwrap();
    let reverse = plan.reverse().unwrap();
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        reverse,
        reverse.hash(),
    )
    .await
    .unwrap();
    let db = engine(&options).await;
    db.execute(&Insert::new("parent").columns(["name"]).values(["next"]))
        .await
        .unwrap();
    assert_eq!(
        db.fetch(&Select::new(["id"]).from("parent").where_eq("name", "next"))
            .await
            .unwrap()[0]
            .try_get::<i64>("id")
            .unwrap(),
        101
    );
    assert_eq!(
        db.fetch(&Select::default().from("cascade"))
            .await
            .unwrap()
            .len(),
        1
    );
    db.close().await.unwrap();
}

/// A nullable addition with a populated default still needs a NULL-only reverse precondition.
async fn nullable_default() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("default.sqlite")).create_if_missing(true);
    let source = RevisionSnapshot::new("workspace", "v1", vec![table()]).unwrap();
    install(&options, &source).await;
    let db = engine(&options).await;
    db.execute(
        &Insert::new("profile")
            .columns(["id", "name"])
            .values(["p1", "Ada"]),
    )
    .await
    .unwrap();
    db.close().await.unwrap();
    let mut target = table();
    target
        .fields
        .push(Field::new("summary", SqlType::Text).default("filled"));
    let target = RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    let reverse = plan.reverse().unwrap();
    assert_eq!(
        reverse.discardable_columns(),
        &[("profile".into(), "summary".into())]
    );
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
    )
    .await
    .unwrap();
    assert!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            reverse,
            reverse.hash()
        )
        .await
        .is_err()
    );
    let db = engine(&options).await;
    db.execute(
        &stackpress_sql::Update::new("profile")
            .set("summary", SqlValue::Null)
            .where_eq("id", "p1"),
    )
    .await
    .unwrap();
    db.close().await.unwrap();
    MigrationRunner::apply(
        SqliteMaintenance::open(options).await.unwrap(),
        reverse,
        reverse.hash(),
    )
    .await
    .unwrap();
}
/// Native fixture tools are test-only and independently killed/reaped on timeout or unwind.
fn fixture_command(mut command: std::process::Command) {
    use std::{
        process::{Child, Stdio},
        time::Instant,
    };
    struct Guard(Child);
    impl Drop for Guard {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let file = std::fs::File::create(log.path()).unwrap();
    let mut child = Guard(
        command
            .stdout(Stdio::from(file.try_clone().unwrap()))
            .stderr(Stdio::from(file))
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    let status = loop {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "native fixture timed out"
        );
        assert!(log.as_file().metadata().unwrap().len() < 128 * 1024);
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    assert!(
        status.success(),
        "{}",
        std::fs::read_to_string(log.path()).unwrap()
    );
}

/// Fixed PostgreSQL/MySQL SQL snapshots are renderer evidence, never native application evidence.
fn rendered_snapshots() {
    let table = Create::new("entry")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let from = RevisionSnapshot::new("snapshot-app", "v1", vec![table.clone()]).unwrap();
    let mut target = table;
    target.fields[1].name = "heading".into();
    target.fields.push(Field::new("summary", SqlType::Text));
    target.indexes.push(Index::new("entry_identity", ["id"]));
    let to = RevisionSnapshot::new("snapshot-app", "v2", vec![target]).unwrap();
    for (dialect, name, expected) in [
        (
            DialectId::Postgres,
            "postgres",
            [
                include_str!("fixtures/m3/postgres-baseline.sql"),
                include_str!("fixtures/m3/postgres-up.sql"),
                include_str!("fixtures/m3/postgres-down.sql"),
            ],
        ),
        (
            DialectId::Mysql,
            "mysql",
            [
                include_str!("fixtures/m3/mysql-baseline.sql"),
                include_str!("fixtures/m3/mysql-up.sql"),
                include_str!("fixtures/m3/mysql-down.sql"),
            ],
        ),
    ] {
        let baseline = MigrationPlanner::baseline(&from, dialect).unwrap();
        let forward = MigrationPlanner::between(
            &from,
            &to,
            RenameMap::new().field("entry", "name", "heading").unwrap(),
            dialect,
        )
        .unwrap();
        let reverse = forward.reverse().unwrap();
        assert!(
            forward
                .findings()
                .iter()
                .all(|f| f.kind != "plain_integer_primary_key_may_reuse_deleted_ids")
        );
        for ((phase, plan), expected) in
            [("baseline", &baseline), ("up", &forward), ("down", reverse)]
                .into_iter()
                .zip(expected)
        {
            let text = plan
                .queries()
                .iter()
                .map(|q| {
                    assert!(q.binds().is_empty());
                    format!("{};\n", q.sql())
                })
                .collect::<String>();
            if let Some(root) = std::env::var_os("STACKPRESS_S6_SNAPSHOT_OUTPUT") {
                std::fs::write(
                    std::path::PathBuf::from(root).join(format!("{name}-{phase}.sql")),
                    &text,
                )
                .unwrap();
            } else {
                assert_eq!(text, expected, "{name}-{phase}");
            }
        }
    }
}
