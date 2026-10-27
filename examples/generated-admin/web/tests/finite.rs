//! Real executable SSR/CSR output, run in a reaped child with a parent watchdog.
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Reap the finite output process even when a parent assertion fails.
struct Guard(Child);
impl Drop for Guard {
    /// A successful exit and a watchdog cancellation both relinquish the child.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The native host emits useful SSR or the explicit empty CSR root without opening a socket.
#[test]
fn executable_modes_are_finite() {
    for csr in [false, true] {
        let log =
            std::env::temp_dir().join(format!("admin-web-test-{}-{csr}.html", std::process::id()));
        let file = std::fs::File::create(&log).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_generated-admin-web"));
        if csr {
            command.arg("--csr");
        }
        let mut child = Guard(
            command
                .stdout(Stdio::from(file))
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let start = Instant::now();
        loop {
            assert!(
                start.elapsed() < Duration::from_secs(10),
                "SSR process timed out"
            );
            if let Some(status) = child.0.try_wait().unwrap() {
                assert!(status.success());
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        let html = std::fs::read_to_string(&log).unwrap();
        std::fs::remove_file(log).unwrap();
        assert!(html.starts_with("<!doctype html>"));
        assert_eq!(html.contains("Generated workspace admin"), !csr);
        assert!(!html.contains("Unable to render"));
        assert!(!html.contains("ADMIN_PRIVATE_DEFAULT_SENTINEL"));
    }
}
