//! Explicit application connection fields and finite per-operation policy.

use serde::{Deserialize, Serialize};
use sqlx::ConnectOptions;
use sqlx::postgres::{PgConnectOptions, PgSslMode};
use stackpress_sql::{ErrorKind, Result, SqlError};
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// Server identity verification is the default; plaintext requires explicit selection.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum PostgresTls {
    /// Verify the server identity using the bundled WebPKI certificate roots.
    #[default]
    VerifyFull,
    /// Verify the server identity using the supplied PEM certificate authority file.
    CertificateAuthority {
        /// Application-owned path; never sourced from a query or URL.
        path: PathBuf,
    },
    /// Explicit local/development opt-in; no encryption or downgrade negotiation.
    Plaintext,
}

/// Application-owned connection configuration; Debug hides all connection fields.
///
/// Serialization contains the password and is an explicit secret-handling operation.
/// Open never creates databases, reads a passfile, migrates, or retries operations.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[must_use = "use the configured database options"]
pub struct PostgresOptions {
    /// Explicit TCP hostname; Unix sockets and URL syntax are excluded.
    pub host: String,
    /// Explicit nonzero TCP port.
    #[serde(default = "default_port")]
    pub port: u16,
    /// Existing database selected by the application.
    pub database: String,
    /// Explicit authentication identity.
    pub user: String,
    /// Caller-supplied secret; an empty password is still an explicit choice.
    pub password: String,
    /// Server certificate/identity policy; defaults to verified WebPKI roots.
    #[serde(default)]
    pub tls: PostgresTls,
    /// Maximum native connections, 1..=128; minimum is always zero.
    #[serde(default = "default_connections")]
    pub max_connections: u32,
    /// Immediate admission bound including held transactions, 1..=4096.
    #[serde(default = "default_admission")]
    pub max_in_flight: usize,
    /// Whole eager-open/acquisition budget in milliseconds, 1..=300000.
    #[serde(default = "default_acquire")]
    pub acquire_timeout_ms: u64,
    /// Whole prepare/bind/execute/decode/control budget, 1..=300000 milliseconds.
    #[serde(default = "default_statement")]
    pub statement_timeout_ms: u64,
    /// Retirement budget before forced transport disposal, 1..=300000 milliseconds.
    #[serde(default = "default_cleanup")]
    pub cleanup_timeout_ms: u64,
}

impl fmt::Debug for PostgresOptions {
    /// Keep ordinary configuration logs free of credentials and connection identifiers.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PostgresOptions")
            .field("connection", &"[redacted]")
            .field("max_connections", &self.max_connections)
            .field("max_in_flight", &self.max_in_flight)
            .finish_non_exhaustive()
    }
}

impl PostgresOptions {
    /// Select explicit connection fields with verified TLS and finite defaults.
    /// The default port is 5432; no ambient connection fields are adopted.
    pub fn new(
        host: impl Into<String>,
        database: impl Into<String>,
        user: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        // Keep secrets as data, never as an interpolated driver URL.
        Self {
            host: host.into(),
            port: default_port(),
            database: database.into(),
            user: user.into(),
            password: password.into(),
            tls: PostgresTls::default(),
            max_connections: default_connections(),
            max_in_flight: default_admission(),
            acquire_timeout_ms: default_acquire(),
            statement_timeout_ms: default_statement(),
            cleanup_timeout_ms: default_cleanup(),
        }
    }

    /// Reject invalid fields or unlimited budgets before native IO.
    ///
    /// # Errors
    /// Returns Configuration without echoing the secret, host, user, or database.
    pub fn validate(&self) -> Result<()> {
        // Connection fields cannot trigger a URL parser or a Unix socket fallback.
        if self.host.is_empty()
            || self.host.starts_with('/')
            || self.host.contains("://")
            || self.host.chars().any(char::is_control)
            || self.port == 0
            || [&self.database, &self.user]
                .iter()
                .any(|value| value.is_empty() || value.chars().any(char::is_control))
            || self.password.contains('\0')
            || !(1..=128).contains(&self.max_connections)
            || !(1..=4096).contains(&self.max_in_flight)
            || [
                self.acquire_timeout_ms,
                self.statement_timeout_ms,
                self.cleanup_timeout_ms,
            ]
            .iter()
            .any(|value| !(1..=300000).contains(value))
        {
            return Err(invalid_options());
        }
        // A selected CA file must have a usable application-supplied path.
        if let PostgresTls::CertificateAuthority { path } = &self.tls
            && (path.as_os_str().is_empty() || path.to_string_lossy().chars().any(char::is_control))
        {
            return Err(invalid_options());
        }
        Ok(())
    }

    /// Construct backend policy without a passfile or arbitrary connection URL.
    pub(crate) fn connect_options(&self) -> Result<PgConnectOptions> {
        // SQLx 0.9's constructor reads these ambient fields with no public clearing API.
        // Reject them instead of silently adopting certificates or arbitrary startup policy.
        // All other PG fields are explicitly overwritten below, and pgpass is never read.
        if ["PGPgROOTCERT", "PGPgCERT", "PGPgKEY", "PGOPTIONS"]
            .iter()
            .any(|name| std::env::var_os(name).is_some())
        {
            return Err(SqlError::new(
                ErrorKind::Configuration,
                "ambient PostgreSQL TLS/startup options are unsupported; use explicit options",
            ));
        }
        let options = PgConnectOptions::new_without_pgpass()
            .host(&self.host)
            .port(self.port)
            .database(&self.database)
            .username(&self.user)
            .password(&self.password)
            .statement_cache_capacity(0)
            .application_name("stackpress")
            .options([("standard_conforming_strings", "on")])
            .disable_statement_logging();
        // Never downgrade opportunistically: select exactly one TLS policy.
        Ok(match &self.tls {
            PostgresTls::VerifyFull => options.ssl_mode(PgSslMode::VerifyFull),
            PostgresTls::CertificateAuthority { path } => {
                options.ssl_mode(PgSslMode::VerifyFull).ssl_root_cert(path)
            }
            PostgresTls::Plaintext => options.ssl_mode(PgSslMode::Disable),
        })
    }

    /// Convert the validated whole-open/acquire deadline to Tokio's duration.
    pub(crate) fn acquire_timeout(&self) -> Duration {
        Duration::from_millis(self.acquire_timeout_ms)
    }
    /// Convert the validated statement deadline without a zero/unlimited sentinel.
    pub(crate) fn statement_timeout(&self) -> Duration {
        Duration::from_millis(self.statement_timeout_ms)
    }
    /// Bound graceful transport disposal while retaining both permits.
    pub(crate) fn cleanup_timeout(&self) -> Duration {
        Duration::from_millis(self.cleanup_timeout_ms)
    }
}

/// Report configuration rejection without exposing application data.
fn invalid_options() -> SqlError {
    SqlError::new(
        ErrorKind::Configuration,
        "invalid PostgreSQL connection fields or finite budget",
    )
}

/// Use the conventional backend port when serialized config omits it.
fn default_port() -> u16 {
    5432
}

/// Bound native connections by default.
fn default_connections() -> u32 {
    4
}

/// Bound admitted application work by default.
fn default_admission() -> usize {
    128
}

/// Bound eager open and pool acquisition by default.
fn default_acquire() -> u64 {
    5000
}

/// Bound full statement/control IO by default.
fn default_statement() -> u64 {
    30000
}

/// Bound graceful retirement before forced disposal by default.
fn default_cleanup() -> u64 {
    5000
}
