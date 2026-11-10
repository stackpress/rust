//! Shared test-only child observation; fixture creation and cleanup remain backend-owned.
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

/// Preserve native runner errors without formatting their potentially sensitive payloads.
type ProcessResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Wait without trusting an in-runtime timeout; timeout always kills and reaps the child.
pub(crate) fn watchdog(mut child: Child, deadline: Duration) -> ProcessResult<ExitStatus> {
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) => {}
            Err(error) => {
                // Even observation failures must not leave a native scenario running.
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
        }
        if start.elapsed() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(
                "native scenario process watchdog timed out; child killed and reaped".into(),
            );
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}
