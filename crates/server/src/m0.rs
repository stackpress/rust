//! M0 proofs against real host/lib values; policy breadth remains in later milestones.

use std::future::Future;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;
use std::time::Duration;

use stackpress_lib::{
    DeliveryState, DispatchOutcome, EmitterBuilder, Error, Flow, Invocation, Nest, Request,
    Response, Result,
};

use super::*;
use crate::context::CallContext;

#[path = "../../lib/tests/support/watchdog.rs"]
mod watchdog;

/// Typed test input seeds the real registry before M1 implements JSON section decoding.
struct Config {
    /// Owned text proves config references remain usable across suspension.
    label: String,
    /// Counts actual constructed service releases independently of Server ownership.
    drops: Arc<AtomicUsize>,
}

/// A real service with no hidden framework lock and explicit internal synchronization.
struct Profiles {
    /// Borrowed operation source value.
    label: String,
    /// Two children must execute concurrently to pass this rendezvous.
    barrier: tokio::sync::Barrier,
    /// Externally observed resource lifetime.
    drops: Arc<AtomicUsize>,
}

impl Drop for Profiles {
    /// Observe release without relying on an engine/global singleton.
    fn drop(&mut self) {
        // Only the explicit Arc service allocation owns this destructor.
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

/// Construct the service in Config through the public StartupContext API.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    // Copy only construction inputs before taking the mutable registration borrow.
    let config = start.config::<Config>()?;
    let label = config.label.clone();
    let drops = Arc::clone(&config.drops);
    start.register(Arc::new(Profiles {
        label,
        barrier: tokio::sync::Barrier::new(2),
        drops,
    }))?;
    Ok(Flow::Continue)
}

/// Borrow both config and service through the requested Invocation shortcuts.
async fn leaf(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // These references, not raw pointers or cloned payloads, survive the await.
    let service = inv.service::<Profiles>()?;
    let config = inv.config::<Config>()?;
    let label = service.label.as_str();
    service.barrier.wait().await;
    assert_eq!(label, config.label);
    res.results(&serde_json::json!({
        "correlation": inv.correlation_id()?, "stage": format!("{:?}", inv.stage()?),
        "path": inv.path().iter().map(|frame| frame.event.as_str()).collect::<Vec<_>>(),
    }))?;
    Ok(Flow::Continue)
}

/// Prove an explicit reused-pair nested delegation without a second lifecycle.
async fn echo(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Yield while retaining the actual service and context borrowed by the parent.
    let borrowed = inv.service::<Profiles>()?;
    tokio::task::yield_now().await;
    req.data.set("echo", borrowed.label.as_str())?;
    res.results(&req.data)?;
    Ok(Flow::Continue)
}

/// Concurrent child resolves preserve both parent context identity and diagnostic ancestry.
async fn parent(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let host = inv.host()?;
    let id = inv.correlation_id()?.expect("explicit proof root");
    let shared = inv.shared_service::<Profiles>()?;
    assert!(std::ptr::eq(shared.as_ref(), host.service::<Profiles>()?));
    let (left, right) = tokio::try_join!(
        inv.resolve("leaf", Nest::new()),
        inv.resolve("leaf", Nest::new()),
    )?;
    // Each child reports its own listener path and this root's correlation/stage.
    for result in [left, right] {
        let value = result.response.results.expect("leaf result").to_json()?;
        assert_eq!(value["correlation"], id);
        assert_eq!(value["stage"], "Operation");
        assert_eq!(value["path"], serde_json::json!(["parent", "leaf"]));
    }
    assert!(std::ptr::eq(host, inv.host()?));
    assert_eq!(inv.event(), "parent");
    assert_eq!(inv.path().len(), 1);
    inv.delegate("echo", req, res).await
}

/// Cooperative child abort is an explicit Flow through the delegation helper.
async fn abort(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    // No HTTP status is manufactured by a raw child abort.
    Ok(Flow::Abort)
}

/// A route directly delegates to the same reusable operation after capture injection.
async fn profile_route(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // The DX helper retains missing/abort/error behavior while making delegation one line.
    inv.delegate("echo", req, res).await
}

/// Install operation listeners after Config has constructed the service.
async fn listen(start: &mut StartupContext) -> Result<Flow> {
    assert_eq!(start.stage(), StartupStage::Listen);
    start.on("parent", parent)?;
    start.on("leaf", leaf)?;
    start.on("echo", echo)?;
    start.on("abort", abort)?;
    Ok(Flow::Continue)
}

/// Register the route through the same real host wrapper used by later HTTP work.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    assert_eq!(start.stage(), StartupStage::Route);
    start.get("/profiles/:id", profile_route)?;
    Ok(Flow::Continue)
}

