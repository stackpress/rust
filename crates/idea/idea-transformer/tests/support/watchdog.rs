//! File-backed bounded diagnostics and exact-child kill/reap supervision for native tests.
use std::io::{self, Read, Seek};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// Reaped status and bounded diagnostics from one owned test process.
pub(crate) struct Completion {
    /// Status after normal exit or watchdog termination has been reaped.
    pub(crate) status: ExitStatus,
    /// Whether this owner terminated the process at the deadline.
    pub(crate) timed_out: bool,
    /// At most 64 KiB of diagnostic text read after child exit.
    pub(crate) output: String,
}

/// Retain cleanup authority through IO errors and assertion unwind.
struct ChildOwner(Child);
impl Drop for ChildOwner {
    /// Never leave a blocked child after its parent returns.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run one exact child, bounding wall time independently of its executor.
pub(crate) fn run(command: &mut Command, _: &str, deadline: Duration) -> io::Result<Completion> {
    // A unique anonymous file cannot fill a pipe or create another Cargo target directory.
    let mut output = tempfile::tempfile()?;
    command
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::from(output.try_clone()?));
    let mut child = ChildOwner(command.spawn()?);
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child.0.try_wait()? {
            break (status, false);
        }
        if started.elapsed() >= deadline {
            child.0.kill()?;
            break (child.0.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    output.rewind()?;
    let mut bytes = Vec::new();
    output.take(64 * 1024).read_to_end(&mut bytes)?;
    Ok(Completion {
        status,
        timed_out,
        output: String::from_utf8_lossy(&bytes).into_owned(),
    })
}
