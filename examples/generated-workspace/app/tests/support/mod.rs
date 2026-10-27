//! Real SQLite adapter with an observation-only wrapper at the Database/TransactionSession seam.
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
    TransactionSession,
};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU8, Ordering},
};
/// Native fixture retaining its owned directory until every pool/transaction is closed.
#[allow(
    dead_code,
    reason = "one real adapter fixture shared by independent integration test binaries"
)]
pub struct Fixture {
    pub engine: Engine,
    pub calls: Arc<Mutex<Vec<(String, usize)>>>,
    pub fault: Arc<AtomicU8>,
    pub submitted: Arc<tokio::sync::Notify>,
    pub _root: tempfile::TempDir,
}
/// Record one call before delegating to the real backend without changing its policy/results.
struct Counted {
    inner: SqliteDatabase,
    calls: Arc<Mutex<Vec<(String, usize)>>>,
    fault: Arc<AtomicU8>,
    submitted: Arc<tokio::sync::Notify>,
}
/// Preserve exact native transaction affinity while observing each statement.
struct Session {
    inner: Box<dyn TransactionSession>,
    calls: Arc<Mutex<Vec<(String, usize)>>>,
    fault: Arc<AtomicU8>,
    submitted: Arc<tokio::sync::Notify>,
}
impl Database for Counted {
    fn dialect(&self) -> DialectId {
        self.inner.dialect()
    }
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.calls
            .lock()
            .unwrap()
            .push((query.sql().into(), query.binds().len()));
        let write = query.sql().starts_with("INSERT")
            || query.sql().starts_with("UPDATE")
            || query.sql().starts_with("DELETE");
        let fault = self.fault.clone();
        let submitted = self.submitted.clone();
        let future = self.inner.run(query, limits);
        Box::pin(async move {
            let result = future.await?;
            if write {
                if fault
                    .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    return Err(lost());
                }
                if fault
                    .compare_exchange(4, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    submitted.notify_one();
                    std::future::pending::<()>().await;
                }
            }
            Ok(result)
        })
    }
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            Ok(Box::new(Session {
                inner: self.inner.begin(options).await?,
                calls: self.calls.clone(),
                fault: self.fault.clone(),
                submitted: self.submitted.clone(),
            }) as Box<dyn TransactionSession>)
        })
    }
    fn close(&self) -> SqlFuture<'_, ()> {
        self.inner.close()
    }
}
impl TransactionSession for Session {
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.calls
            .lock()
            .unwrap()
            .push((query.sql().into(), query.binds().len()));
        let write = query.sql().starts_with("INSERT")
            || query.sql().starts_with("UPDATE")
            || query.sql().starts_with("DELETE");
        let fault = self.fault.clone();
        let submitted = self.submitted.clone();
        let future = self.inner.run(query, limits);
        Box::pin(async move {
            let result = future.await?;
            if write {
                if fault
                    .compare_exchange(1, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    return Err(lost());
                }
                if fault
                    .compare_exchange(4, 0, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
                {
                    submitted.notify_one();
                    std::future::pending::<()>().await;
                }
            }
            Ok(result)
        })
    }
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            self.inner.commit().await?;
            if self
                .fault
                .compare_exchange(2, 0, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Err(lost());
            }
            Ok(())
        })
    }
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            self.inner.rollback().await?;
            if self
                .fault
                .compare_exchange(3, 0, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Err(stackpress_sql::SqlError::with_source(
                    stackpress_sql::ErrorKind::Busy,
                    "rollback acknowledgement failed",
                    LostAck,
                ));
            }
            Ok(())
        })
    }
}
/// Install the actual generated table definitions into a unique file; no SQL substitute fixture.
pub async fn fixture() -> Fixture {
    let root = tempfile::tempdir_in("/private/tmp").unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let database = SqliteDatabase::open(
        SqliteOptions::file(root.path().join("fixture.sqlite")).create_if_missing(true),
    )
    .await
    .unwrap();
    let fault = Arc::new(AtomicU8::new(0));
    let submitted = Arc::new(tokio::sync::Notify::new());
    let engine = Engine::new(Counted {
        inner: database,
        calls: calls.clone(),
        fault: fault.clone(),
        submitted: submitted.clone(),
    });
    for table in generated_workspace_app::generated::stores::schema::definition().unwrap() {
        engine
            .execute_plan(&table.create().render(engine.dialect()).unwrap())
            .await
            .unwrap();
    }
    Fixture {
        engine,
        calls,
        fault,
        submitted,
        _root: root,
    }
}

/// Concrete private fault retained beneath SqlError/DataError; never a public driver text classifier.
#[derive(Debug)]
pub struct LostAck;
impl std::fmt::Display for LostAck {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PRIVATE_LOST_ACK_SENTINEL")
    }
}
impl std::error::Error for LostAck {}
/// Model a lost acknowledgement only after the real backend's effect completed successfully.
fn lost() -> stackpress_sql::SqlError {
    stackpress_sql::SqlError::with_source(
        stackpress_sql::ErrorKind::Driver,
        "write acknowledgement lost",
        LostAck,
    )
}
