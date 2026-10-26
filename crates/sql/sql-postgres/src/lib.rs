//! Bounded native PostgreSQL execution on the caller's Tokio runtime.
//!
//! Importing this crate performs no IO. Open is eager; it never creates a database.
//! Driver types remain private. Share an Engine through Arc, not a cloned database.
//! M0 conservatively retires every completed lease, including ordinary success;
//! connection/cache reuse and the full codec/outcome matrix await later milestones.
//!
//! Cancellation loses observation: a write or commit may already have taken effect.
//! Cleanup retains admission through bounded transport disposal while the runtime lives.
//! Outside a runtime, Drop disposes sockets by RAII, without claiming remote rollback
//! acknowledgement. Drain application tasks and await close before runtime shutdown.
//! Result caps bound retained first-party copies, not driver packet/server allocations.
//! Native date/decimal/UUID/array codecs and automatic retries are excluded.
//!
//! ```no_run
//! use stackpress_sql::Engine;
//! use stackpress_sql_postgres::{PostgresDatabase, PostgresOptions};
//! # async fn application(secret: String) -> stackpress_sql::Result<()> {
//! let database = PostgresDatabase::open(PostgresOptions::new(
//!     "db.internal", "application", "application", secret,
//! )).await?;
//! let engine = Engine::new(database);
//! engine.close().await?;
//! # Ok(())
//! # }
//! ```

//! A database owner cannot be cloned; share its Engine explicitly.
//!
//! ```compile_fail
//! use stackpress_sql_postgres::PostgresDatabase;
//! fn clone_owner(database: &PostgresDatabase) { let _: PostgresDatabase = database.clone(); }
//! ```
//!
//! A transaction retains one exclusive mutable borrow across each awaited query.
//!
//! ```compile_fail
//! use stackpress_sql::{TransactionSession, Query, QueryLimits};
//! async fn overlap(session: &mut dyn TransactionSession, query: Query) {
//!     let first = session.run(query.clone(), QueryLimits::default());
//!     let second = session.run(query, QueryLimits::default());
//!     first.await.unwrap();
//!     second.await.unwrap();
//! }
//! ```
//!
//! Completion consumes session ownership when the future is constructed.
//!
//! ```compile_fail
//! use stackpress_sql::TransactionSession;
//! fn reuse(session: Box<dyn TransactionSession>) {
//!     let completion = session.commit();
//!     drop(session);
//!     drop(completion);
//! }
//! ```

mod collect;
mod execute;
mod lease;
mod options;
mod session;

use lease::{Cleanup, Lease};
pub use options::{PostgresOptions, PostgresTls};
use session::PostgresSession;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use stackpress_sql::{
    Database, DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

/// Non-Clone, eagerly opened database owner; native pool handles never escape.
pub struct PostgresDatabase {
    /// Native capacity bounds checked-out, acquiring, and retiring connections.
    pool: PgPool,
    /// Application admission includes pool waiters and transaction lifetime.
    admission: Arc<Semaphore>,
    /// Tracks every Drop-triggered retirement through cancellation/runtime teardown.
    cleanup: Arc<Cleanup>,
    /// Maximum prepare/execution/control duration per operation.
    statement_timeout: Duration,
    /// Maximum local graceful retirement duration.
    cleanup_timeout: Duration,
}
impl PostgresDatabase {
    /// Validate explicit fields and eagerly connect to an existing database.
    ///
    /// # Errors
    /// Invalid options, TLS/authentication/connection failures and finite open deadlines
    /// fail without publishing a partial owner. Public errors never echo native messages.
    pub async fn open(options: PostgresOptions) -> Result<Self> {
        options.validate()?;
        let connect = options.connect_options()?;
        // The outer deadline bounds the entire handshake, not just time waiting in a pool.
        let pool = tokio::time::timeout(
            options.acquire_timeout(),
            PgPoolOptions::new()
                .max_connections(options.max_connections)
                .min_connections(0)
                .acquire_timeout(options.acquire_timeout())
                .connect_with(connect),
        )
        .await
        .map_err(|cause| {
            SqlError::with_source(
                ErrorKind::PoolTimedOut,
                "PostgreSQL open deadline exceeded",
                cause,
            )
        })?
        .map_err(execute::driver_error)?;
        Ok(Self {
            pool,
            admission: Arc::new(Semaphore::new(options.max_in_flight)),
            cleanup: Arc::new(Cleanup::default()),
            statement_timeout: options.statement_timeout(),
            cleanup_timeout: options.cleanup_timeout(),
        })
    }
    /// Bound pending work immediately; no unbounded admission queue is created.
    fn admit(&self) -> Result<OwnedSemaphorePermit> {
        self.admission
            .clone()
            .try_acquire_owned()
            .map_err(|cause| match cause {
                TryAcquireError::Closed => {
                    SqlError::new(ErrorKind::Closed, "PostgreSQL database is closed")
                }
                TryAcquireError::NoPermits => {
                    SqlError::new(ErrorKind::Limit, "PostgreSQL admission limit exceeded")
                }
            })
    }
    /// Publish coupled ownership only after a bounded native acquisition completes.
    async fn lease(&self) -> Result<Lease> {
        let permit = self.admit()?;
        // Cancellation before publication leaves SQLx responsible for unpublished IO.
        let connection = self.pool.acquire().await.map_err(execute::driver_error)?;
        Ok(Lease::new(
            connection,
            permit,
            self.cleanup_timeout,
            self.cleanup.clone(),
        ))
    }
}
impl Database for PostgresDatabase {
    /// Select exactly the backend's renderer and native syntax.
    fn dialect(&self) -> DialectId {
        DialectId::Postgres
    }
    /// Execute on one owned lease, retiring it before returning the complete result.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            // Direct trait users receive the same pre-effect validation as Engine users.
            execute::validate(&query, limits)?;
            let mut lease = self.lease().await?;
            let result =
                execute::timed_run(lease.connection()?, query, limits, self.statement_timeout)
                    .await;
            lease::completed(result, lease.retire().await)
        })
    }
    /// Publish a transaction only after BEGIN is acknowledged on its exclusive lease.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            let mut lease = self.lease().await?;
            // Failure/cancellation before publication retires uncertain native BEGIN state.
            if let Err(primary) =
                execute::control(lease.connection()?, "BEGIN", self.statement_timeout).await
            {
                return lease::completed(Err(primary), lease.retire().await);
            }
            Ok(
                Box::new(PostgresSession::new(lease, self.statement_timeout))
                    as Box<dyn TransactionSession>,
            )
        })
    }
    /// Shut admission, wake pool waiters and await held/retiring leases; re-awaitable.
    fn close(&self) -> SqlFuture<'_, ()> {
        Box::pin(async move {
            self.admission.close();
            self.pool.close().await;
            self.cleanup.wait().await
        })
    }
}
#[cfg(test)]
mod tests;
