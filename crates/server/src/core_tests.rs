//! Preserved M0 ownership/context proofs against the integrated production host and lib.

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
    // Both children must reach the real rendezvous; a serialized callback lock would deadlock.
    service.barrier.wait().await;
    // Observe the same borrowed service/config values after suspension without payload cloning.
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
    // Retain the original host borrow and root identity before either child suspends.
    let host = inv.host()?;
    let id = inv.correlation_id()?.expect("explicit proof root");
    let shared = inv.shared_service::<Profiles>()?;
    assert!(std::ptr::eq(shared.as_ref(), host.service::<Profiles>()?));
    // Poll distinct owned request/response pairs concurrently through the same parent context.
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
    // Children must not replace the parent's host pointer, event or ancestry when they return.
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
    // Verify phase advancement before installing reusable operation handlers.
    assert_eq!(start.stage(), StartupStage::Listen);
    start.on("parent", parent)?;
    start.on("leaf", leaf)?;
    start.on("echo", echo)?;
    start.on("abort", abort)?;
    Ok(Flow::Continue)
}

/// Register the route through the same real host wrapper used by later HTTP work.
async fn routes(start: &mut StartupContext) -> Result<Flow> {
    // HTTP exposure belongs to Route, after the operation/service phases completed.
    assert_eq!(start.stage(), StartupStage::Route);
    start.get("/profiles/:id", profile_route)?;
    Ok(Flow::Continue)
}

/// Select a function plugin without a macro or trait implementation.
fn plugin(app: &mut ServerBuilder) -> Result<()> {
    // Selection schedules named phases; no service or operation runs during this function.
    app.on_config(configure)?;
    app.on_listen(listen)?;
    app.on_route(routes)?;
    Ok(())
}

/// Seed only typed input data privately; all task/wrapper/runtime work is production code.
fn builder(drops: &Arc<AtomicUsize>) -> Result<ServerBuilder> {
    // Keep non-serializable drop counters as private test inputs; normal config uses typed
    // sections.
    let mut builder = ServerBuilder::new();
    // These private typed counters isolate ownership proof from serialized config layering.
    builder.state.config.insert(Arc::new(Config {
        label: "configured".into(),
        drops: Arc::clone(drops),
    }))?;
    Ok(builder)
}

