//! Async orchestration over explicit event sources; no detached blocking pipe readers.

use std::collections::VecDeque;
use std::future::Future;
use std::io::{self, IsTerminal, Write};

use tokio::sync::{mpsc, watch};

use super::render::Renderer;
use super::{
    PromptAction, PromptError, PromptEvent, PromptOptions, PromptResult, PromptState,
    RenderOptions, Validation,
};

/// Explicit shareable cancellation signal; cloning shares only the named signal.
#[derive(Clone, Debug)]
pub struct PromptCancellation(watch::Sender<bool>);

impl Default for PromptCancellation {
    /// Create an uncancelled signal.
    fn default() -> Self {
        // Default owns an independent signal, not a process-global cancellation slot.
        Self::new()
    }
}

impl PromptCancellation {
    /// Create an uncancelled signal independent of process-wide signal handlers.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptCancellation;
    /// let cancellation = PromptCancellation::new();
    /// assert!(!cancellation.is_cancelled());
    /// ```
    pub fn new() -> Self {
        // Keep the sender even without listeners so cancellation is retained for future
        // subscribers.
        Self(watch::channel(false).0)
    }

    /// Cancel permanently, waking all current waiters even if no input is ready.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptCancellation;
    /// let cancellation = PromptCancellation::new();
    /// let shared = cancellation.clone();
    /// shared.cancel();
    /// assert!(cancellation.is_cancelled());
    /// ```
    pub fn cancel(&self) {
        // send_replace records permanent cancellation even when no driver is currently awaiting it.
        self.0.send_replace(true);
    }

    /// Inspect cancellation before acquiring input or invoking callbacks.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptCancellation;
    /// let cancellation = PromptCancellation::new();
    /// assert!(!cancellation.is_cancelled());
    /// cancellation.cancel();
    /// assert!(cancellation.is_cancelled());
    /// ```
    pub fn is_cancelled(&self) -> bool {
        // The short watch borrow is released here, never carried across user code or an await.
        *self.0.borrow()
    }

    /// Wait without losing a cancellation that happened before subscription.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptCancellation;
    /// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
    /// let cancellation = PromptCancellation::new();
    /// cancellation.cancel();
    /// cancellation.cancelled().await; // Late subscribers observe the retained cancellation.
    /// # });
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub async fn cancelled(&self) {
        // Subscribe before inspecting state inside wait_for to avoid a check/subscribe race.
        let mut receiver = self.0.subscribe();
        // The sender is owned by self for this borrow, so channel closure is impossible here.
        let _ = receiver.wait_for(|cancelled| *cancelled).await;
    }
}

/// Cancellation-safe event source; dropping `next` must not lose an already consumed event.
///
/// Implementations must not block the executor or hide an unjoinable input worker.
/// This trait is the actual substitution boundary for buffers, channels and native TTY input.
pub trait PromptInput: Send {
    /// Return one event or EOF; cancellation may drop this future at any await.
    fn next(&mut self) -> impl Future<Output = io::Result<Option<PromptEvent>>> + Send;
}

impl PromptInput for mpsc::Receiver<PromptEvent> {
    /// Receive one bounded channel event; Tokio recv is cancellation-safe.
    async fn next(&mut self) -> io::Result<Option<PromptEvent>> {
        // Dropping recv before readiness does not remove an event from the caller's queue.
        Ok(self.recv().await)
    }
}

/// Finite supplied events; construction never reads native stdin.
pub struct BufferedInput<I> {
    /// Iterator produces events lazily, avoiding a second whole-input allocation.
    events: I,
}

impl<I: Iterator<Item = PromptEvent> + Send> BufferedInput<I> {
    /// Wrap supplied events/fixtures with no hidden thread, runtime or I/O.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{BufferedInput, PromptEvent};
    /// let input = BufferedInput::new([PromptEvent::Enter].into_iter());
    /// ```
    pub fn new(events: I) -> Self {
        // Own only the iterator; callers retain responsibility for the supplied input workload.
        Self { events }
    }
}

