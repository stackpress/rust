//! Format fresh in-memory Rust files before acquiring an output writer lock.

use std::collections::BTreeMap;
use std::io::{self, Read, Write};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::{OutputPlan, Result, TransformError};

/// Format Rust through stdin/stdout, avoiding formatter writes outside the staged plan.
pub(super) fn formatted(plan: &OutputPlan) -> Result<BTreeMap<String, (String, String)>> {
    // Keep declared bytes inspectable; only this bounded copy is replaced by formatter output.
    let mut files = plan.files.clone();
    for (path, (_, text)) in &mut files {
        // Non-Rust artifacts retain their exact bytes, including the legacy newline policy.
        if path.ends_with(".rs") {
            // No native file path is passed, and child module traversal is explicitly disabled.
            let mut command = Command::new("rustfmt");
            command.args([
                "--edition",
                "2024",
                "--config",
                "skip_children=true",
                "--emit",
                "stdout",
            ]);
            *text = run_formatter(text, &mut command)?;
        }
    }
    // Formatting may expand source; enforce the same total budget again before installation.
    if files.values().map(|(_, text)| text.len()).sum::<usize>() > ARTIFACT_BYTES {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Limit,
            "formatted output byte budget exceeded",
        ));
    }
    Ok(files)
}

/// Final artifacts share a 1 MiB aggregate budget, independently of subprocess diagnostics.
const ARTIFACT_BYTES: usize = 1024 * 1024;
/// Each formatter stdout stream is capped before retaining the next chunk.
const STDOUT_BYTES: usize = ARTIFACT_BYTES;
/// Diagnostic bytes have their own smaller ceiling, even on successful formatter exit.
const STDERR_BYTES: usize = 64 * 1024;
/// One formatter may run for ten seconds, including pipe input/output and exit.
const FORMATTER_TIME: Duration = Duration::from_secs(10);

/// Own child termination/reaping before any scoped pipe worker is joined.
struct ChildOwner(Child);
impl Drop for ChildOwner {
    /// Kill on every error/unwind path; wait also reaps already-exited children.
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Completed bounded pipe operations, each sent exactly once to the supervisor.
enum PipeResult {
    /// Stdin reached EOF after all source bytes were written.
    Input,
    /// Capped stdout retained for artifact decoding.
    Output(Vec<u8>),
    /// Capped stderr was fully drained and discarded.
    Diagnostics,
}

/// Run the installed formatter under native pipe and runtime supervision.
fn run_formatter(text: &str, command: &mut Command) -> Result<String> {
    supervise(text, command, FORMATTER_TIME)
}

/// Drain both output pipes while writing input, with kill/reap before worker joins on failure.
fn supervise(text: &str, command: &mut Command, time_limit: Duration) -> Result<String> {
    let started = Instant::now();
    let child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // Move the owner inside the scope: all return paths drop it before scope joins pipe workers.
    // The trusted formatter must not delegate inherited pipes to independently surviving children.
    std::thread::scope(move |scope| {
        let mut child = ChildOwner(child);
        let receiver = start_pipes(scope, text, &mut child.0)?;
        let bytes = collect(&mut child.0, &receiver, started, time_limit)?;
        String::from_utf8(bytes).map_err(|cause| {
            TransformError::caused(
                crate::TransformErrorKind::Io,
                "formatter output is not UTF-8",
                cause,
            )
        })
    })
}

/// Start the three pipe operations under the caller's scope and child cleanup authority.
fn start_pipes<'scope>(
    scope: &'scope std::thread::Scope<'scope, '_>,
    text: &'scope str,
    child: &mut Child,
) -> Result<mpsc::Receiver<Result<PipeResult>>> {
    let mut input = child
        .stdin
        .take()
        .ok_or_else(|| io::Error::other("formatter stdin missing"))?;
    let output = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::other("formatter stdout missing"))?;
    let diagnostics = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::other("formatter stderr missing"))?;
    let (sender, receiver) = mpsc::channel();
    let send_input = sender.clone();
    // Fallible thread creation also drops the child owner before joining already-started work.
    std::thread::Builder::new()
        .name("formatter-input".into())
        .spawn_scoped(scope, move || {
            let result = input
                .write_all(text.as_bytes())
                .map(|()| PipeResult::Input)
                .map_err(TransformError::from);
            drop(input); // Deliver EOF even when the write failed.
            let _ = send_input.send(result);
        })?;
    let send_output = sender.clone();
    std::thread::Builder::new()
        .name("formatter-output".into())
        .spawn_scoped(scope, move || {
            let _ = send_output.send(read_capped(output, STDOUT_BYTES).map(PipeResult::Output));
        })?;
    std::thread::Builder::new()
        .name("formatter-diagnostics".into())
        .spawn_scoped(scope, move || {
            let _ = sender
                .send(read_capped(diagnostics, STDERR_BYTES).map(|_| PipeResult::Diagnostics));
        })?;
    Ok(receiver)
}

/// Supervise finite pipe completion and process exit; the caller kills/reaps on any error.
fn collect(
    child: &mut Child,
    receiver: &mpsc::Receiver<Result<PipeResult>>,
    started: Instant,
    time_limit: Duration,
) -> Result<Vec<u8>> {
    let mut finished = 0;
    let mut bytes = Vec::new();
    let status = loop {
        // Stream errors take precedence over an otherwise successful process exit.
        match receiver.recv_timeout(Duration::from_millis(5)) {
            Ok(result) => {
                if let PipeResult::Output(output) = result? {
                    bytes = output;
                }
                finished += 1;
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) if finished == 3 => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                return Err(io::Error::other("formatter pipe worker stopped").into());
            }
        }
        if let Some(status) = child.try_wait()?
            && finished == 3
        {
            break status;
        }
        // A stuck writer, either reader, or a child that closed its pipes is still bounded.
        if started.elapsed() >= time_limit {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Limit,
                "formatter time limit exceeded",
            ));
        }
        // Disconnected channels return immediately; avoid spinning while awaiting child exit.
        if finished == 3 {
            std::thread::sleep(Duration::from_millis(5));
        }
    };
    if !status.success() {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Other,
            "Rust formatting failed",
        ));
    }
    Ok(bytes)
}

/// Check each chunk before retention, keeping stdout and stderr budgets independent.
fn read_capped(mut input: impl Read, limit: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let count = match input.read(&mut chunk) {
            Err(cause) if cause.kind() == io::ErrorKind::Interrupted => continue,
            result => result?,
        };
        if count == 0 {
            return Ok(output);
        }
        if count > limit.saturating_sub(output.len()) {
            return Err(TransformError::classified(
                crate::TransformErrorKind::Limit,
                "formatter stream byte limit exceeded",
            ));
        }
        output.extend_from_slice(&chunk[..count]);
    }
}

#[cfg(test)]
mod tests;
