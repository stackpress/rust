//! M0 compile/runtime evidence for three-argument Send async handlers.
//!
//! These scenarios now run against the production values and emitter.
//! Only the receipt helpers below are test-specific; the M0 toy registry is gone.

use std::process::Command;
use std::sync::Arc;
use std::time::Duration;

use stackpress_lib::{DispatchOutcome, Error, Flow};

#[path = "support/signature.rs"]
mod signature;
#[path = "support/watchdog.rs"]
mod watchdog;

use signature::{EmitterBuilder, EventEmitter, Invocation, Request, Response, Result, handler};

/// Compile-time assertion about a value's real returned future type.
fn require_send<T: Send>(_: &T) {
    // The generic bound is the proof: no runtime polling or assertion can substitute for future
    // Send.
}

/// Compile-time assertion that immutable registrations may be shared across threads.
fn require_send_sync<T: Send + Sync>() {
    // Compile the table's sharing contract without creating a hidden Clone alias.
}

/// Read numeric test state through the public typed dot-path API.
fn value(req: &Request) -> Result<usize> {
    // Use public typed data access so the signature proof depends on production request values.

    Ok(req.data.path_as("value")?.unwrap_or(0))
}
/// Snapshot ordinary request receipts without exposing internal storage.
fn request_steps(req: &Request) -> Result<Vec<String>> {
    // Decode only ordinary receipt data; no fixture storage bypasses the real Nest API.

    Ok(req.data.path_as("steps")?.unwrap_or_default())
}
/// Append one request receipt through the production value API.
fn record_request(req: &mut Request, step: String) -> Result<()> {
    // Replace the ordinary receipt list after a checked typed read and serialization.

    let mut steps = request_steps(req)?;
    steps.push(step);
    req.data.set(
        "steps",
        stackpress_lib::Value::try_from(serde_json::to_value(steps)?)?,
    )?;
    Ok(())
}
/// Snapshot response receipts from the structured result.
fn response_steps(res: &Response) -> Result<Vec<String>> {
    // Absent structured output has an empty receipt list; malformed output remains an error.

    res.result()
        .map(|value| Ok(serde_json::from_value(value.to_json()?)?))
        .unwrap_or(Ok(Vec::new()))
}
/// Append a response receipt using Serialize-based results.
fn record_response(res: &mut Response, step: String) -> Result<()> {
    // Typed Serialize results exercise the same response mutation boundary as application handlers.

    let mut steps = response_steps(res)?;
    steps.push(step);
    res.results(&steps)?;
    Ok(())
}
/// Render root-first production diagnostic frames for receipt assertions.
fn path(invocation: &Invocation<'_>) -> String {
    // Materialize ancestry only for assertions, preserving its public root-first order.

    invocation
        .path()
        .iter()
        .map(|frame| frame.event.as_str())
        .collect::<Vec<_>>()
        .join(" > ")
}

/// Named handler mutating both arguments after an actual executor suspension.
async fn leaf(req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    // Retain the borrowed inputs across a real yield rather than an always-ready await.
    tokio::task::yield_now().await;
    let increment: usize = "2".parse()?;
    req.data.set("value", (value(req)? + increment) as u64)?;
    record_request(req, path(&invocation))?;
    record_response(res, format!("{}:{}", invocation.event(), value(req)?))?;
    Ok(Flow::Continue)
}

/// Named parent proves nested emit and reuse of its borrowed inputs after await.
async fn parent(req: &mut Request, res: &mut Response, invocation: Invocation<'_>) -> Result<Flow> {
    // Child and parent use the same values sequentially, without a mutex/alias.
    req.data.set("value", (value(req)? + 1) as u64)?;
    let child = invocation.emit("leaf", req, res).await?;
    assert_eq!(child, DispatchOutcome::Completed);
    assert_eq!(invocation.event(), "parent");
    assert_eq!(path(&invocation), "parent");

    // The parent's metadata remains intact after its child suspended and returned.
    record_request(req, invocation.event().to_owned())?;
    record_response(res, "parent returned".to_owned())?;
    Ok(Flow::Continue)
}

