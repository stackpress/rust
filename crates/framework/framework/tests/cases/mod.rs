//! Source parity, resource ownership and registration boundaries of the aggregate.

mod catalog;
mod closeout;
#[cfg(feature = "codegen")]
mod codegen;
mod commands;
mod language;
mod ownership;
mod sql;
mod view;

use serde_json::{Value, json};
use stackpress_framework::{client::Client, commands::Bindings, register};
use stackpress_lib::{Flow, Invocation, Request, Response, Result, queue::task};
use stackpress_server::{
    ConfigLayers, ServerBuilder, ServerInvocationExt, StartupContext, StartupSpec, StartupStage,
};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

/// Build an application-selected aggregate without a native execution host.
fn application(config: Value) -> ServerBuilder {
    let mut app = ServerBuilder::new();
    let mut layers = ConfigLayers::new();
    layers.value("application", config).unwrap();
    app.config_layers(layers).unwrap();
    app.plugin("framework", register).unwrap();
    app
}

/// Public capability used by no-global-guard and same-route cooperation tests.
async fn public(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&"public")?;
    Ok(Flow::Continue)
}

/// Observe real provider execution, independent of registration/startup phases.
async fn command(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.service::<AtomicUsize>()?.fetch_add(1, Ordering::SeqCst);
    res.results(&inv.event())?;
    Ok(Flow::Abort)
}

/// A no-config host still contributes source commands/auth events and the default CSRF mechanism.
#[tokio::test]
async fn absent_client_and_config_need_no_engine_or_startup_io() {
    let host = application(json!({})).build().await.unwrap();
    let client = host.host().service::<Client>().unwrap();
    assert!(client.get(true).unwrap().is_none());
    assert!(client.get(false).is_err());
    assert_eq!(
        host.host()
            .service::<stackpress_framework_csrf::Service>()
            .unwrap()
            .name(),
        "csrf"
    );
    assert!(host.host().service::<Bindings>().is_err());
    assert!(
        host.host()
            .service::<stackpress_framework_session::Service>()
            .is_err()
    );
    let patterns: Vec<_> = host
        .listeners()
        .map(|listener| listener.pattern.clone())
        .collect();
    let oracle: Value =
        serde_json::from_str(include_str!("../fixtures/source-order.json")).unwrap();
    let commands = oracle["command_order"].as_array().unwrap();
    for (index, name) in commands.iter().enumerate() {
        assert_eq!(&patterns[index], name.as_str().unwrap());
    }
    assert_eq!(
        &patterns[commands.len()..],
        &["auth-signup", "auth-signin", "auth-signout"]
    );
    assert!(host.resolve("serve", json!({})).await.is_err());
}

