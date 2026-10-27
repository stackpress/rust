//! Generated atomic refusals must never enter any executor method, even on a failed owner.
use super::{DataErrorKind, DataResult, TestResult, observe};
use generated_workspace_app::generated::stores::{
    profile::{self, Profiles},
    project::{self, Projects},
    task::{self, Tasks},
};
use stackpress_framework_schema::ValueContext;
use stackpress_sql::{
    Database, DialectId, Engine, ErrorKind, Query, QueryLimits, QueryResult, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use std::sync::Arc;

/// A native-entry sentinel, with both successful and failing BEGIN controls.
struct Sentinel {
    dialect: DialectId,
    fail_begin: bool,
}
/// Only reached by a regression; its observed completion would violate zero-entry assertions.
struct Session;
impl Database for Sentinel {
    /// Select the real dialect capability checks in the generated code.
    fn dialect(&self) -> DialectId {
        self.dialect
    }
    /// Never emulate SQL or manufacture a successful generated result.
    fn run(&self, _: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async {
            Err(SqlError::new(
                ErrorKind::Driver,
                "unexpected executor entry",
            ))
        })
    }
    /// A failing BEGIN must not mask a known pure Unsupported result.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        Box::pin(async move {
            if self.fail_begin {
                return Err(SqlError::new(ErrorKind::Driver, "unexpected BEGIN entry"));
            }
            Ok(Box::new(Session) as Box<dyn TransactionSession>)
        })
    }
    /// No transport or database was opened by this sentinel.
    fn close(&self) -> SqlFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
impl TransactionSession for Session {
    /// An unexpected read remains a failure, rather than a fake successful query.
    fn run(&mut self, _: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        Box::pin(async { Err(SqlError::new(ErrorKind::Driver, "unexpected session entry")) })
    }
    /// Observation wrappers expose this completion if preflight regresses.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async { Ok(()) })
    }
    /// Observation wrappers expose this cleanup if preflight regresses.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        Box::pin(async { Ok(()) })
    }
}
/// Check kind without Debug-formatting a native/data-layer cause on unexpected failure.
pub(super) fn refused<T>(result: DataResult<T>, kind: DataErrorKind) {
    match result {
        Ok(_) => panic!("operation unexpectedly succeeded"),
        Err(error) => assert_eq!(error.kind(), kind),
    }
}
/// Run the same atomic query envelope on each actual generated model family.
async fn search(model: usize, owner: &Engine, query: serde_json::Value) -> DataResult<()> {
    match model {
        0 => Profiles::search_atomic(
            owner,
            serde_json::from_value::<profile::Query>(query).unwrap(),
        )
        .await
        .map(|_| ()),
        1 => Projects::search_atomic(
            owner,
            serde_json::from_value::<project::Query>(query).unwrap(),
        )
        .await
        .map(|_| ()),
        2 => Tasks::search_atomic(owner, serde_json::from_value::<task::Query>(query).unwrap())
            .await
            .map(|_| ()),
        _ => unreachable!(),
    }
}
/// All typed inputs are valid; the backend capability is the only refused boundary.
async fn batch(model: usize, owner: &Engine) -> DataResult<()> {
    let context = ValueContext::default();
    match model {
        0 => Profiles::batch_atomic(
            owner,
            vec![profile::BatchInput::Create {
                input: serde_json::from_value(serde_json::json!({"id":"u","name":"Owner"}))
                    .unwrap(),
            }],
            &context,
        )
        .await
        .map(|_| ()),
        1 => Projects::batch_atomic(
            owner,
            vec![project::BatchInput::Create {
                input: serde_json::from_value(
                    serde_json::json!({"id":"p","ownerId":"u","title":"Project"}),
                )
                .unwrap(),
            }],
            &context,
        )
        .await
        .map(|_| ()),
        2 => Tasks::batch_atomic(
            owner,
            vec![task::BatchInput::Create {
                input: serde_json::from_value(
                    serde_json::json!({"id":"t","projectId":"p","title":"Task"}),
                )
                .unwrap(),
            }],
            &context,
        )
        .await
        .map(|_| ()),
        _ => unreachable!(),
    }
}
/// Both sentinel modes and every generated family must stay at zero explicit executor entries.
pub(super) async fn check() -> TestResult<()> {
    for fail_begin in [false, true] {
        for dialect in [DialectId::Postgres, DialectId::Mysql] {
            for model in 0..3 {
                let path = ["projects.tasks", "tasks.project", "project.owner"][model];
                for query in [
                    serde_json::json!({"q":"needle"}),
                    serde_json::json!({"total":true}),
                    serde_json::json!({"includes":[path]}),
                ] {
                    let (owner, calls) = observe::engine(Arc::new(Sentinel {
                        dialect,
                        fail_begin,
                    }));
                    refused(
                        search(model, &owner, query).await,
                        DataErrorKind::Unsupported,
                    );
                    assert_eq!(calls.snapshot(), observe::Snapshot::default());
                    owner.close().await?;
                }
                // Invalid paths retain their prior validation precedence over the snapshot gate.
                let (owner, calls) = observe::engine(Arc::new(Sentinel {
                    dialect,
                    fail_begin,
                }));
                refused(
                    search(
                        model,
                        &owner,
                        serde_json::json!({"total":true,"includes":["missing"]}),
                    )
                    .await,
                    DataErrorKind::Validation,
                );
                assert_eq!(calls.snapshot(), observe::Snapshot::default());
                if dialect == DialectId::Mysql {
                    refused(batch(model, &owner).await, DataErrorKind::Unsupported);
                    assert_eq!(calls.snapshot(), observe::Snapshot::default());
                }
                owner.close().await?;
            }
        }
    }
    println!("generated atomic preflight: all model/dialect/failing-BEGIN cases have zero entries");
    Ok(())
}
