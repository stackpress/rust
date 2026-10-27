//! Core executor-policy contracts only. Native IO/affinity/cancellation are proved by
//! the SQLite adapter's independent tests, not by this immediate-future test driver.

use stackpress_sql::*;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// Observable calls at the core/driver boundary.
#[derive(Default)]
struct Calls {
    /// Ordered lifecycle calls issued by Engine/Transaction.
    events: Vec<&'static str>,
    /// Actual SQL/parameter data and policy forwarded to run.
    queries: Vec<(Query, QueryLimits)>,
    /// Force statement failure for primary-error/cleanup tests.
    fail_run: bool,
    /// Force rollback failure without replacing the primary error.
    fail_rollback: bool,
    /// Statement ordinal at which to fail, for ordered plan tests.
    fail_at: Option<usize>,
    /// Statement ordinal whose future remains pending until the owner is cancelled.
    pending_at: Option<usize>,
    /// Durable fake state, populated only by an explicit session commit.
    committed: Vec<String>,
    /// Leases currently owned by published sessions.
    active_sessions: usize,
}

/// Runtime-free immediate driver for inspecting only the core forwarding boundary.
struct Driver(Arc<Mutex<Calls>>);

impl Database for Driver {
    /// Select the only M1 renderer.
    fn dialect(&self) -> DialectId {
        DialectId::Sqlite
    }
    /// Record a query and return owned row data or a typed failure.
    fn run(&self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.0.lock().unwrap().events.push("database-run");
        run(&self.0, query, limits)
    }
    /// Publish an owned exclusive test session, recording acquisition explicitly.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        let mut calls = self.0.lock().unwrap();
        calls.events.push("begin");
        calls.active_sessions += 1;
        Box::pin(std::future::ready(Ok(Box::new(Session {
            calls: self.0.clone(),
            completed: false,
            staged: Vec::new(),
        })
            as Box<dyn TransactionSession>)))
    }
    /// Record explicit close without any runtime dependency.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.0.lock().unwrap().events.push("close");
        Box::pin(std::future::ready(Ok(())))
    }
}

/// Test-only exclusive driver session. Native rollback-on-drop is adapter-owned.
struct Session {
    /// Shared committed state and observable test driver calls.
    calls: Arc<Mutex<Calls>>,
    /// Completion consumes the session and prevents a second rollback during Drop.
    completed: bool,
    /// Successful session statements become durable only if commit moves them to Calls.
    staged: Vec<String>,
}

impl Drop for Session {
    /// Model the driver contract so cancellation proves the core drops its owned session.
    fn drop(&mut self) {
        if !self.completed {
            let mut calls = self.calls.lock().unwrap();
            calls.events.push("drop-rollback");
            calls.active_sessions -= 1;
            self.staged.clear();
        }
    }
}

impl TransactionSession for Session {
    /// Observe transaction-local forwarding independently from shared run.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.calls.lock().unwrap().events.push("session-run");
        let sql = query.sql().to_owned();
        let future = run(&self.calls, query, limits);
        Box::pin(async move {
            let result = future.await?;
            self.staged.push(sql);
            Ok(result)
        })
    }
    /// Observe consuming completion.
    fn commit(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        self.completed = true;
        let mut calls = self.calls.lock().unwrap();
        calls.events.push("commit");
        calls.active_sessions -= 1;
        calls.committed.append(&mut self.staged);
        Box::pin(std::future::ready(Ok(())))
    }
    /// Return a secondary cleanup failure only when explicitly configured.
    fn rollback(mut self: Box<Self>) -> SqlFuture<'static, ()> {
        self.completed = true;
        let mut calls = self.calls.lock().unwrap();
        calls.events.push("rollback");
        calls.active_sessions -= 1;
        self.staged.clear();
        let result = if calls.fail_rollback {
            Err(SqlError::new(ErrorKind::Closed, "test cleanup failed"))
        } else {
            Ok(())
        };
        Box::pin(std::future::ready(result))
    }
}

