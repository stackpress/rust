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
        self.0.lock().unwrap().events.push("begin");
        Box::pin(std::future::ready(Ok(
            Box::new(Session(self.0.clone())) as Box<dyn TransactionSession>
        )))
    }
    /// Record explicit close without any runtime dependency.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.0.lock().unwrap().events.push("close");
        Box::pin(std::future::ready(Ok(())))
    }
}

/// Test-only exclusive driver session. Native rollback-on-drop is adapter-owned.
struct Session(Arc<Mutex<Calls>>);

impl TransactionSession for Session {
    /// Observe transaction-local forwarding independently from shared run.
    fn run(&mut self, query: Query, limits: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.0.lock().unwrap().events.push("session-run");
        run(&self.0, query, limits)
    }
    /// Observe consuming completion.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.0.lock().unwrap().events.push("commit");
        Box::pin(std::future::ready(Ok(())))
    }
    /// Return a secondary cleanup failure only when explicitly configured.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        let mut calls = self.0.lock().unwrap();
        calls.events.push("rollback");
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
    let result = if calls.fail_run {
        Err(SqlError::new(ErrorKind::Driver, "test statement failed"))
    } else {
        Ok(QueryResult {
            rows: vec![Row::new(vec![("id".into(), 7.into())])],
            affected: 1,
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
