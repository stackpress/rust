//! Finite native codegen commands shared by selected and disabled feature proofs.

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Native CLI commands are bounded by a kill-and-reap deadline and preserve OS argument boundaries.
pub fn command(input: &Path, output: &Path, arguments: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_profile-codegen"))
        .args(["emit", "generate", "--input"])
        .arg(input)
        .arg("--output")
        .arg(output)
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            panic!("codegen CLI timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
