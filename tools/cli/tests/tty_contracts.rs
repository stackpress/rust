//! Actual PTY/raw-mode proof with an external kill/reap watchdog; no native listener.

#![cfg(feature = "tty")]

use std::future::Future;
use std::io::{self, IsTerminal, Write};
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::Poll;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal;
use stackpress_cli::tty::TtySession;

#[path = "../../../crates/lib/tests/support/watchdog.rs"]
mod watchdog;

/// Probe-only bounded input worker; the full prompt driver is deliberately a later milestone.
struct Worker {
    /// Framework-owned stop signal; no arbitrary callback can retain it and block cleanup.
    stop: Arc<AtomicBool>,
    /// Every exit joins this exact worker, never a detached stdin reader.
    join: Option<JoinHandle<io::Result<()>>>,
}

impl Worker {
    /// Spawn a reader that only polls under the acquired real terminal lease.
    fn start(session: TtySession) -> Self {
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let join = thread::spawn(move || {
            let mut session = session;
            // The finite poll interval lets drop interrupt idle reads without new input.
            while !worker_stop.load(Ordering::Acquire) {
                if let Some(Event::Key(key)) = session.poll(Duration::from_millis(20))?
                    && key.code == KeyCode::Char('c')
                    && key.modifiers.contains(KeyModifiers::CONTROL)
                {
                    break;
                }
            }
            session.close()
        });
        Self {
            stop,
            join: Some(join),
        }
    }

    /// Await a real Ctrl-C event without creating a second reader competing with the worker.
    fn wait_for_key(&mut self) {
        let started = Instant::now();
        while !self.join.as_ref().unwrap().is_finished() {
            assert!(
                started.elapsed() < Duration::from_secs(2),
                "Ctrl-C event deadline"
            );
            thread::sleep(Duration::from_millis(5));
        }
    }
}

impl Drop for Worker {
    /// Stop/join the controlled worker; the parent process watchdog catches any violated bound.
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(join) = self.join.take() {
            join.join()
                .expect("worker panic")
                .expect("worker terminal I/O");
        }
    }
}

/// Exercise raw-state ownership on a real PTY with native input but no prompt implementation.
#[tokio::test(flavor = "current_thread")]
#[ignore = "run only by the PTY watchdog parent"]
async fn tty_native_child() -> io::Result<()> {
    assert!(io::stdin().is_terminal());
    assert!(!terminal::is_raw_mode_enabled()?);
    {
        let mut session = TtySession::acquire()?;
        assert!(terminal::is_raw_mode_enabled()?);
        assert_eq!(
            TtySession::acquire().err().unwrap().kind(),
            io::ErrorKind::AlreadyExists
        );
        session.close()?;
        session.close()?;
        assert!(session.poll(Duration::ZERO).is_err());
    }
    assert!(!terminal::is_raw_mode_enabled()?);

    // Exercise an actual idle native poll before testing the worker's stop/join path.
    let mut idle = TtySession::acquire()?;
    let started = Instant::now();
    assert!(idle.poll(Duration::from_millis(20))?.is_none());
    assert!(started.elapsed() < Duration::from_millis(250));
    drop(idle);

    // Idle worker drop must stop without input and restore mode before returning.
    let worker = Worker::start(TtySession::acquire()?);
    let started = Instant::now();
    drop(worker);
    assert!(started.elapsed() < Duration::from_millis(250));
    assert!(!terminal::is_raw_mode_enabled()?);

    // A pending async validator owns the guard; dropping its future performs actual cleanup.
    let mut pending = Box::pin(async {
        let _session = TtySession::acquire()?;
        std::future::pending::<io::Result<()>>().await
    });
    std::future::poll_fn(|context| {
        assert!(pending.as_mut().poll(context).is_pending());
        Poll::Ready(())
    })
    .await;
    assert!(terminal::is_raw_mode_enabled()?);
    drop(pending);
    assert!(!terminal::is_raw_mode_enabled()?);

    // Panic propagation is not framework recovery; RAII still restores the terminal lease.
    let unwind = std::panic::catch_unwind(|| {
        let _session = TtySession::acquire().expect("panic guard");
        panic!("intentional terminal guard unwind");
    });
    assert!(unwind.is_err());
    assert!(!terminal::is_raw_mode_enabled()?);

    // Preserve raw mode that was already enabled by an explicit enclosing owner.
    terminal::enable_raw_mode()?;
    drop(TtySession::acquire()?);
    assert!(terminal::is_raw_mode_enabled()?);
    terminal::disable_raw_mode()?;

    // Parent writes Ctrl-C only after the input lease is raw and worker owns the reader.
    let mut worker = Worker::start(TtySession::acquire()?);
    println!("PTY_READY");
    io::stdout().flush()?;
    worker.wait_for_key();
    drop(worker);
    assert!(!terminal::is_raw_mode_enabled()?);
    println!("PTY_RESTORED");
    io::stdout().flush()?;
    // The parent checks actual termios while the session leader is still alive;
    // macOS revokes the slave after leader exit, making post-exit ioctl invalid.
    let mut acknowledgement = String::new();
    io::stdin().read_line(&mut acknowledgement)?;
    assert_eq!(acknowledgement.trim(), "verified");
    Ok(())
}

/// A non-TTY stdin never starts a hidden blocking read or changes raw mode.
#[test]
fn non_tty_input_is_rejected_without_reading() {
    if !io::stdin().is_terminal() {
        assert_eq!(
            TtySession::acquire().err().unwrap().kind(),
            io::ErrorKind::Unsupported
        );
    }
}

/// Python owns/reaps the PTY child; the outer watchdog also kills/reaps a stalled harness.
#[test]
fn native_guard_poll_cancel_and_drop_are_bounded() {
    let mut command = Command::new("python3");
    command
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/support/tty_probe.py"
        ))
        .arg(std::env::current_exe().expect("test executable"));
    let result = watchdog::run(&mut command, "tty-guard", Duration::from_secs(12))
        .expect("launch/reap PTY harness");
    assert!(!result.timed_out, "{}", result.output);
    assert!(result.status.success(), "{}", result.output);
}
