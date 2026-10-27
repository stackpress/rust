//! Independent parent-process deadlines, including a deliberately stuck negative control.

use std::fs::File;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Ensure every error/unwind path terminates and reaps the child.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    /// Killing an exited child is harmless; waiting reaps it in either case.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run a test child with bounded file-backed diagnostics, independent of Tokio progress.
pub(super) fn run(test: &str, scenario: &str, deadline: Duration) -> (bool, bool, String) {
    run_child(test, scenario, deadline, false)
}

/// Start a negative control's measured deadline only after its prerequisite is acknowledged.
pub(super) fn run_ready(test: &str, scenario: &str, deadline: Duration) -> (bool, bool, String) {
    run_child(test, scenario, deadline, true)
}

/// Acknowledge the child's controlled state through its parent-owned temporary file.
pub(super) fn ready() {
    let path = std::env::var_os("STACKPRESS_SQL_WATCHDOG_READY").unwrap();
    std::fs::write(path, b"ready").unwrap();
}

/// Own process supervision; a separate startup bound prevents a missing handshake from hanging.
fn run_child(
    test: &str,
    scenario: &str,
    deadline: Duration,
    require_ready: bool,
) -> (bool, bool, String) {
    let log = tempfile::NamedTempFile::new().unwrap();
    let ready = tempfile::NamedTempFile::new().unwrap();
    let output = File::create(log.path()).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env("STACKPRESS_SQL_M0_SCENARIO", scenario)
        .env("STACKPRESS_SQL_WATCHDOG_READY", ready.path())
        .stdout(Stdio::from(output.try_clone().unwrap()))
        .stderr(Stdio::from(output));
    let mut child = ChildGuard(command.spawn().unwrap());
    let spawned = Instant::now();
    let mut start = (!require_ready).then_some(spawned);
    let (status, expired, exceeded) = loop {
        // Reject a noisy child before its deadline, so diagnostics cannot fill the disk.
        if log.as_file().metadata().unwrap().len() > 64 * 1024 {
            let _ = child.0.kill();
            break (child.0.wait().unwrap(), false, true);
        }
        if let Some(status) = child.0.try_wait().unwrap() {
            break (status, false, false);
        }
        // A slow process launch must not consume the negative control's measured window.
        if start.is_none() && ready.as_file().metadata().unwrap().len() > 0 {
            start = Some(Instant::now());
        }
        // Unwinding this failure still kills/reaps through ChildGuard; it cannot count as
        // a successful timeout or output-limit proof without reaching the controlled state.
        assert!(
            start.is_some() || spawned.elapsed() < Duration::from_secs(10),
            "watchdog child did not acknowledge its controlled state"
        );
        if start.is_some_and(|start| start.elapsed() >= deadline) {
            child.0.kill().unwrap();
            break (child.0.wait().unwrap(), true, false);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // One extra byte detects growth after the last poll, including a child that exited.
    let mut output = Vec::new();
    File::open(log.path())
        .unwrap()
        .take(64 * 1024 + 1)
        .read_to_end(&mut output)
        .unwrap();
    let exceeded = exceeded || output.len() > 64 * 1024;
    output.truncate(64 * 1024);
    // Native diagnostics can contain non-UTF-8 bytes or end midway through a code point.
    let output = String::from_utf8_lossy(&output).into_owned();
    (status.success() && !exceeded, expired, output)
}