/// Capture policy/input and manufacture only a boundary-test outcome, never native evidence.
fn run(
    calls: &Arc<Mutex<Calls>>,
    query: Query,
    limits: QueryLimits,
) -> SqlFuture<'static, QueryResult> {
    let mut calls = calls.lock().unwrap();
    calls.queries.push((query, limits));
    if calls.pending_at == Some(calls.queries.len()) {
        return Box::pin(std::future::pending());
    }
    let result = if calls.fail_run || calls.fail_at == Some(calls.queries.len()) {
        Err(SqlError::with_source(
            ErrorKind::Driver,
            "test statement failed",
            std::io::Error::other("primary native source"),
        ))
    } else {
        Ok(QueryResult {
            rows: vec![Row::new(vec![("id".into(), 7.into())])],
            affected: calls.queries.len() as u64,
            insert_id: Some(7),
        })
    };
    Box::pin(std::future::ready(result))
}

/// Poll a deliberately immediate test future once; no unsafe executor or timer is needed.
fn complete<T>(future: impl Future<Output = T>) -> T {
    let mut future = std::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("test driver unexpectedly awaited runtime work"),
    }
}

/// Rendering and limits must reject bad input before invoking any driver work.
#[test]
fn engine_validates_before_driver_and_forwards_owned_results() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone())).with_limits(QueryLimits {
        binds: 1,
        rows: 2,
        bytes: 20,
        ..QueryLimits::default()
    });
    assert_eq!(engine.dialect(), DialectId::Sqlite);
    assert!(complete(engine.execute(&Delete::new("p"))).is_err());
    assert!(
        complete(engine.execute(&Insert::new("p").columns(["x", "y"]).values([1, 2]))).is_err()
    );
    assert!(calls.lock().unwrap().events.is_empty());
    let result = complete(engine.execute(&Select::default().from("p").where_eq("id", 7))).unwrap();
    assert_eq!(result.affected, 1);
    assert_eq!(result.insert_id, Some(7));
    assert_eq!(result.rows[0].try_get::<i64>("id").unwrap(), 7);
    let rows = complete(engine.fetch(&Select::default().from("p"))).unwrap();
    assert_eq!(rows[0].try_get::<i64>("id").unwrap(), 7);
    complete(engine.close()).unwrap();
    let calls = calls.lock().unwrap();
    assert_eq!(calls.events, ["database-run", "database-run", "close"]);
    assert_eq!(calls.queries[0].0.binds(), &[7.into()]);
    assert_eq!(calls.queries[0].1.rows, 2);
    assert_eq!(calls.queries[0].1.bytes, 20);
}

/// Transactions receive the frozen policy and forward every query to the owned session.
#[test]
fn transaction_forwarding_completion_and_error_preservation() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone()));
    let mut transaction = complete(engine.begin()).unwrap();
    assert!(complete(transaction.execute(&Delete::new("p"))).is_err());
    assert_eq!(
        complete(transaction.fetch(&Select::default().from("p"))).unwrap()[0]
            .try_get_at::<i64>(0)
            .unwrap(),
        7
    );
    complete(transaction.commit()).unwrap();
    let transaction = complete(engine.begin()).unwrap();
    complete(transaction.rollback()).unwrap();
    let primary = SqlError::with_source(
        ErrorKind::Driver,
        "primary",
        std::io::Error::other("native primary"),
    );
    let transaction = complete(engine.begin()).unwrap();
    let error = complete(transaction.rollback_after(primary));
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert!(error.cleanup_error().is_none());
    calls.lock().unwrap().fail_rollback = true;
    let transaction = complete(engine.begin()).unwrap();
    let error = complete(transaction.rollback_after(SqlError::with_source(
        ErrorKind::Driver,
        "primary",
        std::io::Error::other("native primary"),
    )));
    assert_eq!(error.kind(), ErrorKind::Driver);
    assert_eq!(error.to_string(), "primary");
    assert_eq!(
        std::error::Error::source(&error).unwrap().to_string(),
        "native primary"
    );
    assert_eq!(error.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    assert_eq!(
        calls.lock().unwrap().events,
        [
            "begin",
            "session-run",
            "commit",
            "begin",
            "rollback",
            "begin",
            "rollback",
            "begin",
            "rollback"
        ]
    );
}

/// Errors remain failures instead of becoming empty rows or stale success metadata.
#[test]
fn driver_failure_is_not_swallowed() {
    let calls = Arc::new(Mutex::new(Calls {
        fail_run: true,
        ..Calls::default()
    }));
    let engine = Engine::new(Driver(calls));
    assert_eq!(
        complete(engine.fetch(&Select::default().from("p")))
            .unwrap_err()
            .kind(),
        ErrorKind::Driver
    );
    let mut transaction = complete(engine.begin()).unwrap();
    assert_eq!(
        complete(transaction.execute(&Select::default().from("p")))
            .unwrap_err()
            .kind(),
        ErrorKind::Driver
    );
}

