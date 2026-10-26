//! Detached query data, statement validation, and the single-statement lexical gate.

use crate::{ErrorKind, Result, SqlError, SqlValue};

/// A renderer/backend identity, independent from installed native adapters.
///
/// # Examples
///
/// ```
/// use stackpress_sql::DialectId;
/// assert!(DialectId::Sqlite.capabilities().returning);
/// assert!(DialectId::Postgres.capabilities().cascade);
/// assert!(!DialectId::Mysql.capabilities().transactional_ddl);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialectId {
    /// Native SQLite.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Select};
    /// let dialect = DialectId::Sqlite;
    /// Select::default().from("profile").render(dialect)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Sqlite,
    /// Pure network renderer identity; no production network adapter is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Select};
    /// let dialect = DialectId::Postgres;
    /// Select::default().from("profile").render(dialect)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Postgres,
    /// Pure network renderer identity; no production network adapter is included.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Select};
    /// let dialect = DialectId::Mysql;
    /// Select::default().from("profile").render(dialect)?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Mysql,
}

/// Finite per-statement input and retained-result budgets.
///
/// Defaults: 1 MiB SQL, 1,000 binds, 10,000 retained rows and 16 MiB retained
/// name/value/declaration bytes. Core validates input; adapters must enforce output
/// budgets while collecting, before publishing a partial result. Lower these with
/// [`crate::Engine::with_limits`]; raising them cannot bypass renderer construction budgets.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, ErrorKind, QueryLimits, Render, Select};
/// let limits = QueryLimits {
///     binds: 0,
///     ..QueryLimits::default()
/// };
/// assert_eq!(limits.sql_bytes, 1024 * 1024);
/// assert_eq!(limits.rows, 10_000);
/// assert_eq!(limits.bytes, 16 * 1024 * 1024);
/// let query = Select::default()
///     .from("profile")
///     .where_eq("id", 7)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.validate(limits).unwrap_err().kind(), ErrorKind::Limit);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug)]
pub struct QueryLimits {
    /// Maximum UTF-8 SQL bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryLimits;
    /// let value = QueryLimits::default();
    /// assert_eq!(value.sql_bytes, 1024 * 1024);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub sql_bytes: usize,
    /// Maximum arguments.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryLimits;
    /// let value = QueryLimits::default();
    /// assert_eq!(value.binds, 1000);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub binds: usize,
    /// Maximum retained rows; zero permits only rowless outcomes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryLimits;
    /// let value = QueryLimits::default();
    /// assert_eq!(value.rows, 10_000);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub rows: usize,
    /// Maximum retained column-name/value bytes and bind bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryLimits;
    /// let value = QueryLimits::default();
    /// assert_eq!(value.bytes, 16 * 1024 * 1024);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub bytes: usize,
}

impl Default for QueryLimits {
    /// Use the accepted initial finite budgets.
    fn default() -> Self {
        Self {
            sql_bytes: 1024 * 1024,
            binds: 1000,
            rows: 10_000,
            bytes: 16 * 1024 * 1024,
        }
    }
}

/// One validated, dialect-tagged statement with separate owned arguments.
///
/// Obtain a query through [`crate::Render::render`]. Private fields retain the renderer
/// identity and separate binds; cloning produces independent owned data. Inspecting
/// binds is explicit and may disclose application data. Validation here checks
/// input budgets; native prepared-parameter arity is an adapter responsibility.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
/// let query = Select::new(["id"])
///     .from("profile")
///     .where_eq("id", 7)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.dialect(), DialectId::Sqlite);
/// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
/// assert!(query.sql().contains("WHERE"));
/// query.validate(QueryLimits::default())?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug)]
pub struct Query {
    dialect: DialectId,
    sql: String,
    binds: Vec<SqlValue>,
}

