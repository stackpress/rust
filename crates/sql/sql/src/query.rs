//! Detached query data, minimal SELECT rendering, and the single-statement lexical gate.

use crate::{ErrorKind, Result, SqlError, SqlValue};

/// A renderer/backend identity; only SQLite execution/rendering is proved at M0.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DialectId {
    /// Native SQLite.
    Sqlite,
    /// Reserved renderer identity for a later milestone, not a working adapter.
    Postgres,
    /// Reserved renderer identity for a later milestone, not a working adapter.
    Mysql,
}

/// Finite per-statement input and retained-result budgets.
#[derive(Clone, Copy, Debug)]
pub struct QueryLimits {
    /// Maximum UTF-8 SQL bytes.
    pub sql_bytes: usize,
    /// Maximum arguments.
    pub binds: usize,
    /// Maximum retained rows; zero permits only rowless outcomes.
    pub rows: usize,
    /// Maximum retained column-name/value bytes and bind bytes.
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
#[derive(Clone, Debug)]
pub struct Query {
    dialect: DialectId,
    sql: String,
    binds: Vec<SqlValue>,
}

impl Query {
    /// Inspect statement text independently of IO.
    pub fn sql(&self) -> &str {
        &self.sql
    }

    /// Inspect bound data; none of it is interpolated into statement text.
    pub fn binds(&self) -> &[SqlValue] {
        &self.binds
    }

    /// Return the selected renderer identity.
    pub fn dialect(&self) -> DialectId {
        self.dialect
    }

    /// Enforce finite input budgets before acquiring/preparing on a native connection.
    pub fn validate(&self, limits: QueryLimits) -> Result<()> {
        if self.sql.len() > limits.sql_bytes || self.binds.len() > limits.binds {
            return Err(SqlError::new(ErrorKind::Limit, "SQL input limit exceeded"));
        }
        let mut bytes = 0_usize;
        for value in &self.binds {
            value.validate()?;
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
pub trait Render {
    /// Validate the selected dialect and produce one owned statement.
    fn render(&self, dialect: DialectId) -> Result<Query>;
}

impl Render for Query {
    /// Clone detached data only after checking the backend identity.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        if self.dialect != dialect {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "SQL dialect mismatch",
            ));
        }
        Ok(self.clone())
    }
}

/// Explicit assertion that dynamic SQL was authored/audited as trusted code.
///
/// This is not sanitization. Never pass request text to this constructor.
#[derive(Clone, Debug)]
pub struct TrustedSql(String);

impl TrustedSql {
    /// Mark trusted, manually audited SQL; values still belong in binds.
    pub fn audited(sql: impl Into<String>) -> Self {
        Self(sql.into())
    }
}

/// Detached trusted raw statement. Only anonymous positional `?` markers are admitted.
#[derive(Clone, Debug)]
pub struct Sql {
    query: Query,
}

impl Sql {
    /// Construct caller-authored static SQL, subject to single-statement validation.
    pub fn literal(dialect: DialectId, sql: &'static str) -> Self {
        Self::trusted(dialect, TrustedSql::audited(sql))
    }

    /// Construct explicitly audited dynamic SQL; the marker does not sanitize it.
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
    pub fn bind(mut self, value: impl Into<SqlValue>) -> Self {
        self.query.binds.push(value.into());
        self
    }
}

impl Render for Sql {
    /// Reject unsupported dialects, batches, transaction control and nonanonymous binds.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        let query = self.query.render(dialect)?;
        if dialect != DialectId::Sqlite {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "renderer not implemented at M0",
            ));
        }
        validate_statement(&query.sql)?;
        query.validate(QueryLimits::default())?;
        Ok(query)
    }
}

/// Small owned SELECT builder used by the ownership/host proof.
#[derive(Clone, Debug)]
pub struct Select {
    columns: Vec<String>,
    table: String,
    equality: Option<(String, SqlValue)>,
}

impl Select {
    /// Select explicit identifier segments; `*`/expressions require trusted raw SQL at M0.
    pub fn new(columns: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            columns: columns.into_iter().map(Into::into).collect(),
            table: String::new(),
            equality: None,
        }
    }

    /// Set one literal table-name segment; dots do not imply qualification.
    pub fn from(mut self, table: impl Into<String>) -> Self {
        self.table = table.into();
        self
    }

    /// Add one bound equality predicate, replacing a previous predicate.
    pub fn where_eq(mut self, column: impl Into<String>, value: impl Into<SqlValue>) -> Self {
        self.equality = Some((column.into(), value.into()));
        self
    }
}

