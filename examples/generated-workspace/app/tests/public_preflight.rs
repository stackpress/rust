//! Public generated reads refuse known invalid work before touching the selected executor.
#![cfg(not(feature = "matrix"))]

#[cfg(not(feature = "names"))]
use generated_workspace_app::generated::stores::operations::project as selected;
#[cfg(feature = "names")]
use generated_workspace_app::generated::stores::operations::{_operations as selected, identity};
use serde_json::{Value, json};
use stackpress_framework_sql::{
    DataError, DataErrorKind,
    operations::{Operation, OperationPolicy, Policy, Registration},
};
use stackpress_lib::{Error, Flow, Request, queue::task};
use stackpress_server::{Server, ServerBuilder, StartupContext};
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, Row, SqlError, SqlFuture,
    TransactionOptions, TransactionSession,
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

#[path = "../../support/watchdog.rs"]
mod watchdog;

/// Observe all four executor effects, independently of whether BEGIN would succeed.
#[derive(Default)]
struct Effects {
    begin: AtomicUsize,
    run: AtomicUsize,
    commit: AtomicUsize,
    rollback: AtomicUsize,
}
impl Effects {
    /// Read one exact effect vector after the awaited operation completes.
    fn snapshot(&self) -> [usize; 4] {
        [
            self.begin.load(Ordering::SeqCst),
            self.run.load(Ordering::SeqCst),
            self.commit.load(Ordering::SeqCst),
            self.rollback.load(Ordering::SeqCst),
        ]
    }
}
/// No native resources exist; successful controls return a finite empty search result.
struct Recording {
    dialect: DialectId,
    fail_begin: bool,
    effects: Arc<Effects>,
}
impl Database for Recording {
    /// Exercise the actual generated dialect gates with no network adapter.
    fn dialect(&self) -> DialectId {
        self.dialect
    }
    /// An ordinary pooled execution is visible even if a wrapper accidentally uses one.
    fn run(&self, query: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.effects.run.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move { Ok(empty_result(&query)) })
    }
    /// Count acquisition at method entry, so even an unpolled native request would fail the test.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        self.effects.begin.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move {
            if self.fail_begin {
                return Err(SqlError::new(
                    stackpress_sql::ErrorKind::Busy,
                    "recording BEGIN refused",
                ));
            }
            Ok(Box::new(Session(self.effects.clone())) as Box<dyn TransactionSession>)
        })
    }
    /// There is no connection or pending cleanup in this recording backend.
    fn close(&self) -> SqlFuture<'_, ()> {
        Box::pin(async { Ok(()) })
    }
}
/// Preserve the exact transaction object for successful admission controls.
struct Session(Arc<Effects>);
impl TransactionSession for Session {
    /// COUNT returns one authoritative zero; ordinary pages are empty.
    fn run(&mut self, query: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.0.run.fetch_add(1, Ordering::SeqCst);
        Box::pin(async move { Ok(empty_result(&query)) })
    }
    /// Observe consuming successful completion.
    fn commit(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.0.commit.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
    /// Observe consuming error cleanup, which must be absent on preflight refusal.
    fn rollback(self: Box<Self>) -> SqlFuture<'static, ()> {
        self.0.rollback.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Ok(()) })
    }
}
/// Supply only the two result shapes needed by the supported empty-page controls.
fn empty_result(query: &Query) -> QueryResult {
    QueryResult {
        rows: if query.sql().contains("COUNT(*)") {
            vec![Row::new(vec![("total".into(), 0.into())])]
        } else {
            Vec::new()
        },
        ..Default::default()
    }
}
/// The policy is deliberately separate from decoding and every executor observation.
struct Authorization {
    allow: bool,
    calls: Arc<AtomicUsize>,
}
impl OperationPolicy for Authorization {
    /// Count authorization even for malformed input; refusal must win over decoding.
    fn authorize(&self, _: &'static str, _: Operation, _: &Request) -> stackpress_lib::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.allow {
            Ok(())
        } else {
            Err(Error::new("recording policy refused"))
        }
    }
}
/// Build real generated handlers, policy and Server dispatch without routes or a listener.
async fn host(
    dialect: DialectId,
    fail_begin: bool,
    limits: QueryLimits,
    allow: bool,
) -> (Server, Arc<Effects>, Arc<AtomicUsize>) {
    let effects = Arc::new(Effects::default());
    let calls = Arc::new(AtomicUsize::new(0));
    let engine = Arc::new(
        Engine::new(Recording {
            dialect,
            fail_begin,
            effects: effects.clone(),
        })
        .with_limits(limits),
    );
    let mut builder = ServerBuilder::new();
    let policy = calls.clone();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let engine = engine.clone();
            let calls = policy.clone();
            Box::pin(async move {
                start.register(engine)?;
                start.register(Arc::new(Policy(Arc::new(Authorization { allow, calls }))))?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder
        .on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                let mut registration = Registration::default();
                selected::register(
                    start,
                    &mut registration,
                    "selected",
                    &[Operation::Search, Operation::Detail],
                )?;
                #[cfg(feature = "names")]
                identity::register(start, &mut registration, "private", &[Operation::Detail])?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    (builder.build().await.unwrap(), effects, calls)
}
/// Inspect the concrete data category without depending on private wrapper Display.
fn data_kind(error: &Error) -> DataErrorKind {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(error);
    for _ in 0..8 {
        let current = cause.expect("missing concrete data cause");
        if let Some(data) = current.downcast_ref::<DataError>() {
            return data.kind();
        }
        cause = current.source();
    }
    panic!("data cause exceeds the bounded error chain");
}
/// Assert the requested category and absence of every executor effect through a real handler.
async fn refused(
    dialect: DialectId,
    fail_begin: bool,
    event: &str,
    input: Value,
    limits: QueryLimits,
    kind: DataErrorKind,
) {
    let (app, effects, calls) = host(dialect, fail_begin, limits, true).await;
    let reply = app.call(event, input.clone()).await.unwrap();
    assert!(
        reply.response.error().is_some(),
        "operation unexpectedly succeeded"
    );
    let error = reply.response.into_error();
    assert_eq!(data_kind(&error), kind, "{dialect:?} {event} {input}");
    assert_eq!(effects.snapshot(), [0; 4], "{dialect:?} {event} {input}");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
/// Bound host dispatch and all controls independently of the async executor.
#[test]
fn public_read_preflight() {
    if std::env::var_os("STACKPRESS_S6_SCENARIO").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(prove());
        return;
    }
    let (ok, expired, output) = watchdog::run(
        "public_read_preflight",
        "public-preflight",
        Duration::from_secs(15),
    );
    assert!(ok && !expired, "timeout={expired}\n{output}");
}
/// Exercise dialect, public exposure, exact input limits and successful admission as one matrix.
async fn prove() {
    for dialect in [DialectId::Postgres, DialectId::Mysql] {
        for fail_begin in [false, true] {
            #[cfg(not(feature = "names"))]
            refused(
                dialect,
                fail_begin,
                "selected.search",
                json!({"q":"Ship"}),
                QueryLimits::default(),
                DataErrorKind::Unsupported,
            )
            .await;
            let relation = if cfg!(feature = "names") {
                "row"
            } else {
                "owner"
            };
            for input in [json!({"includes":[relation]}), json!({"total":true})] {
                refused(
                    dialect,
                    fail_begin,
                    "selected.search",
                    input,
                    QueryLimits::default(),
                    DataErrorKind::Unsupported,
                )
                .await;
            }
            #[cfg(feature = "names")]
            refused(
                dialect,
                fail_begin,
                "selected.detail",
                json!({"key":{"id":"p1"}}),
                QueryLimits::default(),
                DataErrorKind::Unsupported,
            )
            .await;
        }
    }
    for dialect in [DialectId::Sqlite, DialectId::Postgres, DialectId::Mysql] {
        let key = json!({"filters":[{"field":"id","predicate":{"op":"eq","value":"p1"}}]});
        for limits in [
            QueryLimits {
                binds: 0,
                ..Default::default()
            },
            QueryLimits {
                bytes: 0,
                ..Default::default()
            },
            QueryLimits {
                sql_bytes: 1,
                ..Default::default()
            },
        ] {
            refused(
                dialect,
                true,
                "selected.search",
                key.clone(),
                limits,
                DataErrorKind::Validation,
            )
            .await;
            #[cfg(not(feature = "names"))]
            refused(
                dialect,
                true,
                "selected.detail",
                json!({"key":{"id":"p1"}}),
                limits,
                DataErrorKind::Validation,
            )
            .await;
        }
        #[cfg(feature = "names")]
        refused(
            dialect,
            true,
            "private.detail",
            json!({"key":{"id":1}}),
            QueryLimits::default(),
            DataErrorKind::Validation,
        )
        .await;
        #[cfg(not(feature = "names"))]
        refused(dialect, true, "selected.search", json!({"sort":[{"field":"title","direction":"asc","nulls":"first"},{"field":"title","direction":"desc","nulls":"last"}]}), QueryLimits::default(), DataErrorKind::Validation).await;
        let relation = if cfg!(feature = "names") {
            "row"
        } else {
            "owner"
        };
        refused(
            dialect,
            true,
            "selected.search",
            json!({"includes":[format!("{relation}.unapproved")]}),
            QueryLimits::default(),
            DataErrorKind::Validation,
        )
        .await;
        // A supported simple search still owns and completes exactly one transaction.
        let (app, effects, _) = host(dialect, false, QueryLimits::default(), true).await;
        let reply = app.call("selected.search", json!({})).await.unwrap();
        assert_eq!(reply.response.try_to_json().unwrap()["code"], 200);
        assert_eq!(effects.snapshot(), [1, 1, 1, 0]);
        // Empty pages may fit zero output budgets; their size is not known before IO.
        let (app, effects, _) = host(
            dialect,
            false,
            QueryLimits {
                rows: 0,
                bytes: 0,
                ..Default::default()
            },
            true,
        )
        .await;
        let reply = app.call("selected.search", json!({})).await.unwrap();
        assert_eq!(reply.response.try_to_json().unwrap()["code"], 200);
        assert_eq!(effects.snapshot(), [1, 1, 1, 0]);
        // Known valid input must still reach acquisition and retain its original failure category.
        let (app, effects, _) = host(dialect, true, QueryLimits::default(), true).await;
        let reply = app.call("selected.search", json!({})).await.unwrap();
        assert_eq!(data_kind(&reply.response.into_error()), DataErrorKind::Sql);
        assert_eq!(effects.snapshot(), [1, 0, 0, 0]);
        // Authorization must win over deliberately malformed input before pure preflight.
        let (app, effects, calls) = host(dialect, true, QueryLimits::default(), false).await;
        let reply = app
            .call("selected.search", json!({"unexpected":true}))
            .await
            .unwrap();
        assert_eq!(reply.response.try_to_json().unwrap()["code"], 403);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(effects.snapshot(), [0; 4]);
    }
    // COUNT has exactly one integer row even when the base table is empty: these output
    // ceilings are known before IO, unlike actual page/related row sizes and lookup keys.
    for limits in [
        QueryLimits {
            rows: 0,
            ..Default::default()
        },
        QueryLimits {
            bytes: 12,
            ..Default::default()
        },
    ] {
        refused(
            DialectId::Sqlite,
            true,
            "selected.search",
            json!({"total":true}),
            limits,
            DataErrorKind::Validation,
        )
        .await;
    }
    let (app, effects, _) = host(
        DialectId::Sqlite,
        false,
        QueryLimits {
            rows: 1,
            bytes: 13,
            ..Default::default()
        },
        true,
    )
    .await;
    let reply = app
        .call("selected.search", json!({"total":true}))
        .await
        .unwrap();
    assert_eq!(reply.response.try_to_json().unwrap()["results"]["total"], 0);
    assert_eq!(effects.snapshot(), [1, 2, 1, 0]);
}