/// Source priority ten is pinned by equal-priority probes on both sides of registration.
#[tokio::test]
async fn schema_priority_and_phase_publication_are_real() {
    let oracle: Value =
        serde_json::from_str(include_str!("../fixtures/source-order.json")).unwrap();
    let priority = oracle["schema_config_priority"].as_i64().unwrap();
    let mut app = ServerBuilder::new();
    app.startup(
        StartupSpec::new(StartupStage::Config).priority(priority),
        task(|start: &mut StartupContext| {
            Box::pin(async move {
                assert!(start.service::<Client>().is_err());
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    register(&mut app).unwrap();
    app.startup(
        StartupSpec::new(StartupStage::Config).priority(priority),
        task(|start: &mut StartupContext| {
            Box::pin(async move {
                assert!(start.service::<Client>()?.get(true)?.is_none());
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.startup(
        StartupSpec::new(StartupStage::Config).priority(11),
        task(|start: &mut StartupContext| {
            Box::pin(async move {
                assert!(start.service::<Client>().is_err());
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.startup(
        StartupSpec::new(StartupStage::Config).priority(9),
        task(|start: &mut StartupContext| {
            Box::pin(async move {
                assert!(start.service::<Client>()?.get(true)?.is_none());
                start.register(Arc::new(AtomicUsize::new(0)))?;
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    app.on_listen(task(|start: &mut StartupContext| {
        Box::pin(async move {
            assert_eq!(start.stage(), StartupStage::Listen);
            assert_eq!(
                start
                    .service::<AtomicUsize>()?
                    .fetch_add(1, Ordering::SeqCst),
                0
            );
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.on_route(task(|start: &mut StartupContext| {
        Box::pin(async move {
            assert_eq!(start.stage(), StartupStage::Route);
            assert_eq!(start.service::<AtomicUsize>()?.load(Ordering::SeqCst), 1);
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    app.build().await.unwrap();
}

/// The real command provider is untouched until explicit raw invocation and preserves Abort.
#[tokio::test]
async fn command_provider_is_lazy_and_errors_are_not_success() {
    let calls = Arc::new(AtomicUsize::new(0));
    let captured = calls.clone();
    let mut app = application(json!({"view":{}}));
    app.on_config(task(move |start: &mut StartupContext| {
        let calls = captured.clone();
        Box::pin(async move {
            let mut bindings = Bindings::default();
            bindings.bind("serve", command);
            start.register(Arc::new(bindings))?;
            start.register(calls)?;
            Ok(Flow::Continue)
        })
    }))
    .unwrap();
    let host = app.build().await.unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    let result = host.resolve("serve", json!({})).await.unwrap();
    assert_eq!(result.outcome, stackpress_lib::DispatchOutcome::Aborted);
    assert_eq!(result.response.results.unwrap().as_str(), Some("serve"));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert!(host.resolve("build", json!({})).await.is_err());
    assert_eq!(
        host.host()
            .service::<stackpress_framework::view::Settings>()
            .unwrap()
            .config,
        json!({})
    );
}

/// Missing access contributes no guard; an empty access map contributes its source allowing guard.
#[tokio::test]
async fn no_global_guard_and_missing_versus_empty_access() {
    for (config, guarded) in [
        (json!({}), false),
        (json!({"session":{}}), false),
        (json!({"session":{"access":{}}}), true),
    ] {
        let mut app = application(config);
        app.get("/public", public).unwrap();
        let host = app.build().await.unwrap();
        assert_eq!(
            host.listeners()
                .any(|listener| listener.pattern == "request"),
            guarded
        );
        let response = host.call("GET /public", json!({})).await.unwrap().response;
        assert_eq!(response.result().unwrap().as_str(), Some("public"));
        assert!(response.error().is_none());
    }
}

/// Same route handlers remain in priority/tie order, and any handler can stop the chain.
#[tokio::test]
async fn g42_all_matching_handlers_keep_priority_ties_and_abort() {
    let mut app = application(json!({}));
    // Register a lower-priority Abort first so the later high-priority callback must sort ahead.
    app.on(
        "GET /shared",
        stackpress_lib::handler(|req: &mut Request, res: &mut Response, _: Invocation<'_>| {
            Box::pin(async move {
                res.results(&req.data.path_as::<String>("order")?)?;
                Ok(Flow::Abort)
            })
        }),
    )
    .unwrap();
    app.on("GET /shared", public).unwrap();
    app.on_priority(
        "GET /shared",
        1,
        stackpress_lib::handler(|req: &mut Request, _: &mut Response, _: Invocation<'_>| {
            Box::pin(async move {
                req.data.set("order", "first")?;
                Ok(Flow::Continue)
            })
        }),
    )
    .unwrap();
    let host = app.build().await.unwrap();
    let result = host.resolve("GET /shared", json!({})).await.unwrap();
    assert_eq!(result.outcome, stackpress_lib::DispatchOutcome::Aborted);
    assert_eq!(result.response.results.unwrap().as_str(), Some("first"));
}

/// Compile-time evidence rejects borrowed static catalogs, native-reader clones and non-Send providers.
#[test]
fn compile_fail_ownership_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/*.rs");
}
