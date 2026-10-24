//! Prompt configuration and callback/error contracts, independent of terminal I/O.

use std::fmt;
use std::future::Future;
use std::io;
use std::pin::Pin;

use regex::Regex;

use super::{InvalidMode, Prefill, Validation};

/// A prompt result preserves cancellation, invalid answers and I/O as different outcomes.
pub type PromptResult<T> = Result<T, PromptError>;

/// Typed prompt termination; validation rejection itself is [`Validation::Invalid`].
#[derive(Debug)]
pub enum PromptError {
    /// Explicit cancellation, including native Ctrl-C.
    Aborted,
    /// Input ended without an accepted answer.
    Eof,
    /// A single supplied answer was rejected; this carries the bounded validation message.
    InvalidAnswer(String),
    /// Native stdin is a pipe, or TTY support is disabled, with no supplied answer/default.
    NonInteractive,
    /// A configured input, error, render or pending-event budget was exceeded.
    Limit(&'static str),
    /// Invalid options or an externally supplied validation-completion event.
    Configuration(&'static str),
    /// Native input, output or terminal ownership failed.
    Io(io::Error),
    /// A user validator/transformer failed; its exact cause is retained.
    Callback(Box<dyn std::error::Error + Send + Sync>),
}

impl PromptError {
    /// Preserve a validator/transformer cause without coercing failure to invalid input.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptError;
    /// use std::error::Error;
    /// let error = PromptError::callback(std::io::Error::other("validation service failed"));
    /// assert!(error.source().is_some());
    /// ```
    pub fn callback(error: impl std::error::Error + Send + Sync + 'static) -> Self {
        // Retain the concrete source behind a Send+Sync owner, without exposing its text by
        // default.
        Self::Callback(Box::new(error))
    }
}

impl fmt::Display for PromptError {
    /// Render concise errors without exposing answer text or arbitrary callback messages.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Error display names the failure class; source/answer text requires explicit inspection.
        match self {
            Self::Aborted => formatter.write_str("prompt was aborted"),
            Self::Eof => formatter.write_str("prompt input ended"),
            Self::InvalidAnswer(_) => formatter.write_str("supplied prompt answer was rejected"),
            Self::NonInteractive => {
                formatter.write_str("prompt requires an explicit answer or terminal")
            }
            Self::Limit(message) | Self::Configuration(message) => formatter.write_str(message),
            Self::Io(_) => formatter.write_str("prompt I/O failed"),
            Self::Callback(_) => formatter.write_str("prompt callback failed"),
        }
    }
}

impl std::error::Error for PromptError {
    /// Expose causes only to callers explicitly requesting developer diagnostics.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Structured rejections/cancellation have no error cause; native and callback failures do.
        match self {
            Self::Io(error) => Some(error),
            Self::Callback(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

impl From<io::Error> for PromptError {
    /// Retain the native input/output failure.
    fn from(error: io::Error) -> Self {
        // Preserve I/O kind and source instead of converting a transport failure to invalid input.
        Self::Io(error)
    }
}

/// One owned, movable validation future; cancellation drops it without spawning a task.
type ValidationFuture = Pin<Box<dyn Future<Output = PromptResult<Validation>> + Send>>;
/// A synchronous validator borrows the submitted answer only for its call.
type SyncValidator = Box<dyn Fn(&str) -> PromptResult<Validation> + Send + Sync>;
/// An asynchronous validator owns its bounded submitted answer.
type AsyncValidator = Box<dyn Fn(String) -> ValidationFuture + Send + Sync>;
/// Display-only transformation; `is_final` distinguishes the accepted final render.
pub type Transformer = Box<dyn Fn(&str, bool) -> PromptResult<String> + Send + Sync>;

/// Explicit sync/async validation without JavaScript truthiness conversions.
pub enum Validator {
    /// Runs on the driver's thread; caller code must not block indefinitely.
    Sync(SyncValidator),
    /// Polled alongside cancellation/input; dropped on cancellation or driver drop.
    Async(AsyncValidator),
}

impl Validator {
    /// Adapt an ordinary borrowed synchronous validator.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{Validation, Validator};
    /// let check = Validator::sync(|value| Ok(if value.is_empty() {
    ///     Validation::Invalid("Required".into())
    /// } else { Validation::Valid }));
    /// ```
    pub fn sync(
        callback: impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static,
    ) -> Self {
        // Erase configuration-time callback type only; execution still borrows one candidate.
        Self::Sync(Box::new(callback))
    }

    /// Adapt a named async function or closure owning its answer, without spawning it.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptResult, Validation, Validator};
    /// async fn check(_: String) -> PromptResult<Validation> {
    ///     Ok(Validation::Valid)
    /// }
    /// let validator = Validator::asynchronous(check);
    /// ```
    pub fn asynchronous<F, Fut>(callback: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = PromptResult<Validation>> + Send + 'static,
    {
        // Future ownership permits cancellation by drop without a hidden spawned task.
        Self::Async(Box::new(move |answer| Box::pin(callback(answer))))
    }
}

/// Finite per-state/per-render budgets; these do not bound arbitrary user callback allocation.
#[derive(Clone, Copy, Debug)]
pub struct PromptLimits {
    /// Maximum UTF-8 bytes of a default, candidate or queued input text in aggregate.
    pub answer_bytes: usize,
    /// Maximum UTF-8 bytes in an error before display sanitization.
    pub error_bytes: usize,
    /// Maximum bytes in one sanitized prompt plus error/terminal framing.
    pub render_bytes: usize,
    /// Maximum queued edit/Enter events while a validator is awaiting completion.
    pub pending_events: usize,
}

impl Default for PromptLimits {
    /// Select conservative defaults; callers may deliberately choose different budgets.
    fn default() -> Self {
        // Bound independent text/render/queue dimensions; callbacks retain their own resource
        // policy.
        Self {
            answer_bytes: 4096,
            error_bytes: 1024,
            render_bytes: 8192,
            pending_events: 64,
        }
    }
}

/// Owned prompt policy; no process streams, locks, signals or runtime tasks are retained.
pub struct PromptOptions {
    /// Question text; terminal control characters are sanitized by the renderer.
    pub message: String,
    /// Used for empty Enter and shown as a preview until cleared or replaced.
    pub default: Option<String>,
    /// Reject an empty candidate before invoking a pattern or user validator.
    pub required: bool,
    /// Tab preview or immediately editable default.
    pub prefill: Prefill,
    /// Preserve or clear the candidate/default after invalid validation.
    pub invalid: InvalidMode,
    /// Immutable Rust regex, with Rust-regex syntax and search semantics.
    pattern: Option<Regex>,
    /// Message on pattern mismatch.
    pub pattern_error: String,
    /// Optional validator, called only after required and regex checks succeed.
    validator: Option<Validator>,
    /// Optional live/final display transformation; the returned answer is never transformed.
    transformer: Option<Transformer>,
    /// Answer/error/render/pending-event budgets.
    pub limits: PromptLimits,
}

impl PromptOptions {
    /// Build optional input with Tab prefill and Keep invalid policy.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, PromptState};
    /// let options = PromptOptions::new("Name?");
    /// assert!(!options.required);
    /// assert_eq!(PromptState::new(&options)?.current(), "");
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        // Start optional, with preview-only defaults and correction-friendly invalid handling.
        Self {
            message: message.into(),
            default: None,
            required: false,
            prefill: Prefill::Tab,
            invalid: InvalidMode::Keep,
            pattern: None,
            pattern_error: "Invalid input".into(),
            validator: None,
            transformer: None,
            limits: PromptLimits::default(),
        }
    }

    /// Compile a Rust regex before input acquisition, retaining search semantics.
    ///
    /// # Errors
    /// Returns regex's syntax/size error without modifying any terminal state.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptOptions;
    /// let options = PromptOptions::new("Code?").pattern("^[A-Z]{3}$")?;
    /// let malformed = String::from("[");
    /// assert!(PromptOptions::new("Code?").pattern(&malformed).is_err());
    /// # Ok::<(), regex::Error>(())
    /// ```
    pub fn pattern(mut self, source: &str) -> Result<Self, regex::Error> {
        // Compile before storing so an invalid pattern cannot escape in the options.
        self.pattern = Some(Regex::new(source)?);
        Ok(self)
    }

    /// Use a precompiled Rust regex without reparsing it.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptOptions;
    /// let pattern = regex::Regex::new("^[0-9]+$")?;
    /// let options = PromptOptions::new("Number?").with_pattern(pattern);
    /// # Ok::<(), regex::Error>(())
    /// ```
    pub fn with_pattern(mut self, pattern: Regex) -> Self {
        // Move the immutable regex into this one prompt policy.
        self.pattern = Some(pattern);
        self
    }

    /// Remove the pattern rule, preserving required/user validation.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptOptions;
    /// let options = PromptOptions::new("Text?").pattern("^yes$")?.without_pattern();
    /// # Ok::<(), regex::Error>(())
    /// ```
    pub fn without_pattern(mut self) -> Self {
        // The required and custom checks remain in their existing order.
        self.pattern = None;
        self
    }

    /// Register an unboxed synchronous validator after built-in checks.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, PromptResult, Validation};
    /// fn check(value: &str) -> PromptResult<Validation> {
    ///     Ok(if value == "admin" {
    ///         Validation::Valid
    ///     } else {
    ///         Validation::Invalid("Use admin".into())
    ///     })
    /// }
    /// let options = PromptOptions::new("Role?").validator(check);
    /// ```
    pub fn validator(
        mut self,
        callback: impl Fn(&str) -> PromptResult<Validation> + Send + Sync + 'static,
    ) -> Self {
        // Adapt once during configuration; submission only borrows the stored function.
        self.validator = Some(Validator::sync(callback));
        self
    }

    /// Register a named async function or closure owning its bounded candidate.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, PromptResult, Validation};
    /// async fn check(value: String) -> PromptResult<Validation> {
    ///     Ok(if value == "admin" {
    ///         Validation::Valid
    ///     } else {
    ///         Validation::Invalid("Use admin".into())
    ///     })
    /// }
    /// let options = PromptOptions::new("Role?").async_validator(check);
    /// // A closure also works; neither form needs caller-side boxing.
    /// let options = options.async_validator(|_| async { Ok(Validation::Valid) });
    /// ```
    pub fn async_validator<F, Fut>(mut self, callback: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = PromptResult<Validation>> + Send + 'static,
    {
        // The adapter boxes the future internally; validation still spawns no task.
        self.validator = Some(Validator::asynchronous(callback));
        self
    }

    /// Remove custom validation, preserving the built-in required/pattern checks.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::{PromptOptions, Validation};
    /// let options = PromptOptions::new("Text?")
    ///     .validator(|_| Ok(Validation::Valid)).without_validator();
    /// ```
    pub fn without_validator(mut self) -> Self {
        // Dropping this callback releases any explicitly captured application owners.
        self.validator = None;
        self
    }

    /// Register an unboxed live/final display transformation without changing accepted answers.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptOptions;
    /// let options = PromptOptions::new("Token?").transform(|value, is_final| {
    ///     Ok(if is_final {
    ///         "[saved]".into()
    ///     } else {
    ///         "*".repeat(value.chars().count())
    ///     })
    /// });
    /// ```
    pub fn transform(
        mut self,
        callback: impl Fn(&str, bool) -> PromptResult<String> + Send + Sync + 'static,
    ) -> Self {
        // Display policy stays separate from candidate validation and answer storage.
        self.transformer = Some(Box::new(callback));
        self
    }

    /// Return to displaying the original answer/default without a custom transformation.
    ///
    /// # Examples
    /// ```
    /// use stackpress_cli::prompt::PromptOptions;
    /// let options = PromptOptions::new("Name?")
    ///     .transform(|_, _| Ok("hidden".into())).without_transform();
    /// ```
    pub fn without_transform(mut self) -> Self {
        // Removing display policy never alters the configured default or validation rules.
        self.transformer = None;
        self
    }

    /// Apply display policy while keeping callback storage private to options.
    pub(crate) fn display_value(&self, value: &str, is_final: bool) -> PromptResult<String> {
        // The renderer owns bounds/sanitization; this boundary owns callback invocation only.
        match &self.transformer {
            Some(transformer) => transformer(value, is_final),
            None => Ok(value.to_owned()),
        }
    }

    /// Validate static sizes before acquiring terminal input or calling user code.
    pub(crate) fn check(&self) -> PromptResult<()> {
        // Zero answer bytes deliberately permits only empty answers; rendering needs framing room.
        if self.limits.render_bytes < 32 || self.limits.pending_events == 0 {
            // Refuse budgets that cannot even frame a prompt or retain one queued edit.
            return Err(PromptError::Configuration(
                "render/pending-event budgets are too small",
            ));
        }
        if self.message.len() > self.limits.render_bytes
            || self
                .default
                .as_ref()
                .is_some_and(|value| value.len() > self.limits.answer_bytes)
            || self.pattern_error.len() > self.limits.error_bytes
        {
            // Reject static text before state cloning, user callbacks or terminal acquisition.
            return Err(PromptError::Limit("prompt options exceed text budgets"));
        }
        Ok(())
    }

    /// Run built-in checks first, then exactly one user validation, preserving callback errors.
    pub(crate) async fn validate(&self, value: String) -> PromptResult<Validation> {
        // Empty required input cannot trigger regex or user work.
        if self.required && value.is_empty() {
            return Ok(Validation::Invalid("You must provide a value".into()));
        }
        // An immutable regex has no JS-style cursor state; rejection owns its configured message.
        if self
            .pattern
            .as_ref()
            .is_some_and(|pattern| !pattern.is_match(&value))
        {
            return Ok(Validation::Invalid(self.pattern_error.clone()));
        }
        // Exactly one custom check runs after built-ins; callback errors terminate the prompt.
        match &self.validator {
            Some(Validator::Sync(callback)) => callback(&value),
            // Await in the driver-owned future so cancellation drops this work and its candidate.
            Some(Validator::Async(callback)) => callback(value).await,
            None => Ok(Validation::Valid),
        }
    }
}
