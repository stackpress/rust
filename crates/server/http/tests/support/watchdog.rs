//! One kill-and-reap boundary for every potentially stalled duplex/TLS scenario.
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

/// Reap even if status inspection or a parent assertion panics while the child is still alive.
struct OwnedChild {
    /// The only process handle; None means try_wait already observed and reaped termination.
    child: Option<Child>,
}
impl Drop for OwnedChild {
    /// Cleanup never masks the parent's original failure with a second panic.
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            // A race with normal exit is harmless; wait is still required after a kill attempt.
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Run one ignored entry in this binary with its original environment selector and finite budget.
pub fn run(test: &str, budget: Duration, variables: &[(&str, &str)]) {
    let child = Command::new(std::env::current_exe().expect("current test binary"))
        .args(["--exact", test, "--ignored", "--nocapture"])
        .envs(variables.iter().copied())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("watchdog child started");
    let mut owner = OwnedChild { child: Some(child) };
    let deadline = Instant::now() + budget;
    loop {
        // try_wait reaps a finished child; disarm Drop before asserting its exit status.
        let status = owner
            .child
            .as_mut()
            .expect("unreaped watchdog child")
            .try_wait()
            .expect("watchdog status inspected");
        if let Some(status) = status {
            owner.child = None;
            assert!(status.success(), "{test} {variables:?} failed: {status}");
            return;
        }
        // Dropping the owner kills and reaps before reporting a timeout as a test failure.
        if Instant::now() >= deadline {
            drop(owner);
            panic!("{test} {variables:?} exceeded {budget:?}; killed and reaped");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