/// All six Engine convenience methods return detached data without touching the driver.
#[test]
fn engine_conveniences_never_attach_or_execute() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone()));
    assert!(
        engine
            .select(["id"])
            .from("p")
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(
        engine
            .insert("p")
            .columns(["id"])
            .values([1])
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(
        engine
            .update("p")
            .set("id", 1)
            .all_rows()
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(
        engine
            .delete("p")
            .all_rows()
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(
        engine
            .create("p")
            .add_field(Field::new("id", SqlType::Integer))
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(
        engine
            .alter("p")
            .remove_field("id")
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(engine.drop_table("p").render(DialectId::Sqlite).is_ok());
    assert!(
        engine
            .rename_table("p", "q")
            .render(DialectId::Sqlite)
            .is_ok()
    );
    assert!(engine.truncate("p").render(DialectId::Sqlite).is_ok());
    assert!(calls.lock().unwrap().events.is_empty());
}

/// Three ordered DDL statements make failure at N and outcome order observable.
fn three_statement_plan(dialect: DialectId) -> QueryPlan {
    Create::new("plan")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("one", ["id"]))
        .add_index(Index::new("two", ["id"]).unique())
        .render(dialect)
        .unwrap()
}

/// The engine acquires once, returns each owned outcome in order and commits only at the end.
#[test]
fn engine_plan_success_and_caller_owned_transaction() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone()));
    let plan = three_statement_plan(DialectId::Sqlite);
    let results = complete(engine.execute_plan(&plan)).unwrap();
    assert_eq!(
        results
            .iter()
            .map(|result| result.affected)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        calls.lock().unwrap().committed,
        plan.queries()
            .iter()
            .map(|query| query.sql().to_owned())
            .collect::<Vec<_>>()
    );
    assert_eq!(calls.lock().unwrap().active_sessions, 0);
    assert_eq!(
        calls.lock().unwrap().events,
        [
            "begin",
            "session-run",
            "session-run",
            "session-run",
            "commit"
        ]
    );
    let mut transaction = complete(engine.begin()).unwrap();
    assert_eq!(complete(transaction.execute_plan(&plan)).unwrap().len(), 3);
    assert_eq!(calls.lock().unwrap().events.last(), Some(&"session-run"));
    complete(transaction.rollback()).unwrap();
    assert_eq!(calls.lock().unwrap().events.last(), Some(&"rollback"));
}

/// Failure stops at N, rolls back and preserves primary plus optional secondary cleanup errors.
#[test]
fn engine_plan_failure_preserves_original_and_cleanup_errors() {
    for fail_rollback in [false, true] {
        let calls = Arc::new(Mutex::new(Calls {
            fail_at: Some(2),
            fail_rollback,
            ..Calls::default()
        }));
        let engine = Engine::new(Driver(calls.clone()));
        let error =
            complete(engine.execute_plan(&three_statement_plan(DialectId::Sqlite))).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::Driver);
        assert_eq!(
            std::error::Error::source(&error).unwrap().to_string(),
            "primary native source"
        );
        assert_eq!(error.cleanup_error().is_some(), fail_rollback);
        if let Some(cleanup) = error.cleanup_error() {
            assert_eq!(cleanup.kind(), ErrorKind::Closed);
        }
        let calls = calls.lock().unwrap();
        assert_eq!(calls.queries.len(), 2);
        assert!(calls.committed.is_empty());
        assert_eq!(calls.active_sessions, 0);
        assert_eq!(
            calls.events,
            ["begin", "session-run", "session-run", "rollback"]
        );
    }
}