/// Select a function plugin without a macro or trait implementation.
fn plugin(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(configure)?;
    app.on_listen(listen)?;
    app.on_route(routes)?;
    Ok(())
}

/// Seed only typed input data privately; all task/wrapper/runtime work is production code.
fn builder(drops: &Arc<AtomicUsize>) -> Result<ServerBuilder> {
    let mut builder = ServerBuilder::new();
    // This fixture is not a claimed implementation of the future config-section policy.
    builder.state.config.insert(Arc::new(Config {
        label: "configured".into(),
        drops: Arc::clone(drops),
    }))?;
    Ok(builder)
}

/// Exercise two concurrently spawned roots against one immutable real host.
async fn host_scenarios() -> BuildResult<()> {
    let drops = Arc::new(AtomicUsize::new(0));
    let mut app = builder(&drops)?;
    app.plugin("profiles", plugin)?;
    let server = Arc::new(app.build().await?);
    let weak = Arc::downgrade(&server.host);
    let mut tasks = Vec::new();
    for id in ["root-one", "root-two"] {
        let server = Arc::clone(&server);
        tasks.push(tokio::spawn(async move {
            // M0 carries explicit fixture correlations; top-level allocation is an M2 concern.
            let context = CallContext {
                host: Arc::clone(&server.host),
                correlation: Some(id.into()),
                stage: LifecycleStage::Operation,
            };
            server
                .emitter
                .resolve_with_context("parent", Nest::new(), &context)
                .await
        }));
    }
    for task in tasks {
        assert_eq!(
            task.await.expect("owned root task")?.outcome,
            DispatchOutcome::Completed
        );
    }
    // The host wrapper overrides aggregate id but leaves query/post storage unchanged.
    let mut req = Request::builder()
        .url("/profiles/path-id?id=query-id")
        .data(serde_json::json!({"id": "explicit-id"}))
        .build()?;
    let mut res = Response::new();
    server
        .emit("GET /profiles/path-id", &mut req, &mut res)
        .await?;
    assert_eq!(req.data.get("id").and_then(|v| v.as_str()), Some("path-id"));
    assert_eq!(
        req.query.get("id").and_then(|v| v.as_str()),
        Some("query-id")
    );

    // Raw host roots expose services, but fabricate no lifecycle stage/correlation.
    assert_eq!(
        server.resolve("missing", Nest::new()).await?.outcome,
        DispatchOutcome::NoListener
    );
    let descriptor = server
        .emitter
        .listeners()
        .find(|item| item.pattern.contains("profiles"))
        .expect("route descriptor");
    assert_eq!(descriptor.plugin.as_deref(), Some("profiles"));
    assert!(descriptor.registered.file.ends_with("m0.rs"));

    // Dropping the last host owner releases the service and proves no emitter/context cycle.
    drop(server);
    assert!(weak.upgrade().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Prove helper semantics separately from host startup breadth.
#[tokio::test]
async fn delegation_and_missing_host_are_explicit() -> Result<()> {
    async fn delegate_abort(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        // Delegation remains a raw nested operation even without a server context.
        inv.delegate("abort", req, res).await
    }
    async fn delegate_missing(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        inv.delegate("absent", req, res).await
    }
    async fn missing_host(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        assert!(inv.host().is_err());
        assert!(inv.config::<Config>().is_err());
        assert!(inv.service::<Profiles>().is_err());
        assert!(inv.shared_service::<Profiles>().is_err());
        assert!(inv.stage().is_err());
        assert!(inv.correlation_id().is_err());
        Ok(Flow::Continue)
    }
    let mut builder = EmitterBuilder::new();
    builder.on("abort", abort)?;
    builder.on("delegated", delegate_abort)?;
    builder.on("missing", delegate_missing)?;
    builder.on("raw", missing_host)?;
    let emitter = builder.build();
    assert_eq!(
        emitter.resolve("delegated", Nest::new()).await?.outcome,
        DispatchOutcome::Aborted
    );
    let error = emitter
        .resolve("missing", Nest::new())
        .await
        .expect_err("required capability");
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<DelegationError>()
    );
    assert_eq!(error.code(), 404);
    emitter.resolve("raw", Nest::new()).await?;
    Ok(())
}

/// Distinguish the abort and original-error paths using explicit typed fixture data.
struct AbortStartup(bool);

/// A configuration task may fail/abort after allocating a real service.
async fn fail_or_stop(start: &mut StartupContext) -> Result<Flow> {
    configure(start).await?;
    if start.config::<AbortStartup>()?.0 {
        Ok(Flow::Abort)
    } else {
        Err(Error::from_source(std::io::Error::other("startup source")))
    }
}

/// Real selection supplies the error's plugin metadata before asynchronous startup.
fn failed_plugin(app: &mut ServerBuilder) -> Result<()> {
    app.on_config(fail_or_stop)
}

/// A suspended task owns its partial state until the build future is dropped.
async fn suspend(start: &mut StartupContext) -> Result<Flow> {
    configure(start).await?;
    std::future::pending().await
}

/// Fail/Abort/cancel paths destroy accumulated services without pretending to roll back effects.
#[tokio::test]
async fn failed_startup_never_publishes_and_cancel_releases() -> Result<()> {
    for abort in [false, true] {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut builder = builder(&drops)?;
        builder.state.config.insert(Arc::new(AbortStartup(abort)))?;
        builder.plugin("failed-plugin", failed_plugin)?;
        let Err(error) = builder.build().await else {
            panic!("partial host escaped")
        };
        assert_eq!(error.stage, StartupStage::Config);
        assert_eq!(error.plugin.as_deref(), Some("failed-plugin"));
        assert!(error.registered.file.ends_with("m0.rs"));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let mut builder = builder(&drops)?;
    builder.on_config(suspend)?;
    let mut future = Box::pin(builder.build());
    // Poll exactly to the first suspension; no timing guess is needed to prove allocation.
    std::future::poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(future);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Output authority belongs to the adapter, not a replaceable Response instance.
#[tokio::test]
async fn response_replacement_cannot_repeat_native_output() -> Result<()> {
    let writes = Arc::new(AtomicUsize::new(0));
    let mut guard = OutputGuard::default();
    let mut response = Response::new();
    response.stop()?;
    // A handler can replace a whole value before any native write; it cannot see guard.
    response = Response::new();
    let count = Arc::clone(&writes);
    guard
        .dispatch(&mut response, move |_| async move {
            count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await?;
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("duplicate write") })
            .await
            .is_err()
    );
    assert_eq!(writes.load(Ordering::SeqCst), 1);
    assert_eq!(guard.state(), DeliveryState::Sent);

    // Cancellation retains Attempted in both owners; replacement cannot grant a retry.
    let mut guard = OutputGuard::default();
    let mut response = Response::new();
    let mut attempt = Box::pin(guard.dispatch(&mut response, |_| std::future::pending()));
    std::future::poll_fn(|context| {
        assert!(attempt.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    drop(attempt);
    assert_eq!(guard.state(), DeliveryState::Attempted);
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("cancel retry") })
            .await
            .is_err()
    );
    Ok(())
}

/// Partial writer errors and suppression/finalized states consume independent native authority.
#[tokio::test]
async fn output_failure_and_suppression_do_not_retry() -> Result<()> {
    let mut guard = OutputGuard::default();
    let mut response = Response::new();
    let bytes = Arc::new(AtomicUsize::new(0));
    let partial = Arc::clone(&bytes);
    assert!(
        guard
            .dispatch(&mut response, move |_| async move {
                partial.fetch_add(3, Ordering::SeqCst);
                Err(Error::new("writer failed after three bytes"))
            })
            .await
            .is_err()
    );
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("partial write retry") })
            .await
            .is_err()
    );
    assert_eq!(guard.state(), DeliveryState::Failed);
    assert_eq!(bytes.load(Ordering::SeqCst), 3);

    let mut guard = OutputGuard::default();
    response.stop()?;
    guard
        .dispatch(&mut response, |_| async { panic!("suppressed writer") })
        .await?;
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("suppression reset") })
            .await
            .is_err()
    );
    assert_eq!(guard.state(), DeliveryState::Stopped);

    // A finalized no-op lib dispatch did not produce a native adapter result.
    let mut guard = OutputGuard::default();
    response.dispatch().await?;
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("foreign delivery") })
            .await
            .is_err()
    );
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("foreign reset") })
            .await
            .is_err()
    );
    assert_eq!(guard.state(), DeliveryState::Failed);
    Ok(())
}

