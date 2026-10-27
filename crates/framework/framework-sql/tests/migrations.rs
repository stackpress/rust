//! Real file-backed M0 rename/addition/ledger/reconciliation proof.
#![cfg(feature = "sqlite-maintenance")]

use std::time::Duration;

use stackpress_framework_sql::{
    DataErrorKind,
    migrations::{
        ApplyOutcome, MigrationPlanner, MigrationRunner, PlanHash, RenameMap, RevisionSnapshot,
    },
};
use stackpress_sql::{Create, DialectId, Engine, Field, Insert, Select, SqlType, Update};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions, maintenance::SqliteMaintenance};

#[path = "../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Kill/reap on a hung migration independently from the child's executor.
#[test]
fn supervised_migration_contract() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_migrations());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "supervised_migration_contract",
        "migrations",
        Duration::from_secs(10),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Build explicit revisions from actual normalized core definitions.
fn revisions() -> (RevisionSnapshot, RevisionSnapshot) {
    let original = Create::new("profile")
        .add_field(Field::new("id", SqlType::Text).not_null())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let mut target = original.clone();
    target.fields[1].name = "heading".into();
    target.fields.push(Field::new("summary", SqlType::Text));
    (
        RevisionSnapshot::new("workspace", "v1", vec![original]).unwrap(),
        RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap(),
    )
}

/// Verify pure refusal policy, atomic execution, reverse preconditions and non-replaying reconciliation.
async fn prove_migrations() {
    prove_competing_migrators().await;
    let (source, target) = revisions();
    let ambiguous_from = Create::new("ambiguous")
        .add_field(Field::new("id", SqlType::Text).not_null())
        .add_field(Field::new("first", SqlType::Text).not_null())
        .add_field(Field::new("last", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let mut ambiguous_to = ambiguous_from.clone();
    ambiguous_to.fields[1].name = "given".into();
    ambiguous_to.fields[2].name = "family".into();
    assert!(
        MigrationPlanner::between(
            &RevisionSnapshot::new("workspace", "ambiguous_old", vec![ambiguous_from]).unwrap(),
            &RevisionSnapshot::new("workspace", "ambiguous_new", vec![ambiguous_to]).unwrap(),
            RenameMap::new(),
            DialectId::Sqlite
        )
        .is_err()
    );
    assert!(
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).is_err()
    );
    let renames = RenameMap::new()
        .field("profile", "name", "heading")
        .unwrap();
    assert!(renames.clone().field("profile", "name", "summary").is_err());
    let mut invalid = target.tables().to_vec();
    invalid[0].fields[2].nullable = false;
    let invalid = RevisionSnapshot::new("workspace", "invalid", invalid).unwrap();
    assert!(
        MigrationPlanner::between(&source, &invalid, renames.clone(), DialectId::Sqlite).is_err()
    );
    let plan = MigrationPlanner::between(&source, &target, renames, DialectId::Sqlite).unwrap();
    assert_eq!(
        plan.hash(),
        MigrationPlanner::between(
            &source,
            &target,
            RenameMap::new()
                .field("profile", "name", "heading")
                .unwrap(),
            DialectId::Sqlite
        )
        .unwrap()
        .hash()
    );
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("schema.sqlite")).create_if_missing(true);
    let baseline = MigrationPlanner::baseline(&source, DialectId::Sqlite).unwrap();
    let open = || SqliteMaintenance::open(options.clone());
    assert_eq!(
        MigrationRunner::apply(open().await.unwrap(), &baseline, baseline.hash())
            .await
            .unwrap(),
        ApplyOutcome::Applied
    );
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    engine
        .execute(
            &Insert::new("profile")
                .columns(["id", "name"])
                .values(["u1", "Ada"]),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
    let wrong = PlanHash::parse(&"0".repeat(64)).unwrap();
    let refusal = MigrationRunner::apply(open().await.unwrap(), &plan, &wrong)
        .await
        .unwrap_err();
    assert_eq!(refusal.kind(), DataErrorKind::Stale);
    assert_eq!(
        MigrationRunner::apply(open().await.unwrap(), &plan, plan.hash())
            .await
            .unwrap(),
        ApplyOutcome::Applied
    );
    // Simulate losing the application acknowledgement after the real commit/close. The next
    // call must reconcile the target and plan hash, with no duplicated ledger or schema effect.
    assert_eq!(
        MigrationRunner::apply(open().await.unwrap(), &plan, plan.hash())
            .await
            .unwrap(),
        ApplyOutcome::AlreadyApplied
    );
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    let rows = engine
        .fetch(&Select::default().from("profile"))
        .await
        .unwrap();
    assert_eq!(rows[0].try_get::<String>("heading").unwrap(), "Ada");
    assert_eq!(rows[0].try_get::<Option<String>>("summary").unwrap(), None);
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        2
    );
    engine.close().await.unwrap();
    let reverse = plan.reverse().unwrap();
    assert_eq!(
        MigrationRunner::apply(open().await.unwrap(), reverse, reverse.hash())
            .await
            .unwrap(),
        ApplyOutcome::Applied
    );
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    assert_eq!(
        engine
            .fetch(&Select::new(["name"]).from("profile"))
            .await
            .unwrap()[0]
            .try_get::<String>("name")
            .unwrap(),
        "Ada"
    );
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        3
    );
    engine.close().await.unwrap();
    MigrationRunner::apply(open().await.unwrap(), &plan, plan.hash())
        .await
        .unwrap();
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    engine
        .execute(
            &Update::new("profile")
                .set("summary", "keep me")
                .where_eq("id", "u1"),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
    assert!(
        MigrationRunner::apply(open().await.unwrap(), reverse, reverse.hash())
            .await
            .is_err()
    );
    let engine = Engine::new(SqliteDatabase::open(options).await.unwrap());
    assert_eq!(
        engine
            .fetch(&Select::new(["summary"]).from("profile"))
            .await
            .unwrap()[0]
            .try_get::<String>("summary")
            .unwrap(),
        "keep me"
    );
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        4
    );
    engine.close().await.unwrap();
}

