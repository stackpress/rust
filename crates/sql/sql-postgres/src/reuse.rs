//! PostgreSQL resets every acknowledged reusable lease outside transaction scope.
use crate::execute::driver_error;
use sqlx::{Connection, Executor, PgConnection};
use stackpress_sql::{Query, Result};

/// All admitted PostgreSQL ordinary statements require an acknowledged DISCARD before reuse.
pub(crate) fn allowed(_: &Query) -> bool {
    true
}
/// Clear temp tables, variables, advisory locks and prepared state, then restore fixed policy.
pub(crate) async fn reset(connection: &mut PgConnection) -> Result<()> {
    // SQLx caching remains disabled. Clear bookkeeping before DISCARD removes native handles.
    connection
        .clear_cached_statements()
        .await
        .map_err(driver_error)?;
    // PostgreSQL wraps multiple simple-query commands in an implicit transaction;
    // DISCARD ALL must be its own acknowledged command outside that block.
    connection
        .execute("DISCARD ALL")
        .await
        .map_err(driver_error)?;
    connection
        .execute("SET application_name = 'stackpress'; SET standard_conforming_strings = on")
        .await
        .map_err(driver_error)?;
    Ok(())
}