/// Exercise two concurrently spawned roots against one immutable real host.
async fn host_scenarios() -> BuildResult<()> {
    // Prepare independently observed resource ownership before compiling the real plugin.
    let drops = Arc::new(AtomicUsize::new(0));
    let mut app = builder(&drops)?;
    app.plugin("profiles", plugin)?;
    // Only explicit Arc sharing makes this non-Clone host available to spawned roots.
    let server = Arc::new(app.build().await?);
    let weak = Arc::downgrade(&server.host);
    let mut tasks = Vec::new();
    // Give each concurrent root its own immutable context rather than a global current-ID slot.
    for id in ["root-one", "root-two"] {
        let server = Arc::clone(&server);
        tasks.push(tokio::spawn(async move {
            // Explicit IDs isolate context inheritance from separately tested top-level allocation.
            let context = CallContext {
                host: Arc::clone(&server.host),
                correlation: Some(id.into()),
                stage: LifecycleStage::Operation,
                failure: None,
                adapter_context: None,
            };
            server
                .emitter
                .resolve_with_context("parent", Nest::new(), &context)
                .await
        }));
    }
    // Join every owned root so no framework test task escapes the watchdog child.
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
    // Observe incoming route precedence while the original query store remains independent.
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
    // Inspect real registration provenance after dispatch, rather than inferring owner from a
    // label.
    let descriptor = server
        .emitter
        .listeners()
        .find(|item| item.pattern.contains("profiles"))
        .expect("route descriptor");
    assert_eq!(descriptor.plugin.as_deref(), Some("profiles"));
    assert!(descriptor.registered.file.ends_with("core_tests.rs"));

    // Dropping the last host owner releases the service and proves no emitter/context cycle.
    drop(server);
    assert!(weak.upgrade().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Prove helper semantics separately from host startup breadth.
#[tokio::test]
async fn delegation_and_missing_host_are_explicit() -> Result<()> {
    /// Delegate a raw child Abort through the public extension helper.
    async fn delegate_abort(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        // A raw child Abort must propagate only through this explicit delegation.
        // Delegation remains a raw nested operation even without a server context.
        inv.delegate("abort", req, res).await
    }
    /// Require a missing capability through the typed delegation error path.
    async fn delegate_missing(
        req: &mut Request,
        res: &mut Response,
        inv: Invocation<'_>,
    ) -> Result<Flow> {
        // Do not silently treat NoListener as a completed delegated capability.
        inv.delegate("absent", req, res).await
    }
    /// Check every typed lookup against an honestly context-free lib invocation.
    async fn missing_host(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        // None of the server shortcuts may infer a host from untrusted input.
        assert!(inv.host().is_err());
        assert!(inv.config::<Config>().is_err());
        assert!(inv.service::<Profiles>().is_err());
        assert!(inv.shared_service::<Profiles>().is_err());
        assert!(inv.stage().is_err());
        assert!(inv.correlation_id().is_err());
        Ok(Flow::Continue)
    }
    // Use raw lib dispatch to isolate delegation mapping and missing-host errors from lifecycle.
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
    // Missing delegation is a typed 404 source, not an apparently successful no-listener outcome.
    let error = emitter
        .resolve("missing", Nest::new())
        .await
        .expect_err("required capability");
    assert!(
        std::error::Error::source(&error)
            .unwrap()
            .is::<DelegationError>()
    );
    assert_eq!(error.message(), "required capability delegation failed");
    assert_eq!(error.public_view().error, "Not Found");
    assert_eq!(error.code(), 404);
    // Plain lib roots remain valid but provide no trusted server registry/context.
    emitter.resolve("raw", Nest::new()).await?;
    Ok(())
}

/// Distinguish the abort and original-error paths using explicit typed fixture data.
struct AbortStartup(bool);

/// A configuration task may fail/abort after allocating a real service.
async fn fail_or_stop(start: &mut StartupContext) -> Result<Flow> {
    // Allocate the real service first so both Abort and error must release attempted construction.
    configure(start).await?;
    // Select the intended failure path from a typed fixture without hiding construction effects.
    if start.config::<AbortStartup>()?.0 {
        Ok(Flow::Abort)
    } else {
        Err(Error::from_source(std::io::Error::other("startup source")))
    }
}

/// Real selection supplies the error's plugin metadata before asynchronous startup.
fn failed_plugin(app: &mut ServerBuilder) -> Result<()> {
    // The selected plugin owns the failing task's metadata even before asynchronous entry.
    app.on_config(fail_or_stop)
}

/// A suspended task owns its partial state until the build future is dropped.
async fn suspend(start: &mut StartupContext) -> Result<Flow> {
    // Construct a real resource before the pending await exposes build cancellation ownership.
    configure(start).await?;
    std::future::pending().await
}

/// Fail/Abort/cancel paths destroy accumulated services without pretending to roll back effects.
#[tokio::test]
async fn failed_startup_never_publishes_and_cancel_releases() -> Result<()> {
    // Exercise both ordinary failure and cooperative startup Abort against fresh resource owners.
    for abort in [false, true] {
        let drops = Arc::new(AtomicUsize::new(0));
        let mut builder = builder(&drops)?;
        builder.state.config.insert(Arc::new(AbortStartup(abort)))?;
        builder.plugin("failed-plugin", failed_plugin)?;
        // Neither path may return a partially usable runtime or a resumable builder.
        let Err(error) = builder.build().await else {
            panic!("partial host escaped")
        };
        // Retained diagnostics must identify the actual phase, selected plugin and external site.
        assert_eq!(error.stage, StartupStage::Config);
        assert_eq!(error.plugin.as_deref(), Some("failed-plugin"));
        assert!(error.registered.file.ends_with("core_tests.rs"));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    // A separate suspended build proves drop cleanup independently from returned errors.
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
    // Cancellation drops the sole attempted construction owner; no background work finishes it.
    drop(future);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    Ok(())
}

/// Output authority belongs to the adapter, not a replaceable Response instance.
#[tokio::test]
async fn response_replacement_cannot_repeat_native_output() -> Result<()> {
    // Keep the native write counter/guard outside the replaceable handler response.
    let writes = Arc::new(AtomicUsize::new(0));
    let mut guard = OutputGuard::default();
    let mut response = Response::new();
    response.stop()?;
    // A handler can replace a whole value before any native write; it cannot see guard.
    response = Response::new();
    let count = Arc::clone(&writes);
    // The first actual writer entry consumes the adapter's independent authority.
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
    // A fresh Response must not make the second writer closure reachable.
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
    // Dropping the actual pending attempt preserves incomplete delivery in the adapter guard.
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
    // Prepare separate value and native-attempt state before simulating a partial write.
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
    // Replacing buffered state after partial output must not grant another native write.
    response = Response::new();
    assert!(
        guard
            .dispatch(&mut response, |_| async { panic!("partial write retry") })
            .await
            .is_err()
    );
    // Observe both the retained terminal state and the exact accepted prefix length.
    assert_eq!(guard.state(), DeliveryState::Failed);
    assert_eq!(bytes.load(Ordering::SeqCst), 3);

    let mut guard = OutputGuard::default();
    // Explicit suppression consumes a new adapter opportunity without ever calling the writer.
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
    // Whole-value replacement cannot erase this adapter's suppression history.
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
    /// Attempt duplicate insertion without replacing the earlier singleton.
    async fn duplicate(start: &mut StartupContext) -> Result<Flow> {
        // The first value must survive a duplicate-type failure.
        start.register(Arc::new(7_u64))?;
        assert!(start.register(Arc::new(9_u64)).is_err());
        assert_eq!(*start.service::<u64>()?, 7);
        Ok(Flow::Continue)
    }
    /// Attempt a service mutation after Config has already frozen the registry.
    async fn wrong_phase(start: &mut StartupContext) -> Result<Flow> {
        // A later phase must fail before inserting or replacing any service.
        start.register(Arc::new(8_u64))?;
        Ok(Flow::Continue)
    }
    /// Provide a compiled no-op registration function for selection validation.
    fn empty(_: &mut ServerBuilder) -> Result<()> {
        // Selection validation is isolated from any callback side effect.
        Ok(())
    }
    // Invalid/duplicate selection must fail before any compiled callback runs.
    let mut app = ServerBuilder::new();
    assert!(app.plugin("bad\nname", empty).is_err());
    app.plugin("single", empty)?;
    assert!(app.plugin("single", empty).is_err());
    app.on_config(duplicate)?;
    app.on_listen(wrong_phase)?;
    // Config succeeds, then Listen's service write must prevent runtime publication.
    let Err(error) = app.build().await else {
        panic!("late service registration escaped")
    };
    assert_eq!(error.stage, StartupStage::Listen);
    // An independently built empty host reports missing values instead of panicking or inferring
    // them.
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
    // Start from a real accepted layer, then attempt each malformed append against the same owner.
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
    // Rejected aggregate input preserves the earlier accepted sequence and its original value.
    assert_eq!(bounded.entries().count(), 1);
    assert_eq!(layers.entries().count(), 1);
    assert_eq!(layers.entries().next().unwrap().1["profiles"]["limit"], 3);
    Ok(())
}

/// Child entry for scenarios that would deadlock under a hidden global callback lock.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by the M0 host watchdog parent"]
async fn host_child() {
    // Bound cooperative async progress inside the child as well as its external process lifetime.
    tokio::time::timeout(Duration::from_secs(2), host_scenarios())
        .await
        .expect("host async deadline")
        .expect("host context proof");
}

/// A parent outside the executor can kill and reap even a synchronously blocked child.
#[test]
fn host_context_and_wrapper_proof_under_watchdog() {
    // Launch only the ignored child marker, avoiding recursively launching this parent test.
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command.args([
        "--ignored",
        "--exact",
        "core_tests::host_child",
        "--nocapture",
    ]);
    // The parent can kill and reap this exact child even if its executor is synchronously blocked.
    let result = watchdog::run(&mut command, "host-context", Duration::from_secs(10))
        .expect("launch/reap host child");
    // A successful proof requires normal child completion, not merely successful watchdog cleanup.
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