impl BufferedInput<std::iter::Empty<PromptEvent>> {
    /// Interpret supplied pipe-style UTF-8 text, coalescing CRLF to one Enter.
    ///
    /// Ctrl-C/Tab/backspace have the same meaning as native events; other controls
    /// are rejected as Text by the machine. EOF does not implicitly press Enter.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{BufferedInput, PromptEvent, PromptInput};
    /// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
    /// let mut input = BufferedInput::lines("Ada\r\n");
    /// assert_eq!(input.next().await?, Some(PromptEvent::Text("A".into())));
    /// # Ok::<(), std::io::Error>(()) })?;
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn lines(text: &str) -> BufferedInput<impl Iterator<Item = PromptEvent> + Send + '_> {
        // Track CR across iterator calls so CRLF submits once without dropping standalone LF.
        let mut after_cr = false;
        BufferedInput::new(text.chars().filter_map(move |character| {
            let skip_lf = after_cr && character == '\n';
            after_cr = character == '\r';
            if skip_lf {
                // The preceding CR already produced Enter; consume only its companion LF.
                return None;
            }
            // Map control characters explicitly; ordinary text remains UTF-8 scalar-safe.
            Some(match character {
                '\r' | '\n' => PromptEvent::Enter,
                '\u{3}' => PromptEvent::Cancel,
                '\t' => PromptEvent::Tab,
                '\u{8}' | '\u{7f}' => PromptEvent::Backspace,
                _ => PromptEvent::Text(character.to_string()),
            })
        }))
    }
}

impl<I: Iterator<Item = PromptEvent> + Send> PromptInput for BufferedInput<I> {
    /// Yield the next supplied event immediately; exhaustion is explicit EOF.
    async fn next(&mut self) -> io::Result<Option<PromptEvent>> {
        // This ready-only path never starts a native read or allocates a whole second event list.
        Ok(self.events.next())
    }
}

/// Prompt using supplied events/output, with no global stdin/stdout or server dispatch.
///
/// Required and pattern checks precede user validation. While validation awaits,
/// bounded edits are queued; cancellation wins and drops the validator future.
/// Queued edits replay only after rejection. EOF after Enter lets that validation
/// finish, then ends input if rejected. Input cannot forge validation completions.
/// Dropping this future drops callbacks/borrows; a native input owner must also be
/// dropped (the [`prompt_native`] wrapper owns it). Writers/callbacks may block.
///
/// # Errors
/// Returns cancellation, EOF, input/output/callback failures or configured limits.
/// It never retries output or starts another prompt after a partial write.
///
/// # Examples
/// ```
/// use stackpress_cli::prompt::{
///     BufferedInput, PromptCancellation, PromptOptions, PromptResult, RenderOptions, prompt,
/// };
/// async fn ask() -> PromptResult<String> {
///     let mut input = BufferedInput::lines("Ada\n");
///     let mut output = Vec::new();
///     prompt(&PromptOptions::new("Name?"), &mut input, &mut output,
///         &PromptCancellation::new(), RenderOptions::default()).await
/// }
/// # let answer = tokio::runtime::Builder::new_current_thread().build()?.block_on(ask())?;
/// # assert_eq!(answer, "Ada");
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub async fn prompt<I: PromptInput>(
    options: &PromptOptions,
    input: &mut I,
    writer: &mut (dyn Write + Send),
    cancellation: &PromptCancellation,
    render: RenderOptions,
) -> PromptResult<String> {
    // Interactive sources can correct invalid input; only native supplied answers terminate on it.
    prompt_inner(options, input, writer, cancellation, render, None).await
}

