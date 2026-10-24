//! Real host/lifecycle CLI contracts with explicit sinks and a kill/reap watchdog.

use std::ffi::OsString;
use std::future::Future;
use std::io::{self, Write};
use std::process::{Command, ExitCode};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use stackpress_cli::prompt::PromptError;
use stackpress_cli::{
    CliContext, CliRun, CliSinks, ConfigLayers, Terminal, TerminalOptions, main_with,
    main_with_args, run,
};
use stackpress_lib::queue::task;
use stackpress_lib::{
    DeliveryState, Error, Flow, Invocation, ParseLimits, Request, Response, Result, SharedNative,
    Value,
};
use stackpress_server::{
    BuildResult, LifecycleStage, Server, ServerBuilder, ServerInvocationExt, StartupContext,
};

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Application-owned observation state; locks are released before any await/delegation.
#[derive(Default)]
struct Trace(Mutex<Vec<String>>);

/// Successful recovery must not call arbitrary source callbacks merely to decide an exit code.
#[derive(Debug)]
struct SourceMustNotRun;
impl std::fmt::Display for SourceMustNotRun {
    /// Its printable identity is safe; only source traversal is deliberately forbidden.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("recovered source")
    }
}
impl std::error::Error for SourceMustNotRun {
    /// Make an unnecessary traversal after successful public recovery fail observably.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        panic!("recovered error source must not be traversed for exit 0");
    }
}

/// An application-defined cyclic cause chain must never trap executable exit classification.
#[derive(Debug)]
struct CyclicSource(Arc<AtomicUsize>);
impl std::fmt::Display for CyclicSource {
    /// Developer formatting remains identifiable without exposing fixture ownership state.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("cyclic application cause")
    }
}
impl std::error::Error for CyclicSource {
    /// Count traversal and deliberately return the same borrowed error forever.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.fetch_add(1, Ordering::SeqCst);
        Some(self)
    }
}

/// A sink panic belongs to its caller and must not cause hidden writer retries or retention.
struct PanickingWriter(Arc<AtomicUsize>);
impl Write for PanickingWriter {
    /// Record entry before unwinding so any unexpected retry remains observable.
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        self.0.fetch_add(1, Ordering::SeqCst);
        panic!("fixture writer panic");
    }

    /// Any flush would incorrectly continue delivery after the panicking write.
    fn flush(&mut self) -> io::Result<()> {
        panic!("a panicking writer must not be flushed");
    }
}

/// Observe trusted metadata and true stage order without placing observations in request data.
async fn audit(req: &mut Request, _: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Every host stage sees the CLI adapter context and selected terminal MIME.
    assert_eq!(req.mimetype(), "terminal/arguments");
    assert!(inv.adapter_context::<CliContext>().is_some());
    inv.service::<Trace>()?
        .0
        .lock()
        .unwrap()
        .push(inv.event().into());
    Ok(Flow::Continue)
}

