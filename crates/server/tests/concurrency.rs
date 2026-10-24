//! External-watchdog proofs for reentrancy, concurrent metadata, cancellation and panic safety.

use stackpress_lib::queue::task;
use stackpress_lib::{Error, Flow, Invocation, Nest, Request, Response, Result};
use stackpress_server::{
    HandleOptions, LifecycleStage as Stage, ServerBuilder, ServerInvocationExt, StartupContext,
    StartupSpec, StartupStage,
};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

#[path = "../../lib/tests/support/watchdog.rs"]
mod watchdog;

/// Shared resource has no framework lock; four concurrent leaves must actually rendezvous.
struct Probe {
    barrier: tokio::sync::Barrier,
    drops: Arc<AtomicUsize>,
}
impl Drop for Probe {
    /// Observe release independently from any host/emitter owner.
    fn drop(&mut self) {
        self.drops.fetch_add(1, Ordering::SeqCst);
    }
}

/// Leaf retains borrowed service, host identity, parent metadata and child ancestry across await.
async fn leaf(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let host = inv.host()?;
    let id = inv.correlation_id()?.unwrap().to_owned();
    let probe = inv.service::<Probe>()?;
    probe.barrier.wait().await;
    assert!(std::ptr::eq(host, inv.host()?));
    assert_eq!(inv.correlation_id()?, Some(id.as_str()));
    assert_eq!(inv.stage()?, Stage::Operation);
    assert_eq!(
        inv.path()
            .iter()
            .map(|f| f.event.as_str())
            .collect::<Vec<_>>(),
        ["parent", "child", "leaf"]
    );
    res.results(&id)?;
    Ok(Flow::Continue)
}
/// Grandchild delegation must not reacquire a registry lock or begin a new lifecycle.
async fn child(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("leaf", req, res).await
}
/// Concurrent resolves have independent request/response/captures and one inherited root ID.
async fn parent(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let id = inv.correlation_id()?.unwrap().to_owned();
    let (a, b) = tokio::try_join!(
        inv.resolve("child", Nest::new()),
        inv.resolve("child", Nest::new())
    )?;
    for result in [a, b] {
        assert_eq!(result.response.results.unwrap().as_str(), Some(id.as_str()));
    }
    assert_eq!(inv.event(), "parent");
    res.results(&id)?;
    Ok(Flow::Continue)
}
/// Same-event bounded recursion mutates only its explicit request and preserves parent ancestry.
async fn recursive(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let depth = req.data.path_as::<u64>("depth")?.unwrap_or(0);
    if depth < 3 {
        req.data.set("depth", depth + 1)?;
        inv.emit("recursive", req, res).await?;
    }
    assert_eq!(inv.path().len(), depth as usize + 1);
    res.results(&true)?;
    Ok(Flow::Continue)
}
/// Nested failure must retain its deepest original listener path.
async fn explode(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    tokio::task::yield_now().await;
    Err(Error::new("private nested failure"))
}
/// Parent propagates a child's error without a second automatic lifecycle.
async fn failing(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    inv.delegate("explode", req, res).await
}
/// Error hook performs nested work while its original immutable failure context remains stable.
async fn recover(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let failure = inv.failure()?.unwrap();
    let original = Arc::clone(&failure.error);
    inv.emit("error-child", req, res).await?;
    assert_eq!(inv.stage()?, Stage::Error);
    assert!(Arc::ptr_eq(&inv.failure()?.unwrap().error, &original));
    *res = Response::new();
    res.results(&true)?;
    Ok(Flow::Continue)
}
/// Nested error work inherits Error as its active stage and the original operation failure.
async fn error_child(_: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    assert_eq!(inv.stage()?, Stage::Error);
    assert_eq!(inv.failure()?.unwrap().stage, Stage::Operation);
    tokio::task::yield_now().await;
    Ok(Flow::Continue)
}
/// Ordinary successful operation for independent post-cancellation/post-panic invocations.
async fn success(_: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    res.results(&true)?;
    Ok(Flow::Continue)
}
/// Panic propagates out of the host without a poisoned framework registry.
async fn panics(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    panic!("expected application panic")
}
/// Pending work exposes completed mutations, but no callback runs after its future is dropped.
async fn pending(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    req.data.set("entered", true)?;
    res.results(&"prefix")?;
    std::future::pending().await
}

