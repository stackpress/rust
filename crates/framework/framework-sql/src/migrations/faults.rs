//! Test-only native boundary failures, cancellation and post-commit acknowledgement loss.

use super::*;
use std::{cell::Cell, time::Duration};
#[path = "../../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;
thread_local! {
    /// Current-thread supervised tests own these counters; no native production state is global.
    static COUNT:Cell<usize>=const{Cell::new(0)};
    static FAIL:Cell<usize>=const{Cell::new(0)};
    static CANCEL:Cell<usize>=const{Cell::new(0)};
    static ENTERED:Cell<bool>=const{Cell::new(false)};
    static LOSE_COMMIT:Cell<bool>=const{Cell::new(false)};
}
/// Inject an actual native SQL failure after a selected acknowledged phase.
pub(super) async fn checkpoint(session: &mut MaintenanceSession) -> DataResult<()> {
    let count = COUNT.with(|c| {
        let n = c.get() + 1;
        c.set(n);
        n
    });
    if FAIL.with(|c| c.get() == count) {
        session
            .execute(&Sql::literal(
                DialectId::Sqlite,
                "SELECT * FROM _sp_injected_missing_table",
            ))
            .await?;
        panic!("injected native failure unexpectedly succeeded");
    }
    if CANCEL.with(|c| c.get() == count) {
        ENTERED.with(|v| v.set(true));
        std::future::pending::<()>().await;
    }
    Ok(())
}
/// Drop the application acknowledgement only after the real commit/close succeeded.
pub(super) fn commit_ack() -> DataResult<()> {
    if LOSE_COMMIT.with(Cell::get) {
        return Err(DataError::new("injected lost migration acknowledgement")
            .with_kind(DataErrorKind::UnknownOutcome));
    }
    Ok(())
}
/// Every hang-class fault/cancel test has an executor-independent parent watchdog.
#[test]
fn supervised_faults() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match scenario.as_str() {
                    "faults" => fault_matrix(false).await,
                    "cancel" => fault_matrix(true).await,
                    "unknown-commit" => unknown_commit().await,
                    _ => panic!("unknown fault scenario"),
                }
            });
        return;
    }
    for scenario in ["faults", "cancel", "unknown-commit"] {
        let (ok, expired, output) = watchdog::run(
            "migrations::sqlite::faults::supervised_faults",
            scenario,
            Duration::from_secs(20),
        );
        assert!(ok && !expired, "{scenario}: timeout={expired}\n{output}");
    }
}
/// Reset one private current-thread fixture's injection state.
fn reset() {
    COUNT.with(|c| c.set(0));
    FAIL.with(|c| c.set(0));
    CANCEL.with(|c| c.set(0));
    ENTERED.with(|c| c.set(false));
    LOSE_COMMIT.with(|c| c.set(false));
}
/// Actual default-change reconstruction plus a secondary index and namespaced ledger.
fn revisions() -> (RevisionSnapshot, MigrationPlan) {
    let table = Create::new("profile")
        .add_field(
            Field::new("id", SqlType::Integer)
                .not_null()
                .auto_increment(),
        )
        .add_field(Field::new("name", SqlType::Text).not_null())
        .primary_key(["id"])
        .definition()
        .unwrap();
    let source = RevisionSnapshot::new("workspace", "v1", vec![table.clone()]).unwrap();
    let mut target = table;
    target.fields[1].default = Some("fallback".into());
    target
        .indexes
        .push(stackpress_sql::Index::new("profile_name", ["name"]));
    let target = RevisionSnapshot::new("workspace", "v2", vec![target]).unwrap();
    let plan =
        MigrationPlanner::between(&source, &target, RenameMap::new(), DialectId::Sqlite).unwrap();
    (source, plan)
}
/// Install/seed one isolated database without injection.
async fn setup(options: &SqliteOptions, source: &RevisionSnapshot) {
    reset();
    let baseline = MigrationPlanner::baseline(source, DialectId::Sqlite).unwrap();
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &baseline,
        baseline.hash(),
    )
    .await
    .unwrap();
    let engine = Engine::new(SqliteDatabase::open(options.clone()).await.unwrap());
    engine
        .execute(
            &Insert::new("profile")
                .columns(["id", "name"])
                .values([SqlValue::Integer(100), "kept".into()]),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
    reset();
}
/// Capture exact catalog plus rows/head/history without depending on unstable process timing.
async fn state(options: &SqliteOptions) -> (Vec<CatalogObject>, String) {
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert!(session.policy().foreign_keys_initially_enabled);
    let catalog = session.catalog().await.unwrap();
    let mut data = String::new();
    for table in [
        "profile",
        "_stackpress_migrations",
        "_stackpress_schema_state",
        "sqlite_sequence",
    ] {
        let rows = session
            .execute(&Select::default().from(table))
            .await
            .unwrap()
            .rows;
        data.push_str(&format!("{rows:?}"));
    }
    session.rollback().await.unwrap();
    (catalog, data)
}
/// Native failures and cancellation at every counted phase preserve exact original state.
async fn fault_matrix(cancel: bool) {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("faults.sqlite")).create_if_missing(true);
    let (source, plan) = revisions();
    setup(&options, &source).await;
    // Measure the actual production sequence, including both ledger writes, once.
    MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
    )
    .await
    .unwrap();
    let count = COUNT.with(Cell::get);
    assert!(count >= 14);
    for boundary in 1..=count {
        let root = tempfile::tempdir_in("/private/tmp").unwrap();
        let options = SqliteOptions::file(root.path().join("fault.sqlite")).create_if_missing(true);
        setup(&options, &source).await;
        let before = state(&options).await;
        if cancel {
            CANCEL.with(|c| c.set(boundary));
        } else {
            FAIL.with(|c| c.set(boundary));
        }
        let owner = SqliteMaintenance::open(options.clone()).await.unwrap();
        if cancel {
            let mut future = Box::pin(MigrationRunner::apply(owner, &plan, plan.hash()));
            loop {
                tokio::select! {result=future.as_mut()=>panic!("cancel boundary {boundary} missed: {result:?}"),_=tokio::task::yield_now()=>{}}
                if ENTERED.with(Cell::get) {
                    break;
                }
            }
            drop(future);
        } else {
            assert!(
                MigrationRunner::apply(owner, &plan, plan.hash())
                    .await
                    .is_err(),
                "fault boundary {boundary}"
            );
        }
        reset();
        assert_eq!(
            state(&options).await,
            before,
            "boundary={boundary}, cancel={cancel}"
        );
    }
    reset();
}
/// A real committed ledger reconciles a lost response without executing the migration twice.
async fn unknown_commit() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = SqliteOptions::file(root.path().join("unknown.sqlite")).create_if_missing(true);
    let (source, plan) = revisions();
    setup(&options, &source).await;
    LOSE_COMMIT.with(|c| c.set(true));
    let result = MigrationRunner::apply(
        SqliteMaintenance::open(options.clone()).await.unwrap(),
        &plan,
        plan.hash(),
    )
    .await
    .unwrap_err();
    assert_eq!(result.kind(), DataErrorKind::UnknownOutcome);
    reset();
    assert_eq!(
        MigrationRunner::apply(
            SqliteMaintenance::open(options.clone()).await.unwrap(),
            &plan,
            plan.hash()
        )
        .await
        .unwrap(),
        ApplyOutcome::AlreadyApplied
    );
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

/// Unknown native declarations can contain UTF-8 outside quoted identifiers without panicking.
#[test]
fn utf8_catalog_tokens() {
    let values = tokens("CREATE VIEW café AS SELECT * FROM profile /* 🐟 */");
    assert!(values.iter().any(|token| token == "profile"));
}
