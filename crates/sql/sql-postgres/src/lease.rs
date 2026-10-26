//! Lease retirement owns both admission and native capacity until transport disposal.

use crate::execute::driver_error;
use sqlx::pool::PoolConnection;
use sqlx::{PgConnection, Postgres};
use stackpress_sql::{ErrorKind, Result, SqlError};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;
use tokio::sync::{Notify, OwnedSemaphorePermit};

/// Counts cleanup tasks even before first poll; retains the first asynchronous failure.
#[derive(Default)]
pub(crate) struct Cleanup {
    /// Incremented before spawning and decremented by a cancellation-safe guard.
    active: AtomicUsize,
    /// Wakes every close observer after a retirement finishes or is aborted.
    changed: Notify,
    /// Bounded storage: repeated failures never grow a background error queue.
    failure: Mutex<Option<SqlError>>,
}
impl Cleanup {
    /// Wait until every registered retirement has disposed of its transport.
    pub(crate) async fn wait(&self) -> Result<()> {
        loop {
            // Register before inspecting the count to avoid a missed completion wakeup.
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.active.load(Ordering::SeqCst) == 0 {
                break;
            }
            changed.await;
        }
        // Return the first background failure once; an interrupted close loses no error.
        let mut failure = self
            .failure
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        failure.take().map_or(Ok(()), Err)
    }
    /// Record one failure without replacing the first cleanup diagnosis.
    fn record(&self, error: SqlError) {
        let mut failure = self
            .failure
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if failure.is_none() {
            *failure = Some(error);
        }
    }
}
/// A registered cleanup remains visible even if its spawned future is never polled.
struct CleanupGuard(Arc<Cleanup>);
impl Drop for CleanupGuard {
    /// Release the tracked task after captured socket/permit ownership is disposed.
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
        self.0.changed.notify_waiters();
    }
}
/// Exclusive connection ownership; only acknowledged, reset leases may return to their pool.
pub(crate) struct Lease {
    /// Taken only by explicit completion or Drop; never returned uncertain to the pool.
    parts: Option<(PoolConnection<Postgres>, OwnedSemaphorePermit)>,
    /// Finite deadline for native close; dropping a timed-out close drops the socket.
    timeout: Duration,
    /// Close observers must include cleanup tasks whose pool slot was already released.
    cleanup: Arc<Cleanup>,
}
impl Lease {
    /// Couple both permits and arm conservative retirement before any application IO.
    pub(crate) fn new(
        connection: PoolConnection<Postgres>,
        permit: OwnedSemaphorePermit,
        timeout: Duration,
        cleanup: Arc<Cleanup>,
    ) -> Self {
        // Lease owns the only release path. Keep SQLx's flag clear until retirement:
        // its flagged Drop always spawns, even after detach, which would panic outside
        // a runtime. Lease Drop below prevents every uncertain return-to-pool path.
        Self {
            parts: Some((connection, permit)),
            timeout,
            cleanup,
        }
    }
    /// Borrow the one native connection; transaction calls retain exclusive affinity.
    pub(crate) fn connection(&mut self) -> Result<&mut PgConnection> {
        self.parts
            .as_mut()
            .map(|(connection, _)| &mut **connection)
            .ok_or_else(|| SqlError::new(ErrorKind::Closed, "PostgreSQL lease has completed"))
    }
    /// Recycle only after full acknowledgement and backend-specific reset complete.
    pub(crate) async fn finish<T>(mut self, primary: Result<T>, reusable: bool) -> Result<T> {
        if primary.is_ok() && reusable {
            // Keep the lease intact across reset; any cancellation still runs retiring Drop.
            let timeout = self.timeout;
            let reset = tokio::time::timeout(timeout, async {
                #[cfg(test)]
                crate::probe::at(crate::probe::Phase::BeforeReuse).await?;
                crate::reuse::reset(self.connection()?).await?;
                #[cfg(test)]
                crate::probe::at(crate::probe::Phase::AfterReset).await?;
                Ok(())
            })
            .await
            .map_err(|cause| {
                SqlError::with_source(
                    ErrorKind::Driver,
                    "database reset deadline exceeded; outcome uncertain",
                    cause,
                )
            })
            .and_then(|result| result);
            if let Err(primary) = reset {
                return completed(Err(primary), self.retire().await);
            }
            // Only this acknowledged/reset path can invoke SQLx's normal return-to-pool Drop.
            if let Some((connection, permit)) = self.parts.take() {
                drop(connection);
                drop(permit);
            }
            primary
        } else {
            completed(primary, self.retire().await)
        }
    }
    /// Retire explicitly so a caller can retain primary and cleanup failures separately.
    pub(crate) async fn retire(mut self) -> Result<()> {
        let Some((connection, permit)) = self.parts.take() else {
            return Ok(());
        };
        // The close future owns the native capacity guard; timeout drops that future/socket.
        let result = close(connection, self.timeout).await;
        drop(permit);
        result
    }
}
impl Drop for Lease {
    /// Transfer uncertain IO to tracked bounded cleanup; outside a runtime, dispose by RAII.
    fn drop(&mut self) {
        let Some((mut connection, permit)) = self.parts.take() else {
            return;
        };
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            // Arm driver retirement before transferring uncertain native IO to a task.
            connection.close_on_drop();
            // Register before spawn, including tasks aborted before their first poll.
            self.cleanup.active.fetch_add(1, Ordering::SeqCst);
            let guard = CleanupGuard(self.cleanup.clone());
            let timeout = self.timeout;
            runtime.spawn(async move {
                // Keep admission coupled until close acknowledges or forced disposal occurs.
                if let Err(error) = close(connection, timeout).await {
                    guard.0.record(error);
                }
                drop(permit);
                drop(guard);
            });
        } else {
            // SQLx detach removes only this checked-out connection. min_connections=0
            // prevents replacement tasks; dropping the raw TCP socket needs no runtime.
            // This is disposal, never pool reuse or acknowledged remote rollback.
            drop(connection.detach());
            drop(permit);
        }
    }
}
/// Bound graceful close while SQLx retains its pool-capacity guard through the await.
async fn close(mut connection: PoolConnection<Postgres>, timeout: Duration) -> Result<()> {
    // The flag protects a close future dropped before it first polls/takes the socket.
    connection.close_on_drop();
    tokio::time::timeout(timeout, async move {
        // Poll the consuming SQLx close once before a private test can pause. SQLx has
        // transferred the raw socket and capacity guard out of PoolConnection by then;
        // dropping a pending close therefore disposes the socket instead of recycling it.
        let mut closing = Box::pin(connection.close());
        let first =
            std::future::poll_fn(|context| std::task::Poll::Ready(closing.as_mut().poll(context)))
                .await;
        #[cfg(test)]
        crate::probe::at(crate::probe::Phase::BeforeClose).await?;
        match first {
            std::task::Poll::Ready(result) => result.map_err(driver_error),
            std::task::Poll::Pending => closing.await.map_err(driver_error),
        }
    })
    .await
    .map_err(|cause| {
        SqlError::with_source(
            ErrorKind::Driver,
            "Postgres cleanup deadline exceeded; remote outcome uncertain",
            cause,
        )
    })?
}
/// Keep an operation's primary failure distinct from transport cleanup failure.
pub(crate) fn completed<T>(primary: Result<T>, cleanup: Result<()>) -> Result<T> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Ok(_), Err(cleanup)) => Err(cleanup),
        (Err(primary), Ok(())) => Err(primary),
        (Err(primary), Err(cleanup)) => Err(primary.with_cleanup(cleanup)),
    }
}
