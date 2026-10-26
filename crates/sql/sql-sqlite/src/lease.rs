//! Coupled work/admission ownership, with acknowledged cleanup while its runtime remains alive.

use sqlx::pool::PoolConnection;
use sqlx::{Sqlite, SqliteConnection};
use stackpress_sql::{ErrorKind, Result, SqlError};
use tokio::sync::OwnedSemaphorePermit;

/// One live pool connection and its admission slot; cancellation transfers both to cleanup.
pub(crate) struct Lease {
    /// Present until explicit normal release or Drop transfers ownership to cleanup.
    parts: Option<(PoolConnection<Sqlite>, OwnedSemaphorePermit)>,
}

impl Lease {
    /// Couple connection ownership to admission before any native command is queued.
    pub(crate) fn new(connection: PoolConnection<Sqlite>, permit: OwnedSemaphorePermit) -> Self {
        Self {
            parts: Some((connection, permit)),
        }
    }

    /// Borrow the one native connection while this lease remains active.
    pub(crate) fn connection(&mut self) -> Result<&mut SqliteConnection> {
        self.parts
            .as_mut()
            .map(|(connection, _)| &mut **connection)
            .ok_or_else(|| SqlError::new(ErrorKind::Closed, "SQLite lease has completed"))
    }

    /// Return a fully acknowledged ordinary statement's connection for reuse.
    pub(crate) fn release(mut self) {
        if let Some((connection, permit)) = self.parts.take() {
            // Application SQL is acknowledged. SQLx may return/ping the idle connection
            // asynchronously, while its own pool slot still bounds connection count.
            drop(connection);
            drop(permit);
        }
    }
}

impl Drop for Lease {
    /// Close uncertain/native-transaction state and retain admission until worker shutdown.
    fn drop(&mut self) {
        let Some((mut connection, permit)) = self.parts.take() else {
            return;
        };
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            // If runtime shutdown cancels the cleanup task, SQLx must retire rather
            // than recycle a connection whose native transaction depth is uncertain.
            connection.close_on_drop();
            runtime.spawn(async move {
                // close retains SQLx's pool permit until the worker acknowledges shutdown;
                // SQLite rolls back an open native transaction when its connection closes.
                let _ = connection.close().await;
                drop(permit);
            });
        } else {
            // Outside a live runtime only nonblocking RAII fallback is possible. It
            // initiates driver teardown, without claiming acknowledged async cleanup.
            // min_connections is fixed at zero, so detaching does not spawn a pool
            // replacement. Dropping the raw worker handle needs no Tokio context.
            drop(connection.detach());
            drop(permit);
        }
    }
}