/// Execute named fixture behavior through production Handler/Invocation/Response types.
async fn operation(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Count real operation entry so adapter retries cannot hide behind equivalent JSON.
    inv.service::<Trace>()?
        .0
        .lock()
        .unwrap()
        .push(inv.event().into());
    match inv.event() {
        "parent" => {
            // Hold the real shared borrow across nesting; no mutable global context is involved.
            let before = inv.adapter_context::<CliContext>().unwrap();
            assert!(before.verbose && before.force);
            assert_eq!(before.brand, "[app]");
            inv.delegate("child", req, res).await?;
            assert!(std::ptr::eq(
                before,
                inv.adapter_context::<CliContext>().unwrap()
            ));
        }
        "child" => {
            // User fields with operator names cannot replace trusted metadata.
            let context = inv.adapter_context::<CliContext>().unwrap();
            assert!(context.force);
            assert!(context.config.is_none());
            assert_eq!(req.data.path_as::<bool>("force")?, Some(false));
            assert_eq!(req.data.path_as::<u32>("adapter_context")?, Some(0));
            res.results(&req.data)?;
        }
        "fail" | "recover" => {
            // A public message is deliberate; the retained developer cause must stay off stdout.
            return Err(Error::new("PRIVATE operation detail")
                .with_cause(io::Error::other("PRIVATE source cause"))
                .with_code(403)
                .with_public_message("Denied"));
        }
        // A typed prompt source, rather than its display text or numeric code, selects exit 130.
        "cancel" => return Err(Error::from_source(PromptError::Aborted)),
        "public-error" => {
            // Even an empty public message with a 2xx code remains a present application error.
            res.apply_error(
                Error::new("PRIVATE applied error")
                    .with_code(200)
                    .with_public_message(""),
            )?;
        }
        "recovered-source" => return Err(Error::new("recovered").with_cause(SourceMustNotRun)),
        "false-cancel" => return Err(Error::new("prompt was aborted").with_code(130)),
        "abort" => return Ok(Flow::Abort),
        "abort-body" => {
            res.results(&0)?;
            return Ok(Flow::Abort);
        }
        "stopped" => {
            // Explicit success plus suppression is different from an unknown empty response.
            res.status_code(204, None)?.stop()?;
        }
        "sent" => {
            // Capability-owned foundation delivery is foreign to the CLI's private sink.
            res.results(&true)?;
            res.dispatch().await?;
        }
        "attempted" => {
            // Drop a pending foreign dispatcher after its first poll consumed output authority.
            // Drop an entered dispatcher future to reach a real incomplete state.
            let mut future = Box::pin(res.dispatch_with(|_| std::future::pending()));
            std::future::poll_fn(|cx| {
                assert!(future.as_mut().poll(cx).is_pending());
                std::task::Poll::Ready(())
            })
            .await;
            drop(future);
        }
        "failed" => {
            // A failed foreign delivery must never acquire a second attempt from the adapter.
            let _ = res
                .dispatch_with(|_| async { Err(Error::new("foreign writer")) })
                .await;
        }
        "replace" => {
            // Foundation finalization does not grant a second adapter writer: sinks stay private.
            res.results(&"old")?;
            res.dispatch().await?;
            *res = Response::new();
            res.results(&"replacement")?;
        }
        "rich" => {
            // Conversion must fail before entering stdout when a result is not JSON-compatible.
            res.results_value(Value::Shared(SharedNative::new(7_u64)))?;
        }
        "large" => {
            // A budget above default is a real response policy, not a detached envelope default.
            let limits = ParseLimits {
                max_bytes: ParseLimits::default().max_bytes + 1024,
                ..ParseLimits::default()
            };
            *res = Response::with_limits(limits);
            res.results(&"x".repeat(ParseLimits::default().max_bytes + 1))?;
        }
        "redirect" => {
            // G14 accepts final redirects without interpreting them as executable exit codes.
            res.status_code(302, None)?;
        }
        "custom" => {
            // Application codes remain representable on the wire but are unsuccessful here.
            res.status_code(700, None)?;
        }
        "null" => {
            // Explicit null has result presence and must not collapse to an empty response.
            res.results_value(Value::Null)?;
        }
        "false" => {
            // Both falsy results and zero totals survive final envelope conversion.
            res.results(&false)?;
            res.set_total(0)?;
        }
        "empty" => {
            // Empty text remains deliberate output rather than triggering unknown-event logic.
            res.results(&"")?;
        }
        "pending" => std::future::pending::<()>().await,
        "panic" => panic!("intentional CLI handler panic"),
        _ => {
            // Echo parsed target values without reconstructing or reparsing argv.
            res.results(&req.data)?;
        }
    }
    Ok(Flow::Continue)
}

/// Deliberately recover only the selected scenario, retaining original diagnostic history.
async fn recover(_: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Keep evidence of the original error stage even after successful public replacement.
    inv.service::<Trace>()?
        .0
        .lock()
        .unwrap()
        .push("error".into());
    // Replacement clears public error state; merely writing results would not establish recovery.
    *res = Response::with_limits(res.limits());
    res.results(&false)?;
    res.set_total(0)?;
    Ok(Flow::Continue)
}

