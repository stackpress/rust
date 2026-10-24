//! Format fresh in-memory Rust files before acquiring an output writer lock.

use std::collections::BTreeMap;
use std::io::Write;
use std::process::{Command, Stdio};

use crate::{OutputPlan, Result, TransformError};

/// Format Rust through stdin/stdout, avoiding formatter writes outside the staged plan.
pub(super) fn formatted(plan: &OutputPlan) -> Result<BTreeMap<String, (String, String)>> {
    // Keep declared bytes inspectable; only this bounded copy is replaced by formatter output.
    let mut files = plan.files.clone();
    for (path, (_, text)) in &mut files {
        // Non-Rust artifacts retain their exact bytes, including the legacy newline policy.
        if path.ends_with(".rs") {
            // No native file path is passed, and child module traversal is explicitly disabled.
            let mut child = Command::new("rustfmt")
                .args([
                    "--edition",
                    "2024",
                    "--config",
                    "skip_children=true",
                    "--emit",
                    "stdout",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;
            // Taking stdin closes it at the end of the arm, delivering EOF before waiting.
            let written = match child.stdin.take() {
                Some(mut input) => input.write_all(text.as_bytes()),
                None => Err(std::io::Error::other("formatter stdin missing")),
            };
            // A broken pipe must not leave a live child behind on an early error return.
            if let Err(error) = written {
                let _ = child.kill();
                let _ = child.wait();
                return Err(error.into());
            }
            // Collect/reap the child before interpreting status or accepting its bytes.
            let output = child.wait_with_output()?;
            if !output.status.success() {
                return Err(TransformError::new("Rust formatting failed"));
            }
            *text = String::from_utf8(output.stdout)
                .map_err(|_| TransformError::new("formatter output is not UTF-8"))?;
        }
    }
    // Formatting may expand source; enforce the same total budget again before installation.
    if files.values().map(|(_, text)| text.len()).sum::<usize>() > 1024 * 1024 {
        return Err(TransformError::new("formatted output byte budget exceeded"));
    }
    Ok(files)
}
