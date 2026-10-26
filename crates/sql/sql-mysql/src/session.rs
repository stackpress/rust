//! Exclusive transaction affinity, rollback-only failures, and consuming completion.

use crate::{
    execute,
    lease::{self, Lease},
};
use stackpress_sql::{
    ErrorKind, Query, QueryLimits, QueryResult, SqlError, SqlFuture, TransactionSession,
};
use std::time::Duration;

/// Published only after native BEGIN acknowledgement; never Clone or shared behind a mutex.
pub(crate) struct MysqlSession {
    /// Owns native capacity and admission through all transaction operations.
    lease: Lease,
    /// Set before any run work; only acknowledged success clears it.
    rollback_only: bool,
    /// Whole operation deadline inherited from database startup.
    timeout: Duration,
}
impl MysqlSession {
    /// Establish a usable session after BEGIN succeeds.
    pub(crate) fn new(lease: Lease, timeout: Duration) -> Self {
        Self {
            lease,
            rollback_only: false,
            timeout,
        }
    }
}
impl TransactionSession for MysqlSession {
    /// Poison before suspension; failed/cancelled runs allow only explicit rollback.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            if self.rollback_only {
                return Err(rollback_only());
            }
            // Poison even rejected statements consistently with SQLite's session contract.
            self.rollback_only = true;
            execute::validate_transaction(&query)?;
            let result =
                execute::timed_run(self.lease.connection()?, query, limits, self.timeout).await;
            if result.is_ok() {
                self.rollback_only = false;
            }
            result
        })
    }
    /// Own the session from future construction; unpolled/cancelled commit retires it.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            let Self {
                mut lease,
                rollback_only: failed,
                timeout,
            } = *self;
            let result = if failed {
                Err(rollback_only())
            } else {
                // Completion can race cancellation; never retry or promise rollback.
                execute::control(lease.connection()?, "COMMIT", timeout).await
            };
            lease::completed(result, lease.retire().await)
        })
    }
    /// Permit explicit rollback after any rejected, failed, or cancelled statement.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            let Self {
                mut lease, timeout, ..
            } = *self;
            let result = execute::control(lease.connection()?, "ROLLBACK", timeout).await;
            lease::completed(result, lease.retire().await)
        })
    }
}
/// Preserve the portable contract even when a native server could continue its transaction.
fn rollback_only() -> SqlError {
    SqlError::new(
        ErrorKind::RollbackOnly,
        "MySQL transaction requires rollback",
    )
}