/// Build a small immutable application and retain its observable stage/capability trace.
async fn app(recovery: bool) -> (Server, Arc<Trace>) {
    // Register an application-owned service through real startup, never a global observation map.
    let trace = Arc::new(Trace::default());
    let service = Arc::clone(&trace);
    let mut builder = ServerBuilder::new();
    builder
        .on_config(task(move |start: &mut StartupContext| {
            let service = Arc::clone(&service);
            Box::pin(async move {
                start.register(service)?;
                Ok(Flow::Continue)
            })
        }))
        .unwrap();
    builder.on("request", audit).unwrap();
    builder.on("response", audit).unwrap();
    // Named fixtures exercise the same immutable host registry used by application plugins.
    for event in [
        "echo",
        "public-error",
        "recovered-source",
        "parent",
        "child",
        "fail",
        "recover",
        "cancel",
        "false-cancel",
        "abort",
        "abort-body",
        "stopped",
        "sent",
        "attempted",
        "failed",
        "replace",
        "rich",
        "large",
        "redirect",
        "custom",
        "null",
        "false",
        "empty",
        "pending",
        "panic",
    ] {
        builder.on(event, operation).unwrap();
    }
    if recovery {
        // Only callers selecting this compiled fixture recover an operation failure.
        builder.on("error", recover).unwrap();
    }
    (builder.build().await.unwrap(), trace)
}

