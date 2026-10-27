//! Native Tokio/SQLx SQLite execution, with finite pool and query budgets.
//!
//! Open is eager and file-backed. Transactions own a lease across awaits; the adapter retires
//! every transaction connection on completion/drop, conservatively covering uncertain
//! native state. Ordinary statement leases are reused. No database is opened by import,
//! no listeners are started, and no SQL operation is automatically retried.
//!
//! Cancelling a future loses observation: a write/commit may already have happened.
//! Drop initiates worker close and native rollback; it does not synchronously acknowledge it.
//! Outside a live runtime, teardown is best effort; neither worker completion nor continued
//! admission coupling is acknowledged. Await close before shutting down the owning runtime.
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
mod session;

#[cfg(feature = "maintenance")]
pub mod maintenance;

use std::sync::Arc;

use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Executor, SqlitePool};
use stackpress_sql::{
    Database, DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

use lease::Lease;
use session::SqliteSession;

pub use options::SqliteOptions;

/// An eagerly opened, bounded native pool. Share the owning Engine through `Arc`.
pub struct SqliteDatabase {
    /// Bounded pool; only fully acknowledged ordinary statements reuse a connection.
    pool: SqlitePool,
    /// Retains admission through native completion while the cleanup runtime remains alive.
    admission: Arc<Semaphore>,
}

impl SqliteDatabase {
    /// Validate options and eagerly open a real file with FULL synchronous and foreign keys.
    /// Writable connections request WAL; read-only connections preserve the file's mode.
    ///
    /// # Errors
    /// Invalid options, unavailable files, permissions and native/open timeouts fail;
    /// there is no fallback to a different file or an in-memory database.
    ///
    /// Cancellation drops an unpublished pool and its initialized connections. Opening with
    /// creation enabled can leave a new empty file; file creation is not transactional.
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
            // Admission bounds even callers waiting for a connection. Cancellation during
            // acquisition drops the permit; no application SQL has been queued yet.
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
            // Couple the permit to its acquired connection before the first native command.
            // Cancelling acquisition itself leaves SQLx responsible for unpublished IO.
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
            Ok(Box::new(SqliteSession::new(lease)) as Box<dyn TransactionSession>)
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

#[cfg(test)]
mod tests;