/// Dialect/budget rejection precedes acquisition; cancellation drops the actual owned session.
#[test]
fn engine_plan_validation_and_cancelled_owner() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let engine = Engine::new(Driver(calls.clone()));
    assert_eq!(
        complete(engine.execute_plan(&three_statement_plan(DialectId::Postgres)))
            .unwrap_err()
            .kind(),
        ErrorKind::Unsupported
    );
    assert!(calls.lock().unwrap().events.is_empty());
    let limited = Engine::new(Driver(calls.clone())).with_limits(QueryLimits {
        sql_bytes: 5,
        ..QueryLimits::default()
    });
    assert_eq!(
        complete(limited.execute_plan(&three_statement_plan(DialectId::Sqlite)))
            .unwrap_err()
            .kind(),
        ErrorKind::Limit
    );
    assert!(calls.lock().unwrap().events.is_empty());
    calls.lock().unwrap().pending_at = Some(2);
    let plan = three_statement_plan(DialectId::Sqlite);
    let mut future = Box::pin(engine.execute_plan(&plan));
    let mut context = Context::from_waker(Waker::noop());
    assert!(future.as_mut().poll(&mut context).is_pending());
    assert_eq!(calls.lock().unwrap().active_sessions, 1);
    assert!(calls.lock().unwrap().committed.is_empty());
    drop(future);
    let calls = calls.lock().unwrap();
    assert_eq!(calls.active_sessions, 0);
    assert!(calls.committed.is_empty());
    assert_eq!(
        calls.events,
        ["begin", "session-run", "session-run", "drop-rollback"]
    );
    assert_eq!(calls.queries.len(), 2);
}

/// Failure or cancellation of a borrowed batch retains the caller's session until rollback.
#[test]
fn caller_owned_plan_retains_guard_after_failure_or_cancellation() {
    for cancelled in [false, true] {
        let calls = Arc::new(Mutex::new(Calls {
            fail_at: (!cancelled).then_some(2),
            pending_at: cancelled.then_some(2),
            ..Calls::default()
        }));
        let engine = Engine::new(Driver(calls.clone()));
        let mut transaction = complete(engine.begin()).unwrap();
        let plan = three_statement_plan(DialectId::Sqlite);
        if cancelled {
            let mut future = Box::pin(transaction.execute_plan(&plan));
            assert!(
                future
                    .as_mut()
                    .poll(&mut Context::from_waker(Waker::noop()))
                    .is_pending()
            );
            drop(future);
            assert_eq!(calls.lock().unwrap().active_sessions, 1);
            complete(transaction.rollback()).unwrap();
        } else {
            let primary = complete(transaction.execute_plan(&plan)).unwrap_err();
            assert_eq!(calls.lock().unwrap().active_sessions, 1);
            let original = complete(transaction.rollback_after(primary));
            assert_eq!(original.kind(), ErrorKind::Driver);
            assert_eq!(
                std::error::Error::source(&original).unwrap().to_string(),
                "primary native source"
            );
        }
        let calls = calls.lock().unwrap();
        assert_eq!(calls.queries.len(), 2);
        assert_eq!(calls.active_sessions, 0);
        assert!(calls.committed.is_empty());
        assert_eq!(
            calls.events,
            ["begin", "session-run", "session-run", "rollback"]
        );
    }
}

/// Budget inspection is a copy-only operation and transactions retain their original policy.
#[test]
fn limits_are_copied_frozen_and_require_no_driver_call() {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let original = QueryLimits {
        sql_bytes: 4000,
        binds: 2,
        rows: 30,
        bytes: 8000,
    };
    let engine = Engine::new(Driver(calls.clone())).with_limits(original);
    let mut copy = engine.limits();
    assert_eq!(
        (copy.sql_bytes, copy.binds, copy.rows, copy.bytes),
        (4000, 2, 30, 8000)
    );
    copy.binds = 999;
    assert_eq!(engine.limits().binds, 2);
    assert_eq!(copy.binds, 999);
    assert!(calls.lock().unwrap().events.is_empty());
    let transaction = complete(engine.begin()).unwrap();
    let before = calls.lock().unwrap().events.clone();
    let engine = engine.with_limits(QueryLimits::default());
    let frozen = transaction.limits();
    assert_eq!(
        (frozen.sql_bytes, frozen.binds, frozen.rows, frozen.bytes),
        (4000, 2, 30, 8000)
    );
    let defaults = engine.limits();
    assert_eq!(
        (
            defaults.sql_bytes,
            defaults.binds,
            defaults.rows,
            defaults.bytes
        ),
        (1024 * 1024, 1000, 10000, 16 * 1024 * 1024)
    );
    assert_eq!(calls.lock().unwrap().events, before);
    complete(transaction.rollback()).unwrap();
}
