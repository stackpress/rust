//! File-backed maintenance proofs. Every hang-class scenario is supervised by a parent process.

use std::sync::mpsc;
use std::time::Duration;

use stackpress_sql::{
    Create, Delete, Engine, Field, ForeignKey, Insert, ReferentialAction, Select, Sql, SqlType,
    SqlValue, TableCommand,
};
use tokio::sync::oneshot;

use super::*;
use crate::SqliteDatabase;

#[path = "../../../../../examples/generated-workspace/support/watchdog.rs"]
mod watchdog;

/// Run native state/cancellation/locking controls only inside independently supervised children.
#[test]
fn supervised_contracts() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        if scenario == "stuck" {
            // A spurious park wakeup must not let the negative control finish.
            loop {
                std::thread::park();
            }
        }
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match scenario.as_str() {
                    "rebuild" => rebuild_and_failed_copy().await,
                    "cancellation" => cancellation().await,
                    "writer" => single_writer().await,
                    "integrity" => commit_rejects_invalid_foreign_keys().await,
                    _ => panic!("unknown maintenance scenario"),
                }
            });
        return;
    }
    for scenario in ["rebuild", "cancellation", "writer", "integrity"] {
        let (ok, expired, output) = watchdog::run(
            "maintenance::tests::supervised_contracts",
            scenario,
            Duration::from_secs(10),
        );
        assert!(ok && !expired, "{scenario}: timeout={expired}\n{output}");
    }
    let (ok, expired, output) = watchdog::run(
        "maintenance::tests::supervised_contracts",
        "stuck",
        Duration::from_millis(150),
    );
    assert!(!ok && expired, "negative control failed: {output}");
}

/// Set explicit finite file/busy budgets on independent temporary databases.
fn options(root: &tempfile::TempDir) -> SqliteOptions {
    let mut options =
        SqliteOptions::file(root.path().join("maintenance.sqlite")).create_if_missing(true);
    options.busy_timeout_ms = 150;
    options
}

/// Open one ordinary Engine for setup/readback; imports and normal application startup install nothing.
async fn engine(options: SqliteOptions) -> Engine {
    Engine::new(SqliteDatabase::open(options).await.unwrap())
}

/// Install typed parent/child/state tables and seed real FK-linked rows.
async fn setup(options: SqliteOptions) {
    let engine = engine(options).await;
    let parent = Create::new("parent")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .primary_key(["id"]);
    engine
        .execute_plan(&parent.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    let mut foreign = ForeignKey::new("child_parent_fk", ["parent_id"], "parent", ["id"]);
    foreign.on_delete = ReferentialAction::Cascade;
    let child = Create::new("child")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("parent_id", SqlType::Integer).not_null())
        .primary_key(["id"])
        .add_foreign_key(foreign);
    engine
        .execute_plan(&child.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    let ledger =
        Create::new("proof_ledger").add_field(Field::new("head", SqlType::Text).not_null());
    engine
        .execute_plan(&ledger.render(DialectId::Sqlite).unwrap())
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("parent")
                .columns(["id", "name"])
                .values([SqlValue::Integer(1), "Ada".into()]),
        )
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("child")
                .columns(["id", "parent_id"])
                .values([1, 1]),
        )
        .await
        .unwrap();
    engine
        .execute(
            &Insert::new("proof_ledger")
                .columns(["head"])
                .values(["original"]),
        )
        .await
        .unwrap();
    engine.close().await.unwrap();
}

