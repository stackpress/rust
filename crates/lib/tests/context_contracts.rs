//! M0 context inheritance and external sealed-supertrait composition proofs.

use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use stackpress_lib::queue::{TaskHandler, TaskQueue, task};
use stackpress_lib::{
    DispatchOutcome, EmitterBuilder, Flow, Handler, HandlerFuture, Invocation, Nest, Request,
    Response, Result, handler,
};

#[path = "support/watchdog.rs"]
mod watchdog;

/// Immutable fixture owned outside every dispatch, including isolated child resolves.
struct Host {
    /// Rendezvous proves children really overlap without a framework callback lock.
    barrier: tokio::sync::Barrier,
    /// A real owned string is borrowed across suspension rather than copied into data.
    label: String,
}

/// Externally compose a Handler using its existing sealed supertrait method.
fn invoke<'a, H: Handler>(
    action: &'a H,
    req: &'a mut Request,
    res: &'a mut Response,
    inv: Invocation<'a>,
) -> HandlerFuture<'a, Result<Flow>> {
    // This test crate cannot name the private trait; its public bound must suffice.
    action.invoke(req, res, inv)
}

/// Externally compose a TaskHandler without adding a public forwarding API.
fn invoke_task<'a, H: TaskHandler<u64>>(
    action: &'a H,
    state: &'a mut u64,
) -> HandlerFuture<'a, Result<Flow>> {
    // The inherited method borrows exactly the callback and mutable state lifetime.
    action.run(state)
}

/// Borrow host state over a rendezvous, then return a JSON-safe value.
async fn leaf(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // A missing or wrong-typed state is not inferred from request data.
    let host = inv.context::<Host>().expect("explicit host");
    assert!(inv.context::<String>().is_none());
    let label = host.label.as_str();
    host.barrier.wait().await;
    assert_eq!(label, host.label);
    res.results(&label)?;
    Ok(Flow::Continue)
}

/// Await concurrent isolated resolves while retaining the parent's original borrow.
async fn parent(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Child calls share immutable host identity but own distinct request/response pairs.
    let borrowed = inv.context::<Host>().expect("host");
    let before = inv.path();
    let (left, right) = tokio::try_join!(
        inv.resolve("leaf", Nest::new()),
        inv.resolve("leaf", Nest::new()),
    )?;
    assert_eq!(left.response.results.unwrap().as_str(), Some("borrowed"));
    assert_eq!(right.response.results.unwrap().as_str(), Some("borrowed"));
    assert!(std::ptr::eq(borrowed, inv.context::<Host>().unwrap()));
    assert_eq!(inv.path()[0].event, before[0].event);
    req.data.set("returned", true)?;
    res.results(&true)?;
    Ok(Flow::Continue)
}

/// Named queue callback used by the external composition probe.
async fn increment(state: &mut u64) -> Result<Flow> {
    // Keep a real state borrow alive across await under the same Send adaptation.
    tokio::task::yield_now().await;
    *state += 1;
    Ok(Flow::Continue)
}

/// Run context, raw-root and wrapper scenarios in a killable child.
async fn scenarios() -> Result<()> {
    // Wrap a named callback without unsealing Handler or changing its public API.
    let mut builder = EmitterBuilder::new();
    builder.on("leaf", leaf)?;
    builder.on(
        "parent",
        handler(|req, res, inv| Box::pin(invoke(&parent, req, res, inv))),
    )?;
    builder.on(
        "raw",
        handler(|_, _, inv| {
            Box::pin(async move {
                assert!(inv.context::<Host>().is_none());
                Ok(Flow::Continue)
            })
        }),
    )?;
    let emitter = Arc::new(builder.build());
    let host = Host {
        barrier: tokio::sync::Barrier::new(2),
        label: "borrowed".into(),
    };
    let mut req = Request::new();
    let mut res = Response::new();
    let future = emitter.emit_with_context("parent", &mut req, &mut res, &host);
    require_send(&future);
    assert_eq!(future.await?, DispatchOutcome::Completed);
    assert_eq!(
        req.data.get("returned").and_then(|v| v.as_bool()),
        Some(true)
    );

    // A fresh raw root does not inherit state from a preceding contextual root.
    emitter.emit("raw", &mut req, &mut res).await?;
    let table = Arc::clone(&emitter);
    tokio::spawn(async move {
        let host = Host {
            barrier: tokio::sync::Barrier::new(2),
            label: "borrowed".into(),
        };
        table
            .resolve_with_context("parent", Nest::new(), &host)
            .await
    })
    .await
    .expect("owned outer task")?;

    // Existing TaskHandler is also externally callable; no forwarding is necessary.
    let mut queue = TaskQueue::new();
    queue.add(task(|state| Box::pin(invoke_task(&increment, state))), 0)?;
    let mut count = 0;
    queue.run(&mut count).await?;
    assert_eq!(count, 1);
    Ok(())
}

/// Assert the concrete future's Send bound at compile time.
fn require_send<T: Send>(_: &T) {
    // A Send callable alone cannot prove that its returned future is Send.
}

/// Internal child entry; the parent supplies the external wall-clock deadline.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run by the context watchdog parent"]
async fn context_child() {
    // An async timeout diagnoses ordinary stalls; the parent also catches blocked threads.
    tokio::time::timeout(Duration::from_secs(2), scenarios())
        .await
        .expect("context async deadline")
        .expect("context scenarios");
}

/// Kill and reap a blocked proof rather than trusting an executor timer alone.
#[test]
fn borrowed_context_and_existing_external_wrappers() {
    // Reuse the already exercised external watchdog with an exact child marker.
    let mut command = Command::new(std::env::current_exe().expect("test executable"));
    command.args(["--ignored", "--exact", "context_child", "--nocapture"]);
    let result = watchdog::run(&mut command, "context", Duration::from_secs(10))
        .expect("launch/reap context child");
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
