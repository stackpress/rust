//! Finite CLI setup/refusal contracts; no case permits reaching a listener bind.
#![cfg(all(feature = "review-preview", debug_assertions))]
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Reap the exact owned child on assertion failure, timeout and ordinary completion.
struct Reap(Child);
impl Drop for Reap {
    /// A second wait is harmless after a cached exit; never leave a timed-out child running.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// Execute one bounded executable action, capturing output in finite owned files.
fn run(args: &[&str]) -> bool {
    let stdout = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let stderr = tempfile::NamedTempFile::new_in("/private/tmp").unwrap();
    let mut child = Reap(
        Command::new(env!("CARGO_BIN_EXE_slice8-review"))
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout.reopen().unwrap()))
            .stderr(Stdio::from(stderr.reopen().unwrap()))
            .spawn()
            .unwrap(),
    );
    let start = Instant::now();
    loop {
        assert!(
            start.elapsed() < Duration::from_secs(10),
            "CLI child deadline"
        );
        assert!(stdout.as_file().metadata().unwrap().len() < 1024 * 1024);
        if let Some(status) = child.0.try_wait().unwrap() {
            return status.success();
        }
        std::thread::sleep(Duration::from_millis(5));
    }
}
/// Actual setup creates a fresh manifest, refuses overwrite and rejects serve before binding.
#[test]
fn cli_setup_and_prebind_refusals() {
    let directory = tempfile::tempdir_in("/private/tmp").unwrap();
    let directory = directory.path().to_str().unwrap();
    assert!(run(&["setup", "--directory", directory]));
    assert!(!run(&["setup", "--directory", directory]));
    assert!(!run(&["setup"]));
    assert!(!run(&["unknown"]));
    assert!(!run(&[]));
    assert!(!run(&["serve", "--port", "0", "--directory", directory]));
    // Assets are deliberately absent, so pipe startup fails before reading input or opening IO.
    assert!(!run(&["pipe", "--port", "3001", "--directory", directory]));
    assert!(!run(&[
        "stream-pipe",
        "--port",
        "3001",
        "--directory",
        directory
    ]));
}
