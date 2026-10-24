//! Pure serialized editing/validation transitions; there is no I/O or async callback here.

use super::{PromptError, PromptOptions, PromptResult};

/// Initial handling of a configured default.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Prefill {
    /// Preview the default, copying it into editing text only on Tab/Enter.
    Tab,
    /// Begin with the default as editable text.
    Editable,
}

/// Invalid candidate policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InvalidMode {
    /// Retain the submitted candidate for correction.
    Keep,
    /// Clear both current input and default so a repeated Enter cannot restore it.
    Clear,
}

/// Exact validation result; invalid input remains editable rather than terminating the driver.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Validation {
    /// Accept the submitted candidate.
    Valid,
    /// Display this bounded message and allow correction.
    Invalid(String),
}

/// Owned event input; text is UTF-8 and backspace removes one Unicode scalar.
#[derive(Debug, Eq, PartialEq)]
pub enum PromptEvent {
    /// Append printable text; controls must be represented by their named events.
    Text(String),
    /// Remove the final scalar, or clear the default preview if input is empty.
    Backspace,
    /// Copy the preview into empty editing text under Tab prefill.
    Tab,
    /// Submit the current value, or the remaining default if current input is empty.
    Enter,
    /// Explicit cancellation invalidates an in-flight validation token.
    Cancel,
    /// End of supplied input; no implicit submission.
    Eof,
    /// Pure-state completion; drivers generate these internally, never trust input to do so.
    Validated {
        /// Token returned by the matching Validate action.
        id: u64,
        /// Checked validator result.
        result: Validation,
    },
}

/// Exhaustive state visible to read-only observers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PromptStatus {
    /// Input can be edited/submitted.
    Editing,
    /// One candidate is being validated; only matching completion or cancellation can change it.
    Validating(u64),
    /// A value was accepted and becomes final display/answer text.
    Complete,
    /// Explicit cancellation; no later completion can revive the state.
    Aborted,
    /// Input ended before an accepted value.
    Ended,
}

/// Requested driver action after one pure transition.
#[derive(Debug, Eq, PartialEq)]
pub enum PromptAction {
    /// Stale completion, repeated terminal event or inapplicable edit; no action.
    None,
    /// Redraw the editing/error state.
    Render,
    /// Validate this candidate once; return the exact token with its result.
    Validate {
        /// Monotonically increasing, checked per-state token.
        id: u64,
        /// Owned bounded candidate.
        value: String,
    },
    /// Return this untransformed accepted value after final rendering.
    Complete(String),
    /// Terminate as explicit cancellation.
    Aborted,
    /// Terminate as end of input.
    Eof,
}

/// Non-Clone mutable prompt state. Pass the same options to every transition/render.
#[derive(Debug)]
pub struct PromptState {
    /// Editable UTF-8 string, never byte-sliced for deletion.
    current: String,
    /// Remaining default preview, cleared independently from the configured default.
    default: String,
    /// Latest bounded invalid-input message; erased on the next edit.
    error: String,
    /// State gates all edits and validation completion.
    status: PromptStatus,
    /// Last issued token; checked increment prevents wraparound reuse.
    generation: u64,
}