/// A real competing process opens before the shared start barrier, then applies the same plan.
#[test]
fn competing_process() {
    use std::io::Read;
    let Some(root) = std::env::var_os("STACKPRESS_S6_RACE_ROOT") else {
        return;
    };
    let root = std::path::PathBuf::from(root);
    let index = std::env::var("STACKPRESS_S6_RACE_INDEX").unwrap();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (source, target) = revisions();
            let plan = MigrationPlanner::between(
                &source,
                &target,
                RenameMap::new()
                    .field("profile", "name", "heading")
                    .unwrap(),
                DialectId::Sqlite,
            )
            .unwrap();
            let mut options = SqliteOptions::file(root.join("race.sqlite"));
            options.busy_timeout_ms = 1000;
            let owner = SqliteMaintenance::open(options).await.unwrap();
            std::fs::write(root.join(format!("ready-{index}")), b"ready").unwrap();
            let mut go = [0_u8];
            std::io::stdin().read_exact(&mut go).unwrap();
            let outcome = MigrationRunner::apply(owner, &plan, plan.hash())
                .await
                .unwrap();
            println!("s6-outcome:{outcome:?}");
        });
}

/// Kill/reap every competing child on timeout, IO failure, assertion or parent unwind.
struct Competitors(Vec<std::process::Child>);
impl Drop for Competitors {
    /// Retain no SQLite worker/process after a failed concurrency proof.
    fn drop(&mut self) {
        for child in &mut self.0 {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Two independent processes barrier-start the same plan; exactly one ledger application wins.
async fn prove_competing_migrators() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use std::time::Instant;
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("race.sqlite")).create_if_missing(true);
    let (source, _) = revisions();
    let baseline = MigrationPlanner::baseline(&source, DialectId::Sqlite).unwrap();
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &baseline,
        baseline.hash(),
    )
    .await
    .unwrap();
    let mut competitors = Competitors(Vec::new());
    let mut logs = Vec::new();
    for index in 0..2 {
        let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
        let file = std::fs::File::create(log.path()).unwrap();
        competitors.0.push(
            Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "competing_process", "--nocapture"])
                .env("STACKPRESS_S6_RACE_ROOT", root.path())
                .env("STACKPRESS_S6_RACE_INDEX", index.to_string())
                .stdin(Stdio::piped())
                .stdout(Stdio::from(file.try_clone().unwrap()))
                .stderr(Stdio::from(file))
                .spawn()
                .unwrap(),
        );
        logs.push(log);
    }
    let start = Instant::now();
    while !(root.path().join("ready-0").exists() && root.path().join("ready-1").exists()) {
        assert!(
            start.elapsed() < Duration::from_secs(3),
            "competing owners did not reach barrier"
        );
        for child in &mut competitors.0 {
            assert!(
                child.try_wait().unwrap().is_none(),
                "competitor failed before barrier"
            );
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    for child in &mut competitors.0 {
        child.stdin.take().unwrap().write_all(b"g").unwrap();
    }
    let mut completed = [false, false];
    while completed != [true, true] {
        assert!(
            start.elapsed() < Duration::from_secs(5),
            "competing migrator timed out"
        );
        for (index, child) in competitors.0.iter_mut().enumerate() {
            if completed[index] {
                continue;
            }
            assert!(logs[index].as_file().metadata().unwrap().len() < 128 * 1024);
            if let Some(status) = child.try_wait().unwrap() {
                let log = std::fs::read_to_string(logs[index].path()).unwrap();
                assert!(status.success(), "{log}");
                completed[index] = true;
            }
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    let output = logs
        .iter()
        .map(|log| std::fs::read_to_string(log.path()).unwrap())
        .collect::<String>();
    assert_eq!(output.matches("s6-outcome:Applied").count(), 1);
    assert_eq!(output.matches("s6-outcome:AlreadyApplied").count(), 1);
    let engine = Engine::new(SqliteDatabase::open(options).await.unwrap());
    assert_eq!(
        engine
            .fetch(&Select::default().from("_stackpress_migrations"))
            .await
            .unwrap()
            .len(),
        2
    );
    engine.close().await.unwrap();
}
