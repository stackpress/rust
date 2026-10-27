//! Whole-batch preflight, two-store atomicity and last-writer-wins concurrency over SQLite.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::{
    contracts::{profile::CreateProfile, project::CreateProject, public::PatchProfile},
    stores::{
        profile::{BatchInput, BatchOutcome, Profiles},
        project::Projects,
    },
};
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::{DataError, DataErrorKind, DataSession};
use stackpress_sql::QueryLimits;
use std::{sync::Arc, time::Duration};
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// Native concurrency and transaction completion run under an independent deadline.
#[test]
fn atomic_batches_and_stores() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "atomic_batches_and_stores",
        "atomic",
        Duration::from_secs(20),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Strict typed create helper used only to arrange meaningful mutation cases.
fn profile(id: &str, name: &str) -> CreateProfile {
    serde_json::from_value(serde_json::json!({"id":id,"name":name})).unwrap()
}
/// All plans validate before writes; the caller/atomic helper retains completion responsibility.
async fn prove() {
    let mut fixture = support::fixture().await;
    let context = ValueContext::default();
    let mut tx = fixture.engine.begin().await.unwrap();
    fixture.calls.lock().unwrap().clear();
    let error = Profiles::batch(
        &mut DataSession::transaction(&mut tx),
        vec![
            BatchInput::Create {
                input: profile("good", "Good"),
            },
            BatchInput::Create {
                input: profile("bad", ""),
            },
        ],
        &context,
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::Validation);
    assert!(fixture.calls.lock().unwrap().is_empty());
    tx.rollback().await.unwrap();
    // A later render/byte-budget refusal must also precede the first valid item's write.
    fixture.engine = fixture.engine.with_limits(QueryLimits {
        bytes: 256,
        ..Default::default()
    });
    let mut oversized = serde_json::json!({"id":"large","name":"Large","address":{"city":"x".repeat(1024),"country":"PH"}});
    let mut tx = fixture.engine.begin().await.unwrap();
    fixture.calls.lock().unwrap().clear();
    assert!(
        Profiles::batch(
            &mut DataSession::transaction(&mut tx),
            vec![
                BatchInput::Create {
                    input: profile("first", "First")
                },
                BatchInput::Create {
                    input: serde_json::from_value(oversized.take()).unwrap()
                }
            ],
            &context
        )
        .await
        .is_err()
    );
    assert!(fixture.calls.lock().unwrap().is_empty());
    tx.rollback().await.unwrap();
    fixture.engine = fixture.engine.with_limits(QueryLimits::default());
    let mut pooled = DataSession::pooled(&fixture.engine);
    assert_eq!(
        Profiles::batch(
            &mut pooled,
            vec![BatchInput::Create {
                input: profile("no-tx", "No")
            }],
            &context
        )
        .await
        .unwrap_err()
        .kind(),
        DataErrorKind::Unsupported
    );
    let result = Profiles::batch_atomic(
        &fixture.engine,
        vec![
            BatchInput::Create {
                input: profile("batch", "First"),
            },
            BatchInput::Update {
                key: serde_json::from_value(serde_json::json!({"id":"batch"})).unwrap(),
                patch: serde_json::from_value(serde_json::json!({"name":"Second"})).unwrap(),
            },
            BatchInput::Upsert {
                key: serde_json::from_value(serde_json::json!({"id":"batch"})).unwrap(),
                input: profile("batch", "Ignored"),
                patch: serde_json::from_value(serde_json::json!({"name":"Third"})).unwrap(),
            },
        ],
        &context,
    )
    .await
    .unwrap();
    assert!(matches!(&result[0],BatchOutcome::Created(row) if row.name=="First"));
    assert!(matches!(&result[1],BatchOutcome::Updated(Some(row)) if row.name=="Second"));
    assert!(matches!(&result[2],BatchOutcome::Upserted(row) if row.name=="Third"));
    assert!(
        Profiles::batch_atomic(
            &fixture.engine,
            vec![
                BatchInput::Create {
                    input: profile("must-rollback", "New")
                },
                BatchInput::Create {
                    input: profile("batch", "Conflict")
                }
            ],
            &context
        )
        .await
        .is_err()
    );
    assert!(
        Profiles::get(&mut pooled, "must-rollback")
            .await
            .unwrap()
            .is_none()
    );
    // Two generated store families use one exact exclusive session, with no hidden pool lease.
    let mut tx = fixture.engine.begin().await.unwrap();
    let mut db = DataSession::transaction(&mut tx);
    Profiles::create(&mut db, profile("owner", "Owner"))
        .await
        .unwrap();
    let project: CreateProject = serde_json::from_value(
        serde_json::json!({"id":"project","title":"Project","ownerId":"owner"}),
    )
    .unwrap();
    Projects::create(&mut db, project).await.unwrap();
    let error = DataError::new("domain step refused after both stores");
    let outcome: Result<(), _> = stackpress_framework_sql::finish(tx, Err(error)).await;
    assert!(outcome.is_err());
    assert!(Profiles::get(&mut pooled, "owner").await.unwrap().is_none());
    assert!(
        Projects::get(&mut pooled, "project")
            .await
            .unwrap()
            .is_none()
    );
    let mut tx = fixture.engine.begin().await.unwrap();
    let mut db = DataSession::transaction(&mut tx);
    Profiles::create(&mut db, profile("owner", "Owner"))
        .await
        .unwrap();
    Projects::create(
        &mut db,
        serde_json::from_value(
            serde_json::json!({"id":"project","title":"Project","ownerId":"owner"}),
        )
        .unwrap(),
    )
    .await
    .unwrap();
    stackpress_framework_sql::finish(tx, Ok(())).await.unwrap();
    assert!(Profiles::get(&mut pooled, "owner").await.unwrap().is_some());
    assert!(
        Projects::get(&mut pooled, "project")
            .await
            .unwrap()
            .is_some()
    );
    // Both concurrent key-scoped updates succeed; no expected-version predicate rejects a writer.
    let engine = Arc::new(fixture.engine);
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut writers = Vec::new();
    for name in ["Writer one", "Writer two"] {
        let engine = engine.clone();
        let barrier = barrier.clone();
        writers.push(tokio::spawn(async move {
            barrier.wait().await;
            let patch: PatchProfile =
                serde_json::from_value(serde_json::json!({"name":name})).unwrap();
            Profiles::update(
                &mut DataSession::pooled(&engine),
                "owner",
                patch,
                &ValueContext::default(),
            )
            .await
            .unwrap()
            .unwrap()
            .name
        }));
    }
    let mut names = Vec::new();
    for writer in writers {
        names.push(writer.await.unwrap());
    }
    names.sort();
    assert_eq!(names, ["Writer one", "Writer two"]);
    let final_name = Profiles::get(&mut DataSession::pooled(&engine), "owner")
        .await
        .unwrap()
        .unwrap()
        .name;
    assert!(names.contains(&final_name));
    engine.close().await.unwrap();
}
