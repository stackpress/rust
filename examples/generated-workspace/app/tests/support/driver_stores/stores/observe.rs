//! Observation at the real executor seam; no query/result policy is changed.
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
    TransactionSession,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Counts every explicit executor entry, without retaining SQL, binds or driver handles.
#[derive(Default)]
pub(super) struct Calls {
    begins: AtomicUsize,
    runs: AtomicUsize,
    commits: AtomicUsize,
    rollbacks: AtomicUsize,
}
/// Stable scalar observation before/after one generated operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct Snapshot {
    pub begins: usize,
    pub runs: usize,
    pub commits: usize,
    pub rollbacks: usize,
}
impl Calls {
    /// Copy counters without resetting another call's observation.
    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            begins: self.begins.load(Ordering::SeqCst),
            runs: self.runs.load(Ordering::SeqCst),
            commits: self.commits.load(Ordering::SeqCst),
            rollbacks: self.rollbacks.load(Ordering::SeqCst),
        }
    }
}
impl Snapshot {
    /// Counters are monotonic; only calls since the selected boundary enter an assertion.
    pub fn since(self, before: Self) -> Self {
        Self {
            begins: self.begins - before.begins,
            runs: self.runs - before.runs,
            commits: self.commits - before.commits,
            rollbacks: self.rollbacks - before.rollbacks,
        }
    }
}
/// Borrowed trait owner remains behind Arc; no alternate connection can enter a store.
struct Observed {
    inner: Arc<dyn Database>,
    calls: Arc<Calls>,
}
/// Exact returned transaction is retained, with no hidden acquisition or affinity change.
struct Session {
    inner: Box<dyn TransactionSession>,
    calls: Arc<Calls>,
}
/// Wrap the actual owner while keeping all observations outside production interfaces.
pub(super) fn engine(database: Arc<dyn Database>) -> (Engine, Arc<Calls>) {
    let calls = Arc::new(Calls::default());
    (
        Engine::new(Observed {
            inner: database,
            calls: calls.clone(),
        }),
        calls,
    )
}
impl Database for Observed {
    /// Preserve the selected renderer identity.
    fn dialect(&self) -> DialectId {
        self.inner.dialect()
    }
    /// Record entry without copying any sensitive statement data.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.calls.runs.fetch_add(1, Ordering::SeqCst);
        self.inner.run(query, limits)
    }
    /// Wrap the exact acknowledged session; do not acquire an alternate connection.
    fn begin(&self, options: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        self.calls.begins.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            Ok(Box::new(Session {
                inner: self.inner.begin(options).await?,
                calls: self.calls.clone(),
            }) as Box<dyn TransactionSession>)
        })
    }
    /// Preserve owner close semantics; close is outside per-operation preflight assertions.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.inner.close()
    }
}
impl TransactionSession for Session {
    /// Forward the same exclusive borrow across the native await.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.calls.runs.fetch_add(1, Ordering::SeqCst);
        self.inner.run(query, limits)
    }
    /// Consuming completion retains native ownership and acknowledgement semantics.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        let Self { inner, calls } = *self;
        calls.commits.fetch_add(1, Ordering::SeqCst);
        inner.commit()
    }
    /// Count explicit cleanup without replacing its result or primary error.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        let Self { inner, calls } = *self;
        calls.rollbacks.fetch_add(1, Ordering::SeqCst);
        inner.rollback()
    }
}
