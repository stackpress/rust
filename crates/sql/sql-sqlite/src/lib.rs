//! Native Tokio/SQLx SQLite execution, with finite pool and query budgets.
//!
//! Open is eager and file-backed. Transactions own a lease across awaits; the adapter retires
//! every transaction connection on completion/drop, conservatively covering uncertain
//! native state. Ordinary statement leases are reused. No database is opened by import,
//! no listeners are started, and no SQL operation is automatically retried.
//!
//! Cancelling a future loses observation: a write/commit may already have happened.
//! Drop initiates worker close and native rollback; it does not synchronously acknowledge it.
//! Result limits bound retained first-party values, not SQLite VM memory/time or a
//! native cell's allocation before decoding. Verification covers native aarch64 macOS only.
//! Statements refresh the native schema and discard cached statement metadata before prepare,
//! favoring correct results after explicit DDL over reuse of prepared statement caches.
//!
//! Native insert IDs are reported only when this statement changes data and changes
//! the connection's last-rowid observation. Reused explicit rowids and WITHOUT ROWID
//! tables can yield `None`; use `INSERT ... RETURNING` for an authoritative generated
//! key on every insert. UPDATE/DELETE/SELECT do not inherit a previous statement's ID.
//!
//! ```no_run
//! use stackpress_sql::{DialectId, Engine, Sql};
//! use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
//! # async fn example() -> stackpress_sql::Result<()> {
//! let engine = Engine::new(SqliteDatabase::open(
//!     SqliteOptions::file("application.sqlite").create_if_missing(true),
//! ).await?);
//! let rows = engine.fetch(&Sql::literal(DialectId::Sqlite, "SELECT ? AS name")
//!     .bind("O'Reilly")).await?;
//! assert_eq!(rows[0].try_get::<String>("name")?, "O'Reilly");
//! engine.close().await?;
//! # Ok(())
//! # }
//! ```

mod collect;
mod execute;
mod lease;
mod options;

use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, SqlitePool};
use stackpress_sql::{
    Database, DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

use lease::Lease;

pub use options::SqliteOptions;

/// An eagerly opened, bounded native pool. Share the owning Engine through `Arc`.
pub struct SqliteDatabase {
    /// Bounded pool; only fully acknowledged ordinary statements reuse a connection.
    pool: SqlitePool,
    /// Counts admitted work through native completion, including detached cleanup.
    admission: Arc<Semaphore>,
}

impl SqliteDatabase {
    /// Validate options and eagerly open a real file with FULL synchronous and foreign keys.
    /// Writable connections request WAL; read-only connections preserve the file's mode.
    ///
    /// # Errors
    /// Invalid options, unavailable files, permissions and native/open timeouts fail;
    /// there is no fallback to a different file or an in-memory database.
    pub async fn open(options: SqliteOptions) -> Result<Self> {
        Self::open_pool(options, SqlitePoolOptions::new()).await
    }

    /// Apply the adapter's finite policy before eagerly opening the native pool.
    /// Keeping pool construction here also permits safe, private transition hooks in tests;
    /// callers of the public API cannot install callbacks or change connection policy.
    async fn open_pool(options: SqliteOptions, pool: SqlitePoolOptions) -> Result<Self> {
        options.validate()?;
        let connect = options.connect_options();
        let pool = pool
            .max_connections(options.max_connections)
            .min_connections(0)
            .acquire_timeout(options.acquire_timeout())
            .connect_with(connect)
            .await
            .map_err(execute::driver_error)?;
        Ok(Self {
            pool,
            admission: Arc::new(Semaphore::new(options.max_in_flight)),
        })
    }

    /// Fail promptly rather than queue unbounded owned requests before pool acquisition.
    fn admit(&self) -> Result<OwnedSemaphorePermit> {
        self.admission
            .clone()
            .try_acquire_owned()
            .map_err(|error| match error {
                TryAcquireError::Closed => {
                    SqlError::new(ErrorKind::Closed, "SQLite database is closed")
                }
                TryAcquireError::NoPermits => {
                    SqlError::new(ErrorKind::Limit, "SQLite admission limit exceeded")
                }
            })
    }
}

impl Database for SqliteDatabase {
    /// Select native SQLite rendering.
    fn dialect(&self) -> DialectId {
        DialectId::Sqlite
    }

    /// Prepare/validate/bind/execute on one lease and release it before returning.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            query.validate(limits)?;
            let permit = self.admit()?;
            let connection = self.pool.acquire().await.map_err(execute::driver_error)?;
            let mut lease = Lease::new(connection, permit);
            let result = execute::run(lease.connection()?, query, limits).await;
            // Only complete, acknowledged statements can return their connection for reuse.
            // Error/cancellation leaves the lease responsible for native shutdown.
            if result.is_ok() {
                lease.release();
            }
            result
        })
    }

    /// Own one native transaction without self-references or lifetime widening.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            let permit = self.admit()?;
            let connection = self.pool.acquire().await.map_err(execute::driver_error)?;
            let mut lease = Lease::new(connection, permit);
            // Internal control commands bypass the public raw gate, on one owned lease.
            // Cancellation/failure closes this connection, rolling back native state.
            lease
                .connection()?
                .execute("BEGIN")
                .await
                .map_err(execute::driver_error)?;
            Ok(Box::new(SqliteSession {
                lease,
                rollback_only: false,
            }) as Box<dyn TransactionSession>)
        })
    }

    /// Wake admission/pool waiters and wait for leases; safe to cancel and re-await.
    fn close(&self) -> SqlFuture<'_, ()> {
        Box::pin(async move {
            self.admission.close();
            self.pool.close().await;
            Ok(())
        })
    }
}

/// One native transaction on a leased connection; lease Drop closes/rolls back native state.
struct SqliteSession {
    /// Owns the connection/admission until completion or native cleanup acknowledgement.
    lease: Lease,
    /// Set before execution, cleared only after a successful acknowledgement.
    rollback_only: bool,
}

impl TransactionSession for SqliteSession {
    /// Retain exclusive affinity and make cancellation/error rollback-only.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            if self.rollback_only {
                return Err(rollback_only());
            }
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

#[cfg(test)]
mod tests;
