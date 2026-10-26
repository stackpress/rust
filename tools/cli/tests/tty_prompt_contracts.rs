//! Exercise the real production prompt/worker in a PTY; Python and Rust both enforce deadlines.

#![cfg(feature = "tty")]

use std::future::Future;
use std::io::{self, Write};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::task::Poll;
use std::time::{Duration, Instant};

use crossterm::terminal;
use stackpress_cli::prompt::{
    PromptCancellation, PromptError, PromptOptions, PromptResult, RenderOptions, prompt_native,
};
use stackpress_cli::tty::{TtyInput, TtySession};

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Notify the PTY owner after native raw-mode acquisition and the first prompt render.
struct ReadyWriter {
    /// Handshake unique to this prompt scenario.
    marker: &'static str,
    /// Emit one handshake, never on each individual edit.
    ready: bool,
    /// Allow an output-error case after real input/validation.
    fail_final: bool,
    /// Write the wrapping proof to the actual PTY so its parent inspects native framing.
    echo: bool,
}
impl Write for ReadyWriter {
    /// Real production input has acquired its lease before its first render reaches this writer.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // Advertise readiness only after the real driver has enabled raw mode; optional echo
        // reaches the PTY.
        if !self.ready {
            assert!(terminal::is_raw_mode_enabled()?);
            println!("{}", self.marker);
            io::stdout().flush()?;
            self.ready = true;
        }
        if self.fail_final && bytes.ends_with(b"\n") && bytes.windows(4).any(|part| part == b"done")
        {
            return Err(io::ErrorKind::BrokenPipe.into());
        }
        if self.echo {
            io::stdout().write_all(bytes)?;
        }
        Ok(bytes.len())
    }
    /// Avoid global stderr; only handshake stdout is flushed explicitly.
    fn flush(&mut self) -> io::Result<()> {
        // Handshake writes flush stdout themselves; this sink has no additional buffered payload.
        Ok(())
    }
}

