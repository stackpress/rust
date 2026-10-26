//! App-owned file/configuration data; there is no request-controlled open path.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use stackpress_sql::{ErrorKind, Result, SqlError};

/// Native file options with finite positive bounds and unknown-field rejection.
///
/// Writable files use WAL/FULL; read-only connections preserve the file's journal mode.
/// Foreign keys and bounded worker/row channels apply to every connection. In-memory
/// mode is not exposed because it cannot satisfy the file persistence contract.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteOptions {
    /// Explicit file path supplied by the application.
    pub path: PathBuf,
    /// Creation is opt-in; missing files otherwise fail eager open.
    pub create_if_missing: bool,
    /// Open an existing file without write permission. Incompatible with creation.
    /// Missing this field in existing serialized M0 configuration means false.
    #[serde(default)]
    pub read_only: bool,
    /// Maximum native connections, from 1 through 128.
    pub max_connections: u32,
    /// Pool acquisition timeout, 1 through 300000 milliseconds.
    pub acquire_timeout_ms: u64,
    /// Native lock busy timeout, 1 through 300000 milliseconds.
    pub busy_timeout_ms: u64,
    /// Immediate admission cap, 1 through 4096 including held transactions.
    pub max_in_flight: usize,
}

impl SqliteOptions {
    /// Select an explicit existing file with the accepted finite defaults.
    pub fn file(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            create_if_missing: false,
            read_only: false,
            max_connections: 4,
            acquire_timeout_ms: 5000,
            busy_timeout_ms: 1000,
            max_in_flight: 128,
        }
    }

    /// Opt into file creation; installation/seeding remains an explicit caller operation.
    pub fn create_if_missing(mut self, create: bool) -> Self {
        self.create_if_missing = create;
        self
    }

    /// Open an existing file without modifying its journal mode or allowing writes.
    /// Creation must remain disabled; validation rejects conflicting options.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql_sqlite::SqliteOptions;
    /// let options = SqliteOptions::file("existing.sqlite").read_only(true);
    /// options.validate()?;
    /// // Validation checks configuration only; open() performs actual native IO.
    /// assert!(options.read_only);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Reject unsupported targets and invalid/unbounded budgets before opening a worker.
    pub fn validate(&self) -> Result<()> {
        // SQLx sets SQLITE_OPEN_URI even with filename(). Reject URI modes and memory
        // targets before they can override the explicit file/persistence contract.
        let path = self.path.to_str().ok_or_else(invalid_options)?;
        if path.is_empty()
            || path == ":memory:"
            || path.starts_with("file:")
            || path.chars().any(char::is_control)
            || (self.read_only && self.create_if_missing)
            || !(1..=128).contains(&self.max_connections)
            || !(1..=300_000).contains(&self.acquire_timeout_ms)
            || !(1..=300_000).contains(&self.busy_timeout_ms)
            || !(1..=4096).contains(&self.max_in_flight)
        {
            return Err(invalid_options());
        }
        Ok(())
    }

    /// Convert the validated millisecond budget to SQLx pool policy.
    pub(crate) fn acquire_timeout(&self) -> Duration {
        Duration::from_millis(self.acquire_timeout_ms)
    }

    /// Configure every connection consistently, without URI parsing or hidden fallback.
    pub(crate) fn connect_options(&self) -> SqliteConnectOptions {
        let options = SqliteConnectOptions::new()
            .filename(&self.path)
            .create_if_missing(self.create_if_missing)
            .read_only(self.read_only)
            .foreign_keys(true)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_millis(self.busy_timeout_ms))
            .command_buffer_size(16)
            .row_buffer_size(16);
        // Changing an existing file's journal mode can require write access. A read-only
        // attachment preserves the mode; writable opens consistently request WAL.
        if self.read_only {
            options
        } else {
            options.journal_mode(SqliteJournalMode::Wal)
        }
    }
}

/// Construct a safe options failure without disclosing the configured path.
fn invalid_options() -> SqlError {
    SqlError::new(
        ErrorKind::Configuration,
        "invalid SQLite file or finite budget",
    )
}
