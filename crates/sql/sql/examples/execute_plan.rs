//! Run with `cargo run -p stackpress-sql --example execute_plan`.
//! Exercise transaction orchestration with a recording adapter; no native SQL runs.

/// Recording-only adapter shared with runnable API examples.
mod support;

use stackpress_sql::{Create, Engine, Field, Index, Result, SqlType};
use support::RecordingDatabase;

/// Complete owned plans and compose a caller-owned transaction on the same guard.
async fn install() -> Result<()> {
    let recorder = RecordingDatabase::default();
    let state = recorder.state.clone();
    let engine = Engine::new(recorder);
    let plan = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("profile_id", ["id"]))
        .render(engine.dialect())?;
    let outcomes = engine.execute_plan(&plan).await?;
    assert_eq!(outcomes.len(), 2);
    assert_eq!(state.lock().unwrap().committed.len(), 2);
    println!("committed {} ordered statement outcomes", outcomes.len());

    // The caller owns completion when composing multiple operations on one transaction.
    let mut transaction = engine.begin().await?;
    transaction.execute_plan(&plan).await?;
    transaction.rollback().await?;
    assert_eq!(state.lock().unwrap().committed.len(), 2);
    assert_eq!(state.lock().unwrap().rollbacks, 1);
    engine.close().await
}

/// Tokio is an existing dev-dependency, used only to run this example's async calls.
fn main() -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("example runtime")
        .block_on(install())
}
