//! Invalid plan data must still be rejected before an executor can acquire a lease.
use super::*;
use crate::{
    Database, Engine, QueryResult, Render, Sql, SqlFuture, TransactionOptions, TransactionSession,
};
use std::future::Future;
use std::task::{Context, Poll, Waker};
/// Any adapter interaction makes this negative control fail immediately.
struct NoIo(DialectId);
impl Database for NoIo {
    /// Select SQLite for the upfront mismatch check.
    fn dialect(&self) -> DialectId {
        self.0
    }
    /// Invalid plans must never reach execution.
    fn run(&self, _: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        panic!("unexpected run")
    }
    /// Invalid plans must never acquire a transaction.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        panic!("unexpected begin")
    }
    /// No close is needed for this zero-resource test adapter.
    fn close(&self) -> SqlFuture<'_, ()> {
        panic!("unexpected close")
    }
}
/// Private-field corruption proves defense in depth even though constructors reject mixing.
#[test]
fn mixed_plan_rejected_before_acquisition() {
    let first = Sql::literal(DialectId::Sqlite, "CREATE TABLE a(id INTEGER)")
        .render(DialectId::Sqlite)
        .unwrap();
    let second = Sql::literal(DialectId::Postgres, "CREATE TABLE b(id BIGINT)")
        .render(DialectId::Postgres)
        .unwrap();
    assert!(QueryPlan::new(vec![first.clone(), second.clone()]).is_err());
    let corrupt = QueryPlan {
        queries: vec![first, second],
    };
    let engine = Engine::new(NoIo(DialectId::Sqlite));
    let mut future = std::pin::pin!(engine.execute_plan(&corrupt));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(_))
    ));
}

/// Empty plans and nontransactional MySQL DDL never acquire a native session.
#[test]
fn empty_plan_is_not_executable() {
    assert!(QueryPlan::new(vec![]).is_err());
    let corrupt = QueryPlan { queries: vec![] };
    let engine = Engine::new(NoIo(DialectId::Sqlite));
    let mut future = std::pin::pin!(engine.execute_plan(&corrupt));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(_))
    ));
}
/// MySQL's implicit DDL commits must not pass through the atomic plan entry point.
#[test]
fn mysql_atomic_plan_rejected_before_acquisition() {
    let plan = Create::new("p")
        .add_field(Field::new("id", crate::SqlType::Integer))
        .render(DialectId::Mysql)
        .unwrap();
    let engine = Engine::new(NoIo(DialectId::Mysql));
    let mut future = std::pin::pin!(engine.execute_plan(&plan));
    assert!(matches!(
        future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Ready(Err(_))
    ));
}
