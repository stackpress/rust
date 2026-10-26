//! Shared CLI diagnostic disclosure and typed cancellation policy; never writes machine stdout.

use std::error::Error as StdError;
use std::io::Write;

use stackpress_lib::{Error, ErrorKind, Result};

use crate::control::{Control, Level, OutputEnvironment};
use crate::prompt::PromptError;

/// Write one human diagnostic through an explicit sink, sanitizing all caller-controlled text.
pub(crate) fn write(
    control: &Control,
    writer: &mut (dyn Write + Send),
    level: Level,
    message: &str,
) -> Result<()> {
    // Borrowed writers have no native TTY identity: Auto stays plain; Always overrides explicitly.
    control
        .write(writer, OutputEnvironment::default(), level, message, &[])
        .map_err(|cause| {
            Error::new("CLI diagnostic write failed")
                .with_kind(ErrorKind::External)
                .with_code(500)
                .with_public_message("Internal Server Error")
                .with_cause(cause)
        })?;
    // Flush is part of delivery; a failed stderr flush cannot be reported as a successful command.
    writer.flush().map_err(|cause| {
        Error::new("CLI diagnostic flush failed")
            .with_kind(ErrorKind::External)
            .with_code(500)
            .with_public_message("Internal Server Error")
            .with_cause(cause)
    })
}

/// Recognize explicit typed prompt cancellation without guessing from a message or status number.
pub(crate) fn prompt_aborted(error: &(dyn StdError + 'static)) -> bool {
    // Borrow each link in place; cause inspection never clones or retains application ownership.
    let mut current = Some(error);
    // Bound traversal because application-defined source chains can be cyclic or arbitrarily deep.
    for _ in 0..=32 {
        let Some(error) = current else { return false };
        if matches!(
            error.downcast_ref::<PromptError>(),
            Some(PromptError::Aborted)
        ) {
            // Only the exact cancellation variant qualifies; invalid/EOF and ordinary 130 do not.
            return true;
        }
        // Advance after the concrete variant check; ordinary messages never imply cancellation.
        current = error.source();
    }
    false
}

/// Opt into developer text for startup/outer errors, with the same finite source-chain policy.
pub(crate) fn developer(error: &(dyn StdError + 'static)) -> String {
    // This helper is called only after a successfully parsed explicit verbose operator flag.
    let mut text = error.to_string();
    let mut current = error.source();
    // A malformed cyclic source chain still produces a finite verbose diagnostic.
    for _ in 0..32 {
        let Some(cause) = current else { break };
        // Keep traversal text separate from wire output; the final human formatter sanitizes it.
        text.push_str("; caused by: ");
        text.push_str(&cause.to_string());
        current = cause.source();
    }
    text
}

#[cfg(test)]
mod tests;
