//! Named-source diagnostics with valid half-open UTF-8 byte ranges.

use std::fmt;

use serde::Serialize;

/// Stable categories for syntax, compilation and bounded-resource failures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ErrorCode {
    /// Input does not match the declaration or value grammar.
    UnexpectedToken,
    /// A required closing delimiter is absent.
    UnexpectedEof,
    /// A declaration kind is unsupported.
    InvalidDeclaration,
    /// A literal or modifier cannot be represented faithfully.
    InvalidLiteral,
    /// A declaration/member name appears twice.
    DuplicateName,
    /// An attribute refers to a declaration not available yet.
    UnknownReference,
    /// Required normalized structure is missing.
    MissingProperty,
    /// Internal normalized input violates its contract.
    ValidationError,
    /// A source, expansion or recursion budget was exceeded.
    Limit,
    /// A referenced environment key was not explicitly supplied.
    MissingEnvironment,
}

/// Safe diagnostic coordinates; display never prints source text or environment values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IdeaError {
    /// Machine-readable failure category.
    pub code: ErrorCode,
    /// Human-readable explanation without source excerpts.
    pub message: String,
    /// Inclusive UTF-8 byte offset, present for public diagnostics.
    pub start: Option<usize>,
    /// Exclusive UTF-8 byte offset, present for public diagnostics.
    pub end: Option<usize>,
    /// Caller-supplied logical file name.
    pub file: String,
    /// One-based source line.
    pub line: usize,
    /// One-based Unicode-scalar column, with tabs counted as one scalar.
    pub column: usize,
    /// Earlier declaration locations associated with a duplicate or related failure.
    pub related: Box<[crate::SourceSpan]>,
}

impl IdeaError {
    /// Construct an internal failure before the public source boundary supplies coordinates.
    pub(crate) fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        // Keeping absent coordinates internal lets callers see one consistent public contract.
        Self {
            code,
            message: message.into(),
            start: None,
            end: None,
            file: String::new(),
            line: 1,
            column: 1,
            related: Box::new([]),
        }
    }

    /// Attach raw internal byte offsets, normalized before returning to a caller.
    pub(crate) fn with_span(mut self, start: usize, end: usize) -> Self {
        self.start = Some(start);
        self.end = Some(end);
        self
    }

    /// Retain an earlier duplicate's byte range for normalization at the source boundary.
    pub(crate) fn with_related(mut self, start: usize, end: usize) -> Self {
        let mut related = self.related.into_vec();
        related.push(crate::SourceSpan {
            file: String::new(),
            start,
            end,
            line: 1,
            column: 1,
        });
        self.related = related.into_boxed_slice();
        self
    }

    /// Clamp EOF and extend partial character endpoints without slicing invalid UTF-8.
    pub(crate) fn locate(mut self, file: &str, source: &str) -> Self {
        let mut start = self.start.unwrap_or(source.len()).min(source.len());
        let mut end = self.end.unwrap_or(start).min(source.len()).max(start);
        // A scanner error may point into the first byte of a multibyte character.
        while !source.is_char_boundary(start) {
            start -= 1;
        }
        while !source.is_char_boundary(end) {
            end += 1;
        }
        let prefix = &source[..start];
        // CRLF is one line break; standalone CR is also a line break.
        let normalized = prefix.replace("\r\n", "\n").replace('\r', "\n");
        self.line = normalized.bytes().filter(|byte| *byte == b'\n').count() + 1;
        self.column = normalized.rsplit('\n').next().unwrap_or("").chars().count() + 1;
        self.start = Some(start);
        self.end = Some(end);
        self.file = file.into();
        for related in self.related.iter_mut() {
            // Related locations originate in the same parse; normalize without source excerpts.
            *related = crate::SourceSpan::new(
                &crate::SourceFile::new(file, source),
                related.start,
                related.end,
            );
        }
        self
    }
}

impl fmt::Display for IdeaError {
    /// Show actionable coordinates without automatically disclosing input contents.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{}:{}: {}",
            self.file, self.line, self.column, self.message
        )
    }
}

impl std::error::Error for IdeaError {}
