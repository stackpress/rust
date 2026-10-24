//! Pure prompt text plus driver-owned terminal framing; caller text cannot generate escapes.

use std::io::Write;
use unicode_width::UnicodeWidthStr;

use crate::control::display_text;

use super::{PromptError, PromptOptions, PromptResult, PromptState, PromptStatus};

/// Explicit output properties; pipes always receive plain text, even with clear-on-done.
///
/// Terminal output wraps using unicode-width's non-CJK width policy. Editing and
/// row boundaries are scalar-based, so a grapheme can split at a boundary; full
/// grapheme segmentation is not provided. Native frames taller than the captured
/// screen height return Limit before writing rather than becoming unerasable scrollback.
///
/// # Examples
/// ```
/// use stackpress_cli::prompt::RenderOptions;
/// // An injected width lets a buffer model a terminal without querying native metadata.
/// let terminal = RenderOptions {
///     is_terminal: true,
///     clear_on_done: true,
///     columns: Some(40),
/// };
/// assert_eq!(terminal.columns, Some(40));
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RenderOptions {
    /// Whether the selected writer is a terminal, independent of the input source.
    pub is_terminal: bool,
    /// Erase terminal prompt on success; plain outputs retain the readable transcript.
    pub clear_on_done: bool,
    /// Inject a terminal width for buffered rendering; None queries native terminal size.
    /// Required for terminal rendering when the `tty` feature is disabled.
    /// The size is captured for this prompt; resizing/external writes during a
    /// prompt require the caller to cancel and start a new output session.
    pub columns: Option<usize>,
}

/// Sanitized display-only text; actual answers are never transformed or sanitized.
#[derive(Debug, Eq, PartialEq)]
pub struct PromptRender {
    /// Question plus live/default/final display text, without newline/ANSI.
    pub prompt: String,
    /// Sanitized validation error, without newline/ANSI.
    pub error: String,
}

impl PromptState {
    /// Render bounded question/value/error text, invoking the display transformer if present.
    ///
    /// # Errors
    /// Returns transformation failures and render limits before writing any bytes.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, PromptState};
    /// let options = PromptOptions::new("Name?").transform(|_, _| Ok("hidden".into()));
    /// let state = PromptState::new(&options)?;
    /// assert_eq!(state.render(&options)?.prompt, "? Name? hidden");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn render(&self, options: &PromptOptions) -> PromptResult<PromptRender> {
        // Accepted empty text stays empty; only a live preview may fall back to the default.
        let is_final = self.status() == PromptStatus::Complete;
        let value = if self.current().is_empty() && !is_final {
            self.default_value()
        } else {
            self.current()
        };
        // User transformation changes display only, and is checked before sanitization/framing.
        let transformed = options.display_value(value, is_final)?;
        if transformed.len() > options.limits.render_bytes {
            // Refuse callback expansion before sanitization allocates a second representation.
            return Err(PromptError::Limit("transformed text exceeds render budget"));
        }
        // Sanitize each display component so only driver-owned framing can move the cursor.
        let message = bounded_display(&options.message, options.limits.render_bytes)?;
        let value = bounded_display(&transformed, options.limits.render_bytes)?;
        let error = bounded_display(self.error_message(), options.limits.render_bytes)?;
        let mut prompt = String::from("?");
        for part in [&message, &value] {
            // Omit empty pieces without dropping meaningful false/zero text already formatted by
            // callers.
            if !part.is_empty() {
                prompt.push(' ');
                prompt.push_str(part);
            }
        }
        // Include fixed cursor commands, separators and newline in the one-write budget.
        if prompt.len().saturating_add(error.len()).saturating_add(32) > options.limits.render_bytes
        {
            return Err(PromptError::Limit("prompt exceeds render budget"));
        }
        Ok(PromptRender { prompt, error })
    }
}

/// Sanitize only already-bounded text, checking the expanded representation as well.
fn bounded_display(text: &str, maximum: usize) -> PromptResult<String> {
    // Bound both raw and expanded printable text; replacement characters may need more UTF-8 bytes.
    if text.len() > maximum {
        return Err(PromptError::Limit("display text exceeds render budget"));
    }
    let display = display_text(text);
    if display.len() > maximum {
        // Sanitization is not allowed to push a valid raw string past the output budget.
        return Err(PromptError::Limit("sanitized text exceeds render budget"));
    }
    Ok(display)
}

/// Driver-local presentation state; writes no text until explicitly painted.
pub(super) struct Renderer {
    /// Chosen output behavior, never guessed from stdin.
    options: RenderOptions,
    /// Plain streams emit initial/final text and errors, not every intermediate edit.
    painted: bool,
    /// Captured width for deterministic wrapping; plain output never queries native metadata.
    columns: usize,
    /// A native frame must fit the visible screen, because cursor-up cannot erase scrollback.
    height: Option<usize>,
    /// Number of physical rows in the last successful terminal paint, including its cursor row.
    rows: usize,
}

impl Renderer {
    /// Begin a fresh prompt output transcript without borrowing a global stream.
    pub(super) fn new(options: RenderOptions) -> PromptResult<Self> {
        // Query native metadata only for an explicitly selected terminal writer.
        let (columns, height) = if options.is_terminal {
            terminal_columns(options.columns)?
        } else {
            // A pipe has no physical rows to redraw and must not query a controlling terminal.
            (0, None)
        };
        // No cursor region is owned until the first write and flush complete successfully.
        Ok(Self {
            options,
            painted: false,
            columns,
            height,
            rows: 0,
        })
    }

