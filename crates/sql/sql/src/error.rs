//! Typed failures independent of a native driver or host error envelope.

use std::error::Error;
use std::fmt;

/// SQL operation result with a typed, source-preserving failure.
pub type Result<T> = std::result::Result<T, SqlError>;

/// Stable classification; callers should not parse driver messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Invalid SQL shape, identifier, or value.
    InvalidQuery,
    /// Backend or operation is outside the implemented capability.
    Unsupported,
    /// Prepared parameter count differs from supplied arguments.
    Arity {
        /// Native parameter count.
        expected: usize,
        /// Supplied argument count.
        actual: usize,
    },
    /// A finite input, output or admission budget was exceeded.
    Limit,
    /// Acquiring a pooled connection exceeded its deadline.
    PoolTimedOut,
    /// Database has closed or its worker is unavailable.
    Closed,
    /// SQLite could not obtain a native lock within the busy budget.
    Busy,
    /// A query failed or was cancelled, making the transaction rollback-only.
    RollbackOnly,
    /// A row lookup or strict conversion failed.
    Decode,
    /// Invalid native options.
    Configuration,
    /// Other native failure; inspect the source explicitly for diagnostics.
    Driver,
}

/// A safe display message and an optional native cause.
///
/// Display omits SQL, binds and paths. Source diagnostics may contain sensitive
/// native details; applications choose their own disclosure policy.
pub struct SqlError {
    /// Stable classification independent of native message text.
    kind: ErrorKind,
    /// Safe public text; never interpolates SQL/binds/paths.
    message: &'static str,
    /// Retained for explicit diagnostics, omitted from implicit formatting.
    source: Option<Box<dyn Error + Send + Sync>>,
    /// A secondary explicit rollback failure; the primary kind/source remain intact.
    cleanup: Option<Box<SqlError>>,
}

impl fmt::Debug for SqlError {
    /// Keep default debug logs free of native details; inspect source explicitly instead.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SqlError")
            .field("kind", &self.kind)
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl SqlError {
    /// Construct a failure with a static, non-sensitive message.
    pub fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            message,
            source: None,
            cleanup: None,
        }
    }

    /// Preserve a driver cause without including it in the display message.
    pub fn with_source(
        kind: ErrorKind,
        message: &'static str,
        source: impl Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message,
            source: Some(Box::new(source)),
            cleanup: None,
        }
    }

    /// Return the stable failure classification.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Retain a secondary cleanup failure without replacing the primary native cause.
    pub fn with_cleanup(mut self, cleanup: SqlError) -> Self {
        self.cleanup = Some(Box::new(cleanup));
        self
    }

    /// Inspect explicit rollback's secondary failure, including its own native source.
    pub fn cleanup_error(&self) -> Option<&SqlError> {
        self.cleanup.as_deref()
    }
}

impl fmt::Display for SqlError {
    /// Render only the intentionally safe public message.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl Error for SqlError {
    /// Expose the original native cause to explicitly trusted diagnostics.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_deref()
            .map(|error| error as &(dyn Error + 'static))
    }
}
