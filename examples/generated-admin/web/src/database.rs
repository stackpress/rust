//! Real SQLite lifecycle and an explicit finite-proof lost-acknowledgement adapter.
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
    TransactionSession,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

/// One native database; a fault is armed only by the proof's trusted process input.
struct Observed {
    inner: SqliteDatabase,
    fault: Arc<AtomicBool>,
}
impl Database for Observed {
    /// Preserve the actual adapter dialect.
    fn dialect(&self) -> DialectId {
        self.inner.dialect()
    }
    /// Report uncertainty only after a real autocommit write completed at SQLite.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        let write = query.sql().starts_with("INSERT")
            || query.sql().starts_with("UPDATE")
            || query.sql().starts_with("DELETE");
        let fault = self.fault.clone();
        let future = self.inner.run(query, limits);
        Box::pin(async move {
            let result = future.await?;
            if write && fault.swap(false, Ordering::SeqCst) {
                return Err(stackpress_sql::SqlError::with_source(
                    stackpress_sql::ErrorKind::Driver,
                    "acknowledgement lost",
                    LostAck,
                ));
            }
            Ok(result)
        })
    }
    /// Every transaction retains the exact SQLite executor and its real commit/rollback behavior.
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        let future = self.inner.begin(options);
        let fault = self.fault.clone();
        Box::pin(async move {
            Ok(Box::new(ObservedSession {
                inner: future.await?,
                fault,
                wrote: false,
            }) as Box<dyn TransactionSession>)
        })
    }
    /// Release every pool resource explicitly.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.inner.close()
    }
}

/// Preserve the actual SQLite session; inject uncertainty only after its durable commit.
struct ObservedSession {
    inner: Box<dyn TransactionSession>,
    fault: Arc<AtomicBool>,
    wrote: bool,
}
impl TransactionSession for ObservedSession {
    /// Delegate plans unchanged, recording whether this transaction attempted a mutation.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.wrote |= query.sql().starts_with("INSERT")
            || query.sql().starts_with("UPDATE")
            || query.sql().starts_with("DELETE");
        self.inner.run(query, limits)
    }
    /// Await the real commit before injecting the trusted lost-acknowledgement fault.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            self.inner.commit().await?;
            if self.wrote && self.fault.swap(false, Ordering::SeqCst) {
                return Err(stackpress_sql::SqlError::with_source(
                    stackpress_sql::ErrorKind::Driver,
                    "commit acknowledgement lost",
                    LostAck,
                ));
            }
            Ok(())
        })
    }
    /// Rollback delegates unchanged and never consumes the commit-only fault.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.inner.rollback()
    }
}
/// Private typed fault; public operation errors must never disclose its source text.
#[derive(Debug)]
struct LostAck;
impl std::fmt::Display for LostAck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ADMIN_PRIVATE_LOST_ACK")
    }
}
impl std::error::Error for LostAck {}

/// Open the caller's exact SQLite file, without touching any other proof's database.
/// # Errors
/// Retains the actual adapter error; no native diagnostics are sent to the browser.
pub async fn open(path: &Path, fault: bool) -> stackpress_sql::Result<Arc<Engine>> {
    let inner = SqliteDatabase::open(SqliteOptions::file(path).create_if_missing(true)).await?;
    Ok(Arc::new(Engine::new(Observed {
        inner,
        fault: Arc::new(AtomicBool::new(fault)),
    })))
}

/// Install the exact generated table definitions into a newly owned fixture.
/// # Errors
/// Propagates generated definition/render/execution failures before initial data is submitted.
pub async fn initialize(engine: &Engine) -> stackpress_lib::Result<()> {
    let definitions = crate::generated::stores::schema::definition().map_err(host_error)?;
    for table in definitions {
        engine
            .execute_plan(
                &table
                    .create()
                    .render(engine.dialect())
                    .map_err(host_error)?,
            )
            .await
            .map_err(host_error)?;
    }
    Ok(())
}

/// Keep native causes under one operation-specific private context.
fn host_error(cause: impl std::error::Error + Send + Sync + 'static) -> stackpress_lib::Error {
    stackpress_lib::Error::new("admin fixture database initialization failed").with_cause(cause)
}

/// Open an already-created review fixture and return its one-shot acknowledgement control.
/// # Errors
/// Refuses a missing database and retains concrete SQLite failures.
#[cfg(all(feature = "review-preview", debug_assertions))]
pub async fn open_controlled(
    path: &Path,
) -> stackpress_sql::Result<(Arc<Engine>, Arc<AtomicBool>)> {
    let inner = SqliteDatabase::open(SqliteOptions::file(path).create_if_missing(false)).await?;
    let fault = Arc::new(AtomicBool::new(false));
    let engine = Arc::new(Engine::new(Observed {
        inner,
        fault: fault.clone(),
    }));
    Ok((engine, fault))
}
