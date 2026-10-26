//! Owned failures, source chains and explicit public/developer diagnostic views.
//!
//! ```
//! use stackpress_lib::{Error, ErrorKind};
//! let error = Error::invalid("expected an object").at_path("profile");
//! assert_eq!(error.kind(), ErrorKind::InvalidInput);
//! assert_eq!(error.path(), Some("profile"));
//! ```

use std::backtrace::Backtrace;
use std::fmt;
use std::panic::Location;

use serde::Serialize;

/// A framework operation's fallible result.
pub type Result<T> = std::result::Result<T, Error>;

/// Stable categories; messages describe the particular failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum ErrorKind {
    /// Invalid input, path, header, cookie or value shape.
    InvalidInput,
    /// An explicit size/depth/count budget was exceeded.
    Limit,
    /// Pattern syntax or registration identity is invalid.
    Registration,
    /// A read or send was already attempted or finalized.
    State,
    /// A callback or underlying library returned an ordinary error.
    External,
    /// A domain handler deliberately reported a failure.
    Domain,
}

/// A captured source location, independent of a native/V8 stack format.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct SourceLocation {
    /// Source filename captured at the actual call site.
    pub file: String,
    /// One-based source line.
    pub line: u32,
    /// One-based source column.
    pub column: u32,
}

impl SourceLocation {
    /// Capture the caller of this synchronous constructor.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::SourceLocation;
    /// let site = SourceLocation::caller();
    /// assert!(site.line > 0);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn caller() -> Self {
        // Location forwarding preserves an external caller through tracked wrappers.
        let caller = Location::caller();
        Self {
            file: caller.file().to_owned(),
            line: caller.line(),
            column: caller.column(),
        }
    }
}

/// Byte offsets identifying a bounded range in a source/input document.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct SourceSpan {
    /// Inclusive starting byte offset.
    pub start: usize,
    /// Exclusive ending byte offset.
    pub end: usize,
}

/// One immutable listener phase in an asynchronous logical call path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct InvocationFrame {
    /// Actual emitted operation/event.
    pub event: String,
    /// Original matcher text, not a compiled expression used as identity.
    pub pattern: String,
    /// Opaque listener identity rendered for diagnostics.
    pub listener: String,
    /// Explicit label or a useful generated listener description.
    pub label: String,
    /// Optional plugin owner supplied at registration.
    pub plugin: Option<String>,
    /// Higher priority executes earlier.
    pub priority: i64,
    /// Before, action or after; independent of a future host lifecycle.
    pub phase: String,
    /// Original registration call site.
    pub registered: SourceLocation,
    /// Replacement call site when a listener was deliberately customized.
    pub replaced: Option<SourceLocation>,
    /// Actual before/after hook registration site when that phase runs.
    pub phase_registered: Option<SourceLocation>,
}

/// Public error data; source paths, causes and backtraces are absent.
#[derive(Clone, Debug, Serialize)]
pub struct PublicError {
    /// Application status code; an adapter validates native HTTP legality.
    pub code: u16,
    /// Status label, kept separate from execution outcomes.
    pub status: String,
    /// Intentionally disclosed message.
    pub error: String,
    /// Explicit field validation messages.
    pub errors: serde_json::Value,
}

/// Diagnostic state is boxed to keep Result's error representation small.
struct Details {
    /// Stable classification separate from application status and message spelling.
    kind: ErrorKind,
    /// Original developer message; never automatically disclosed on the wire.
    message: String,
    /// Optional intentional disclosure; absence produces a generic public description.
    public_message: Option<String>,
    /// Application status value; a later native adapter validates its own range.
    code: u16,
    /// Deliberately supplied public validation detail payload.
    errors: serde_json::Value,
    /// Optional nested data context enriched without replacing origin/cause.
    path: Option<String>,
    /// Optional checked source byte range.
    span: Option<SourceSpan>,
    /// Original synchronous construction location retained across awaits.
    origin: SourceLocation,
    /// Unique owned original source chain; Error intentionally has no Clone.
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
    /// Complete deepest logical path, attached once to avoid repeated outer prefixes.
    frames: Vec<InvocationFrame>,
    /// Best-effort native capture, independently optional/disabled by the runtime environment.
    backtrace: Backtrace,
}

