//! Separate bounded output captures and process deadlines for real application binaries.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// Unique names within one test process; exclusive directory creation prevents stale reuse.
static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

/// Maximum retained stdout or stderr, independently of their contents.
const OUTPUT_LIMIT: u64 = 64 * 1024;

/// One exclusively created ignored directory, removed by its owner on success/error/unwind.
pub struct Fixture(PathBuf);

impl Fixture {
    /// Create an isolated directory beneath this checkout's ignored target tree.
    pub fn new() -> io::Result<Self> {
        // The shared parent may exist; the owned child must be newly created, never adopted.
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/lane-d-cli-fixtures");
        fs::create_dir_all(&base)?;
        let path = base.join(format!(
            "{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path)?;
        Ok(Self(path))
    }

    /// Locate a test-owned file using a fixed fixture basename.
    pub fn path(&self, name: &str) -> PathBuf {
        // Test callers supply local literal basenames, never command-line or user paths.
        self.0.join(name)
    }
}

impl Drop for Fixture {
    /// Remove only the directory this exact owner created.
    fn drop(&mut self) {
        // Cleanup cannot replace an earlier assertion/I/O failure with another panic.
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Own a spawned process until it has been killed/reaped even on an early return or panic.
struct ChildGuard(Child);

impl Drop for ChildGuard {
    /// Stop an unfinished child and wait for it; completed children are harmless to revisit.
    fn drop(&mut self) {
        // Child::wait retains an observed exit status, preventing an escaped zombie on any path.
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Completed child state and independently retained raw machine/human output.
pub struct Completion {
    /// Actual process exit status after reaping.
    pub status: ExitStatus,
    /// True when the external deadline forced termination.
    pub timed_out: bool,
    /// Raw bounded stdout, never combined with diagnostics.
    pub stdout: Vec<u8>,
    /// Raw bounded stderr, inspected separately from the machine envelope.
    pub stderr: Vec<u8>,
}

/// Run a command under an independent wall-clock deadline, killing and reaping on expiry.
pub fn run(command: &mut Command, deadline: Duration) -> io::Result<Completion> {
    // Regular files avoid full-pipe deadlocks; a unique fixture isolates concurrent tests.
    let fixture = Fixture::new()?;
    let stdout = fixture.path("stdout");
    let stderr = fixture.path("stderr");
    command
        .stdin(Stdio::null())
        .stdout(File::create(&stdout)?)
        .stderr(File::create(&stderr)?);

    // Declare the guard after the fixture so unwind reaps the child before deleting its captures.
    let mut child = ChildGuard(command.spawn()?);
    let started = Instant::now();
    let (status, timed_out) = loop {
        // Excess output is a harness failure; the guard still kills/reaps before cleanup.
        if fs::metadata(&stdout)?.len() > OUTPUT_LIMIT
            || fs::metadata(&stderr)?.len() > OUTPUT_LIMIT
        {
            return Err(io::Error::other("CLI child exceeded its capture limit"));
        }

        // Observe completion before expiry so a finished child is never classified as timed out.
        if let Some(status) = child.0.try_wait()? {
            break (status, false);
        }
        if started.elapsed() >= deadline {
            // Kill the exact owned process and reap it before returning any output or status.
            child.0.kill()?;
            break (child.0.wait()?, true);
        }

        // Finite polling keeps the parent responsive without spinning or awaiting the child.
        std::thread::sleep(Duration::from_millis(10));
    };

    // Recheck bounded reads after exit to catch growth since the final metadata observation.
    Ok(Completion {
        status,
        timed_out,
        stdout: read_capture(&stdout)?,
        stderr: read_capture(&stderr)?,
    })
}

/// Read at most one capture ceiling plus a detection byte, rejecting truncated evidence.
fn read_capture(path: &Path) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take(OUTPUT_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    // Never silently truncate an oversized machine result into an apparently valid envelope.
    if bytes.len() as u64 > OUTPUT_LIMIT {
        return Err(io::Error::other("CLI capture exceeded its read limit"));
    }
    Ok(bytes)
}