/// A real checked rebuild preserves inbound CASCADE children; failed copy restores all state.
async fn rebuild_and_failed_copy() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = options(&root);
    setup(options.clone()).await;
    // These ordinary policy/transaction commands remain rejected by the unchanged lexical gate.
    for command in ["PRAGMA foreign_keys = OFF", "BEGIN", "COMMIT"] {
        assert!(
            Sql::literal(DialectId::Sqlite, command)
                .render(DialectId::Sqlite)
                .is_err()
        );
    }
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert_eq!(
        session.policy(),
        RebuildPolicy {
            foreign_keys_initially_enabled: true,
            foreign_keys_disabled_before_begin: true
        }
    );
    let original = session.catalog().await.unwrap();
    let replacement = Create::new("replacement")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("heading", SqlType::Text).not_null())
        .add_field(Field::new("required", SqlType::Text).not_null())
        .primary_key(["id"]);
    for query in replacement.render(DialectId::Sqlite).unwrap().queries() {
        session.execute(query).await.unwrap();
    }
    let copy = CopyPlan::new(
        "parent",
        "replacement",
        &[
            CopyColumn {
                destination: "id".into(),
                source: CopySource::Column("id".into()),
            },
            CopyColumn {
                destination: "heading".into(),
                source: CopySource::Column("name".into()),
            },
        ],
    )
    .unwrap();
    assert!(session.execute(copy.query()).await.is_err());
    assert_eq!(
        session
            .execute(&Select::default().from("parent"))
            .await
            .unwrap_err()
            .kind(),
        ErrorKind::Closed
    );
    session.rollback().await.unwrap();
    // Native retirement was acknowledged; a fresh connection must see exact schema/head/data.
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert_eq!(session.catalog().await.unwrap(), original);
    assert_eq!(
        session
            .execute(&Select::new(["head"]).from("proof_ledger"))
            .await
            .unwrap()
            .rows[0]
            .try_get::<String>("head")
            .unwrap(),
        "original"
    );
    let replacement = Create::new("replacement")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("heading", SqlType::Text).not_null())
        .primary_key(["id"]);
    for query in replacement.render(DialectId::Sqlite).unwrap().queries() {
        session.execute(query).await.unwrap();
    }
    assert_eq!(session.execute(copy.query()).await.unwrap().affected, 1);
    session
        .execute(&TableCommand::drop_table("parent"))
        .await
        .unwrap();
    session
        .execute(&TableCommand::rename("replacement", "parent"))
        .await
        .unwrap();
    assert_eq!(
        session
            .execute(&Select::default().from("child"))
            .await
            .unwrap()
            .rows
            .len(),
        1
    );
    assert!(session.foreign_key_check().await.unwrap().is_empty());
    session
        .execute(
            &stackpress_sql::Update::new("proof_ledger")
                .set("head", "rebuilt")
                .all_rows(),
        )
        .await
        .unwrap();
    session.commit().await.unwrap();
    let engine = engine(options).await;
    // Reopening proves the schema/data replacement and ledger head shared one real commit.
    assert_eq!(
        engine
            .fetch(&Select::new(["head"]).from("proof_ledger"))
            .await
            .unwrap()[0]
            .try_get::<String>("head")
            .unwrap(),
        "rebuilt"
    );
    assert_eq!(
        engine
            .fetch(&Select::new(["heading"]).from("parent"))
            .await
            .unwrap()[0]
            .try_get::<String>("heading")
            .unwrap(),
        "Ada"
    );
    // Real subsequent FK enforcement and cascade behavior survive the dedicated OFF policy.
    assert!(
        engine
            .execute(
                &Insert::new("child")
                    .columns(["id", "parent_id"])
                    .values([2, 999])
            )
            .await
            .is_err()
    );
    engine
        .execute(&Delete::new("parent").where_eq("id", 1))
        .await
        .unwrap();
    assert!(
        engine
            .fetch(&Select::default().from("child"))
            .await
            .unwrap()
            .is_empty()
    );
    engine.close().await.unwrap();
}

/// Pause the native worker at a mutation/commit boundary, independently of executor scheduling.
async fn gate(
    slot: &mut ConnectionSlot,
    commit: bool,
) -> (oneshot::Receiver<()>, mpsc::Sender<()>) {
    let (ready, entered) = oneshot::channel();
    let (release, go) = mpsc::channel();
    let mut ready = Some(ready);
    let mut pause = move || {
        if let Some(ready) = ready.take() {
            ready.send(()).unwrap();
            go.recv_timeout(Duration::from_secs(3)).unwrap();
        }
    };
    let mut handle = slot
        .connection
        .as_mut()
        .unwrap()
        .lock_handle()
        .await
        .unwrap();
    if commit {
        handle.set_commit_hook(move || {
            pause();
            true
        });
    } else {
        handle.set_update_hook(move |_| pause());
    }
    (entered, release)
}

/// Barrier-controlled write cancellation retires its slot; lost commit acknowledgement is reconciled by data.
async fn cancellation() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = options(&root);
    setup(options.clone()).await;
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    let (mut entered, release) = gate(&mut session.slot, false).await;
    let insert = Insert::new("proof_ledger")
        .columns(["head"])
        .values(["cancelled"]);
    let mut pending = Box::pin(session.execute(&insert));
    tokio::select! {
        ready = &mut entered => ready.unwrap(),
        result = pending.as_mut() => panic!("mutation missed native gate: {result:?}"),
    }
    drop(pending);
    assert!(session.slot.connection.is_none());
    assert!(session.slot.cleanup.is_some());
    release.send(()).unwrap();
    session.rollback().await.unwrap();
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert_eq!(
        session
            .execute(&Select::default().from("proof_ledger"))
            .await
            .unwrap()
            .rows
            .len(),
        1
    );
    session
        .execute(
            &Insert::new("proof_ledger")
                .columns(["head"])
                .values(["committed without acknowledgement"]),
        )
        .await
        .unwrap();
    let (entered, release) = gate(&mut session.slot, true).await;
    let task = tokio::spawn(session.commit());
    entered.await.unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    release.send(()).unwrap();
    // Acquiring the writer waits for retirement: the ledger tells us commit really happened.
    let mut observed = SqliteMaintenance::open(options)
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert_eq!(
        observed
            .execute(&Select::default().from("proof_ledger"))
            .await
            .unwrap()
            .rows
            .len(),
        2
    );
    assert!(observed.policy().foreign_keys_initially_enabled);
    observed.rollback().await.unwrap();
}