/// Owned failure with an original cause and invocation-local breadcrumbs.
///
/// It is not Clone: copying a report must not quietly alias an owned cause.
/// Display/Debug are developer views; public_view is the deliberate wire view.
pub struct Error(Box<Details>);

impl Error {
    /// Construct an undisclosed domain failure, defaulting to application 500.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let error = Error::new("private diagnostic");
    /// assert_eq!(error.public_view().error, "Internal Server Error");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn new(message: impl Into<String>) -> Self {
        // Capture once at construction, not again when outer dispatch annotates it.
        Self(Box::new(Details {
            kind: ErrorKind::Domain,
            message: message.into(),
            public_message: None,
            code: 500,
            errors: serde_json::json!({}),
            path: None,
            span: None,
            origin: SourceLocation::caller(),
            cause: None,
            frames: Vec::new(),
            backtrace: Backtrace::capture(),
        }))
    }

    /// Report a checked input failure with an intentionally public description.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::invalid("expected an object").code(), 400);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn invalid(message: impl Into<String>) -> Self {
        // Input descriptions are deliberately public, unlike arbitrary domain/source messages.
        let message = message.into();
        Self::new(message.clone())
            .with_kind(ErrorKind::InvalidInput)
            .with_code(400)
            .with_public_message(message)
    }

    /// Report exhaustion of an explicit resource budget.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::limit("too deep").kind(), ErrorKind::Limit);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn limit(message: impl Into<String>) -> Self {
        // Resource exhaustion remains a checked input failure with a distinct stable category.
        Self::invalid(message).with_kind(ErrorKind::Limit)
    }

    /// Preserve an arbitrary owned Send + Sync cause for normal question-mark flow.
    ///
    /// Display is `external operation failed`; the underlying explanation is available
    /// through [`std::error::Error::source`], retaining its concrete type for downcasting.
    /// This deliberately differs from copying the source's Display into the wrapper.
    /// At a known operation boundary prefer [`Error::new`] with distinct context and
    /// [`Error::with_cause`]. Developer formatters should print each layer once;
    /// neither Display nor source traversal is a public disclosure policy.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let cause = "invalid".parse::<u32>().unwrap_err();
    /// let error = Error::from_source(cause);
    /// assert!(std::error::Error::source(&error).is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[track_caller]
    pub fn from_source(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        // Describe this generic boundary without repeating the original cause's explanation.
        Self::new("external operation failed")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    }
    /// Attach an owned cause deliberately, preserving its own nested chain.
    /// The wrapper message should describe distinct operation context rather than
    /// repeat the cause's Display. This method never flattens or deduplicates sources.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let error = Error::new("operation failed").with_cause(std::io::Error::other("reader"));
    /// assert!(std::error::Error::source(&error).is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn with_cause(mut self, cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        // Ownership is explicit and non-Clone so attaching a source cannot silently alias a report.
        self.0.cause = Some(Box::new(cause));
        self
    }

    /// Inspect the stable failure category.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::invalid("bad input").kind(), ErrorKind::InvalidInput);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn kind(&self) -> ErrorKind {
        // Return stable classification independently of message spelling or response status.
        self.0.kind
    }
    /// Inspect the application status code.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::invalid("bad input").code(), 400);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn code(&self) -> u16 {
        // Application codes remain values; a transport adapter checks native legality later.
        self.0.code
    }
    /// Inspect the original developer message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::new("private").message(), "private");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn message(&self) -> &str {
        // Developer inspection may expose the original message; wire conversion uses public_view
        // instead.
        &self.0.message
    }
    /// Inspect a qualified nested data path when supplied.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(
    ///     Error::invalid("type").at_path("profile.age").path(),
    ///     Some("profile.age")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn path(&self) -> Option<&str> {
        // Borrow optional nested context without manufacturing a path when none was supplied.
        self.0.path.as_deref()
    }
    /// Inspect the captured construction location.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert!(Error::new("failure").origin().line > 0);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn origin(&self) -> &SourceLocation {
        // Retain the construction site instead of replacing it with an outer async polling
        // location.
        &self.0.origin
    }
    /// Inspect asynchronous logical breadcrumbs, root first.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert!(Error::new("uninvoked").frames().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn frames(&self) -> &[InvocationFrame] {
        // The innermost annotated failure owns the complete root-first invocation path.
        &self.0.frames
    }
    /// Inspect the original optional source span.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(
    ///     Error::invalid("syntax")
    ///         .with_span(2, 5)?
    ///         .span()
    ///         .unwrap()
    ///         .end,
    ///     5
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn span(&self) -> Option<SourceSpan> {
        // A span is optional structured source context, not a parsed native backtrace string.
        self.0.span
    }
    /// Inspect the original native capture, which may be disabled/unavailable.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let error = Error::new("failure");
    /// let _capture = error.backtrace();
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn backtrace(&self) -> &Backtrace {
        // Native backtraces can be disabled; logical frames and source origins remain usable
        // independently.
        &self.0.backtrace
    }
    /// Set an intentional typed category without replacing the original cause.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(
    ///     Error::new("failure").with_kind(ErrorKind::External).kind(),
    ///     ErrorKind::External
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn with_kind(mut self, kind: ErrorKind) -> Self {
        // Change classification without destroying the original message, source or registration
        // frames.
        self.0.kind = kind;
        self
    }
    /// Set an application code without treating it as a dispatch outcome.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(Error::new("conflict").with_code(409).code(), 409);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn with_code(mut self, code: u16) -> Self {
        // Store the application status separately from cooperative dispatch flow.
        self.0.code = code;
        self
    }
    /// Choose a message safe for public disclosure.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(
    ///     Error::new("private")
    ///         .with_public_message("Try again")
    ///         .public_view()
    ///         .error,
    ///     "Try again"
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn with_public_message(mut self, message: impl Into<String>) -> Self {
        // Disclosure must be intentional rather than inferred from an internal error Display.
        self.0.public_message = Some(message.into());
        self
    }
    /// Supply explicitly public validation details.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let error = Error::invalid("validation").with_errors(serde_json::json!({"id": "required"}));
    /// assert_eq!(error.public_view().errors["id"], "required");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn with_errors(mut self, errors: serde_json::Value) -> Self {
        // Only explicitly supplied validation fields belong in the public error envelope.
        self.0.errors = errors;
        self
    }
    /// Attach a nested value path while retaining the original construction site.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// assert_eq!(
    ///     Error::invalid("type").at_path("profile.age").path(),
    ///     Some("profile.age")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    #[must_use = "the enriched error must be used"]
    pub fn at_path(mut self, path: impl Into<String>) -> Self {
        // Attach data context while retaining the error's original source location/cause.
        self.0.path = Some(path.into());
        self
    }
    /// Attach a checked source span.
    ///
    /// # Errors
    /// Rejects a reversed byte range.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let error = Error::invalid("syntax").with_span(2, 5)?;
    /// assert_eq!(error.span().unwrap().start, 2);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn with_span(mut self, start: usize, end: usize) -> Result<Self> {
        // Validate ordering before storing byte offsets; reversed ranges are checked failures.

        // A reversed range cannot describe a source span, so leave it unstored and return input
        // failure.
        if start > end {
            return Err(Self::invalid("source span is reversed"));
        }
        self.0.span = Some(SourceSpan { start, end });
        Ok(self)
    }
    /// Attach a complete deepest invocation path once, avoiding repeated prefixes.
    pub(crate) fn with_frames(mut self, frames: Vec<InvocationFrame>) -> Self {
        // The innermost dispatcher has the most complete path; outer ? preserves it.
        if self.0.frames.is_empty() {
            self.0.frames = frames;
        }
        self
    }
    /// Convert only explicitly public fields, never developer paths or source chains.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Error, ErrorKind};
    /// let public = Error::new("private cause")
    ///     .with_public_message("Request failed")
    ///     .public_view();
    /// assert_eq!(public.error, "Request failed");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn public_view(&self) -> PublicError {
        // Select only intentionally public fields; origin, frames, causes and backtraces never
        // enter this object.
        let status = crate::status::Status::from_code(self.code());
        PublicError {
            code: self.code(),
            status: status.message.to_string(),
            error: self
                .0
                .public_message
                .clone()
                .unwrap_or_else(|| "Internal Server Error".into()),
            // Deep-copy ordinary public fields; the private source chain is deliberately absent.
            errors: self.0.errors.clone(),
        }
    }
}