/// Share editing/rendering while making one-answer rejection explicit for the native adapter.
async fn prompt_inner<I: PromptInput>(
    options: &PromptOptions,
    input: &mut I,
    writer: &mut (dyn Write + Send),
    cancellation: &PromptCancellation,
    render: RenderOptions,
    supplied_answer: Option<&str>,
) -> PromptResult<String> {
    // Already-cancelled work must not call transformers, write a prompt, or acquire more input.
    if cancellation.is_cancelled() {
        return Err(PromptError::Aborted);
    }
    // Prepare checked editing/output state before the first side effect.
    let mut state = match supplied_answer {
        // Supplied input is a complete candidate, not a text edit appended to a default.
        Some(answer) => PromptState::supplied(options, answer)?,
        None => PromptState::new(options)?,
    };
    let mut renderer = Renderer::new(render)?;
    let mut queue = VecDeque::new();
    let mut ended = false;
    renderer.paint(options, &state, writer)?;
    loop {
        // An already cancelled signal wins over buffered edits and ready validation.
        if cancellation.is_cancelled() {
            return Err(PromptError::Aborted);
        }
        // Replayed edits retain their original order; only an empty queue consults the source
        // again.
        let event = if let Some(event) = queue.pop_front() {
            event
        } else if ended {
            // EOF observed during validation becomes terminal only after queued corrections run
            // out.
            PromptEvent::Eof
        } else {
            // Cancellation is biased ahead of a ready input, and losing next() must be safe to
            // drop.
            tokio::select! {
                biased;
                _ = cancellation.cancelled() => return Err(PromptError::Aborted),
                event = input.next() => event?.unwrap_or(PromptEvent::Eof),
            }
        };
        // Only the private validator path may create a completion accepted by the pure machine.
        check_input_event(&event)?;
        let mut action = state.apply(options, event)?;
        if let PromptAction::Validate { id, value } = action {
            // Borrow input/state serially; validation can queue edits but never mutates this state.
            let result =
                validate(options, value, input, cancellation, &mut queue, &mut ended).await?;
            // A callback can cancel during its own ready poll, after select's first signal check.
            // Recheck before committing the token so that completion cannot revive cancelled work.
            if cancellation.is_cancelled() {
                return Err(PromptError::Aborted);
            }
            // Only this private completion path can accept an externally supplied answer.
            action = state.apply(options, PromptEvent::Validated { id, result })?;
            if supplied_answer.is_some() && matches!(action, PromptAction::Render) {
                // A finite supplied answer cannot be corrected; preserve its actual rejection.
                renderer.paint(options, &state, writer)?;
                return Err(PromptError::InvalidAnswer(state.error_message().into()));
            }
        }
        match action {
            PromptAction::Complete(value) => {
                // Final output must succeed before an accepted value is returned to the caller.
                renderer.paint(options, &state, writer)?;
                return Ok(value);
            }
            // Editing/rejection requests a redraw; cancellation/EOF return their distinct outcomes.
            PromptAction::Render => renderer.paint(options, &state, writer)?,
            PromptAction::Aborted => return Err(PromptError::Aborted),
            PromptAction::Eof => return Err(PromptError::Eof),
            PromptAction::None => {}
            PromptAction::Validate { .. } => {
                // complete() only accepts/rejects; it cannot recursively request another
                // validation.
                unreachable!("completion never starts another validation")
            }
        }
    }
}

/// Validate once, buffering bounded edits without concurrent state writes.
async fn validate<I: PromptInput>(
    options: &PromptOptions,
    value: String,
    input: &mut I,
    cancellation: &PromptCancellation,
    queue: &mut VecDeque<PromptEvent>,
    ended: &mut bool,
) -> PromptResult<Validation> {
    // Pin the sole validation future locally; every early return drops it instead of detaching it.
    let validation = options.validate(value);
    tokio::pin!(validation);
    // A prior rejected validation may leave edits after its next Enter in the queue.
    // Include those retained payloads before accepting input for this validation.
    let mut bytes = queue.iter().try_fold(0usize, |bytes, event| {
        // Non-text keys cost an event slot, while text also consumes the aggregate payload budget.
        let length = match event {
            PromptEvent::Text(text) => text.len(),
            _ => 0,
        };
        bytes
            .checked_add(length)
            .ok_or(PromptError::Limit("queued input size overflow"))
    })?;
    loop {
        // Each iteration polls cancellation first, then completion, then newly available input.
        tokio::select! {
            biased;
            _ = cancellation.cancelled() => return Err(PromptError::Aborted),
            // Ready sync/built-in results win over buffered EOF and follow-up edits.
            result = &mut validation => return result,
            event = input.next(), if !*ended => {
                // A closed channel or exhausted buffer is EOF; no read is retried after that point.
                let event = event?.unwrap_or(PromptEvent::Eof);
                check_input_event(&event)?;
                match event {
                    // Cancellation abandons queued edits and drops the pending validator
                    // immediately.
                    PromptEvent::Cancel => return Err(PromptError::Aborted),
                    // Submission happened already: allow its validation to complete before EOF
                    // wins.
                    PromptEvent::Eof => *ended = true,
                    event => {
                        // Bound both allocation dimensions before retaining this pending edit.
                        if let PromptEvent::Text(text) = &event {
                            bytes = bytes.checked_add(text.len()).ok_or(
                                PromptError::Limit("queued input size overflow"),
                            )?;
                        }
                        if queue.len() >= options.limits.pending_events
                            || bytes > options.limits.answer_bytes
                        {
                            return Err(PromptError::Limit("pending prompt input exceeds budget"));
                        }
                        // Defer state mutation until the current candidate has been rejected.
                        queue.push_back(event);
                    }
                }
            }
        }
    }
}

