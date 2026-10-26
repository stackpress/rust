//! Exclusive native transaction state, consuming completion and rollback-only cancellation.

use sqlx::Executor;
use stackpress_sql::{
    ErrorKind, Query, QueryLimits, QueryResult, SqlError, SqlFuture, TransactionSession,
};

use crate::execute;
use crate::lease::Lease;

/// One native transaction on a leased connection; lease Drop closes/rolls back native state.
pub(super) struct SqliteSession {
    /// Owns the connection/admission until completion or native cleanup acknowledgement.
    lease: Lease,
    /// Set before execution, cleared only after a successful acknowledgement.
    rollback_only: bool,
}

impl SqliteSession {
    /// Publish a usable transaction only after BEGIN has been acknowledged on this lease.
    pub(super) fn new(lease: Lease) -> Self {
        Self {
            lease,
            rollback_only: false,
        }
    }
}

impl TransactionSession for SqliteSession {
    /// Retain exclusive affinity and make cancellation/error rollback-only.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            if self.rollback_only {
                return Err(rollback_only());
            }
            // Poison before the first possible suspension, including schema/counter IO.
            // Exclusive connection borrowing preserves affinity without a mutex guard.
            self.rollback_only = true;
            let result = execute::run(self.lease.connection()?, query, limits).await;
            // Dropping this future before acknowledgement leaves rollback_only set.
            if result.is_ok() {
                self.rollback_only = false;
            }
            result
        })
    }

    /// Consume ownership; cancellation may leave a committed or rolled-back outcome.
    fn commit(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            if self.rollback_only {
                return Err(rollback_only());
            }
            // Ownership stays inside this future; dropping it retires the lease even if
            // COMMIT completed natively before its response reached the caller.
            self.lease
                .connection()?
                .execute("COMMIT")
                .await
                .map_err(execute::driver_error)?;
            // The consumed session retires its lease after native acknowledgement.
            Ok(())
        })
    }

    /// Explicitly roll back even after a statement failure; retirement covers cleanup failure.
    fn rollback(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            // Cleanup remains available after a failed run. Cancellation still consumes
            // the session and transfers its lease to nonblocking retirement.
            self.lease
                .connection()?
                .execute("ROLLBACK")
                .await
                .map_err(execute::driver_error)?;
            Ok(())
        })
    }
}

/// Explain why an uncertain/failed statement cannot be followed by commit or another query.
fn rollback_only() -> SqlError {
    SqlError::new(
        ErrorKind::RollbackOnly,
        "SQLite transaction requires rollback",
    )
}