/// A second dedicated owner fails finitely while the first writer is held, then recovers cleanly.
async fn single_writer() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = options(&root);
    setup(options.clone()).await;
    let first = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    let second = SqliteMaintenance::open(options.clone()).await.unwrap();
    let error = second.begin_rebuild().await.err().unwrap();
    assert_eq!(error.kind(), ErrorKind::Busy);
    first.rollback().await.unwrap();
    SqliteMaintenance::open(options)
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap()
        .rollback()
        .await
        .unwrap();
}

/// A direct maintenance caller cannot bypass the final FK check or commit a partial ledger.
async fn commit_rejects_invalid_foreign_keys() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = options(&root);
    setup(options.clone()).await;
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    session
        .execute(
            &Insert::new("proof_ledger")
                .columns(["head"])
                .values(["must roll back"]),
        )
        .await
        .unwrap();
    session
        .execute(
            &Insert::new("child")
                .columns(["id", "parent_id"])
                .values([2, 999]),
        )
        .await
        .unwrap();
    let violations = session.foreign_key_check().await.unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].table, "child");
    assert_eq!(violations[0].parent, "parent");
    assert_eq!(violations[0].rowid, Some(2));
    let failure = session.commit().await.unwrap_err();
    assert_eq!(failure.kind(), ErrorKind::InvalidQuery);
    // The consumed rejection awaited rollback/retirement before we reopen the ordinary pool.
    let engine = engine(options).await;
    assert_eq!(
        engine
            .fetch(&Select::default().from("child"))
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        engine
            .fetch(&Select::default().from("proof_ledger"))
            .await
            .unwrap()
            .len(),
        1
    );
    engine.close().await.unwrap();
}

