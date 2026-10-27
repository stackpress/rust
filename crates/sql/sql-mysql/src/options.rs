//! Explicit application connection fields and finite per-operation policy.

use serde::{Deserialize, Serialize};
use sqlx::ConnectOptions;
use sqlx::mysql::{MySqlConnectOptions, MySqlSslMode};
use stackpress_sql::{ErrorKind, Result, SqlError};
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;

/// Server identity verification is the default; plaintext requires explicit selection.
///
/// Select policy explicitly; choosing a CA path does not read its file until open.
/// ```
/// use stackpress_sql_mysql::{MysqlOptions, MysqlTls};
/// let mut options = MysqlOptions::new("db.internal", "application", "application", "");
/// options.tls = MysqlTls::VerifyIdentity;
/// options.validate()?;
/// options.tls = MysqlTls::CertificateAuthority { path: "certificates/ca.pem".into() };
/// options.validate()?;
/// options.tls = MysqlTls::Plaintext;
/// options.validate()?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum MysqlTls {
    /// Verify the server identity using the bundled WebPKI certificate roots.
    #[default]
    VerifyIdentity,
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
///
/// Lower pool/deadline budgets on application-owned options before eager open.
/// ```
/// use stackpress_sql_mysql::MysqlOptions;
/// let mut options = MysqlOptions::new("db.internal", "application", "application", "");
/// options.max_connections = 2;
/// options.max_in_flight = 16;
/// options.acquire_timeout_ms = 2_000;
/// options.statement_timeout_ms = 10_000;
/// options.cleanup_timeout_ms = 1_000;
/// options.validate()?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[must_use = "use the configured database options"]
pub struct MysqlOptions {
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
    pub tls: MysqlTls,
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
    /// Per-reset and per-retirement budget, 1..=300000 milliseconds each.
    #[serde(default = "default_cleanup")]
    pub cleanup_timeout_ms: u64,
}

impl fmt::Debug for MysqlOptions {
    /// Keep ordinary configuration logs free of credentials and connection identifiers.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MysqlOptions")
            .field("connection", &"[redacted]")
            .field("max_connections", &self.max_connections)
            .field("max_in_flight", &self.max_in_flight)
            .finish_non_exhaustive()
    }
}

impl MysqlOptions {
    /// Select explicit connection fields with verified TLS and finite defaults.
    /// The default port is 3306; no ambient connection fields are adopted.
    ///
    /// ```
    /// use stackpress_sql_mysql::MysqlOptions;
    /// let options = MysqlOptions::new("db.internal", "application", "application", "");
    /// assert_eq!(options.port, 3306);
    /// options.validate()?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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
            tls: MysqlTls::default(),
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
    ///
    /// ```
    /// use stackpress_sql_mysql::MysqlOptions;
    /// use stackpress_sql::ErrorKind;
    /// let mut options = MysqlOptions::new("db.internal", "application", "application", "");
    /// options.statement_timeout_ms = 0;
    /// assert_eq!(options.validate().unwrap_err().kind(), ErrorKind::Configuration);
    /// ```
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
        if let MysqlTls::CertificateAuthority { path } = &self.tls
            && (path.as_os_str().is_empty() || path.to_string_lossy().chars().any(char::is_control))
        {
            return Err(invalid_options());
        }
        Ok(())
    }

    /// Construct backend policy without a passfile or arbitrary connection URL.
    pub(crate) fn connect_options(&self) -> Result<MySqlConnectOptions> {
        let options = MySqlConnectOptions::new()
            .host(&self.host)
            .port(self.port)
            .database(&self.database)
            .username(&self.user)
            .statement_cache_capacity(0)
            .charset("utf8mb4")
            .collation("utf8mb4_unicode_ci")
            .no_engine_substitution(true)
            .disable_statement_logging();
        // SQLx treats Some("") as a password scramble, unlike no password. Empty is
        // explicitly passwordless; MySQL options have no ambient-password fallback.
        let options = if self.password.is_empty() {
            options
        } else {
            options.password(&self.password)
        };
        // Never downgrade opportunistically: select exactly one TLS policy.
        Ok(match &self.tls {
            MysqlTls::VerifyIdentity => options.ssl_mode(MySqlSslMode::VerifyIdentity),
            MysqlTls::CertificateAuthority { path } => {
                options.ssl_mode(MySqlSslMode::VerifyIdentity).ssl_ca(path)
            }
            MysqlTls::Plaintext => options.ssl_mode(MySqlSslMode::Disabled),
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
    /// Bound each reset/disposal phase while retaining both permits.
    pub(crate) fn cleanup_timeout(&self) -> Duration {
        Duration::from_millis(self.cleanup_timeout_ms)
    }
}

/// Report configuration rejection without exposing application data.
fn invalid_options() -> SqlError {
    SqlError::new(
        ErrorKind::Configuration,
        "invalid MySQL connection fields or finite budget",
    )
}

/// Use the conventional backend port when serialized config omits it.
fn default_port() -> u16 {
    3306
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

/// Bound each reset/retirement phase before forced disposal by default.
fn default_cleanup() -> u64 {
    5000
}
