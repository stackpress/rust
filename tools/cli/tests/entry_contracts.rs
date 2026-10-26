//! Supplied executable inputs prove bounded bootstrap/config ordering without global argv mutation.

use std::ffi::OsString;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use serde::{Deserialize, Serialize};
use stackpress_cli::{CliContext, CliSinks, ConfigLayers, main_with_args};
use stackpress_lib::{Flow, Invocation, ParseLimits, Request, Response, Result};
use stackpress_server::{BuildResult, Server, ServerBuilder, ServerInvocationExt};

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Allocate only process-local fixture names; each newly created directory has one cleanup owner.
static FIXTURE: AtomicUsize = AtomicUsize::new(0);

/// A new ignored directory never overlaps tracked files or another fixture's cleanup authority.
struct Fixture(PathBuf);
impl Fixture {
    /// Create an exclusively new directory beneath this worktree's ignored target output.
    fn new() -> Self {
        // Isolate file creation/cleanup from every tracked path and concurrently running fixture.
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/cli-entry-fixtures");
        fs::create_dir_all(&base).unwrap();
        let path = base.join(format!(
            "{}-{}",
            std::process::id(),
            FIXTURE.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    /// Write a supplied data fixture, preserving explicit spaces and unusual file suffixes.
    fn file(&self, name: &str, bytes: &[u8]) -> PathBuf {
        // Preserve exact caller-supplied names so extension guessing cannot pass unnoticed.
        let path = self.0.join(name);
        fs::write(&path, bytes).unwrap();
        path
    }
}
impl Drop for Fixture {
    /// Remove only the directory this exact helper created, including after a failed assertion.
    fn drop(&mut self) {
        // Failed tests still release only their own newly allocated fixture tree.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The compiled application explicitly owns its typed configuration section.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Settings {
    /// Public fixture result; no full config dump is performed by the adapter.
    label: String,
}

/// Reject invalid typed values during the real ServerBuilder decoding phase.
fn valid(settings: &Settings) -> Result<()> {
    // Exercise application validation separately from syntax and serde decoding failures.
    if settings.label.is_empty() {
        return Err(stackpress_lib::Error::invalid("label is empty"));
    }
    Ok(())
}

/// Expose selected test data deliberately so the adapter's input boundary is observable.
async fn inspect(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    // Only trusted adapter context may say a file was selected; target data remains independent.
    let context = inv.adapter_context::<CliContext>().unwrap();
    assert!(!req.data.has("config"));
    res.results(&serde_json::json!({
        "label": inv.config::<Settings>()?.label,
        "config_selected": context.config.is_some(),
        "target": req.data.to_json()?,
    }))?;
    Ok(Flow::Continue)
}

/// Compose defaults then supplied layers before publishing a real immutable server.
async fn configured(layers: ConfigLayers) -> BuildResult<Server> {
    // The application claims its schema and applies CLI file layers before startup decoding.
    let mut app = ServerBuilder::new();
    app.config_section(
        "test",
        Settings {
            label: "default".into(),
        },
        valid,
    )?;
    app.config_layers(layers)?;
    app.on("inspect", inspect)?;
    app.build().await
}

/// Capture actual FnOnce entry count along with exact machine/human sink bytes.
async fn invoke(args: &[OsString]) -> (ExitCode, Vec<u8>, Vec<u8>, usize) {
    // Own a non-clone constructor capture and independent buffers for this one executable call.
    let calls = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&calls);
    let owned = Box::new("FnOnce composition fixture".to_owned());
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let exit = main_with_args(
        args,
        &mut CliSinks::new(&mut stdout, &mut stderr),
        move |layers| async move {
            // A unique owned capture proves the constructor need not implement Fn or Clone.
            assert_eq!(*owned, "FnOnce composition fixture");
            observed.fetch_add(1, Ordering::SeqCst);
            configured(layers).await
        },
    )
    .await;
    (exit, stdout, stderr, calls.load(Ordering::SeqCst))
}

/// Supply one exact file path without shell quoting or argument re-tokenization.
fn with_file(path: &Path) -> Vec<OsString> {
    // Keep a path containing spaces in one OS argument, exactly like native argv acquisition.
    [
        OsString::from("-b"),
        path.as_os_str().to_owned(),
        "emit".into(),
        "inspect".into(),
    ]
    .into()
}

/// Load the explicit file before build; post-event bootstrap flags stay target data.
async fn bootstrap_and_boundary() {
    // Arrange an unconventional but explicit filename and observe the actual compiled config.
    let fixture = Fixture::new();
    let path = fixture.file(
        "explicit data.any-suffix",
        br#"{"test":{"label":"from file"}}"#,
    );
    let (exit, stdout, stderr, calls) = invoke(&with_file(&path)).await;
    // Successful data decoding precedes one builder call and adds nothing to human output.
    assert_eq!(exit, ExitCode::SUCCESS);
    assert_eq!(calls, 1);
    let wire: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
    assert_eq!(wire["results"]["label"], "from file");
    assert_eq!(wire["results"]["config_selected"], true);
    assert!(stderr.is_empty());

    // Short/long spellings after the event are handled exclusively by the lib target parser.
    for target in [
        vec!["-b", "literal file"],
        vec!["--bootstrap", "literal file"],
    ] {
        let mut args = vec![OsString::from("emit"), OsString::from("inspect")];
        args.extend(target.into_iter().map(OsString::from));
        let (exit, stdout, stderr, calls) = invoke(&args).await;
        // If the adapter steals a target bootstrap flag, the file/default/context checks fail.
        assert_eq!(exit, ExitCode::SUCCESS);
        assert_eq!(calls, 1);
        assert!(stderr.is_empty());
        let wire: serde_json::Value = serde_json::from_slice(&stdout).unwrap();
        assert_eq!(wire["results"]["label"], "default");
        assert_eq!(wire["results"]["config_selected"], false);
        assert!(
            wire["results"]["target"]["b"] == true
                || wire["results"]["target"]["bootstrap"] == "literal file"
        );
    }
}

/// Byte ceiling and parser/encoding/file errors fail before the builder is consumed.
async fn file_limits_and_errors() {
    // JSON whitespace reaches the exact byte ceiling without exceeding parser value limits.
    let fixture = Fixture::new();
    let limit = ParseLimits::default().max_bytes;
    let mut exact = br#"{"test":{"label":"at limit"}}"#.to_vec();
    exact.resize(limit, b' ');
    let path = fixture.file("exact", &exact);
    assert_eq!(invoke(&with_file(&path)).await.0, ExitCode::SUCCESS);
    // One detection byte, bad syntax/encoding, wrong roots and ordinary I/O all reject pre-build.
    exact.push(b' ');
    let excessive = fixture.file("too large", &exact);
    let invalid = fixture.file("malformed", b"PRIVATE malformed JSON");
    let encoding = fixture.file("non UTF8", &[0xff]);
    let array = fixture.file("array root", b"[]");
    let missing = fixture.0.join("config");
    // An alternate .json path exists but must never be guessed or searched automatically.
    fixture.file("config.json", b"{}");
    for path in [
        &excessive, &invalid, &encoding, &array, &missing, &fixture.0,
    ] {
        let (exit, stdout, stderr, calls) = invoke(&with_file(path)).await;
        // Neither file contents nor paths may appear in default diagnostics.
        assert_eq!(exit, ExitCode::from(2));
        assert_eq!(calls, 0);
        assert!(stdout.is_empty());
        assert_eq!(stderr, b"bootstrap configuration failed\n");
    }
    // Unknown/invalid typed sections are build failures after exactly one constructor entry.
    for text in [
        r#"{"unclaimed":{"PRIVATE":"value"}}"#,
        r#"{"test":{"label":false}}"#,
        r#"{"test":{"label":""}}"#,
    ] {
        let path = fixture.file("typed", text.as_bytes());
        let (exit, stdout, stderr, calls) = invoke(&with_file(&path)).await;
        assert_eq!(exit, ExitCode::from(2));
        assert_eq!(calls, 1);
        assert!(stdout.is_empty());
        assert_eq!(stderr, b"application startup failed\n");
    }
    let mut verbose = with_file(&missing);
    verbose.insert(0, "-v".into());
    // The same missing file exposes its retained cause only after checked operator verbosity.
    let (exit, stdout, stderr, calls) = invoke(&verbose).await;
    assert_eq!(exit, ExitCode::from(2));
    assert!(stdout.is_empty());
    assert_eq!(calls, 0);
    assert!(String::from_utf8(stderr).unwrap().contains("caused by:"));
}

/// Invalid argv never calls the application builder and never writes machine output.
async fn usage_errors() {
    // Cover the operator/event boundary and the approved lib grammar before any file/build work.
    for args in [
        vec![],
        vec!["emit"],
        vec!["-x", "emit", "inspect"],
        vec!["-b"],
        vec!["emit", "inspect", "--dangling"],
    ] {
        let args = args.into_iter().map(OsString::from).collect::<Vec<_>>();
        let (exit, stdout, stderr, calls) = invoke(&args).await;
        // A usage report is human-only and must not consume the application constructor.
        assert_eq!(exit, ExitCode::from(2));
        assert_eq!(calls, 0);
        assert!(stdout.is_empty());
        assert!(!stderr.is_empty());
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        // Native non-UTF-8 values fail with their original argv index rather than lossy decoding.
        let args = [OsString::from("emit"), OsString::from_vec(vec![0xff])];
        let (exit, stdout, stderr, calls) = invoke(&args).await;
        assert_eq!(exit, ExitCode::from(2));
        assert_eq!(calls, 0);
        assert!(stdout.is_empty());
        assert_eq!(stderr, b"argument 1 is not UTF-8\n");
    }
}

/// A dead diagnostic sink supersedes the original startup/usage code with output failure.
struct Broken;
impl Write for Broken {
    /// Preserve the actual BrokenPipe cause without accepting bytes.
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::ErrorKind::BrokenPipe.into())
    }
    /// Nothing is buffered by this deliberately broken fixture.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Prove a pre-run diagnostic failure cannot turn into a fallback stdout message or a build entry.
async fn diagnostic_error() {
    // Fail before a Terminal exists while also making stderr unavailable.
    let mut stdout = Vec::new();
    let mut stderr = Broken;
    let exit = main_with_args(
        &[],
        &mut CliSinks::new(&mut stdout, &mut stderr),
        |_| async {
            panic!("usage failure must not call the builder");
        },
    )
    .await;
    // Output failure wins over usage status and never falls back to machine stdout.
    assert_eq!(exit, ExitCode::from(1));
    assert!(stdout.is_empty());
}

/// A sibling future must advance before bootstrap parsing can consume the builder.
async fn executor_progress() {
    let fixture = Fixture::new();
    let path = fixture.file("progress", b"{}");
    let progressed = std::sync::atomic::AtomicBool::new(false);
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let args = with_file(&path);
    let mut sinks = CliSinks::new(&mut stdout, &mut stderr);
    let entry = main_with_args(&args, &mut sinks, |layers| {
        assert!(
            progressed.load(Ordering::SeqCst),
            "bootstrap blocked executor progress"
        );
        configured(layers)
    });
    let (exit, ()) = tokio::join!(biased; entry, async {
        progressed.store(true, Ordering::SeqCst);
    });
    assert_eq!(exit, ExitCode::SUCCESS);
}

/// Unsupported acquisition never waits for a FIFO writer or invokes the builder.
#[cfg(unix)]
async fn special_bootstrap() {
    let fixture = Fixture::new();
    let fifo = fixture.0.join("fifo");
    assert!(
        Command::new("mkfifo")
            .arg(&fifo)
            .status()
            .unwrap()
            .success()
    );
    for path in [&fifo, Path::new("/dev/null")] {
        let (exit, stdout, _, calls) = invoke(&with_file(path)).await;
        assert_eq!(exit, ExitCode::from(2));
        assert!(stdout.is_empty());
        assert_eq!(calls, 0);
    }
}

/// All data-file tests have an inner async deadline and the independent external watchdog.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by entry watchdog parent"]
async fn entry_child() {
    // A timer protects cooperative futures; the parent separately bounds synchronous blocking.
    tokio::time::timeout(Duration::from_secs(2), async {
        bootstrap_and_boundary().await;
        file_limits_and_errors().await;
        usage_errors().await;
        diagnostic_error().await;
        executor_progress().await;
        #[cfg(unix)]
        special_bootstrap().await;
    })
    .await
    .expect("entry scenario deadline");
}

/// Kill and reap a blocked startup/file fixture even if its executor cannot run the inner timer.
#[test]
fn entry_contracts_are_watchdog_bounded() {
    // Exact ignored-child selection avoids recursive spawning; the helper always kills/reaps.
    let result = watchdog::run(
        Command::new(std::env::current_exe().unwrap()).args([
            "--ignored",
            "--exact",
            "entry_child",
            "--nocapture",
        ]),
        "cli-entry",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