impl PromptState {
    /// Validate options and create initial editing state without invoking user code.
    ///
    /// # Errors
    /// Rejects invalid option sizes/budgets.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, PromptState, PromptStatus};
    /// let state = PromptState::new(&PromptOptions::new("Name?"))?;
    /// assert_eq!(state.status(), PromptStatus::Editing);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(options: &PromptOptions) -> PromptResult<Self> {
        // Check configured budgets before cloning either preview or editable default.
        options.check()?;
        let default = options.default.clone().unwrap_or_default();
        let current = if options.prefill == Prefill::Editable {
            // Editable input starts as real candidate text, not merely display preview.
            default.clone()
        } else {
            // Tab mode retains the independent preview until an explicit editing action uses it.
            String::new()
        };
        // Generation zero has never been issued; the first Enter reserves token one.
        Ok(Self {
            current,
            default,
            error: String::new(),
            status: PromptStatus::Editing,
            generation: 0,
        })
    }

    /// Construct one supplied candidate independently of preview/Editable prefill policy.
    pub(super) fn supplied(options: &PromptOptions, answer: &str) -> PromptResult<Self> {
        // Share edit rules and check before making an owned copy of the supplied answer.
        check_text(answer, options.limits.answer_bytes)?;
        let mut state = Self::new(options)?;
        // Explicit input, including empty text, replaces both preview and Editable prefill.
        state.current = answer.to_owned();
        state.default.clear();
        Ok(state)
    }

    /// Borrow current editing/final text, excluding a Tab preview.
    pub fn current(&self) -> &str {
        // Observers cannot alter the frozen validation candidate through this borrow.
        &self.current
    }

    /// Borrow the remaining default preview.
    pub fn default_value(&self) -> &str {
        // Return live preview state, which can differ from the original configured default.
        &self.default
    }

    /// Borrow the last invalid-input message.
    pub fn error_message(&self) -> &str {
        // Expose the checked invalid message without permitting edits to validation state.
        &self.error
    }

    /// Inspect the editing/validation/finalization state.
    pub fn status(&self) -> PromptStatus {
        // Copy only the enum/token; mutable candidate storage remains owned by this machine.
        self.status
    }

    /// Apply one bounded edit, submit, cancellation or validation completion.
    ///
    /// Edits/repeated Enter during validation are ignored by the pure machine;
    /// the driver separately queues a bounded number and replays after rejection.
    /// Stale completions and all events after termination are ignored.
    ///
    /// # Errors
    /// Rejects controls in Text, answer/error limits and validation-token exhaustion.
    /// Failed checks leave the prior state unchanged.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptAction, PromptEvent, PromptOptions, PromptState};
    /// let options = PromptOptions::new("Name?");
    /// let mut state = PromptState::new(&options)?;
    /// state.apply(&options, PromptEvent::Text("Ada😀".into()))?;
    /// state.apply(&options, PromptEvent::Backspace)?;
    /// assert_eq!(state.current(), "Ada");
    /// assert!(matches!(state.apply(&options, PromptEvent::Enter)?,
    ///     PromptAction::Validate { value, .. } if value == "Ada"));
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn apply(
        &mut self,
        options: &PromptOptions,
        event: PromptEvent,
    ) -> PromptResult<PromptAction> {
        // Terminal states cannot be revived by edits, EOF, or a late validation completion.
        if matches!(
            self.status,
            PromptStatus::Complete | PromptStatus::Aborted | PromptStatus::Ended
        ) {
            return Ok(PromptAction::None);
        }
        match event {
            PromptEvent::Cancel => {
                // Invalidate the pending token before the caller drops any async validator.
                self.status = PromptStatus::Aborted;
                Ok(PromptAction::Aborted)
            }
            PromptEvent::Eof => {
                // EOF terminates editing without silently accepting a partial/default candidate.
                self.status = PromptStatus::Ended;
                Ok(PromptAction::Eof)
            }
            // Completion has a separate token gate; input drivers must never trust this source
            // event.
            PromptEvent::Validated { id, result } => self.complete(options, id, result),
            // The driver queues edits during validation rather than mutating its frozen candidate.
            _ if self.status != PromptStatus::Editing => Ok(PromptAction::None),
            PromptEvent::Enter => {
                // Reserve a fresh checked ID before changing the candidate or validation state.
                let id = self
                    .generation
                    .checked_add(1)
                    .ok_or(PromptError::Limit("validation token exhausted"))?;
                // Freeze exactly the candidate displayed/submitted by this generation.
                if self.current.is_empty() {
                    self.current.clone_from(&self.default);
                }
                // Publish the candidate/token pair atomically within this serialized transition.
                self.generation = id;
                self.status = PromptStatus::Validating(id);
                Ok(PromptAction::Validate {
                    id,
                    value: self.current.clone(),
                })
            }
            PromptEvent::Text(text) => {
                // Check controls/remaining bytes before appending; failed edits retain old state.
                check_text(
                    &text,
                    options
                        .limits
                        .answer_bytes
                        .saturating_sub(self.current.len()),
                )?;
                // Successful editing invalidates the previous rejection message.
                self.current.push_str(&text);
                self.error.clear();
                Ok(PromptAction::Render)
            }
            PromptEvent::Backspace => {
                // String::pop removes one full scalar, including four-byte emoji.
                if self.current.pop().is_none() {
                    // With no real input left, backspace explicitly dismisses the default preview.
                    self.default.clear();
                }
                // Correction removes stale error text before the driver redraws.
                self.error.clear();
                Ok(PromptAction::Render)
            }
            PromptEvent::Tab => {
                // Only an empty Tab-prefill candidate can import the remaining default.
                if options.prefill == Prefill::Tab && self.current.is_empty() {
                    // This becomes editable text; a following backspace edits it normally.
                    self.current.clone_from(&self.default);
                    self.error.clear();
                    return Ok(PromptAction::Render);
                }
                // Editable/nonempty candidates retain their text rather than inserting a tab byte.
                Ok(PromptAction::None)
            }
        }
    }

    /// Apply only a matching completion; invalid messages are checked before mutation.
    fn complete(
        &mut self,
        options: &PromptOptions,
        id: u64,
        result: Validation,
    ) -> PromptResult<PromptAction> {
        // Ignore stale generations without changing either accepted state or current error text.
        if self.status != PromptStatus::Validating(id) {
            return Ok(PromptAction::None);
        }
        match result {
            Validation::Valid => {
                // Final state owns the raw answer; rendering transformations cannot change it.
                self.error.clear();
                self.status = PromptStatus::Complete;
                Ok(PromptAction::Complete(self.current.clone()))
            }
            Validation::Invalid(message) => {
                // An oversized user message is a terminal limit failure, not partial state
                // mutation.
                if message.len() > options.limits.error_bytes {
                    return Err(PromptError::Limit("validation error exceeds byte budget"));
                }
                // Clear removes the preview so Enter cannot restore an invalid default.
                if options.invalid == InvalidMode::Clear {
                    self.current.clear();
                    self.default.clear();
                }
                // Commit the checked rejection and reopen editing for the selected Keep/Clear
                // policy.
                self.error = message;
                self.status = PromptStatus::Editing;
                Ok(PromptAction::Render)
            }
        }
    }
}

/// Check supplied/edit text before allocation or mutation, preserving control/byte precedence.
fn check_text(text: &str, remaining: usize) -> PromptResult<()> {
    // Control bytes belong to typed events, never a raw text candidate.
    if text.chars().any(char::is_control) {
        return Err(PromptError::Configuration(
            "Text must contain no control characters",
        ));
    }
    // Byte budgets bound memory while scalar editing preserves valid UTF-8.
    if text.len() > remaining {
        return Err(PromptError::Limit("prompt answer exceeds byte budget"));
    }
    Ok(())
}