/// Prove dropping a pending validator releases its actual callback-owned resource.
struct ValidationResource(Arc<AtomicUsize>);
impl Drop for ValidationResource {
    /// Record destruction without accessing the terminal worker or its lease.
    fn drop(&mut self) {
        // Count destruction of the resource held inside the actual pending async validator.
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

/// Run a named native prompt with an independent two-second async deadline.
async fn native(
    options: &PromptOptions,
    marker: &'static str,
    fail_final: bool,
) -> PromptResult<String> {
    // Give each native scenario its own writer handshake and independent async deadline.
    let mut output = ReadyWriter {
        marker,
        ready: false,
        fail_final,
        echo: false,
    };
    tokio::time::timeout(
        Duration::from_secs(2),
        prompt_native(
            options,
            None,
            &mut output,
            &PromptCancellation::new(),
            RenderOptions::default(),
        ),
    )
    .await
    .expect("native prompt deadline")
}

/// A no-input pending future owns and joins the production worker when it is dropped.
async fn pending_driver_drop() {
    // Poll the owning prompt until pending, then drop it while its native worker is still idle.
    let options = PromptOptions::new("Drop?");
    let cancellation = PromptCancellation::new();
    let mut output = Vec::new();
    let mut future = Box::pin(prompt_native(
        &options,
        None,
        &mut output,
        &cancellation,
        RenderOptions::default(),
    ));
    std::future::poll_fn(|context| {
        assert!(future.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    assert!(terminal::is_raw_mode_enabled().unwrap());
    let started = Instant::now();
    drop(future);
    assert!(started.elapsed() < Duration::from_millis(250));
    wait_for_cleanup().await;
}

/// Production success, cancellation during validation, output failure, drop and panic cleanup.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run by PTY parent only"]
async fn tty_prompt_child() -> io::Result<()> {
    // First prove pending drop and exclusive ownership before accepting parent-driven native
    // input.
    pending_driver_drop().await;
    let mut input = TtyInput::acquire(PromptCancellation::new())?;
    assert_eq!(
        TtySession::acquire().err().unwrap().kind(),
        io::ErrorKind::AlreadyExists
    );
    input.close().await?;
    input.close().await?;
    drop(input);
    assert!(!terminal::is_raw_mode_enabled()?);

    // Python configures eight columns; None must resolve that real native terminal width.
    assert_eq!(terminal::size()?.0, 8);
    // Reject a native frame taller than the screen before any prompt is painted into scrollback.
    let mut oversized_output = Vec::new();
    let oversized = tokio::time::timeout(
        Duration::from_secs(2),
        prompt_native(
            &PromptOptions::new("x".repeat(200)),
            None,
            &mut oversized_output,
            &PromptCancellation::new(),
            RenderOptions {
                is_terminal: true,
                ..RenderOptions::default()
            },
        ),
    )
    .await
    .expect("oversized native frame deadline");
    assert!(matches!(oversized, Err(PromptError::Limit(_))));
    assert!(oversized_output.is_empty());
    assert!(!terminal::is_raw_mode_enabled()?);
    let mut output = ReadyWriter {
        marker: "WRAP_READY",
        ready: false,
        fail_final: false,
        echo: true,
    };
    let answer = tokio::time::timeout(
        Duration::from_secs(2),
        prompt_native(
            &PromptOptions::new("界😀abcdefg"),
            None,
            &mut output,
            &PromptCancellation::new(),
            RenderOptions {
                is_terminal: true,
                clear_on_done: true,
                columns: None,
            },
        ),
    )
    .await
    .expect("native wrapping deadline")
    .unwrap();
    assert_eq!(answer, "界");
    assert!(!terminal::is_raw_mode_enabled()?);
    println!("WRAP_DONE");
    io::stdout().flush()?;

    // Observe actual queue saturation before dropping; no sleep guesses establish this ordering.
    let input = TtyInput::acquire(PromptCancellation::new())?;
    println!("BACKPRESSURE_READY");
    io::stdout().flush()?;
    tokio::time::timeout(Duration::from_secs(2), async {
        while input.buffered_events() < 32 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("native queue saturation deadline");
    drop(input);
    wait_for_cleanup().await;
    // Parent writes exactly 33 keys: the 33rd may be held by the sender or remain in the OS.
    // Drain only with a ready poll under a new lease, never an unbounded read.
    let mut drain = TtySession::acquire()?;
    while drain.poll(Duration::ZERO)?.is_some() {}
    drain.close()?;

    let options = PromptOptions::new("Name?");
    assert_eq!(
        native(&options, "SUCCESS_READY", false).await.unwrap(),
        "Ada"
    );
    assert!(!terminal::is_raw_mode_enabled()?);
    let mut options = PromptOptions::new("Cancel validation?");
    let dropped = Arc::new(AtomicUsize::new(0));
    let count = Arc::clone(&dropped);
    options = options.async_validator(move |_| {
        let count = Arc::clone(&count);
        async move {
            let _resource = ValidationResource(count);
            println!("VALIDATION_PENDING");
            io::stdout().flush()?;
            std::future::pending().await
        }
    });
    assert!(matches!(
        native(&options, "VALIDATE_READY", false).await,
        Err(PromptError::Aborted)
    ));
    assert_eq!(dropped.load(Ordering::SeqCst), 1);
    assert!(!terminal::is_raw_mode_enabled()?);

    assert!(matches!(
        native(&PromptOptions::new("Output?"), "OUTPUT_READY", true).await,
        Err(PromptError::Io(_))
    ));
    assert!(!terminal::is_raw_mode_enabled()?);
    assert!(matches!(
        native(&PromptOptions::new("EOF?"), "EOF_READY", false).await,
        Err(PromptError::Eof)
    ));
    assert!(!terminal::is_raw_mode_enabled()?);

    let panicked = tokio::spawn(async {
        let mut options = PromptOptions::new("Panic?");
        options = options.validator(|_| panic!("intentional native validator panic"));
        let _ = native(&options, "PANIC_READY", false).await;
    })
    .await
    .unwrap_err();
    assert!(panicked.is_panic());
    wait_for_cleanup().await;

    // The production wrapper must preserve a pre-existing enclosing raw-mode owner too.
    terminal::enable_raw_mode()?;
    pending_driver_drop_raw().await;
    assert!(terminal::is_raw_mode_enabled()?);
    terminal::disable_raw_mode()?;
    println!("PROMPT_RESTORED");
    io::stdout().flush()?;
    let mut acknowledgement = String::new();
    io::stdin().read_line(&mut acknowledgement)?;
    assert_eq!(acknowledgement.trim(), "verified");
    Ok(())
}

/// Drop under an existing raw-mode owner without restoring it to an assumed false state.
async fn pending_driver_drop_raw() {
    // An enclosing raw-mode owner must survive a nested production worker being dropped.
    let input = TtyInput::acquire(PromptCancellation::new()).unwrap();
    drop(input);
    // Raw mode belongs to the enclosing owner; reacquisition proves the nested lease was released.
    wait_for_lease().await;
}

/// Wait for lease release rather than assuming asynchronous Drop has already restored native mode.
async fn wait_for_lease() {
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match TtySession::acquire() {
                Ok(mut session) => {
                    session.close().unwrap();
                    break;
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    tokio::task::yield_now().await
                }
                Err(error) => panic!("terminal reacquisition: {error}"),
            }
        }
    })
    .await
    .expect("terminal cleanup deadline");
}

/// The worker keeps the lease until restoration, so reacquisition is the cleanup handshake.
async fn wait_for_cleanup() {
    wait_for_lease().await;
    assert!(!terminal::is_raw_mode_enabled().unwrap());
}

/// Ten-second watchdog owns Python; Python kills/reaps the exact PTY child on every failure.
#[test]
fn production_native_prompt_restores_and_reaps() {
    // Python owns the PTY child; the outer watchdog additionally owns and reaps Python on failure.
    let result = watchdog::run(
        Command::new("python3")
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/tty_prompt_probe.py"
            ))
            .arg(std::env::current_exe().unwrap()),
        "tty-prompt",
        Duration::from_secs(10),
    )
    .unwrap();
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