impl Render for Select {
    /// Quote identifiers and bind data, without interpreting either as SQL syntax.
    fn render(&self, dialect: DialectId) -> Result<Query> {
        if dialect != DialectId::Sqlite {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "renderer not implemented at M0",
            ));
        }
        if self.columns.is_empty() {
            return Err(invalid_sql());
        }
        let columns = self
            .columns
            .iter()
            .map(|column| identifier(column))
            .collect::<Result<Vec<_>>>()?;
        let mut sql = format!(
            "SELECT {} FROM {}",
            columns.join(", "),
            identifier(&self.table)?
        );
        let mut binds = Vec::new();
        if let Some((column, value)) = &self.equality {
            // Identifier syntax and the value take distinct construction paths.
            sql.push_str(&format!(" WHERE {} = ?", identifier(column)?));
            binds.push(value.clone());
        }
        let query = Query {
            dialect,
            sql,
            binds,
        };
        query.validate(QueryLimits::default())?;
        Ok(query)
    }
}

/// Quote one identifier segment, rejecting empty/control-containing input.
fn identifier(name: &str) -> Result<String> {
    if name.is_empty() || name.chars().any(char::is_control) {
        return Err(invalid_sql());
    }
    Ok(format!("\"{}\"", name.replace('"', "\"\"")))
}

/// Construct a safe lexical/identifier failure without echoing caller input.
fn invalid_sql() -> SqlError {
    SqlError::new(
        ErrorKind::InvalidQuery,
        "invalid SQL statement or identifier",
    )
}

/// Admit one SQLite statement while respecting comments and every SQLite quote form.
fn validate_statement(sql: &str) -> Result<()> {
    let bytes = sql.as_bytes();
    let mut index = 0;
    let mut ended = false;
    let mut first_word = None;
    while index < bytes.len() {
        let byte = bytes[index];
        // Whitespace/comments remain legal after the final statement terminator.
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if bytes[index..].starts_with(b"--") {
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if bytes[index..].starts_with(b"/*") {
            index += 2;
            while index + 1 < bytes.len() && !bytes[index..].starts_with(b"*/") {
                index += 1;
            }
            if index + 1 >= bytes.len() {
                return Err(invalid_sql());
            }
            index += 2;
            continue;
        }
        // A second executable token is a batch, even if no whitespace separates it.
        if ended || byte == 0 {
            return Err(invalid_sql());
        }
        if byte == b';' {
            ended = true;
            index += 1;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`' | b'[') {
            let close = if byte == b'[' { b']' } else { byte };
            index += 1;
            loop {
                if index >= bytes.len() || bytes[index] == 0 {
                    return Err(invalid_sql());
                }
                if bytes[index] == close {
                    index += 1;
                    // Bracket identifiers terminate at the first right bracket.
                    if close != b']' && bytes.get(index) == Some(&close) {
                        index += 1;
                        continue;
                    }
                    break;
                }
                index += 1;
            }
            continue;
        }
        if matches!(byte, b':' | b'@' | b'$')
            || (byte == b'?' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit))
        {
            return Err(SqlError::new(
                ErrorKind::Unsupported,
                "only anonymous SQL parameters are supported",
            ));
        }
        if byte.is_ascii_alphabetic() {
            let start = index;
            while index < bytes.len()
                && (bytes[index].is_ascii_alphanumeric() || bytes[index] == b'_')
            {
                index += 1;
            }
            first_word.get_or_insert(&sql[start..index]);
            continue;
        }
        index += 1;
    }
    // Connection policy and transaction control cannot be bypassed through raw SQL.
    let word = first_word.ok_or_else(invalid_sql)?.to_ascii_uppercase();
    if !matches!(
        word.as_str(),
        "SELECT"
            | "WITH"
            | "INSERT"
            | "UPDATE"
            | "DELETE"
            | "CREATE"
            | "DROP"
            | "ALTER"
            | "EXPLAIN"
    ) {
        return Err(SqlError::new(
            ErrorKind::Unsupported,
            "SQL command is outside the ordinary statement API",
        ));
    }
    Ok(())
}
