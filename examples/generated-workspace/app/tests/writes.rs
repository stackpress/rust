//! Actual generated targeted writes and their authoritative/no-change outcomes.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::{
    contracts::{profile::CreateProfile, public::PatchProfile},
    stores::profile::{ProfileStore, Profiles},
};
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::{DataErrorKind, DataSession, InsertOutcome};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{sync::Arc, time::Duration};

#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Parent supervision covers the competing writers and native close, including executor stalls.
#[test]
fn generated_targeted_writes() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove_writes());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "generated_targeted_writes",
        "generated-writes",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}

/// Deserialize the same strict trusted DTO consumed by generated domain calls.
fn create(id: &str, name: &str) -> CreateProfile {
    serde_json::from_value(serde_json::json!({"id":id, "name":name})).unwrap()
}

/// Keep the typed patch independent of insert values so the conflict branch is observable.
fn patch(name: &str) -> PatchProfile {
    serde_json::from_value(serde_json::json!({"name":name})).unwrap()
}

/// Writes use statement-local RETURNING and the caller's exact transaction, with no retry.
async fn prove_writes() {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let engine = Arc::new(Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("writes.sqlite")).create_if_missing(true),
        )
        .await
        .unwrap(),
    ));
    engine
        .execute_plan(
            &ProfileStore::table()
                .unwrap()
                .create()
                .render(engine.dialect())
                .unwrap(),
        )
        .await
        .unwrap();
    let context = ValueContext::default();
    let mut session = DataSession::pooled(&engine);
    assert!(matches!(
        Profiles::create_if_absent(&mut session, create("a", "Ada"), &context)
            .await
            .unwrap(),
        InsertOutcome::Inserted(_)
    ));
    assert!(matches!(
        Profiles::create_if_absent(&mut session, create("a", "Ignored"), &context)
            .await
            .unwrap(),
        InsertOutcome::AlreadyExists
    ));
    assert_eq!(
        Profiles::get(&mut session, "a")
            .await
            .unwrap()
            .unwrap()
            .name,
        "Ada"
    );
    assert_eq!(
        Profiles::upsert(
            &mut session,
            "a",
            create("a", "Ignored insert"),
            patch("Updated"),
            &context
        )
        .await
        .unwrap()
        .name,
        "Updated"
    );
    assert_eq!(
        Profiles::upsert(
            &mut session,
            "b",
            create("b", "New"),
            patch("Ignored patch"),
            &context
        )
        .await
        .unwrap()
        .name,
        "New"
    );
    assert_eq!(
        Profiles::upsert(
            &mut session,
            "wrong",
            create("b", "Wrong"),
            patch("Wrong"),
            &context
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Validation
    );
    assert!(
        Profiles::update(&mut session, "missing", patch("Absent"), &context)
            .await
            .unwrap()
            .is_none()
    );
    assert_eq!(
        Profiles::update(&mut session, "a", patch("Last writer"), &context)
            .await
            .unwrap()
            .unwrap()
            .name,
        "Last writer"
    );
    assert!(
        Profiles::update(&mut session, "a", PatchProfile::default(), &context)
            .await
            .is_err()
    );
    assert!(
        Profiles::upsert(
            &mut session,
            "bad",
            create("bad", ""),
            patch("Good"),
            &context
        )
        .await
        .is_err()
    );
    assert!(Profiles::get(&mut session, "bad").await.unwrap().is_none());
    // Successful calls on a borrowed transaction do not publish or commit anything themselves.
    let mut transaction = engine.begin().await.unwrap();
    let mut tx_session = DataSession::transaction(&mut transaction);
    Profiles::upsert(
        &mut tx_session,
        "a",
        create("a", "Ignored"),
        patch("Rolled back"),
        &context,
    )
    .await
    .unwrap();
    Profiles::create_if_absent(&mut tx_session, create("rolled_back", "New"), &context)
        .await
        .unwrap();
    transaction.rollback().await.unwrap();
    assert_eq!(
        Profiles::get(&mut session, "a")
            .await
            .unwrap()
            .unwrap()
            .name,
        "Last writer"
    );
    assert!(
        Profiles::get(&mut session, "rolled_back")
            .await
            .unwrap()
            .is_none()
    );
    // Real concurrent generated calls enter the same key without a check-then-create window.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut workers = Vec::new();
    for name in ["First", "Second"] {
        let engine = engine.clone();
        let barrier = barrier.clone();
        workers.push(tokio::spawn(async move {
            barrier.wait().await;
            Profiles::create_if_absent(
                &mut DataSession::pooled(&engine),
                create("race", name),
                &ValueContext::default(),
            )
            .await
            .unwrap()
        }));
    }
    let mut inserted = 0;
    let mut existing = 0;
    for worker in workers {
        match worker.await.unwrap() {
            InsertOutcome::Inserted(_) => inserted += 1,
            InsertOutcome::AlreadyExists => existing += 1,
        }
    }
    assert_eq!((inserted, existing), (1, 1));
    assert!(
        ["First", "Second"].contains(
            &Profiles::get(&mut session, "race")
                .await
                .unwrap()
                .unwrap()
                .name
                .as_str()
        )
    );
    engine.close().await.unwrap();
}