impl fmt::Display for Error {
    /// Render the developer message and qualified data path.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display is a developer view: include the original message and optional qualified data
        // context.
        write!(formatter, "{}", self.0.message)?;
        if let Some(path) = &self.0.path {
            write!(formatter, " at {path}")?;
        }
        Ok(())
    }
}

impl fmt::Debug for Error {
    /// Render deliberate diagnostic state without any request/session values.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Debug is also a developer view, but never pulls request/session payloads into the
        // diagnostic.
        formatter
            .debug_struct("Error")
            .field("kind", &self.kind())
            .field("message", &self.message())
            .field("path", &self.path())
            .field("origin", &self.origin())
            .field("frames", &self.frames())
            .field("cause", &self.0.cause)
            .finish_non_exhaustive()
    }
}

impl std::error::Error for Error {
    /// Preserve the original cause for downcasting and nested diagnostic tools.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Expose the original owned trait object so ordinary source-chain tools and downcasts still
        // work.
        self.0
            .cause
            .as_deref()
            .map(|cause| cause as &dyn std::error::Error)
    }
}

impl From<serde_json::Error> for Error {
    /// Keep the serialization/parser cause rather than only its formatted message.
    #[track_caller]
    fn from(cause: serde_json::Error) -> Self {
        // Tracked conversion retains the underlying error instead of flattening it to a String.
        Self::new("JSON operation failed")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    }
}
impl From<std::io::Error> for Error {
    /// Preserve an adapter's original I/O failure.
    #[track_caller]
    fn from(cause: std::io::Error) -> Self {
        // Tracked conversion retains the underlying error instead of flattening it to a String.
        Self::new("I/O operation failed")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    }
}
impl From<std::num::ParseIntError> for Error {
    /// Permit ordinary numeric validation through question-mark in handlers.
    #[track_caller]
    fn from(cause: std::num::ParseIntError) -> Self {
        // Tracked conversion retains the underlying error instead of flattening it to a String.
        Self::new("integer parsing failed")
            .with_kind(ErrorKind::External)
            .with_cause(cause)
    }
}