/// Singleton service state and failed registration never become silent replacement.
#[tokio::test]
async fn startup_and_registry_missing_values_are_explicit() -> Result<()> {
    async fn duplicate(start: &mut StartupContext) -> Result<Flow> {
        start.register(Arc::new(7_u64))?;
        assert!(start.register(Arc::new(9_u64)).is_err());
        assert_eq!(*start.service::<u64>()?, 7);
        Ok(Flow::Continue)
    }
    async fn wrong_phase(start: &mut StartupContext) -> Result<Flow> {
        start.register(Arc::new(8_u64))?;
        Ok(Flow::Continue)
    }
    fn empty(_: &mut ServerBuilder) -> Result<()> {
        Ok(())
    }
    let mut app = ServerBuilder::new();
    assert!(app.plugin("bad\nname", empty).is_err());
    app.plugin("single", empty)?;
    assert!(app.plugin("single", empty).is_err());
    app.on_config(duplicate)?;
    app.on_listen(wrong_phase)?;
    let Err(error) = app.build().await else {
        panic!("late service registration escaped")
    };
    assert_eq!(error.stage, StartupStage::Listen);
    let server = ServerBuilder::new()
        .build()
        .await
        .map_err(|error| error.error)?;
    assert!(server.host().config::<u64>().is_err());
    assert!(server.host().service::<u64>().is_err());
    assert!(server.host().shared_service::<u64>().is_err());
    Ok(())
}

