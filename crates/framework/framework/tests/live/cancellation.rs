//! Suspend real DDL around acknowledgement boundaries; the shared parent kills deadlocked children.

use super::support;
use stackpress_sql::{
    Database, DialectId, Engine, ErrorKind, Query, QueryLimits, QueryResult, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use tokio::sync::Notify;

/// The provider owns no background task or lock across await. Notify is an explicit test barrier.
struct Suspended {
    inner: Arc<Engine>,
    at: usize,
    after: bool,
    calls: Arc<AtomicUsize>,
    entered: Arc<Notify>,
}
impl Database for Suspended {
    fn dialect(&self) -> DialectId {
        self.inner.dialect()
    }
    fn run(&self, query: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async move {
            let index = self.calls.fetch_add(1, Ordering::SeqCst);
            if index == self.at && !self.after {
                self.entered.notify_one();
                std::future::pending::<()>().await;
            }
            let result = self.inner.execute(&query).await?;
            if index == self.at && self.after {
                // Native DDL has completed, but the outer caller has not received its result.
                self.entered.notify_one();
                std::future::pending::<()>().await;
            }
            Ok(result)
        })
    }
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async {
            Err(SqlError::new(
                ErrorKind::Unsupported,
                "fixture only observes non-atomic execution",
            ))
        })
    }
    fn close(&self) -> SqlFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}

/// Cancellation before submission, after an acknowledged prefix and after lost acknowledgement.
pub(super) async fn prove(inner: Arc<Engine>) -> support::TestResult<()> {
    for (case, at, after) in [(0, 0, false), (1, 1, false), (2, 0, true)] {
        let calls = Arc::new(AtomicUsize::new(0));
        let entered = Arc::new(Notify::new());
        let engine = Engine::new(Suspended {
            inner: inner.clone(),
            at,
            after,
            calls: calls.clone(),
            entered: entered.clone(),
        });
        let names = [
            format!("g48_cancel_{case}_a"),
            format!("g48_cancel_{case}_b"),
        ];
        let queries: Vec<_> = names
            .iter()
            .map(|name| {
                inner
                    .create(name)
                    .add_field(stackpress_sql::Field::new(
                        "id",
                        stackpress_sql::SqlType::Integer,
                    ))
                    .render(inner.dialect())
                    .unwrap()
                    .queries()[0]
                    .clone()
            })
            .collect();
        drop(engine.execute_non_atomic(&queries));
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let mut future = Box::pin(engine.execute_non_atomic(&queries));
        tokio::select! {
            _ = entered.notified() => {},
            result = &mut future => panic!("expected owned suspension, got {result:?}"),
        }
        drop(future);
        assert_eq!(calls.load(Ordering::SeqCst), at + 1);
        assert_eq!(super::exists(&inner, &names[0]).await, at == 1 || after);
        assert!(!super::exists(&inner, &names[1]).await);
        // There is no task to outlive cancellation; retiring the wrapper never closes the app engine.
        engine.close().await?;
    }
    Ok(())
}