    /// Prepare a full bounded render before the single writer call; failures propagate.
    pub(super) fn paint(
        &mut self,
        options: &PromptOptions,
        state: &PromptState,
        writer: &mut (dyn Write + Send),
    ) -> PromptResult<()> {
        let final_state = state.status() == PromptStatus::Complete;
        // Even clear-on-done validates transformation, so callback failure is not hidden by policy.
        let render = state.render(options)?;
        let mut rows = 0;
        let text = if self.options.is_terminal {
            // Erase from the cursor's actual last row back to the first owned row.
            let prefix = clear_rows(self.rows);
            if final_state && self.options.clear_on_done {
                // Erase the previous render without painting an accepted value the caller asked to
                // hide.
                prefix
            } else {
                // Error text participates in the same display-width layout as question and answer.
                let error = if render.error.is_empty() {
                    String::new()
                } else {
                    format!(" [{}]", render.error)
                };
                let layout = wrap(&format!("{}{error}", render.prompt), self.columns)?;
                // Never scroll an owned native frame out of reach of its later clear/redraw.
                if self.height.is_some_and(|height| layout.rows > height) {
                    return Err(PromptError::Limit("prompt exceeds terminal screen height"));
                }
                rows = layout.rows;
                format!(
                    "{prefix}{}{}",
                    layout.text,
                    if final_state { "\r\n" } else { "" }
                )
            }
        } else if !self.painted || final_state {
            // Plain transcripts expose a readable initial/final prompt without cursor commands.
            format!("{}\n", render.prompt)
        } else if !render.error.is_empty() {
            // Emit each actual rejection, while ordinary intermediate edits stay quiet in pipes.
            format!("{}\n", render.error)
        } else {
            // There is no new readable transcript entry for an intermediate successful edit.
            String::new()
        };
        // Include row erasure/wrapping overhead before any partial native output is possible.
        if text.len() > options.limits.render_bytes {
            return Err(PromptError::Limit("terminal framing exceeds render budget"));
        }
        // Any partial write/flush error terminates the driver; bookkeeping is never used for a
        // retry.
        writer.write_all(text.as_bytes())?;
        writer.flush()?;
        // Only successful output commits cursor bookkeeping; failures terminate the driver.
        self.painted = true;
        self.rows = rows;
        Ok(())
    }
}

/// Resolve nonzero injected width or native dimensions without consuming terminal input.
fn terminal_columns(columns: Option<usize>) -> PromptResult<(usize, Option<usize>)> {
    // Explicit buffer dimensions override OS metadata; native mode uses crossterm's checked query.
    let (columns, height) = match columns {
        Some(columns) => (columns, None),
        #[cfg(feature = "tty")]
        None => {
            // Native dimensions bound both wrapping and the rows that cursor-up can still reach.
            let (columns, rows) = crossterm::terminal::size()?;
            (usize::from(columns), Some(usize::from(rows)))
        }
        #[cfg(not(feature = "tty"))]
        None => {
            return Err(PromptError::Configuration(
                "terminal rendering requires a width",
            ));
        }
    };
    // A zero-width native PTY cannot establish any meaningful cursor layout.
    if columns == 0 || height == Some(0) {
        return Err(PromptError::Configuration("terminal width must be nonzero"));
    }
    Ok((columns, height))
}

/// Text with explicit CRLF row boundaries and a known last cursor row.
struct Layout {
    /// Printable content with driver-owned row breaks, never implicit auto-wrap dependence.
    text: String,
    /// One row even for an empty string.
    rows: usize,
}

/// Wrap at scalar boundaries using Unicode display width, including emoji sequence widths.
///
/// Prefix measurement stays within one bounded row's text, including zero-width sequences.
/// This is not grapheme segmentation: a cluster can split if its width grows at a row boundary.
fn wrap(text: &str, columns: usize) -> PromptResult<Layout> {
    // char_indices provides valid UTF-8 boundaries for every committed row slice.
    let mut output = String::new();
    let mut start = 0;
    let mut end = 0;
    let mut rows = 1;
    for (offset, character) in text.char_indices() {
        let next = offset + character.len_utf8();
        // Measure the complete row prefix so ZWJ/variation sequences use library width policy.
        if text[start..next].width() > columns {
            if start == end {
                // One display scalar wider than the whole terminal has no representable row.
                return Err(PromptError::Configuration(
                    "terminal is too narrow for display text",
                ));
            }
            // Commit the last fitting prefix before advancing to a fresh physical row.
            output.push_str(&text[start..end]);
            // CR cancels a full last cell's pending auto-wrap before advancing exactly one row.
            output.push_str("\r\n");
            rows += 1;
            start = end;
            if text[start..next].width() > columns {
                // A wide scalar at the new row still cannot be emitted on an undersized terminal.
                return Err(PromptError::Configuration(
                    "terminal is too narrow for display text",
                ));
            }
        }
        // Carry the final fitting scalar boundary into the next prefix measurement.
        end = next;
    }
    // Keep the cursor on the last row, even when its final cell has pending auto-wrap.
    output.push_str(&text[start..]);
    Ok(Layout { text: output, rows })
}

/// Erase every owned physical row and return the cursor to the beginning of the first row.
fn clear_rows(rows: usize) -> String {
    // First paint also establishes column zero; callers own the output region for the session.
    let mut output = String::from("\r");
    for row in 0..rows {
        if row > 0 {
            // Move to the preceding owned row only after clearing the cursor's current row.
            output.push_str("\x1b[1A\r");
        }
        // Erasing the full row also removes residual text when the new layout shrinks.
        output.push_str("\x1b[2K");
    }
    output
}