/// Test the bounded data-input seam without claiming M1 merge/section policy coverage.
#[test]
fn explicit_config_layers_validate_before_append() -> Result<()> {
    let mut layers = ConfigLayers::new();
    layers.json("explicit file", r#"{"profiles":{"limit":3}}"#)?;
    assert!(layers.json("invalid", "[]").is_err());
    assert!(layers.json("bad\nlabel", "{}").is_err());
    assert!(layers.json("invalid json", "{").is_err());
    // Aggregate budget matters even when every individual layer is valid.
    let payload = serde_json::json!({"value": "x".repeat(600_000)});
    let mut bounded = ConfigLayers::new();
    bounded.value("first", payload.clone())?;
    assert!(bounded.value("second", payload).is_err());
    assert_eq!(bounded.entries().count(), 1);
    assert_eq!(layers.entries().count(), 1);
    assert_eq!(layers.entries().next().unwrap().1["profiles"]["limit"], 3);
    Ok(())
}

/// Child entry for scenarios that would deadlock under a hidden global callback lock.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by the M0 host watchdog parent"]
async fn host_child() {
    tokio::time::timeout(Duration::from_secs(2), host_scenarios())
        .await
        .expect("host async deadline")
        .expect("host context proof");
}

/// A parent outside the executor can kill and reap even a synchronously blocked child.
#[test]
fn host_context_and_wrapper_proof_under_watchdog() {
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command.args(["--ignored", "--exact", "m0::host_child", "--nocapture"]);
    let result = watchdog::run(&mut command, "host-context", Duration::from_secs(10))
        .expect("launch/reap host child");
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
