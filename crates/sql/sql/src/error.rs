//! Typed failures independent of a native driver or host error envelope.

use std::error::Error;
use std::fmt;

/// SQL operation result with a typed, source-preserving failure.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{Result, Row};
/// fn decode_id(row: &Row) -> Result<i64> {
///     row.try_get("id")
/// }
/// assert_eq!(decode_id(&Row::new(vec![("id".into(), 7.into())]))?, 7);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub type Result<T> = std::result::Result<T, SqlError>;

/// Stable classification; callers should not parse driver messages.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{ErrorKind, SqlError};
/// use std::error::Error;
/// let primary = SqlError::with_source(
///     ErrorKind::Driver,
///     "statement failed",
///     std::io::Error::other("native detail"),
/// )
/// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
/// assert_eq!(primary.kind(), ErrorKind::Driver);
/// assert_eq!(primary.to_string(), "statement failed");
/// assert!(primary.source().is_some());
/// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Invalid SQL shape, identifier, or value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::InvalidQuery, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    InvalidQuery,
    /// Backend or operation is outside the implemented capability.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Unsupported, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Unsupported,
    /// Prepared parameter count differs from supplied arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(
    ///     ErrorKind::Arity {
    ///         expected: 2,
    ///         actual: 1,
    ///     },
    ///     "operation failed",
    /// );
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Arity {
        /// Native parameter count.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{ErrorKind, SqlError};
        /// let error = SqlError::new(
        ///     ErrorKind::Arity {
        ///         expected: 2,
        ///         actual: 1,
        ///     },
        ///     "operation failed",
        /// );
        /// assert_eq!(error.to_string(), "operation failed");
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        expected: usize,
        /// Supplied argument count.
        ///
        /// # Examples
        ///
        /// ```
        /// use stackpress_sql::{ErrorKind, SqlError};
        /// let error = SqlError::new(
        ///     ErrorKind::Arity {
        ///         expected: 2,
        ///         actual: 1,
        ///     },
        ///     "operation failed",
        /// );
        /// assert_eq!(error.to_string(), "operation failed");
        /// # Ok::<(), stackpress_sql::SqlError>(())
        /// ```
        actual: usize,
    },
    /// A finite input, output or admission budget was exceeded.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Limit, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Limit,
    /// Acquiring a pooled connection exceeded its deadline.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::PoolTimedOut, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    PoolTimedOut,
    /// Database has closed or its worker is unavailable.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Closed, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Closed,
    /// SQLite could not obtain a native lock within the busy budget.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Busy, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Busy,
    /// A query failed or was cancelled, making the transaction rollback-only.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::RollbackOnly, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    RollbackOnly,
    /// A row lookup or strict conversion failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Decode, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Decode,
    /// Invalid native options.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Configuration, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Configuration,
    /// Other native failure; inspect the source explicitly for diagnostics.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// let error = SqlError::new(ErrorKind::Driver, "operation failed");
    /// assert_eq!(error.to_string(), "operation failed");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Driver,
}

/// A safe display message and an optional native cause.
///
/// Display omits SQL, binds and paths. Source diagnostics may contain sensitive
/// native details; applications choose their own disclosure policy.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{ErrorKind, SqlError};
/// use std::error::Error;
/// let primary = SqlError::with_source(
///     ErrorKind::Driver,
///     "statement failed",
///     std::io::Error::other("native detail"),
/// )
/// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
/// assert_eq!(primary.kind(), ErrorKind::Driver);
/// assert_eq!(primary.to_string(), "statement failed");
/// assert!(primary.source().is_some());
/// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// use std::error::Error;
    /// let primary = SqlError::with_source(
    ///     ErrorKind::Driver,
    ///     "statement failed",
    ///     std::io::Error::other("native detail"),
    /// )
    /// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    /// assert_eq!(primary.kind(), ErrorKind::Driver);
    /// assert_eq!(primary.to_string(), "statement failed");
    /// assert!(primary.source().is_some());
    /// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self {
            kind,
            message,
            source: None,
            cleanup: None,
        }
    }

    /// Preserve a driver cause without including it in the display message.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// use std::error::Error;
    /// let primary = SqlError::with_source(
    ///     ErrorKind::Driver,
    ///     "statement failed",
    ///     std::io::Error::other("native detail"),
    /// )
    /// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    /// assert_eq!(primary.kind(), ErrorKind::Driver);
    /// assert_eq!(primary.to_string(), "statement failed");
    /// assert!(primary.source().is_some());
    /// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// use std::error::Error;
    /// let primary = SqlError::with_source(
    ///     ErrorKind::Driver,
    ///     "statement failed",
    ///     std::io::Error::other("native detail"),
    /// )
    /// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    /// assert_eq!(primary.kind(), ErrorKind::Driver);
    /// assert_eq!(primary.to_string(), "statement failed");
    /// assert!(primary.source().is_some());
    /// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Retain a secondary cleanup failure without replacing the primary native cause.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// use std::error::Error;
    /// let primary = SqlError::with_source(
    ///     ErrorKind::Driver,
    ///     "statement failed",
    ///     std::io::Error::other("native detail"),
    /// )
    /// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    /// assert_eq!(primary.kind(), ErrorKind::Driver);
    /// assert_eq!(primary.to_string(), "statement failed");
    /// assert!(primary.source().is_some());
    /// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn with_cleanup(mut self, cleanup: SqlError) -> Self {
        self.cleanup = Some(Box::new(cleanup));
        self
    }

    /// Inspect explicit rollback's secondary failure, including its own native source.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ErrorKind, SqlError};
    /// use std::error::Error;
    /// let primary = SqlError::with_source(
    ///     ErrorKind::Driver,
    ///     "statement failed",
    ///     std::io::Error::other("native detail"),
    /// )
    /// .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    /// assert_eq!(primary.kind(), ErrorKind::Driver);
    /// assert_eq!(primary.to_string(), "statement failed");
    /// assert!(primary.source().is_some());
    /// assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
