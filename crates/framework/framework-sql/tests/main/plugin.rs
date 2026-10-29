//! Startup dependencies, bounded inventories and shared operation ownership.
use stackpress_framework_sql::{
    operations::{Operation, OperationPolicy, Policy, Registration},
    plugin::{self, Contributions, RegisterOperations},
};
use stackpress_lib::{Error, Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::{
    Database, DialectId, Engine, Query, QueryLimits, QueryResult, SqlFuture, TransactionOptions,
    TransactionSession,
};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

/// Every adapter action is observable; a registrar must use none of them.
struct UnusedDatabase(Arc<AtomicUsize>);
impl Database for UnusedDatabase {
    /// Dialect inspection requires no connection.
    fn dialect(&self) -> DialectId {
        DialectId::Sqlite
    }
    /// Record any inappropriate startup query without hiding it behind a successful mock.
    fn run(&self, _: Query, _: QueryLimits) -> SqlFuture<'_, QueryResult> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(stackpress_sql::SqlError::new(
                stackpress_sql::ErrorKind::Driver,
                "unexpected database access",
            ))
        })
    }
    /// Beginning a transaction is also forbidden during registration.
    fn begin(&self, _: TransactionOptions) -> SqlFuture<'_, Box<dyn TransactionSession>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(stackpress_sql::SqlError::new(
                stackpress_sql::ErrorKind::Driver,
                "unexpected transaction",
            ))
        })
    }
    /// The application retains ownership; startup does not close its engine.
    fn close(&self) -> SqlFuture<'_, ()> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Box::pin(async {
            Err(stackpress_sql::SqlError::new(
                stackpress_sql::ErrorKind::Driver,
                "unexpected close",
            ))
        })
    }
}
/// No implicit grant is inferred from an Engine or a selected plugin.
struct Deny;
impl OperationPolicy for Deny {
    /// Remain a refusing policy even though startup checks that the service exists.
    fn authorize(&self, _: &'static str, _: Operation, _: &Request) -> Result<()> {
        Err(Error::new("private policy refusal"))
    }
}
/// Callback order and count belong to this one build, not process-global state.
struct Trace(Mutex<Vec<usize>>);
/// A real registered listener proves contributions survive into the normal dispatcher.
async fn answer(_: &mut Request, response: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    response.results(&"registered")?;
    Ok(Flow::Continue)
}
/// Register a unique operation per callback with the caller's shared ownership marker.
fn counted(start: &mut StartupContext, marker: &mut Registration) -> Result<()> {
    let index = {
        let mut trace = start.service::<Trace>()?.0.lock().unwrap();
        let index = trace.len();
        trace.push(index);
        index
    };
    let prefix = format!("family{index}");
    marker.claim(&prefix, &[Operation::Get])?;
    start.on(&format!("{prefix}.get"), answer)?;
    Ok(())
}
/// Two independent callbacks must not claim one operation under separate markers.
fn conflicting(start: &mut StartupContext, marker: &mut Registration) -> Result<()> {
    marker.claim("same", &[Operation::Get])?;
    counted(start, marker)
}
/// Callback failures must prevent publication of a partially registered host.
fn failing(_: &mut StartupContext, _: &mut Registration) -> Result<()> {
    Err(Error::invalid("contribution refused"))
}
/// Supply only the explicitly selected dependencies and retain observability outside startup.
fn fixture(
    engine: bool,
    policy: bool,
    callbacks: Option<&'static [RegisterOperations]>,
) -> (ServerBuilder, Arc<AtomicUsize>, Arc<Trace>) {
    let mut app = ServerBuilder::new();
    let calls = Arc::new(AtomicUsize::new(0));
    let trace = Arc::new(Trace(Mutex::new(Vec::new())));
    let database = Arc::new(Engine::new(UnusedDatabase(calls.clone())));
    let retained = trace.clone();
    app.on_config(task(move |start: &mut StartupContext| {
        let database = database.clone();
        let trace = retained.clone();
        Box::pin(async move {
            start.register(trace)?;
            if engine {
                start.register(database)?;
            }
            if policy {
                start.register(Arc::new(Policy(Arc::new(Deny))))?;
            }
            if let Some(operations) = callbacks {
                start.register(Arc::new(Contributions { operations }))?;
            }
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    (app, calls, trace)
}
/// Actual listeners are registered once, in order, with zero database effects at startup.
#[tokio::test]
async fn selected_and_maximum_inventory_use_one_host_without_database_setup() {
    for callbacks in [
        &[counted as RegisterOperations, counted][..],
        &[counted as RegisterOperations; 128][..],
    ] {
        // Both arrays are promoted static compiled inventories.
        let (mut app, calls, trace) = fixture(true, true, Some(callbacks));
        app.plugin("sql", plugin::register).unwrap();
        let server = app.build().await.unwrap();
        assert!(server.host().service::<plugin::Service>().is_ok());
        // Callback functions retain the SQL Listen task's selected plugin identity.
        assert!(
            server
                .listeners()
                .all(|listener| listener.plugin.as_deref() == Some("sql"))
        );
        assert_eq!(
            trace.0.lock().unwrap().as_slice(),
            (0..callbacks.len()).collect::<Vec<_>>()
        );
        let reply = server
            .call("family0.get", serde_json::json!({}))
            .await
            .unwrap();
        assert!(reply.report.failures.is_empty());
        assert!(
            matches!(reply.response.body(), stackpress_lib::Body::Value(stackpress_lib::Value::String(value)) if value == "registered")
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
/// Missing resources and invalid inventory sizes fail before any contribution executes.
#[tokio::test]
async fn missing_dependencies_and_invalid_counts_register_nothing() {
    for (engine, policy, callbacks) in [
        (false, true, Some(&[counted as RegisterOperations][..])),
        (true, false, Some(&[counted as RegisterOperations][..])),
        (true, true, None),
        (true, true, Some(&[][..])),
        (true, true, Some(&[counted as RegisterOperations; 129][..])),
    ] {
        let (mut app, calls, trace) = fixture(engine, policy, callbacks);
        app.plugin("sql", plugin::register).unwrap();
        assert!(app.build().await.is_err());
        assert!(trace.0.lock().unwrap().is_empty());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
/// Aliased double selection and callback conflicts fail through singleton/marker contracts.
#[tokio::test]
async fn duplicate_plugins_callbacks_and_callback_errors_publish_no_server() {
    for (twice, callbacks, completed) in [
        (true, &[counted as RegisterOperations][..], 0),
        (
            false,
            &[conflicting as RegisterOperations, conflicting][..],
            1,
        ),
        (
            false,
            &[counted as RegisterOperations, failing, counted][..],
            1,
        ),
        (false, &[failing as RegisterOperations, counted][..], 0),
    ] {
        let (mut app, calls, trace) = fixture(true, true, Some(callbacks));
        app.plugin("sql", plugin::register).unwrap();
        if twice {
            app.plugin("sql-alias", plugin::register).unwrap();
        }
        app.on_route(task(|start: &mut StartupContext| {
            Box::pin(async move {
                // A Listen failure must prevent the later Route phase from running.
                start.service::<Trace>()?.0.lock().unwrap().push(999);
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
        let error = app.build().await.err().expect("registration must fail");
        assert_eq!(
            error.stage,
            if twice {
                stackpress_server::StartupStage::Config
            } else {
                stackpress_server::StartupStage::Listen
            }
        );
        // Earlier callbacks have observable effects: failure stops later work without
        // rolling them back. A failed build still cannot publish a partial Server.
        assert_eq!(
            trace.0.lock().unwrap().as_slice(),
            (0..completed).collect::<Vec<_>>()
        );
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
/// Compiling the feature cannot activate it, and direct callbacks remain usable independently.
#[tokio::test]
async fn disabled_plugin_and_direct_registration_keep_their_contracts() {
    let (app, calls, trace) = fixture(true, true, Some(&[counted]));
    let server = app.build().await.unwrap();
    assert!(server.host().service::<plugin::Service>().is_err());
    assert!(trace.0.lock().unwrap().is_empty());
    let (mut app, _, _) = fixture(true, true, None);
    app.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            counted(start, &mut Registration::default())?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let server = app.build().await.unwrap();
    assert!(server.host().service::<plugin::Service>().is_err());
    assert!(
        server
            .call("family0.get", serde_json::json!({}))
            .await
            .unwrap()
            .report
            .failures
            .is_empty()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
}

/// Resources contributed by a later plugin cannot satisfy an earlier Config callback.
#[tokio::test]
async fn wrong_order_and_declared_missing_dependencies_fail_before_listen() {
    /// Queue a policy provider under its own selected plugin identity.
    fn policy_provider(app: &mut ServerBuilder) -> Result<()> {
        app.on_config(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.register(Arc::new(Policy(Arc::new(Deny))))?;
                Ok(Flow::Continue)
            })
        }))?;
        Ok(())
    }
    let (mut app, calls, trace) = fixture(true, false, Some(&[counted]));
    app.plugin("sql", plugin::register).unwrap();
    app.plugin("policy", policy_provider).unwrap();
    assert!(app.build().await.is_err());
    assert!(trace.0.lock().unwrap().is_empty());
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let (mut app, _, trace) = fixture(true, true, Some(&[counted]));
    app.plugin_with(
        stackpress_server::PluginSpec::new("sql").requires(&["absent"]),
        plugin::register,
    )
    .unwrap();
    assert_eq!(
        app.build().await.err().unwrap().stage,
        stackpress_server::StartupStage::Registration
    );
    assert!(trace.0.lock().unwrap().is_empty());
}

/// A suspended later Listen task cannot retain the SQL build's resources after cancellation.
#[tokio::test]
async fn cancelled_build_releases_sql_startup_state_without_database_effects() {
    use std::future::Future;

    /// Suspend after SQL's synchronous contribution, before the Route phase can start.
    fn suspend(app: &mut ServerBuilder) -> Result<()> {
        app.on_listen(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.service::<Trace>()?.0.lock().unwrap().push(128);
                std::future::pending().await
            })
        }))?;
        app.on_route(task(|start: &mut StartupContext| {
            Box::pin(async move {
                start.service::<Trace>()?.0.lock().unwrap().push(999);
                Ok(Flow::Continue)
            })
        }))?;
        Ok(())
    }

    let (mut app, calls, trace) = fixture(true, true, Some(&[counted]));
    app.plugin("sql", plugin::register).unwrap();
    app.plugin("suspended", suspend).unwrap();
    let mut build = Box::pin(app.build());
    // Poll to the actual suspension, matching the server's existing cancellation proof.
    // No scheduler delay or background task establishes this interleaving.
    std::future::poll_fn(|context| {
        assert!(build.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert_eq!(trace.0.lock().unwrap().as_slice(), &[0, 128]);
    assert!(Arc::strong_count(&trace) > 1);
    drop(build);
    // Cancellation releases services and callbacks without closing the app-owned database
    // or rolling back the earlier callback's trace. Route never ran and no Server escaped.
    assert_eq!(Arc::strong_count(&trace), 1);
    assert_eq!(Arc::strong_count(&calls), 1);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert_eq!(trace.0.lock().unwrap().as_slice(), &[0, 128]);
}
