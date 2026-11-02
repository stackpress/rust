//! Fixed-context template failures with optional source byte ranges.

use std::{fmt, ops::Range};

/// A template operation's typed failure result.
pub type TemplateResult<T> = Result<T, TemplateError>;

/// Stable template failure categories, without input values in diagnostic text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateErrorKind {
    /// Empty/ambiguous delimiters, zero budgets, or unsupported depth settings.
    InvalidOptions,
    /// Malformed tags, paths, arguments, or block structure.
    Syntax,
    /// An inline helper was not explicitly registered at compilation.
    UnknownHelper,
    /// A helper name is already registered or reserved.
    DuplicateHelper,
    /// The root is not an object or scalar pairs contain duplicate keys.
    InvalidData,
    /// A visited object member, array element, or binding is absent.
    MissingValue,
    /// A visited value has the wrong type for its operation.
    InvalidType,
    /// A source/data/output/depth/iteration/step budget was exceeded.
    Limit,
    /// Trusted compiled helper code reported a static-context failure.
    Helper,
}

/// A fixed message, category, and optional span; no source/data is retained.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateError {
    /// Stable failure classification.
    kind: TemplateErrorKind,
    /// Caller-safe static context, never interpolated input.
    message: &'static str,
    /// Template byte offsets when the failure is associated with a tag/node.
    span: Option<Range<usize>>,
}

impl TemplateError {
    /// Report a custom helper failure without retaining dynamic input.
    pub fn helper(message: &'static str) -> Self {
        Self::new(TemplateErrorKind::Helper, message)
    }

    /// Inspect the failure category independently of diagnostic wording.
    pub fn kind(&self) -> TemplateErrorKind {
        self.kind
    }

    /// Inspect an inclusive-start/exclusive-end template byte range, when known.
    pub fn span(&self) -> Option<Range<usize>> {
        self.span.clone()
    }

    /// Construct a private engine failure with static context.
    pub(super) fn new(kind: TemplateErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            message,
            span: None,
        }
    }

    /// Attach the nearest source location without replacing a more precise range.
    pub(super) fn at(mut self, span: Range<usize>) -> Self {
        if self.span.is_none() {
            self.span = Some(span);
        }
        self
    }
}

impl fmt::Display for TemplateError {
    /// Format only trusted context and byte offsets, never source/data values.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "template {:?}: {}", self.kind, self.message)?;
        if let Some(span) = &self.span {
            write!(formatter, " at bytes {}..{}", span.start, span.end)?;
        }
        Ok(())
    }
}

impl std::error::Error for TemplateError {}
