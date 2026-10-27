//! Parent-process supervision independent of a blocked async executor or SQLite worker.

use std::fs::File;
use std::io::Read;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

/// Always terminate and reap a child when its parent unwinds or reports a failure.
struct ChildGuard(Child);
impl Drop for ChildGuard {
    /// Kill is harmless for an exited child; wait reaps in both cases.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run one test child under a monotonic deadline with bounded file-backed diagnostics.
pub fn run(test: &str, scenario: &str, deadline: Duration) -> (bool, bool, String) {
    let log = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let file = File::create(log.path()).unwrap();
    let mut child = ChildGuard(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", test, "--nocapture"])
            .env("STACKPRESS_S6_SCENARIO", scenario)
            .stdout(Stdio::from(file.try_clone().unwrap()))
            .stderr(Stdio::from(file))
            .spawn()
            .unwrap(),
    );
    let started = Instant::now();
    let (success, expired) = loop {
        if log.as_file().metadata().unwrap().len() > 128 * 1024 {
            break (false, false);
        }
        if let Some(status) = child.0.try_wait().unwrap() {
            break (status.success(), false);
        }
        if started.elapsed() >= deadline {
            break (false, true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    // Terminate/reap before reading, including a deliberately stuck negative control.
    let _ = child.0.kill();
    let _ = child.0.wait();
    let mut output = Vec::new();
    File::open(log.path())
        .unwrap()
        .take(128 * 1024 + 1)
        .read_to_end(&mut output)
        .unwrap();
    (
        success && output.len() <= 128 * 1024,
        expired,
        String::from_utf8_lossy(&output).into_owned(),
    )
}