/// Fail after a suspension so nested named-handler propagation exercises `?`.
async fn failing(
    req: &mut Request,
    res: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    // Observable prefix mutations must survive the later ordinary failure.
    record_request(req, path(&invocation))?;
    record_response(res, "before error".to_owned())?;
    tokio::task::yield_now().await;
    let _: usize = "invalid number".parse()?;
    Ok(Flow::Continue)
}

/// Named parent propagates a child error without entering its trailing mutation.
async fn failing_parent(
    req: &mut Request,
    res: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    // The child future must be Send even when it returns an error after awaiting.
    invocation.emit("failing", req, res).await?;
    record_response(res, "unreachable after error".to_owned())?;
    Ok(Flow::Continue)
}

/// Run two nested invocations concurrently with distinct mutable value pairs.
async fn concurrent_parent(
    req: &mut Request,
    res: &mut Response,
    invocation: Invocation<'_>,
) -> Result<Flow> {
    // Each child owns its request/response; only the immutable Invocation is shared.
    let mut left_req = Request::default();
    let mut left_res = Response::default();
    let mut right_req = Request::new();
    right_req.data.set("value", 10)?;
    let mut right_res = Response::default();
    let (left, right) = tokio::join!(
        invocation.emit("leaf", &mut left_req, &mut left_res),
        invocation.emit("leaf", &mut right_req, &mut right_res),
    );
    assert_eq!(left?, DispatchOutcome::Completed);
    assert_eq!(right?, DispatchOutcome::Completed);

    // Copy only ordinary receipt data, preserving separate child state and paths.
    assert_eq!(value(&left_req)?, 2);
    assert_eq!(value(&right_req)?, 12);
    assert_eq!(request_steps(&left_req)?, ["concurrent > leaf"]);
    assert_eq!(request_steps(&right_req)?, ["concurrent > leaf"]);
    record_request(req, invocation.event().to_owned())?;
    let mut combined = response_steps(&left_res)?;
    combined.extend(response_steps(&right_res)?);
    res.results(&combined)?;
    Ok(Flow::Continue)
}

/// Compose named functions and a captured-configuration closure without macros.
fn make_emitter() -> Result<EventEmitter> {
    // Compose both primary named handlers and the secondary captured-config helper against
    // production types.

    let mut builder = EmitterBuilder::new();
    builder.on("leaf", leaf)?;
    builder.on("parent", parent)?;
    builder.on("failing", failing)?;
    builder.on("failing-parent", failing_parent)?;
    builder.on("concurrent", concurrent_parent)?;

    // Deep-copy this owned string per call; no implicit shared mutable capture.
    let configured_label = String::from("configured closure");
    builder.on(
        "closure",
        handler(move |req, res, invocation| {
            let label = configured_label.clone();
            Box::pin(async move {
                // A helper closure can await nested dispatch with all three arguments.
                let child = invocation.emit("leaf", req, res).await?;
                assert_eq!(child, DispatchOutcome::Completed);
                assert_eq!(invocation.event(), "closure");
                record_request(req, label.clone())?;
                record_response(res, label)?;
                Ok(Flow::Continue)
            })
        }),
    )?;
    Ok(builder.build())
}

