//! Faults injected only after real SQLite effects prove unknown acknowledgements are never replayed.
#![cfg(not(any(feature = "matrix", feature = "names")))]
use generated_workspace_app::generated::{
    contracts::profile::CreateProfile,
    stores::profile::{BatchInput, Profiles},
};
use stackpress_framework_schema::ValueContext;
use stackpress_framework_sql::{DataError, DataErrorKind, DataSession};
use std::{
    error::Error as _,
    sync::{Arc, atomic::Ordering},
    time::Duration,
};
mod support;
#[path = "../../support/watchdog.rs"]
mod watchdog;
/// A parent bounds deliberately pending acknowledgements independently of the child's executor.
#[test]
fn lost_and_cancelled_write_acknowledgements() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "lost_and_cancelled_write_acknowledgements",
        "outcomes",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Minimal typed input; the fixture injects loss after the actual native result.
fn input(id: &str) -> CreateProfile {
    serde_json::from_value(serde_json::json!({"id":id,"name":"Acknowledgement test"})).unwrap()
}
/// Observe actual durable effects, typed causes and explicit cleanup after uncertain outcomes.
async fn prove() {
    let fixture = support::fixture().await;
    let engine = Arc::new(fixture.engine);
    fixture.calls.lock().unwrap().clear();
    fixture.fault.store(1, Ordering::SeqCst);
    let error = Profiles::create(&mut DataSession::pooled(&engine), input("lost-write"))
        .await
        .unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::UnknownOutcome);
    assert!(!error.to_string().contains("PRIVATE_"));
    let native = error
        .source()
        .unwrap()
        .downcast_ref::<stackpress_sql::SqlError>()
        .unwrap();
    assert!(native.source().unwrap().is::<support::LostAck>());
    assert_eq!(
        fixture.calls.lock().unwrap().len(),
        1,
        "no write replay or implicit refetch"
    );
    assert!(
        Profiles::get(&mut DataSession::pooled(&engine), "lost-write")
            .await
            .unwrap()
            .is_some()
    );
    // Commit completes natively, but its acknowledgement is lost to the atomic helper.
    fixture.calls.lock().unwrap().clear();
    fixture.fault.store(2, Ordering::SeqCst);
    let error = Profiles::batch_atomic(
        &engine,
        vec![BatchInput::Create {
            input: input("lost-commit"),
        }],
        &ValueContext::default(),
    )
    .await
    .unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::UnknownOutcome);
    assert_eq!(fixture.calls.lock().unwrap().len(), 1);
    assert!(
        Profiles::get(&mut DataSession::pooled(&engine), "lost-commit")
            .await
            .unwrap()
            .is_some()
    );
    // Cancellation after the real autocommit effect is not a guarantee of rollback.
    fixture.calls.lock().unwrap().clear();
    fixture.fault.store(4, Ordering::SeqCst);
    let db = engine.clone();
    let writer = tokio::spawn(async move {
        Profiles::create(&mut DataSession::pooled(&db), input("cancelled-ack")).await
    });
    tokio::time::timeout(Duration::from_secs(2), fixture.submitted.notified())
        .await
        .unwrap();
    writer.abort();
    assert!(writer.await.unwrap_err().is_cancelled());
    assert_eq!(fixture.calls.lock().unwrap().len(), 1);
    assert!(
        Profiles::get(&mut DataSession::pooled(&engine), "cancelled-ack")
            .await
            .unwrap()
            .is_some()
    );
    // Rollback is explicitly awaited, and a secondary failure never replaces the primary cause.
    let transaction = engine.begin().await.unwrap();
    fixture.fault.store(3, Ordering::SeqCst);
    let primary = DataError::new("domain workflow refused")
        .with_kind(DataErrorKind::Validation)
        .with_cause(support::LostAck);
    let result: Result<(), _> = stackpress_framework_sql::finish(transaction, Err(primary)).await;
    let error = result.unwrap_err();
    assert_eq!(error.kind(), DataErrorKind::Validation);
    assert!(error.source().unwrap().is::<support::LostAck>());
    assert_eq!(
        error.cleanup_error().unwrap().kind(),
        stackpress_sql::ErrorKind::Busy
    );
    engine.close().await.unwrap();
}