/// Run all deadlock-class production-path scenarios inside an independently killable process.
async fn scenarios() -> Result<()> {
    let drops = Arc::new(AtomicUsize::new(0));
    let service = Arc::new(Probe {
        barrier: tokio::sync::Barrier::new(4),
        drops: Arc::clone(&drops),
    });
    let weak = Arc::downgrade(&service);
    let mut app = ServerBuilder::new();
    app.correlation_prefix("test-host")?;
    assert!(app.correlation_prefix("bad\n").is_err());
    app.on_config(task(move |start: &mut StartupContext| {
        let service = Arc::clone(&service);
        Box::pin(async move {
            start.register(service)?;
            Ok(Flow::Continue)
        })
    }))?;
    app.on("parent", parent)?;
    app.on("child", child)?;
    app.on("leaf", leaf)?;
    app.on("recursive", recursive)?;
    app.on("failing", failing)?;
    app.on("explode", explode)?;
    app.on("error", recover)?;
    app.on("error-child", error_child)?;
    app.on("success", success)?;
    app.on("panic", panics)?;
    app.on("pending", pending)?;
    let app = Arc::new(app.build().await.map_err(|e| e.error)?);
    let mut roots = Vec::new();
    for _ in 0..2 {
        let app = Arc::clone(&app);
        roots.push(tokio::spawn(async move {
            app.call("parent", Nest::new()).await
        }));
    }
    let mut ids = Vec::new();
    for root in roots {
        ids.push(root.await.unwrap()?.report.correlation_id);
    }
    assert_ne!(ids[0], ids[1]);
    assert!(ids[0].as_str().starts_with("test-host:"));
    app.call("recursive", Nest::new()).await?;
    let reply = app.call("failing", Nest::new()).await?;
    assert_eq!(
        reply.report.failures[0]
            .error
            .frames()
            .iter()
            .map(|f| f.event.as_str())
            .collect::<Vec<_>>(),
        ["failing", "explode"]
    );
    assert!(reply.report.is_success(&reply.response));
    // Many independently spawned roots share only the explicitly Arc-owned immutable host.
    let mut tasks = Vec::new();
    for _ in 0..64 {
        let app = Arc::clone(&app);
        tasks.push(tokio::spawn(async move {
            app.call("success", Nest::new()).await
        }));
    }
    let mut unique = std::collections::HashSet::new();
    for task in tasks {
        assert!(unique.insert(task.await.unwrap()?.report.correlation_id));
    }
    let task_app = Arc::clone(&app);
    assert!(
        tokio::spawn(async move { task_app.call("panic", Nest::new()).await })
            .await
            .unwrap_err()
            .is_panic()
    );
    assert!(
        app.call("success", Nest::new())
            .await?
            .report
            .failures
            .is_empty()
    );
    let mut req = Request::new();
    let mut res = Response::new();
    let mut future = Box::pin(app.handle("pending", &mut req, &mut res));
    std::future::poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    drop(future);
    assert_eq!(req.data.path_as::<bool>("entered")?, Some(true));
    assert_eq!(res.result().unwrap().as_str(), Some("prefix"));
    assert!(
        app.call("success", Nest::new())
            .await?
            .report
            .failures
            .is_empty()
    );
    drop(app);
    assert!(weak.upgrade().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    handling_cancellation().await?;
    startup_cancellation().await?;
    Ok(())
}

/// Suspend only the explicitly marked call, allowing a subsequent full lifecycle to succeed.
async fn maybe_pending(req: &mut Request, res: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    if req.data.path_as::<bool>("cancel")?.unwrap_or(false) {
        req.data.set("entered", true)?;
        std::future::pending::<()>().await;
    }
    res.results(&true)?;
    Ok(Flow::Continue)
}

/// Fail without a preliminary suspension so the cancellation seam reaches Error itself.
async fn immediate_failure(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
    Err(Error::new("immediate operation failure"))
}

/// Every lifecycle await can be cancelled without later hooks or retained adapter ownership.
async fn handling_cancellation() -> Result<()> {
    for stage_event in ["request", "work", "response", "error"] {
        let mut app = ServerBuilder::new();
        app.on(stage_event, maybe_pending)?;
        if stage_event == "error" {
            app.on("work", immediate_failure)?;
        } else if stage_event != "work" {
            app.on("work", success)?;
        }
        app.on("success", success)?;
        let app = app.build().await.map_err(|e| e.error)?;
        let drops = Arc::new(AtomicUsize::new(0));
        let adapter = Arc::new(Probe {
            barrier: tokio::sync::Barrier::new(1),
            drops: Arc::clone(&drops),
        });
        let mut req = Request::builder()
            .data(serde_json::json!({"cancel":true}))
            .build()?;
        let mut res = Response::new();
        let mut future = Box::pin(app.handle_with_options(
            "work",
            &mut req,
            &mut res,
            HandleOptions {
                adapter_context: Some(adapter),
            },
        ));
        // Poll to the actual handler suspension; no timer guess establishes the interleaving.
        std::future::poll_fn(|cx| {
            assert!(future.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(future);
        assert_eq!(req.data.path_as::<bool>("entered")?, Some(true));
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert!(
            app.call("success", Nest::new())
                .await?
                .report
                .failures
                .is_empty()
        );
    }
    Ok(())
}

/// Cancellation in every startup phase drops the consumed resource owner without replay.
async fn startup_cancellation() -> Result<()> {
    for stage in [
        StartupStage::Config,
        StartupStage::Listen,
        StartupStage::Route,
    ] {
        let drops = Arc::new(AtomicUsize::new(0));
        let probe = Arc::new(Probe {
            barrier: tokio::sync::Barrier::new(1),
            drops: Arc::clone(&drops),
        });
        let mut app = ServerBuilder::new();
        app.on_config(task(move |start: &mut StartupContext| {
            let probe = Arc::clone(&probe);
            Box::pin(async move {
                start.register(probe)?;
                Ok(Flow::Continue)
            })
        }))?;
        app.startup(
            StartupSpec::new(stage).priority(-1),
            task(|_: &mut StartupContext| Box::pin(std::future::pending())),
        )?;
        let mut future = Box::pin(app.build());
        std::future::poll_fn(|cx| {
            assert!(future.as_mut().poll(cx).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        drop(future);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
    // A validator panic likewise propagates without framework-global poisoning.
    let mut validator_app = ServerBuilder::new();
    validator_app.config_section("panic", 1_u64, panic_validator)?;
    assert!(
        tokio::spawn(validator_app.build())
            .await
            .err()
            .expect("validator panic")
            .is_panic()
    );
    // A panicking startup callback cannot poison an unrelated fresh builder.
    let mut app = ServerBuilder::new();
    app.on_config(task(|_: &mut StartupContext| {
        Box::pin(async { panic!("startup panic") })
    }))?;
    assert!(
        tokio::spawn(app.build())
            .await
            .err()
            .expect("startup panic")
            .is_panic()
    );
    ServerBuilder::new().build().await.map_err(|e| e.error)?;
    Ok(())
}

/// User validator panics are not caught or relabeled as ordinary validation errors.
fn panic_validator(_: &u64) -> Result<()> {
    panic!("validator panic")
}

/// Child has a cooperative async bound in addition to the parent's external deadline.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "launched only by external watchdog parent"]
async fn concurrency_child() {
    tokio::time::timeout(Duration::from_secs(2), scenarios())
        .await
        .expect("async deadline")
        .expect("production host concurrency scenarios");
}

/// A blocked executor cannot prevent this parent from killing and reaping the exact child.
#[test]
fn concurrent_nested_cancel_and_panic_under_watchdog() {
    let mut command = Command::new(std::env::current_exe().unwrap());
    command.args(["--ignored", "--exact", "concurrency_child", "--nocapture"]);
    let completion =
        watchdog::run(&mut command, "host-lifecycle", Duration::from_secs(10)).unwrap();
    assert!(!completion.timed_out, "{}", completion.output);
    assert!(completion.status.success(), "{}", completion.output);
}
