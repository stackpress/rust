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
    let log = tempfile::NamedTempFile::new().unwrap();
    let output = File::create(log.path()).unwrap();
    let mut command = Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", test, "--ignored", "--nocapture"])
        .env("STACKPRESS_SQL_M0_SCENARIO", scenario)
        .stdout(Stdio::from(output.try_clone().unwrap()))
        .stderr(Stdio::from(output));
    let mut child = ChildGuard(command.spawn().unwrap());
    let start = Instant::now();
    let (status, expired) = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break (status, false);
        }
        if start.elapsed() >= deadline {
            child.0.kill().unwrap();
            break (child.0.wait().unwrap(), true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let mut output = String::new();
    File::open(log.path())
        .unwrap()
        .take(64 * 1024)
        .read_to_string(&mut output)
        .unwrap();
    (status.success(), expired, output)
}
