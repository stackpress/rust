//! App-owned file/configuration data; there is no request-controlled open path.

use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous};
use stackpress_sql::{ErrorKind, Result, SqlError};

/// Native file options with finite positive bounds and unknown-field rejection.
///
/// WAL, FULL synchronous, foreign keys ON, and bounded worker/row channels are fixed
/// M0 policy. Read-only and in-memory modes need separate proofs and are not exposed.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SqliteOptions {
    /// Explicit file path supplied by the application.
    pub path: PathBuf,
    /// Creation is opt-in; missing files otherwise fail eager open.
    pub create_if_missing: bool,
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

    /// Reject unsupported targets and invalid/unbounded budgets before opening a worker.
    pub fn validate(&self) -> Result<()> {
        // SQLx sets SQLITE_OPEN_URI even with filename(). Reject URI modes and memory
        // targets before they can override the explicit file/persistence contract.
        let path = self.path.to_str().ok_or_else(invalid_options)?;
        if path.is_empty()
            || path == ":memory:"
            || path.starts_with("file:")
            || path.chars().any(char::is_control)
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
        SqliteConnectOptions::new()
            .filename(&self.path)
            .create_if_missing(self.create_if_missing)
            .foreign_keys(true)
            .journal_mode(SqliteJournalMode::Wal)
            .synchronous(SqliteSynchronous::Full)
            .busy_timeout(Duration::from_millis(self.busy_timeout_ms))
            .command_buffer_size(16)
            .row_buffer_size(16)
    }
}

/// Construct a safe options failure without disclosing the configured path.
fn invalid_options() -> SqlError {
    SqlError::new(
        ErrorKind::Configuration,
        "invalid SQLite file or finite budget",
    )
}
