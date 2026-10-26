// Recording adapter for the runnable examples and doctests.
//
// This fixture records transaction ownership and statement order. It does not parse
// SQL, store tables, or demonstrate native database behavior.

use std::sync::{Arc, Mutex};

use stackpress_sql::{
    Database, DialectId, Query, QueryLimits, QueryResult, Row, SqlFuture, TransactionOptions,
    TransactionSession,
};

/// Shared observations, kept outside the engine so examples can inspect completion.
#[derive(Default)]
pub struct State {
    /// Statements published by ordinary runs or successful commits.
    pub committed: Vec<Query>,
    /// Uncommitted sessions explicitly rolled back or dropped.
    pub rollbacks: usize,
}

/// A SQLite-identity recorder; use a native adapter for actual storage.
#[derive(Clone, Default)]
pub struct RecordingDatabase {
    /// Shared statement history and rollback count.
    pub state: Arc<Mutex<State>>,
}

/// One exclusive recorder session, publishing staged statements only on commit.
struct Session {
    /// Shared observation history; locked only for synchronous updates.
    state: Arc<Mutex<State>>,
    /// Statements staged exclusively by this session.
    pending: Vec<Query>,
    /// Set after commit/rollback so Drop cannot count completion twice.
    completed: bool,
}

/// Return a fixed owned result so row decoding can be demonstrated without a server.
fn outcome() -> QueryResult {
    QueryResult {
        rows: vec![Row::new(vec![("id".into(), 7_i64.into())])],
        affected: 1,
        insert_id: Some(7),
    }
}

impl Database for RecordingDatabase {
    /// Advertise the example renderer identity without claiming native SQL execution.
    fn dialect(&self) -> DialectId {
        DialectId::Sqlite
    }

    /// Publish an ordinary validated statement immediately.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            query.validate(limits)?;
            self.state.lock().unwrap().committed.push(query);
            Ok(outcome())
        })
    }

    /// Create one exclusive staging area for pending statements.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            Ok(Box::new(Session {
                state: Arc::clone(&self.state),
                pending: Vec::new(),
                completed: false,
            }) as Box<dyn TransactionSession>)
        })
    }

    /// Complete without waiting because the recorder holds no native resources.
    fn close(&self) -> SqlFuture<'_, ()> {
        // There are no native leases to drain in this recording-only fixture.
        Box::pin(async { Ok(()) })
    }
}

impl TransactionSession for Session {
    /// Retain a validated statement until explicit session completion.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            query.validate(limits)?;
            self.pending.push(query);
            Ok(outcome())
        })
    }

    /// Publish staged statements in order, consuming the session.
    fn commit(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            self.state
                .lock()
                .unwrap()
                .committed
                .append(&mut self.pending);
            self.completed = true;
            Ok(())
        })
    }

    /// Record explicit rollback; Drop then discards pending statements.
    fn rollback(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async move {
            self.state.lock().unwrap().rollbacks += 1;
            self.completed = true;
            Ok(())
        })
    }
}

impl Drop for Session {
    /// Discard unfinished work after an early return or cancellation.
    fn drop(&mut self) {
        // Cancellation discards pending work; never publish it as committed history.
        if !self.completed {
            self.state.lock().unwrap().rollbacks += 1;
        }
    }
}
