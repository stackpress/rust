//! Setup policy contracts; the live aggregate suite supplies native MySQL/PostgreSQL evidence.

#[path = "../../../sql/sql/examples/support/mod.rs"]
mod recorder;

use stackpress_framework_sql::setup;
use stackpress_sql::{Create, DialectId, Engine, Field, Index, Render, Select, SqlType};

/// Both atomic entry points retain ordered owned results and publish only complete scripts.
#[tokio::test]
async fn g48_atomic_setup_plan_and_script_results() {
    let database = recorder::RecordingDatabase::default();
    let state = database.state.clone();
    let engine = Engine::new(database);
    let plan = Create::new("setup")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("setup_id", ["id"]))
        .render(DialectId::Sqlite)
        .unwrap();
    assert_eq!(setup::execute_plan(&engine, &plan).await.unwrap().len(), 2);
    let queries = plan.queries();
    assert_eq!(setup::execute(&engine, queries).await.unwrap().len(), 2);
    assert!(setup::execute(&engine, &[]).await.unwrap().is_empty());
    assert_eq!(state.lock().unwrap().committed.len(), 4);
    engine.close().await.unwrap();
}

/// A later failure still rolls back the atomic path rather than publishing a successful prefix.
#[tokio::test]
async fn g48_atomic_setup_retains_rollback_context() {
    let database = recorder::RecordingDatabase::default();
    let state = database.state.clone();
    let engine = Engine::new(database);
    let first = Select::new(["id"])
        .from("setup")
        .render(DialectId::Sqlite)
        .unwrap();
    let wrong = Select::new(["id"])
        .from("setup")
        .render(DialectId::Postgres)
        .unwrap();
    let error = setup::execute(&engine, &[first, wrong]).await.unwrap_err();
    assert!(error.plan_progress().is_none());
    let state = state.lock().unwrap();
    assert_eq!(state.rollbacks, 1);
    assert!(state.committed.is_empty());
}