#[cfg(test)]
mod tests {
    use super::{Error, InvocationFrame, SourceLocation};
    use std::error::Error as _;

    /// Wrapping and enrichment preserve inner origin, logical frames and downcastable causes.
    #[test]
    fn contextual_layers_retain_invocation_metadata_and_disclosure() {
        let frame = InvocationFrame {
            event: "read".into(),
            pattern: "read".into(),
            listener: "r1:l1".into(),
            label: "reader".into(),
            plugin: Some("storage".into()),
            priority: 7,
            phase: "action".into(),
            registered: SourceLocation::caller(),
            replaced: None,
            phase_registered: None,
        };
        let inner = Error::from_source(std::io::Error::other("private path"))
            .with_frames(vec![frame.clone()]);
        let origin = inner.origin().clone();
        // An outer operation owns its own context; the source keeps the inner metadata intact.
        let outer = Error::new("startup storage read failed").with_cause(inner);
        let inner = outer.source().unwrap().downcast_ref::<Error>().unwrap();
        assert_eq!(inner.origin(), &origin);
        assert_eq!(inner.frames(), &[frame]);
        assert!(inner.source().unwrap().is::<std::io::Error>());
        let public = serde_json::to_string(&outer.public_view()).unwrap();
        assert!(!public.contains("private path"));
        assert!(!public.contains("startup storage"));
        assert_eq!(outer.public_view().error, "Internal Server Error");
    }
}