/// Exercise positive scenarios under both executor and external wall-clock bounds.
async fn run_signature_scenarios() -> Result<()> {
    // Keep all concurrency/reentrancy/cancellation scenarios inside one externally bounded child
    // process.

    require_send_sync::<EventEmitter>();
    let emitter = Arc::new(make_emitter()?);

    // Named async functions mutate both values through a nested suspension.
    let mut req = Request::default();
    let mut res = Response::default();
    let future = emitter.emit("parent", &mut req, &mut res);
    require_send(&future);
    assert_eq!(future.await?, DispatchOutcome::Completed);
    assert_eq!(value(&req)?, 3);
    assert_eq!(request_steps(&req)?, ["parent > leaf", "parent"]);
    assert_eq!(response_steps(&res)?, ["leaf:3", "parent returned"]);

    // The helper supplies closure lifetimes and preserves captured configuration.
    let mut req = Request::default();
    let mut res = Response::default();
    assert_eq!(
        emitter.emit("closure", &mut req, &mut res).await?,
        DispatchOutcome::Completed
    );
    assert_eq!(
        request_steps(&req)?,
        ["closure > leaf", "configured closure"]
    );
    assert_eq!(response_steps(&res)?, ["leaf:2", "configured closure"]);

    // Concurrent roots retain the table explicitly through Arc, not registry Clone.
    let named_table = Arc::clone(&emitter);
    let named = tokio::spawn(async move {
        let mut req = Request::default();
        let mut res = Response::default();
        named_table.emit("parent", &mut req, &mut res).await?;
        Ok::<_, Error>((req, res))
    });
    let closure_table = Arc::clone(&emitter);
    let closure = tokio::spawn(async move {
        let mut req = Request::default();
        let mut res = Response::default();
        closure_table.emit("closure", &mut req, &mut res).await?;
        Ok::<_, Error>((req, res))
    });
    let (named_req, named_res) = named.await.map_err(Error::from_source)??;
    let (closure_req, closure_res) = closure.await.map_err(Error::from_source)??;
    assert_eq!(request_steps(&named_req)?, ["parent > leaf", "parent"]);
    assert_eq!(response_steps(&named_res)?, ["leaf:3", "parent returned"]);
    assert_eq!(
        request_steps(&closure_req)?,
        ["closure > leaf", "configured closure"]
    );
    assert_eq!(
        response_steps(&closure_res)?,
        ["leaf:2", "configured closure"]
    );

    // Separate child values allow concurrent nested dispatch without serializing it.
    let mut req = Request::default();
    let mut res = Response::default();
    emitter.emit("concurrent", &mut req, &mut res).await?;
    assert_eq!(request_steps(&req)?, ["concurrent"]);
    assert_eq!(response_steps(&res)?, ["leaf:2", "leaf:12"]);

    // Ordinary errors propagate through named async handlers without losing mutation.
    let mut req = Request::default();
    let mut res = Response::default();
    let error = emitter
        .emit("failing-parent", &mut req, &mut res)
        .await
        .expect_err("a nested parse error must reach the root");
    assert!(
        std::error::Error::source(&error)
            .is_some_and(|source| source.is::<std::num::ParseIntError>())
    );
    assert_eq!(
        error
            .frames()
            .iter()
            .map(|frame| frame.event.as_str())
            .collect::<Vec<_>>(),
        ["failing-parent", "failing"]
    );
    assert_eq!(request_steps(&req)?, ["failing-parent > failing"]);
    assert_eq!(response_steps(&res)?, ["before error"]);

    // A rendezvous would deadlock under a hidden per-handler/global dispatch lock.
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    let mut paired = EmitterBuilder::new();
    paired.on("concurrent", concurrent_parent)?;
    paired.on(
        "leaf",
        handler(move |req, res, inv| {
            let barrier = Arc::clone(&barrier);
            Box::pin(async move {
                barrier.wait().await;
                leaf(req, res, inv).await
            })
        }),
    )?;
    paired
        .build()
        .emit("concurrent", &mut Request::new(), &mut Response::new())
        .await?;

    // Reentrant recursion is bounded before any unbounded allocation or hang.
    async fn recurse(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
        // Emit the same event to prove the per-ancestry depth guard terminates a real handler
        // cycle.

        inv.emit("loop", req, res).await?;
        Ok(Flow::Continue)
    }
    let mut recursive = EmitterBuilder::new();
    recursive.depth_limit(4)?.on("loop", recurse)?;
    let error = recursive
        .build()
        .emit("loop", &mut Request::new(), &mut Response::new())
        .await
        .expect_err("bounded recursion");
    assert_eq!(error.frames().len(), 4);
    assert_eq!(error.kind(), stackpress_lib::ErrorKind::Limit);

    // A panic is not caught by the framework; a fresh invocation remains usable.
    async fn panic_handler(_: &mut Request, _: &mut Response, _: Invocation<'_>) -> Result<Flow> {
        // A deliberate post-yield panic must propagate without poisoning an internal dispatch lock.

        tokio::task::yield_now().await;
        panic!("intentional panic proof")
    }
    let mut panic_builder = EmitterBuilder::new();
    panic_builder.on("panic", panic_handler)?;
    panic_builder.on("leaf", leaf)?;
    let panic_table = Arc::new(panic_builder.build());
    let task_table = Arc::clone(&panic_table);
    let panic = tokio::spawn(async move {
        task_table
            .emit("panic", &mut Request::new(), &mut Response::new())
            .await
    })
    .await
    .expect_err("panic propagates out of framework");
    assert!(panic.is_panic());
    assert_eq!(
        panic_table
            .emit("leaf", &mut Request::new(), &mut Response::new())
            .await?,
        DispatchOutcome::Completed
    );

    // Dropping a pending dispatch stops future callbacks and retains prefix mutation.
    let entered = Arc::new(tokio::sync::Notify::new());
    let signal = Arc::clone(&entered);
    let mut pending = EmitterBuilder::new();
    pending.on(
        "wait",
        handler(move |req, _res, _inv| {
            let signal = Arc::clone(&signal);
            Box::pin(async move {
                req.data.set("started", true)?;
                signal.notify_one();
                std::future::pending::<()>().await;
                Ok(Flow::Continue)
            })
        }),
    )?;
    pending.on("wait", leaf)?;
    let pending = pending.build();
    let mut cancelled_req = Request::new();
    let mut cancelled_res = Response::new();
    let mut future = pending.emit("wait", &mut cancelled_req, &mut cancelled_res);
    tokio::select! {
        _ = entered.notified() => {},
        _ = &mut future => panic!("pending handler completed"),
    }
    drop(future);
    assert_eq!(cancelled_req.data.path_as::<bool>("started")?, Some(true));
    assert!(cancelled_req.data.path("value").is_none());

    // Queue cancellation consumes the selected task and leaves later work available.
    async fn wait_task(state: &mut Vec<&'static str>) -> Result<Flow> {
        // Record entry before never becoming ready, making queue cancellation and consumption
        // observable.

        state.push("entered");
        std::future::pending::<Result<Flow>>().await
    }
    async fn next_task(state: &mut Vec<&'static str>) -> Result<Flow> {
        // This successor may run only after the cancelled selected task has been consumed.

        state.push("next");
        Ok(Flow::Continue)
    }
    let mut tasks = stackpress_lib::queue::TaskQueue::new();
    tasks.add(wait_task, 1)?.add(next_task, 0)?;
    let mut state = Vec::new();
    let future = tasks.run(&mut state);
    require_send(&future);
    let mut future = Box::pin(future);
    // The other branch yields once so the queue enters its never-ready task.
    tokio::select! {
        biased;
        _ = &mut future => panic!("pending task completed"),
        _ = tokio::task::yield_now() => {},
    }
    drop(future);
    assert_eq!(state, ["entered"]);
    assert_eq!(tasks.len(), 1);
    assert_eq!(
        tasks.run(&mut state).await?,
        stackpress_lib::queue::QueueOutcome::Completed
    );
    assert_eq!(state, ["entered", "next"]);

    // A cancelled loader consumes the native attempt without installing snapshot edits.
    let entered = tokio::sync::Notify::new();
    let signal = &entered;
    let mut cancelled_load = Request::new();
    let future = cancelled_load.load_with(move |mut view| async move {
        view.data.set("snapshot-only", true)?;
        signal.notify_one();
        std::future::pending::<Result<stackpress_lib::LoadResult>>().await
    });
    require_send(&future);
    let mut future = Box::pin(future);
    tokio::select! {
        _ = entered.notified() => {},
        _ = &mut future => panic!("pending loader completed"),
    }
    drop(future);
    assert_eq!(
        cancelled_load.load_state(),
        stackpress_lib::LoadState::Attempted
    );
    assert!(cancelled_load.data.path("snapshot-only").is_none());
    assert_eq!(
        cancelled_load
            .load()
            .await
            .expect_err("no native retry")
            .kind(),
        stackpress_lib::ErrorKind::State
    );

    // Output cancellation likewise records the attempt before a pending writer yields.
    let entered = tokio::sync::Notify::new();
    let signal = &entered;
    let mut cancelled_output = Response::new();
    cancelled_output.results(&"buffered")?;
    let future = cancelled_output.dispatch_with(move |mut view| async move {
        view.data.set("snapshot-only", true)?;
        signal.notify_one();
        std::future::pending::<Result<()>>().await
    });
    require_send(&future);
    let mut future = Box::pin(future);
    tokio::select! {
        _ = entered.notified() => {},
        _ = &mut future => panic!("pending dispatcher completed"),
    }
    drop(future);
    assert_eq!(
        cancelled_output.delivery_state(),
        stackpress_lib::DeliveryState::Attempted
    );
    assert!(cancelled_output.data().path("snapshot-only").is_none());
    assert_eq!(
        cancelled_output.result().and_then(|v| v.as_str()),
        Some("buffered")
    );
    assert_eq!(
        cancelled_output
            .dispatch()
            .await
            .expect_err("no native retry")
            .kind(),
        stackpress_lib::ErrorKind::State
    );

    /// Native-reader panics propagate after the unique attempt has already been consumed.
    async fn panic_loader(_: stackpress_lib::RequestView) -> Result<stackpress_lib::LoadResult> {
        // This is immediate user failure; the framework must retain Attempted through unwind.
        panic!("intentional loader panic")
    }
    /// Native-writer panics cannot justify replaying possibly emitted bytes.
    async fn panic_writer(_: stackpress_lib::ResponseView) -> Result<()> {
        // A caller-owned writer is allowed to unwind; the response must remain frozen.
        panic!("intentional writer panic")
    }
    let mut panic_load = Request::new();
    let mut future = Box::pin(panic_load.load_with(panic_loader));
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    // One poll enters the immediate panic; the outer process watchdog covers blocking regressions.
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::future::Future::poll(future.as_mut(), &mut context)
        }))
        .is_err()
    );
    drop(future);
    assert_eq!(
        panic_load.load_state(),
        stackpress_lib::LoadState::Attempted
    );
    assert!(panic_load.load().await.is_err());
    let mut panic_output = Response::new();
    let mut future = Box::pin(panic_output.dispatch_with(panic_writer));
    assert!(
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            std::future::Future::poll(future.as_mut(), &mut context)
        }))
        .is_err()
    );
    drop(future);
    assert_eq!(
        panic_output.delivery_state(),
        stackpress_lib::DeliveryState::Attempted
    );
    assert!(panic_output.dispatch().await.is_err());

    // No listeners remains an ordinary outcome even with Send erasure.
    assert_eq!(
        emitter.emit("missing", &mut req, &mut res).await?,
        DispatchOutcome::NoListener
    );
    Ok(())
}