/// Supervise the additive G35 cases without changing existing maintenance scenarios.
#[test]
fn supervised_sequences() {
    if let Ok(scenario) = std::env::var("STACKPRESS_S6_SCENARIO") {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                match scenario.as_str() {
                    "sequence-basic" => sequence_basic().await,
                    "sequence-invalid" => sequence_invalid().await,
                    "sequence-cancel" => sequence_cancel().await,
                    _ => panic!("unknown sequence scenario"),
                }
            });
        return;
    }
    for scenario in ["sequence-basic", "sequence-invalid", "sequence-cancel"] {
        let (ok, expired, output) = watchdog::run(
            "maintenance::tests::supervised_sequences",
            scenario,
            Duration::from_secs(10),
        );
        assert!(ok && !expired, "{scenario}: timeout={expired}\n{output}");
    }
}
/// Install independent automatic/plain/empty tables with a deleted historical maximum.
async fn sequence_setup(options: SqliteOptions) {
    let engine = engine(options).await;
    for (name, automatic) in [
        ("automatic", true),
        ("empty", true),
        ("other", true),
        ("plain", false),
    ] {
        let mut field = Field::new("id", SqlType::Integer).not_null();
        field.auto_increment = automatic;
        engine
            .execute_plan(
                &Create::new(name)
                    .add_field(field)
                    .primary_key(["id"])
                    .render(DialectId::Sqlite)
                    .unwrap(),
            )
            .await
            .unwrap();
    }
    for id in 1..=100 {
        engine
            .execute(&Insert::new("automatic").columns(["id"]).values([id]))
            .await
            .unwrap();
    }
    engine
        .execute(&Sql::literal(
            DialectId::Sqlite,
            "DELETE FROM automatic WHERE id > 1",
        ))
        .await
        .unwrap();
    engine
        .execute(&Insert::new("other").columns(["id"]).values([500]))
        .await
        .unwrap();
    engine.close().await.unwrap();
}
/// Monotonic restore handles absent rows, preserves other tables, and rolls back atomically.
async fn sequence_basic() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = self::options(&root);
    sequence_setup(options.clone()).await;
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert_eq!(
        session.sequence_high_water("automatic").await.unwrap(),
        Some(100)
    );
    assert_eq!(session.sequence_high_water("empty").await.unwrap(), None);
    assert_eq!(session.sequence_high_water("plain").await.unwrap(), None);
    session
        .preserve_sequence_high_water("empty", 10)
        .await
        .unwrap();
    session
        .preserve_sequence_high_water("automatic", 99)
        .await
        .unwrap();
    assert_eq!(
        session.sequence_high_water("automatic").await.unwrap(),
        Some(100)
    );
    session
        .preserve_sequence_high_water("automatic", 150)
        .await
        .unwrap();
    assert_eq!(
        session.sequence_high_water("automatic").await.unwrap(),
        Some(150)
    );
    assert_eq!(
        session.sequence_high_water("other").await.unwrap(),
        Some(500)
    );
    assert!(
        session
            .preserve_sequence_high_water("plain", 100)
            .await
            .is_err()
    );
    session.rollback().await.unwrap();
    let mut session = SqliteMaintenance::open(options)
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert!(session.policy().foreign_keys_initially_enabled);
    assert_eq!(
        session.sequence_high_water("automatic").await.unwrap(),
        Some(100)
    );
    assert_eq!(session.sequence_high_water("empty").await.unwrap(), None);
    session
        .preserve_sequence_high_water("empty", 0)
        .await
        .unwrap();
    session
        .preserve_sequence_high_water("automatic", 200)
        .await
        .unwrap();
    session.commit().await.unwrap();
}
/// Corrupt fixtures cannot be mistaken for an absent/valid sequence, or gain setter authority.
async fn sequence_invalid() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = self::options(&root);
    sequence_setup(options.clone()).await;
    for invalid in ["negative", "text", "duplicate"] {
        let mut session = SqliteMaintenance::open(options.clone())
            .await
            .unwrap()
            .begin_rebuild()
            .await
            .unwrap();
        for name in [
            "missing",
            "sqlite_sequence",
            "automatic;DROP TABLE other",
            "../automatic",
        ] {
            assert!(session.sequence_high_water(name).await.is_err());
        }
        assert!(
            session
                .preserve_sequence_high_water("automatic", -1)
                .await
                .is_err()
        );
        match invalid {
            "negative" => {
                session
                    .execute(
                        &stackpress_sql::Update::new("sqlite_sequence")
                            .set("seq", -1)
                            .where_eq("name", "automatic"),
                    )
                    .await
                    .unwrap();
            }
            "text" => {
                session
                    .execute(
                        &stackpress_sql::Update::new("sqlite_sequence")
                            .set("seq", "bad")
                            .where_eq("name", "automatic"),
                    )
                    .await
                    .unwrap();
            }
            _ => {
                session
                    .execute(
                        &Insert::new("sqlite_sequence")
                            .columns(["name", "seq"])
                            .values([SqlValue::Text("automatic".into()), SqlValue::Integer(2)]),
                    )
                    .await
                    .unwrap();
            }
        }
        assert!(session.sequence_high_water("automatic").await.is_err());
        assert!(
            session
                .preserve_sequence_high_water("automatic", 250)
                .await
                .is_err()
        );
        session.rollback().await.unwrap();
    }
    // No automatic table means no sqlite_sequence system table at all.
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = self::options(&root);
    let mut session = SqliteMaintenance::open(options)
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    session
        .execute(
            &Create::new("plain")
                .add_field(Field::new("AUTOINCREMENT", SqlType::Text).default("AUTOINCREMENT"))
                .add_field(Field::new("id", SqlType::Integer).not_null())
                .primary_key(["id"])
                .render(DialectId::Sqlite)
                .unwrap()
                .queries()[0],
        )
        .await
        .unwrap();
    assert_eq!(session.sequence_high_water("plain").await.unwrap(), None);
    assert!(
        session
            .preserve_sequence_high_water("plain", 1)
            .await
            .is_err()
    );
    session.rollback().await.unwrap();
    assert!(!has_autoincrement(
        "CREATE TABLE plain (id INTEGER PRIMARY KEY /* AUTOINCREMENT */, note TEXT DEFAULT 'AUTOINCREMENT') -- AUTOINCREMENT"
    ));
    assert!(has_autoincrement(
        "CREATE TABLE x (id INTEGER PRIMARY KEY autoincrement)"
    ));
}
/// Cancel after the real sequence mutation enters SQLite's native update hook.
async fn sequence_cancel() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let options = self::options(&root);
    sequence_setup(options.clone()).await;
    let mut session = SqliteMaintenance::open(options.clone())
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    let (mut entered, release) = gate(&mut session.slot, false).await;
    let mut pending = Box::pin(session.preserve_sequence_high_water("automatic", 200));
    tokio::select! {ready=&mut entered=>ready.unwrap(),result=pending.as_mut()=>panic!("sequence mutation missed native gate: {result:?}")}
    drop(pending);
    assert!(session.slot.connection.is_none());
    release.send(()).unwrap();
    session.rollback().await.unwrap();
    let mut session = SqliteMaintenance::open(options)
        .await
        .unwrap()
        .begin_rebuild()
        .await
        .unwrap();
    assert!(session.policy().foreign_keys_initially_enabled);
    assert_eq!(
        session.sequence_high_water("automatic").await.unwrap(),
        Some(100)
    );
    session.rollback().await.unwrap();
}
