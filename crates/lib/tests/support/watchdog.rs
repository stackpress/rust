//! External process deadlines for tests that could block an executor thread.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

/// A completed/reaped child and the bounded output useful in failure reports.
pub(crate) struct Completion {
    /// Actual child exit status, including a watchdog termination if it timed out.
    pub(crate) status: ExitStatus,
    /// Whether the deadline caused the parent to terminate the child.
    pub(crate) timed_out: bool,
    /// At most 64 KiB of diagnostic output, never an unbounded pipe buffer.
    pub(crate) output: String,
}

/// Own a child until explicit completion or cleanup on every error/unwind path.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    /// Prevent a subprocess escaping when a parent assertion or I/O step fails.
    fn drop(&mut self) {
        // kill is harmless after exit; wait always reaps a terminated child.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Run a command with a wall-clock deadline independent of its executor.
pub(crate) fn run(
    command: &mut Command,
    scenario: &str,
    deadline: Duration,
) -> io::Result<Completion> {
    // Keep receipts in this build's ignored target directory, after M0 cleaning.
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/m0");
    fs::create_dir_all(&directory)?;
    let output_path = directory.join(format!("{}-{scenario}.log", std::process::id()));
    let output = File::create(&output_path)?;
    command.stdout(Stdio::from(output.try_clone()?));
    command.stderr(Stdio::from(output));

    // Own the process immediately so early returns and panics also clean it up.
    let mut child = ChildGuard(command.spawn()?);
    let started = Instant::now();
    let (status, timed_out) = loop {
        // Poll exit before checking the deadline so an already completed child wins.
        if let Some(status) = child.0.try_wait()? {
            break (status, false);
        }

        // A blocked child cannot prevent this parent from killing and reaping it.
        if started.elapsed() >= deadline {
            child.0.kill()?;
            break (child.0.wait()?, true);
        }

        // Short bounded polling avoids a busy loop without hiding a long sleep.
        std::thread::sleep(Duration::from_millis(10));
    };

    // Read only a bounded prefix after exit; no child can block on a full pipe.
    let mut bytes = Vec::new();
    File::open(output_path)?
        .take(64 * 1024)
        .read_to_end(&mut bytes)?;
    Ok(Completion {
        status,
        timed_out,
        output: String::from_utf8_lossy(&bytes).into_owned(),
    })
}