/// Parse caller argv and run against independent buffer sinks without native process I/O.
async fn invoke(app: &Server, args: &[&str]) -> (CliRun, Vec<u8>, Vec<u8>) {
    // Each action owns parsed input and distinct sinks, keeping native process I/O out of tests.
    let args = args.iter().map(|arg| (*arg).into()).collect::<Vec<_>>();
    let terminal = Terminal::parse(&args, ParseLimits::default()).unwrap();
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let result = run(app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    (result, stdout, stderr)
}

/// Assert exactly one machine line and decode its public envelope.
fn json(bytes: &[u8]) -> serde_json::Value {
    // A single raw delimiter proves strings were escaped and no progress/branding leaked.
    assert!(bytes.ends_with(b"\n"));
    assert_eq!(bytes.iter().filter(|byte| **byte == b'\n').count(), 1);
    assert!(!bytes.contains(&0x1b));
    serde_json::from_slice(bytes).unwrap()
}

/// One lifecycle, nested metadata, strict target argv and exact machine/human separation.
async fn success_and_metadata() {
    // Arrange conflicting operator/target keys plus a single argument containing spaces/symbols.
    let (app, trace) = app(false).await;
    let args = [
        "-vf",
        "emit",
        "parent",
        "--force",
        "false",
        "--adapter_context",
        "0",
        "--name",
        "Ada & Grace = two words",
    ]
    .map(String::from);
    let terminal = Terminal::parse_with_options(
        &args,
        ParseLimits::default(),
        TerminalOptions {
            brand: "[app]".into(),
            ..Default::default()
        },
    )
    .unwrap();
    // Execute through the real host with branded human output and independent machine bytes.
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let result = run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    let wire = json(&stdout);
    // Nested context stays trusted while target values and one top-level lifecycle are preserved.
    assert_eq!(result.exit_code, 0);
    assert_eq!(wire["results"]["force"], false);
    assert_eq!(wire["results"]["name"], "Ada & Grace = two words");
    assert_eq!(
        *trace.0.lock().unwrap(),
        ["request", "parent", "child", "response"]
    );
    assert_eq!(
        result
            .report
            .stages
            .iter()
            .filter(|s| s.stage == LifecycleStage::Input)
            .count(),
        1
    );
    assert_eq!(
        result.report.stages.last().unwrap().delivery_state,
        DeliveryState::Sent
    );
    assert!(!stdout.windows(5).any(|part| part == b"[app]"));
    assert!(
        String::from_utf8(stderr)
            .unwrap()
            .starts_with("[app] call ")
    );
}

/// Status/results presence, unknown/application errors, deliberate recovery, Abort and suppression.
async fn lifecycle_results() {
    // Verify presence independently from truthiness before testing status/error exit policy.
    let (application, _) = app(false).await;
    for (event, expected) in [
        ("false", serde_json::json!(false)),
        ("null", serde_json::Value::Null),
        ("empty", serde_json::json!("")),
        ("abort-body", serde_json::json!(0)),
    ] {
        let (result, stdout, stderr) = invoke(&application, &["emit", event]).await;
        assert_eq!(result.exit_code, 0);
        assert_eq!(json(&stdout)["results"], expected);
        assert!(stderr.is_empty());
        if event == "false" {
            assert_eq!(json(&stdout)["total"], 0);
        }
    }
    for (event, code, exit) in [
        ("unknown", 404, 1),
        ("fail", 403, 1),
        ("abort", 204, 0),
        ("redirect", 302, 0),
        ("custom", 700, 1),
    ] {
        // Every ordinary Open response emits one envelope, including failure/unknown outcomes.
        let (result, stdout, stderr) = invoke(&application, &["emit", event]).await;
        assert_eq!(result.exit_code, exit);
        assert_eq!(json(&stdout)["code"], code);
        assert!(!String::from_utf8_lossy(&stdout).contains("PRIVATE"));
        assert!(!String::from_utf8_lossy(&stderr).contains("PRIVATE"));
        assert_eq!(!stderr.is_empty(), exit != 0);
    }
    let (result, stdout, _) = invoke(&application, &["-v", "emit", "fail"]).await;
    // Verbosity changes only developer stderr; stdout contains solely intentional public data.
    assert_eq!(result.exit_code, 1);
    assert_eq!(json(&stdout)["error"], "Denied");
    let (_, _, detail) = invoke(&application, &["-v", "emit", "fail"]).await;
    assert!(
        String::from_utf8(detail)
            .unwrap()
            .contains("PRIVATE source cause")
    );
    let (recovered, _) = app(true).await;
    // Successful replacement keeps diagnostic history while clearing the public failure state.
    let (result, stdout, stderr) = invoke(&recovered, &["emit", "recover"]).await;
    assert_eq!(result.exit_code, 0);
    assert!(!result.report.failures.is_empty());
    assert_eq!(json(&stdout)["results"], false);
    assert_eq!(json(&stdout)["total"], 0);
    assert!(stderr.is_empty());
    let (result, stdout, stderr) = invoke(&application, &["emit", "stopped"]).await;
    assert_eq!(result.exit_code, 0);
    assert!(stdout.is_empty() && stderr.is_empty());
    assert_eq!(
        result.report.stages.last().unwrap().delivery_state,
        DeliveryState::Stopped
    );
}

/// Public error presence and target verbose data cannot imply success or developer disclosure.
async fn public_error_and_diagnostic_failures() {
    let (application, _) = app(false).await;
    let (result, stdout, _) = invoke(&application, &["emit", "public-error"]).await;
    assert_eq!(result.exit_code, 1);
    assert_eq!(json(&stdout)["error"], "");
    assert_eq!(json(&stdout)["code"], 200);
    let (_, stdout, detail) = invoke(&application, &["-v", "emit", "public-error"]).await;
    assert!(!String::from_utf8_lossy(&stdout).contains("PRIVATE"));
    assert!(
        String::from_utf8(detail)
            .unwrap()
            .contains("PRIVATE applied error")
    );
    let (_, _, detail) = invoke(&application, &["emit", "fail", "--verbose", "true"]).await;
    assert!(!String::from_utf8(detail).unwrap().contains("PRIVATE"));

    // A stderr error after a successful stdout write is terminal, with no second stdout attempt.
    let mut stdout = Writer::new(usize::MAX);
    let mut stderr = Writer::new(0);
    let terminal = Terminal::parse(
        &["-v".into(), "emit".into(), "echo".into()],
        Default::default(),
    )
    .unwrap();
    let result = run(
        &application,
        terminal,
        &mut CliSinks::new(&mut stdout, &mut stderr),
    )
    .await
    .unwrap();
    assert_eq!(result.exit_code, 1);
    assert!(result.report.terminal_failure);
    assert_eq!(stdout.calls, 1);
    assert_eq!(stderr.calls, 1);
    json(&stdout.bytes);

    // Output failure outranks typed prompt cancellation at the executable boundary.
    let (application, _) = app(false).await;
    let mut stdout = Writer::new(0);
    let mut stderr = Vec::new();
    let args = [OsString::from("emit"), OsString::from("cancel")];
    let exit = main_with_args(
        &args,
        &mut CliSinks::new(&mut stdout, &mut stderr),
        move |_| async move { Ok(application) },
    )
    .await;
    assert_eq!(exit, ExitCode::from(1));
    assert_eq!(stdout.calls, 1);
}

/// Stateful writer observations distinguish one attempt from retries after a partial/broken sink.
struct Writer {
    /// Accepted bytes remain observable even when the next write fails.
    bytes: Vec<u8>,
    /// Remaining acceptance budget; zero fails all further writes.
    remaining: usize,
    /// Physical calls expose accidental retries even when they accept no bytes.
    calls: usize,
    /// Independently fail after write success to test final flush handling.
    fail_flush: bool,
}
impl Writer {
    /// Construct an ordinary or deliberately partial sink.
    fn new(remaining: usize) -> Self {
        Self {
            bytes: Vec::new(),
            remaining,
            calls: 0,
            fail_flush: false,
        }
    }
}
impl Write for Writer {
    /// Record each actual entry; allow a prefix and then return the original OS failure kind.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A finite acceptance budget distinguishes one failed attempt from a restarted envelope.
        self.calls += 1;
        if self.remaining == 0 {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        let length = bytes.len().min(self.remaining);
        self.bytes.extend_from_slice(&bytes[..length]);
        self.remaining -= length;
        Ok(length)
    }
    /// Flush failures are delivery failures even after all JSON bytes were accepted.
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush {
            Err(io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(())
        }
    }
}

/// Output authority survives response replacement, foreign finalization and partial/sink failures.
async fn delivery_results() {
    // Foreign finalization and conversion failure must produce no physical adapter write.
    let (app, trace) = app(false).await;
    for event in ["sent", "attempted", "failed", "rich"] {
        let (result, stdout, _) = invoke(&app, &["emit", event]).await;
        assert_eq!(result.exit_code, 1);
        assert!(result.report.terminal_failure);
        assert!(stdout.is_empty());
        assert!(
            result
                .report
                .failures
                .iter()
                .any(|f| f.stage == LifecycleStage::Output)
        );
    }
    let mut stdout = Writer::new(usize::MAX);
    let mut stderr = Vec::new();
    let terminal = Terminal::parse(&["emit".into(), "replace".into()], Default::default()).unwrap();
    let result = run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
        .await
        .unwrap();
    assert_eq!(result.exit_code, 0);
    assert_eq!(stdout.calls, 1);
    assert_eq!(json(&stdout.bytes)["results"], "replacement");
    // Zero-byte, partial-byte and flush failures each preserve their original OS cause.
    for (remaining, flush, expected_calls) in [(0, false, 1), (3, false, 2), (usize::MAX, true, 1)]
    {
        let mut stdout = Writer::new(remaining);
        stdout.fail_flush = flush;
        let terminal =
            Terminal::parse(&["emit".into(), "echo".into()], Default::default()).unwrap();
        let result = run(&app, terminal, &mut CliSinks::new(&mut stdout, &mut stderr))
            .await
            .unwrap();
        assert_eq!(result.exit_code, 1);
        assert_eq!(stdout.calls, expected_calls);
        let cause = result.report.failures.last().unwrap().error.as_ref();
        assert_eq!(
            std::error::Error::source(cause)
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::BrokenPipe
        );
    }
    let (result, stdout, _) = invoke(&app, &["emit", "large"]).await;
    // Serialization uses the replacement response's enlarged policy, not detached defaults.
    assert_eq!(result.exit_code, 0);
    assert_eq!(
        json(&stdout)["results"].as_str().unwrap().len(),
        ParseLimits::default().max_bytes + 1
    );
    // Output failures run no lifecycle error hook; a successful handler is entered once per call.
    assert!(!trace.0.lock().unwrap().iter().any(|event| event == "error"));
}

/// The owning run future can be dropped/panicked without poisoning later independent host calls.
async fn cancellation_and_panic() {
    // Poll to a pending application future without sleeps, then cancel by dropping its owner.
    let (app, trace) = app(false).await;
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let mut sinks = CliSinks::new(&mut stdout, &mut stderr);
    let terminal = Terminal::parse(&["emit".into(), "pending".into()], Default::default()).unwrap();
    let mut future = Box::pin(run(&app, terminal, &mut sinks));
    std::future::poll_fn(|cx| {
        assert!(future.as_mut().poll(cx).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    assert!(
        trace
            .0
            .lock()
            .unwrap()
            .iter()
            .any(|event| event == "pending")
    );
    drop(future);
    assert!(stdout.is_empty());
    let app = Arc::new(app);
    let weak = Arc::downgrade(&app);
    let shared = Arc::clone(&app);
    // A spawned owning task isolates the intentionally propagating handler panic.
    assert!(
        tokio::spawn(async move { invoke(&shared, &["emit", "panic"]).await })
            .await
            .unwrap_err()
            .is_panic()
    );
    // Panic at the external sink boundary after the host finished; the adapter must not retry.
    let calls = Arc::new(AtomicUsize::new(0));
    let observation = Arc::clone(&calls);
    let shared = Arc::clone(&app);
    assert!(
        tokio::spawn(async move {
            let terminal =
                Terminal::parse(&["emit".into(), "echo".into()], Default::default()).unwrap();
            let mut writer = PanickingWriter(observation);
            let mut stderr = Vec::new();
            run(
                &shared,
                terminal,
                &mut CliSinks::new(&mut writer, &mut stderr),
            )
            .await
        })
        .await
        .unwrap_err()
        .is_panic()
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    // Neither cancellation nor either panic poisons a later call or retains the compiled host.
    assert_eq!(invoke(&app, &["emit", "echo"]).await.0.exit_code, 0);
    drop(app);
    assert!(weak.upgrade().is_none());
}

/// Both typed cancellation inspection and opt-in developer formatting bound cyclic causes.
async fn cyclic_sources_are_bounded() {
    for verbose in [false, true] {
        // A real startup error carries the malicious source; no fake host report is involved.
        let count = Arc::new(AtomicUsize::new(0));
        let source = CyclicSource(Arc::clone(&count));
        let mut args = vec![OsString::from("emit"), OsString::from("echo")];
        if verbose {
            args.insert(0, OsString::from("-v"));
        }
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let exit = main_with_args(
            &args,
            &mut CliSinks::new(&mut stdout, &mut stderr),
            move |_| async move { Err(Error::new("startup failed").with_cause(source).into()) },
        )
        .await;
        // A finite traversal rejects cancellation guessing and keeps the startup exit policy.
        assert_eq!(exit, ExitCode::from(2));
        assert!(stdout.is_empty());
        assert!((1..=66).contains(&count.load(Ordering::SeqCst)));
        assert_eq!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("cyclic application cause"),
            verbose
        );
    }
}

/// The executable boundary recognizes typed cancellation, while recovery and ordinary text differ.
async fn executable_exits() {
    // The supplied-argv seam exercises the exact native executable policy with buffer sinks.
    for (event, code) in [
        ("echo", 0),
        ("unknown", 1),
        ("cancel", 130),
        ("false-cancel", 1),
    ] {
        let (app, _) = app(false).await;
        let argv = [OsString::from("emit"), OsString::from(event)];
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let exit = main_with_args(
            &argv,
            &mut CliSinks::new(&mut stdout, &mut stderr),
            move |_| async move { Ok(app) },
        )
        .await;
        assert_eq!(exit, ExitCode::from(code));
        json(&stdout);
    }
    for (verbose, cancelled) in [(false, false), (true, false), (false, true)] {
        // Startup has no partial Server; distinguish ordinary build failure from typed abort.
        let args = if verbose {
            vec!["-v", "emit", "echo"]
        } else {
            vec!["emit", "echo"]
        };
        let args = args.into_iter().map(OsString::from).collect::<Vec<_>>();
        let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
        let exit = main_with_args(
            &args,
            &mut CliSinks::new(&mut stdout, &mut stderr),
            move |_| async move {
                Err::<Server, _>(
                    if cancelled {
                        Error::from_source(PromptError::Aborted)
                    } else {
                        Error::new("PRIVATE startup cause")
                    }
                    .into(),
                )
            },
        )
        .await;
        assert_eq!(exit, ExitCode::from(if cancelled { 130 } else { 2 }));
        assert!(stdout.is_empty());
        assert_eq!(
            String::from_utf8(stderr)
                .unwrap()
                .contains("PRIVATE startup cause"),
            verbose
        );
    }
}

/// Successful recovery short-circuits cancellation inspection, and tiny valid argv still runs.
async fn exit_precedence_and_small_limits() {
    // A recovered error contains a source callback that panics if unnecessarily traversed.
    let (recovered, _) = app(true).await;
    let args = [OsString::from("emit"), OsString::from("recovered-source")];
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let exit = main_with_args(
        &args,
        &mut CliSinks::new(&mut stdout, &mut stderr),
        move |_| async move { Ok(recovered) },
    )
    .await;
    assert_eq!(exit, ExitCode::SUCCESS);
    assert_eq!(json(&stdout)["results"], false);
    assert!(stderr.is_empty());

    // The whole argv fits eight bytes; constant request scaffolding must not consume that budget.
    let (application, _) = app(false).await;
    let terminal = Terminal::parse(
        &["emit".into(), "echo".into()],
        ParseLimits {
            max_bytes: 8,
            ..ParseLimits::default()
        },
    )
    .unwrap();
    let result = run(
        &application,
        terminal,
        &mut CliSinks::new(&mut stdout, &mut stderr),
    )
    .await;
    assert_eq!(result.unwrap().exit_code, 0);
}

/// Bound each scenario independently; the parent also survives a blocked executor thread.
async fn bounded(future: impl Future<Output = ()>) {
    // Bound cooperative async work without using scheduler sleeps to force an interleaving.
    tokio::time::timeout(Duration::from_secs(2), future)
        .await
        .expect("CLI scenario deadline");
}

/// Exact helper entry prevents recursive parent spawning and contains all real host tests.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by the external CLI watchdog"]
async fn run_child() {
    // Each scenario gets its own two-second deadline inside the externally bounded child.
    bounded(success_and_metadata()).await;
    bounded(lifecycle_results()).await;
    bounded(delivery_results()).await;
    bounded(cancellation_and_panic()).await;
    bounded(executable_exits()).await;
    bounded(exit_precedence_and_small_limits()).await;
    bounded(public_error_and_diagnostic_failures()).await;
    bounded(cyclic_sources_are_bounded()).await;
}

/// Enforce the ten-second kill/reap rule for lifecycle/delegation/cancellation scenarios.
#[test]
fn cli_execution_is_watchdog_bounded() {
    // A blocked callback/writer cannot prevent the independent parent from killing/reaping.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "run_child",
            "--nocapture",
        ]),
        "cli-run",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}

/// Native wrapper reads the real test process argv once and rejects its harness-only flags.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by native-entry watchdog parent"]
async fn native_entry_child() {
    // Native stream handles and the complete entry future must remain Send across awaits.
    bounded(async {
        let exit = tokio::spawn(main_with(forbidden_build)).await.unwrap();
        assert_eq!(exit, ExitCode::from(2));
    })
    .await;
}

/// A typed named constructor makes any accidental build entry immediately observable.
async fn forbidden_build(_: ConfigLayers) -> BuildResult<Server> {
    panic!("invalid harness argv must not invoke the builder");
}

/// Cover actual process argv/sink acquisition without adding a stock CLI binary.
#[test]
fn native_entry_rejects_harness_flags_without_building() {
    // Real native argv is intentionally invalid for this library; full success uses buffer tests.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "native_entry_child",
            "--nocapture",
        ]),
        "cli-native-entry",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