impl Query {
    /// Check identity before cloning or validation so mismatch error precedence is stable.
    fn check_dialect(&self, dialect: DialectId) -> Result<()> {
        if self.dialect != dialect {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "SQL dialect mismatch",
            ));
        }
        Ok(())
    }

    /// Freeze AST-generated SQL and parameters under the default finite budgets.
    pub(crate) fn bound(dialect: DialectId, sql: String, binds: Vec<SqlValue>) -> Result<Self> {
        let query = Self {
            dialect,
            sql,
            binds,
        };
        query.validate(QueryLimits::default())?;
        Ok(query)
    }

    /// Inspect statement text independently of IO.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
    /// let query = Select::new(["id"])
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.dialect(), DialectId::Sqlite);
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// assert!(query.sql().contains("WHERE"));
    /// query.validate(QueryLimits::default())?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Inspect bound data; none of it is interpolated into statement text.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
    /// let query = Select::new(["id"])
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.dialect(), DialectId::Sqlite);
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// assert!(query.sql().contains("WHERE"));
    /// query.validate(QueryLimits::default())?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn binds(&self) -> &[SqlValue] {
        &self.binds
    }

    /// Return the selected renderer identity.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
    /// let query = Select::new(["id"])
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.dialect(), DialectId::Sqlite);
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// assert!(query.sql().contains("WHERE"));
    /// query.validate(QueryLimits::default())?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn dialect(&self) -> DialectId {
        self.dialect
    }

    /// Enforce finite input budgets before acquiring/preparing on a native connection.
    ///
    /// # Errors
    /// Returns [`crate::ErrorKind::Limit`] for SQL/bind/byte budgets or [`crate::ErrorKind::InvalidQuery`]
    /// for nonfinite reals. PostgreSQL NUL text returns [`crate::ErrorKind::Unsupported`].
    /// Output limits and native parameter arity are enforced by adapters.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
    /// let query = Select::new(["id"])
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.dialect(), DialectId::Sqlite);
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// assert!(query.sql().contains("WHERE"));
    /// query.validate(QueryLimits::default())?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn validate(&self, limits: QueryLimits) -> Result<()> {
        if self.sql.len() > limits.sql_bytes || self.binds.len() > limits.binds {
            return Err(SqlError::new(ErrorKind::Limit, "SQL input limit exceeded"));
        }
        let mut bytes = 0_usize;
        for value in &self.binds {
            value.validate_bind(self.dialect)?;
            // Checked accounting rejects overflow before the comparison can wrap.
            bytes = bytes
                .checked_add(value.retained_bytes())
                .ok_or_else(input_limit)?;
            if bytes > limits.bytes {
                return Err(input_limit());
            }
        }
        Ok(())
    }
}

/// Construct the common bind-byte budget failure.
fn input_limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL bind byte limit exceeded")
}

/// Pure rendering; it neither acquires a connection nor executes effects.
///
/// Implementations return detached data and must reject unsupported clauses rather
/// than omit them. [`crate::Create`] and [`crate::Alter`] instead render a [`crate::QueryPlan`].
///
/// # Errors
/// Rendering rejects invalid shapes, identifiers, values, budgets, or dialect capabilities.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
/// let query = Select::new(["id"])
///     .from("profile")
///     .where_eq("id", 7)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.dialect(), DialectId::Sqlite);
/// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
/// assert!(query.sql().contains("WHERE"));
/// query.validate(QueryLimits::default())?;
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub trait Render {
    /// Validate the selected dialect and produce one owned statement.
    ///
    /// # Errors
    /// Returns a typed validation, limit, or unsupported-dialect error before IO.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, QueryLimits, Render, Select, SqlValue};
    /// let query = Select::new(["id"])
    ///     .from("profile")
    ///     .where_eq("id", 7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.dialect(), DialectId::Sqlite);
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// assert!(query.sql().contains("WHERE"));
    /// query.validate(QueryLimits::default())?;
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    fn render(&self, dialect: DialectId) -> Result<Query>;
}

