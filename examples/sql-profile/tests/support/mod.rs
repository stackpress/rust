//! Exact-child process deadlines and bounded captures, independent of runtime progress.

use std::fs::File;
use std::io::{self, Read};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use tempfile::NamedTempFile;

/// Own one process until every return/unwind path has killed and reaped it.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    /// Reap even when an assertion or capture error interrupts ordinary completion.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Independently captured raw output and the actual reaped process status.
pub struct Completion {
    /// Native exit status, including signal termination.
    pub status: ExitStatus,
    /// Whether the parent deadline forced termination.
    pub expired: bool,
    /// Bounded machine stdout, with framing retained exactly.
    pub stdout: Vec<u8>,
    /// Bounded human/test diagnostics, never combined with stdout.
    pub stderr: Vec<u8>,
}

/// Run only the supplied child, with no shell/stdin and no pipe-buffer deadlock.
pub fn run(command: &mut Command, deadline: Duration) -> io::Result<Completion> {
    let stdout = NamedTempFile::new()?;
    let stderr = NamedTempFile::new()?;
    command
        .stdin(Stdio::null())
        .stdout(File::create(stdout.path())?)
        .stderr(File::create(stderr.path())?);
    // This owner drops before the capture files on every path, including output overrun.
    let mut child = ChildGuard(command.spawn()?);
    let start = Instant::now();
    let (status, expired) = loop {
        if stdout.as_file().metadata()?.len() > 64 * 1024
            || stderr.as_file().metadata()?.len() > 64 * 1024
        {
            return Err(io::Error::other("child exceeded its output capture limit"));
        }
        if let Some(status) = child.0.try_wait()? {
            break (status, false);
        }
        if start.elapsed() >= deadline {
            child.0.kill()?;
            break (child.0.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    Ok(Completion {
        status,
        expired,
        stdout: capture(stdout.as_file())?,
        stderr: capture(stderr.as_file())?,
    })
}

/// Retain at most the finite capture limit, with one byte to detect growth since the last poll.
fn capture(file: &File) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    file.take(64 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 64 * 1024 {
        return Err(io::Error::other("child output capture grew past its limit"));
    }
    Ok(bytes)
}
