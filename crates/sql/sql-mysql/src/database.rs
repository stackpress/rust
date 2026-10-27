//! Eager database ownership, admission and existing Database trait integration.

use crate::lease::{self, Cleanup, Lease};
use crate::options::MysqlOptions;
use crate::session::MysqlSession;
use crate::{execute, reuse};
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{Executor, MySqlConnection, MySqlPool};
use stackpress_sql::{
    Database, DialectId, ErrorKind, Query, QueryLimits, QueryResult, Result, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{OwnedSemaphorePermit, Semaphore, TryAcquireError};

/// Non-Clone, eagerly opened database owner; native pool handles never escape.
///
/// Use the compiled example on [`Self::open`] to place this owner behind an Engine.
pub struct MysqlDatabase {
    /// Native capacity bounds checked-out, acquiring, and retiring connections.
    pool: MySqlPool,
    /// Application admission includes pool waiters and transaction lifetime.
    pub(crate) admission: Arc<Semaphore>,
    /// Tracks every Drop-triggered retirement through cancellation/runtime teardown.
    cleanup: Arc<Cleanup>,
    /// Maximum prepare/execution/control duration per operation.
    statement_timeout: Duration,
    /// Maximum local reset or graceful retirement duration.
    cleanup_timeout: Duration,
}
impl MysqlDatabase {
    /// Validate explicit fields and eagerly connect to an existing database.
    ///
    /// # Errors
    /// Invalid options, TLS/authentication/connection failures and finite open deadlines
    /// fail without publishing a partial owner. Public errors never echo native messages.
    ///
    /// ```no_run
    /// use stackpress_sql_mysql::{MysqlDatabase, MysqlOptions};
    /// use stackpress_sql::Engine;
    /// async fn application(secret: String) -> stackpress_sql::Result<()> {
    ///     let options = MysqlOptions::new("db.internal", "application", "application", secret);
    ///     let engine = Engine::new(MysqlDatabase::open(options).await?);
    ///     engine.close().await
    /// }
    /// ```
    pub async fn open(options: MysqlOptions) -> Result<Self> {
        options.validate()?;
        let connect = options.connect_options()?;
        // Configure the pool synchronously; all handshake/session IO stays inside the deadline.
        let pool = MySqlPoolOptions::new()
            .max_connections(options.max_connections)
            .min_connections(0)
            .after_connect(|connection, _| Box::pin(initialize_session(connection)))
            .acquire_timeout(options.acquire_timeout());
        // The outer deadline bounds the entire handshake, not just time waiting in a pool.
        let pool = tokio::time::timeout(options.acquire_timeout(), pool.connect_with(connect))
            .await
            .map_err(|cause| {
                SqlError::with_source(
                    ErrorKind::PoolTimedOut,
                    "MySQL open deadline exceeded",
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
    /// Test-only scalar resource snapshot; no native pool handle leaves its owner.
    #[cfg(test)]
    #[allow(
        dead_code,
        reason = "only the M2 executable consumes this private snapshot"
    )]
    pub(crate) fn pool_state(&self) -> (u32, usize, usize, usize) {
        (
            self.pool.size(),
            self.pool.num_idle(),
            self.cleanup.pending(),
            self.admission.available_permits(),
        )
    }
    /// Bound pending work immediately; no unbounded admission queue is created.
    fn admit(&self) -> Result<OwnedSemaphorePermit> {
        self.admission
            .clone()
            .try_acquire_owned()
            .map_err(|cause| match cause {
                TryAcquireError::Closed => {
                    SqlError::new(ErrorKind::Closed, "MySQL database is closed")
                }
                TryAcquireError::NoPermits => {
                    SqlError::new(ErrorKind::Limit, "MySQL admission limit exceeded")
                }
            })
    }
    /// Publish coupled ownership only after a bounded native acquisition completes.
    pub(crate) async fn lease(&self) -> Result<Lease> {
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
impl Database for MysqlDatabase {
    /// Select exactly the backend's renderer and native syntax.
    fn dialect(&self) -> DialectId {
        DialectId::Mysql
    }
    /// Execute on one owned lease; acknowledge reset or retirement before returning.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            // Direct trait users receive the same pre-effect validation as Engine users.
            execute::validate(&query, limits)?;
            let reusable = reuse::allowed(&query);
            let mut lease = self.lease().await?;
            let result =
                execute::timed_run(lease.connection()?, query, limits, self.statement_timeout)
                    .await;
            lease.finish(result, reusable).await
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
            Ok(Box::new(MysqlSession::new(lease, self.statement_timeout))
                as Box<dyn TransactionSession>)
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

/// Fix only this new connection's quoting/strictness policy before SQLx publishes it.
async fn initialize_session(
    connection: &mut MySqlConnection,
) -> std::result::Result<(), sqlx::Error> {
    // Keep this exact session command inside open's handshake deadline; never SET GLOBAL.
    connection
        .execute("SET SESSION sql_mode = 'STRICT_TRANS_TABLES,NO_ENGINE_SUBSTITUTION,NO_BACKSLASH_ESCAPES,PIPES_AS_CONCAT'")
        .await?;
    Ok(())
}