/// Keep all async scenarios in a child so blocking bugs cannot hang cargo test.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "invoked by the external watchdog test"]
async fn signature_scenarios_child() {
    // The executor deadline catches yielding stalls; the parent watchdog also covers blocked
    // executor threads.

    tokio::time::timeout(Duration::from_secs(2), run_signature_scenarios())
        .await
        .expect("signature scenarios exceeded their executor deadline")
        .expect("signature scenarios failed");
}

/// Run the positive compiler/runtime proof with a 10-second external deadline.
#[test]
fn send_and_nested_handlers_finish_under_watchdog() {
    // Launch the real integration binary so a synchronously blocked child cannot hang cargo test.

    let mut command = Command::new(std::env::current_exe().expect("integration-test executable"));
    command.args([
        "--ignored",
        "--exact",
        "signature_scenarios_child",
        "--nocapture",
    ]);
    let completion = watchdog::run(&mut command, "signature", Duration::from_secs(10))
        .expect("launch/reap signature proof");
    assert!(!completion.timed_out, "{}", completion.output);
    assert!(completion.status.success(), "{}", completion.output);
}

/// Link a UI fixture to this test build's production library, not a toy registry.
fn compiler_command(fixture: &str) -> Command {
    // Link the fixture against this test build's production rlib rather than recompiling a toy
    // adapter.

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("../../target/m0");
    std::fs::create_dir_all(&output).expect("create ignored compiler-output directory");
    let deps = std::env::current_exe()
        .expect("test executable")
        .parent()
        .expect("deps directory")
        .to_owned();
    let rlib = std::fs::read_dir(&deps)
        .expect("built libraries")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name().is_some_and(|name| {
                name.to_string_lossy().starts_with("libstackpress_lib-")
                    && name.to_string_lossy().ends_with(".rlib")
            })
        })
        .max_by_key(|path| path.metadata().and_then(|meta| meta.modified()).ok())
        .expect("production rlib");
    let mut command = Command::new("rustc");
    command
        .arg("--extern")
        .arg(format!("stackpress_lib={}", rlib.display()))
        .arg("-L")
        .arg(format!("dependency={}", deps.display()));
    command
        .args(["--edition=2024", "--crate-type=bin"])
        .arg(root.join("tests/ui").join(fixture))
        .arg("--out-dir")
        .arg(output);
    command
}

