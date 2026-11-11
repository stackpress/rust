//! Repository-only generated stores exercise the actual public driver boundaries.
use super::TestResult;
use stackpress_sql::Database;
use std::sync::Arc;

#[path = "stores/mod.rs"]
mod stores;

/// Preserve the entire moved store sequence, then close its exact native owner.
pub(super) async fn check(database: Arc<dyn Database>) -> TestResult<()> {
    let result = stores::check(database.clone()).await;
    let close = database.close().await;
    result?;
    close?;
    Ok(())
}