/// Validation completions are machine/driver messages, never untrusted source events.
fn check_input_event(event: &PromptEvent) -> PromptResult<()> {
    // External completion tokens could bypass required/custom validation even if the ID were
    // guessed.
    if matches!(event, PromptEvent::Validated { .. }) {
        return Err(PromptError::Configuration(
            "input cannot complete validation",
        ));
    }
    Ok(())
}

/// Prompt native stdin, using stderr (or another explicit writer) for human output.
///
/// An explicit answer bypasses input ownership but still validates/transforms/renders.
/// It replaces any Editable/default prefill, including when explicitly empty.
/// Non-TTY stdin uses the configured default or returns NonInteractive immediately;
/// there is no background pipe reader. With `tty` disabled even a terminal requires
/// an explicit/default answer. Native terminal input uses one joined polling worker.
///
/// # Errors
/// Returns the same failures as [`prompt`], plus native acquisition/cleanup errors.
/// Rejection of a supplied/default answer returns [`PromptError::InvalidAnswer`]
/// with its bounded validation message; it is distinct from missing input/EOF.
/// A primary prompt failure is retained if cleanup also fails; Drop retries raw restoration.
///
/// # Examples
/// ```
/// use stackpress_cli::prompt::{
///     PromptCancellation, PromptOptions, RenderOptions, prompt_native,
/// };
/// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
/// let mut output = Vec::new();
/// let answer = prompt_native(&PromptOptions::new("Name?"), Some("Ada"),
///     &mut output, &PromptCancellation::new(), RenderOptions::default()).await?;
/// assert_eq!(answer, "Ada"); // No terminal is required for an explicit answer.
/// # Ok::<(), Box<dyn std::error::Error>>(()) })?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// Native interactive input requires a real terminal and the `tty` feature:
///
/// ```no_run
/// use std::io::{self, IsTerminal};
/// use stackpress_cli::prompt::{
///     PromptCancellation, PromptOptions, RenderOptions, prompt_native,
/// };
/// # tokio::runtime::Builder::new_current_thread().build()?.block_on(async {
/// let mut output = io::stderr();
/// let render = RenderOptions { is_terminal: output.is_terminal(), ..Default::default() };
/// let answer = prompt_native(&PromptOptions::new("Name?"), None, &mut output,
///     &PromptCancellation::new(), render).await?;
/// # Ok::<(), Box<dyn std::error::Error>>(()) })?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub async fn prompt_native(
    options: &PromptOptions,
    answer: Option<&str>,
    writer: &mut (dyn Write + Send),
    cancellation: &PromptCancellation,
    render: RenderOptions,
) -> PromptResult<String> {
    // Early cancellation must not acquire native ownership or invoke supplied-answer callbacks.
    if cancellation.is_cancelled() {
        return Err(PromptError::Aborted);
    }
    // Check static limits before any native I/O or copying an explicit/default answer.
    options.check()?;
    if let Some(answer) = answer {
        // Explicit answers bypass stdin, but still use the shared checked validation/render
        // pipeline.
        return supplied(options, answer, writer, cancellation, render).await;
    }
    if !io::stdin().is_terminal() || !cfg!(feature = "tty") {
        // Never start a reader that could remain blocked when a pipe has no available data.
        return match &options.default {
            Some(default) => supplied(options, default, writer, cancellation, render).await,
            None => Err(PromptError::NonInteractive),
        };
    }
    #[cfg(feature = "tty")]
    {
        // The wrapper owns the worker across all awaits; drop/unwind performs joined cleanup.
        let mut input = crate::tty::TtyInput::acquire(cancellation.clone())?;
        let result = prompt(options, &mut input, writer, cancellation, render).await;
        // Reap before returning success; preserve a primary failure if restoration also fails.
        let cleanup = input.close().await;
        match result {
            Err(error) => Err(error),
            Ok(value) => {
                // A cleanup failure prevents reporting a successful native prompt.
                cleanup?;
                Ok(value)
            }
        }
    }
    #[cfg(not(feature = "tty"))]
    Err(PromptError::NonInteractive)
}

/// Submit exactly one supplied answer; rejection carries its checked validation message.
async fn supplied(
    options: &PromptOptions,
    answer: &str,
    writer: &mut (dyn Write + Send),
    cancellation: &PromptCancellation,
    render: RenderOptions,
) -> PromptResult<String> {
    // State construction checks/owns the supplied candidate; only submission is an input event.
    let mut input = BufferedInput::new([PromptEvent::Enter].into_iter());
    prompt_inner(
        options,
        &mut input,
        writer,
        cancellation,
        render,
        Some(answer),
    )
    .await
}