/// Compile the same adapter with a non-Send future and require a relevant error.
#[test]
fn registration_rejects_a_non_send_returned_future() {
    // Reject the named function because its returned future retains Rc across await, not because
    // the function item is non-Send.

    let mut command = compiler_command("non_send_handler.rs");
    let completion = watchdog::run(&mut command, "non-send", Duration::from_secs(10))
        .expect("launch/reap compiler rejection proof");
    assert!(!completion.timed_out, "{}", completion.output);
    assert!(
        !completion.status.success(),
        "a non-Send future was accepted"
    );
    assert!(
        completion
            .output
            .contains("cannot be sent between threads safely")
            && completion.output.contains("Rc<()>")
            && completion.output.contains("EmitterBuilder::on"),
        "rejection must concern the returned future's Send bound: {}",
        completion.output
    );
}

/// Prove ownership restrictions with compiler errors against production types.
#[test]
fn stateful_clone_and_concurrent_mutable_aliases_are_rejected() {
    // Require relevant compiler diagnostics so an unrelated syntax failure cannot satisfy an
    // ownership proof.

    for (fixture, scenario, diagnostics) in [
        (
            "stateful_clone.rs",
            "stateful-clone",
            vec!["no method named `clone`", "Request", "Response"],
        ),
        (
            "concurrent_mutable_inputs.rs",
            "mutable-alias",
            vec![
                "cannot borrow `*req` as mutable more than once",
                "cannot borrow `*res` as mutable more than once",
            ],
        ),
        (
            "escaping_invocation.rs",
            "escaping-invocation",
            vec!["lifetime may not live long enough", "'static"],
        ),
        (
            "escaping_context.rs",
            "escaping-context",
            vec!["cannot return value referencing function parameter", "inv"],
        ),
        (
            "non_sync_context.rs",
            "non-sync-context",
            vec!["cannot be shared between threads safely", "Rc<()>"],
        ),
    ] {
        let completion = watchdog::run(
            &mut compiler_command(fixture),
            scenario,
            Duration::from_secs(10),
        )
        .expect("launch/reap ownership compiler proof");
        assert!(!completion.timed_out, "{}", completion.output);
        assert!(
            !completion.status.success(),
            "invalid ownership fixture was accepted"
        );
        for diagnostic in diagnostics {
            assert!(
                completion.output.contains(diagnostic),
                "missing {diagnostic}: {}",
                completion.output
            );
        }
    }
}

/// Simulate a synchronously blocked executor only when a watchdog owns this child.
#[test]
#[ignore = "invoked only by the watchdog self-test"]
fn deliberately_stuck_child() {
    // Running ignored tests manually must not accidentally start an endless child.
    if std::env::var_os("STACKPRESS_M0_WATCHDOG_CHILD").is_none() {
        return;
    }
    loop {
        std::thread::park();
    }
}

/// Demonstrate that the timeout mechanism kills and reaps a non-yielding child.
#[test]
fn watchdog_kills_and_reaps_a_stuck_child() {
    // Self-test the external deadline with a non-yielding child rather than assuming Tokio timeout
    // is sufficient.

    let mut command = Command::new(std::env::current_exe().expect("integration-test executable"));
    command
        .args(["--ignored", "--exact", "deliberately_stuck_child"])
        .env("STACKPRESS_M0_WATCHDOG_CHILD", "1");
    let completion = watchdog::run(&mut command, "watchdog", Duration::from_millis(250))
        .expect("terminate/reap deliberately stuck child");
    assert!(
        completion.timed_out,
        "the stuck child unexpectedly returned"
    );
    assert!(!completion.status.success());
}