impl Render for Query {
    /// Clone detached data only after checking the backend identity.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        self.check_dialect(dialect)?;
        Ok(self.clone())
    }
}

/// Explicit assertion that dynamic SQL was authored/audited as trusted code.
///
/// This is not sanitization. Never pass request text to this constructor.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Render, Sql, SqlValue, TrustedSql};
/// let raw = TrustedSql::audited("SELECT ? AS answer".to_owned());
/// let query = Sql::trusted(DialectId::Sqlite, raw)
///     .bind(7)
///     .render(DialectId::Sqlite)?;
/// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TrustedSql(String);

impl TrustedSql {
    /// Borrow audited text inside the rendering boundary.
    pub(crate) fn text(&self) -> &str {
        &self.0
    }

    /// Mark trusted, manually audited SQL; values still belong in binds.
    ///
    /// This constructor makes an author trust assertion. It does not validate or escape
    /// SQL; the consuming raw/type/default renderer applies its own checks.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue, TrustedSql};
    /// let raw = TrustedSql::audited("SELECT ? AS answer".to_owned());
    /// let query = Sql::trusted(DialectId::Sqlite, raw)
    ///     .bind(7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn audited(sql: impl Into<String>) -> Self {
        Self(sql.into())
    }
}

/// Detached trusted raw statement. SQLite/MySQL use anonymous `?`; PostgreSQL uses native `$n`.
///
/// The lexical gate accepts one ordinary statement, rejects transaction/connection
/// control, batches and NUL bytes, and retains literals without global quote rewriting.
/// SQLite/MySQL use anonymous `?`; PostgreSQL uses native `$n`. Arity is checked by
/// the adapter at prepare time. Network backslash-quoted literals and MySQL executable
/// comments are outside the accepted subset. Bind data instead of splicing it.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
/// let query = Sql::literal(DialectId::Postgres, "SELECT $1 AS answer")
///     .bind(7)
///     .render(DialectId::Postgres)?;
/// assert_eq!(query.sql(), "SELECT $1 AS answer");
/// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug)]
pub struct Sql {
    query: Query,
}

impl Sql {
    /// Construct caller-authored static SQL, subject to single-statement validation.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let query = Sql::literal(DialectId::Postgres, "SELECT $1 AS answer")
    ///     .bind(7)
    ///     .render(DialectId::Postgres)?;
    /// assert_eq!(query.sql(), "SELECT $1 AS answer");
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn literal(dialect: DialectId, sql: &'static str) -> Self {
        Self::trusted(dialect, TrustedSql::audited(sql))
    }

    /// Construct explicitly audited dynamic SQL; the marker does not sanitize it.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue, TrustedSql};
    /// let raw = TrustedSql::audited("SELECT ? AS answer".to_owned());
    /// let query = Sql::trusted(DialectId::Sqlite, raw)
    ///     .bind(7)
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn trusted(dialect: DialectId, sql: TrustedSql) -> Self {
        Self {
            query: Query {
                dialect,
                sql: sql.0,
                binds: Vec::new(),
            },
        }
    }

    /// Append owned data as a bind, preserving lexical SQL separately.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let query = Sql::literal(DialectId::Postgres, "SELECT $1 AS answer")
    ///     .bind(7)
    ///     .render(DialectId::Postgres)?;
    /// assert_eq!(query.sql(), "SELECT $1 AS answer");
    /// assert_eq!(query.binds(), &[SqlValue::Integer(7)]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn bind(mut self, value: impl Into<SqlValue>) -> Self {
        self.query.binds.push(value.into());
        self
    }
}

impl Render for Sql {
    /// Validate one ordinary statement using anonymous SQLite/MySQL or numbered PostgreSQL binds.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        // Reject invalid raw input before cloning its potentially oversized SQL and binds.
        self.query.check_dialect(dialect)?;
        self.query.validate(QueryLimits::default())?;
        crate::statement::validate(&self.query.sql, dialect)?;
        Ok(self.query.clone())
    }
}
