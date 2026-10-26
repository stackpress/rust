//! Parent-side deadlines independent of the reactive executor under test.

use std::{
    fs::File,
    io::Read,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Kill and reap even if a fixture assertion or a watchdog I/O operation unwinds.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    /// Prevent a deadlocked child escaping either the test or the parent process.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run hang-class work with a real wall-clock deadline outside its executor.
pub(super) fn run(mode: &str, test_name: &str) {
    // Keep receipts outside the build trees that the M0 brief requires deleting.
    let directory = std::path::Path::new("/private/tmp/s8-m0");
    std::fs::create_dir_all(directory).unwrap();
    let path = directory.join(format!("async-{}-{}.log", mode, std::process::id()));
    let log = File::create(&path).unwrap();
    // File-backed output cannot deadlock the child on a full captured pipe.
    let child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", test_name, "--nocapture"])
        .env("S8_ASYNC_CHILD", mode)
        .stdout(Stdio::from(log.try_clone().unwrap()))
        .stderr(Stdio::from(log))
        .spawn()
        .unwrap();
    // Own the child before any later operation can fail or unwind.
    let mut child = ChildGuard(child);
    let deadline = Instant::now() + Duration::from_secs(10);
    let status = loop {
        // Observe an already completed child before deciding it exceeded time.
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        // Drop's kill/reap guard executes even when this deadline assertion fails.
        assert!(
            Instant::now() < deadline,
            "async {} timed out; log: {}",
            mode,
            path.display()
        );
        std::thread::sleep(Duration::from_millis(10));
    };
    // Read a bounded diagnostic prefix only after the child has been reaped.
    let mut output = String::new();
    File::open(&path)
        .unwrap()
        .take(64 * 1024)
        .read_to_string(&mut output)
        .unwrap();
    assert!(status.success(), "{output}");
    assert!(
        output
            .lines()
            .any(|line| line == format!("S8_ASYNC_COMPLETE {mode}")),
        "child did not complete its scenario (check exact filter): {output}"
    );
    print!("{output}");
}
