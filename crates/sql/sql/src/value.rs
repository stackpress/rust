//! Owned native values and ordered rows; no connection borrow escapes execution.

use crate::{ErrorKind, Result, SqlError};

/// Explicit native scalar values. JSON and dates remain caller-selected text at M0.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    /// SQL NULL, distinct from missing input.
    Null,
    /// Bound as a boolean; SQLite stores an integer.
    Bool(bool),
    /// Exact signed 64-bit integer.
    Integer(i64),
    /// Finite real number; validation rejects NaN and infinity before IO.
    Real(f64),
    /// Owned UTF-8 text.
    Text(String),
    /// Owned binary bytes.
    Blob(Vec<u8>),
}

impl SqlValue {
    /// Reject values whose native representation is outside this contract.
    pub(crate) fn validate(&self) -> Result<()> {
        if matches!(self, Self::Real(value) if !value.is_finite()) {
            return Err(SqlError::new(
                ErrorKind::InvalidQuery,
                "SQL real must be finite",
            ));
        }
        Ok(())
    }

    /// Count retained value bytes, excluding allocator/container overhead.
    pub fn retained_bytes(&self) -> usize {
        match self {
            Self::Text(value) => value.len(),
            Self::Blob(value) => value.len(),
            Self::Null => 0,
            Self::Bool(_) => 1,
            Self::Integer(_) | Self::Real(_) => 8,
        }
    }
}

impl From<i64> for SqlValue {
    /// Preserve an exact integer.
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}
impl From<bool> for SqlValue {
    /// Preserve a boolean without truthiness coercion.
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
impl From<String> for SqlValue {
    /// Take ownership of text.
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}
impl From<&str> for SqlValue {
    /// Copy borrowed text into an owned bind.
    fn from(value: &str) -> Self {
        Self::Text(value.into())
    }
}

/// Strict conversion from an owned row cell, with no lossy coercion.
pub trait FromSqlValue: Sized {
    /// Return an owned value or a typed NULL/type/range error.
    fn from_sql(value: &SqlValue) -> Result<Self>;
}

impl FromSqlValue for String {
    /// Decode text only.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        match value {
            SqlValue::Text(value) => Ok(value.clone()),
            _ => Err(SqlError::new(ErrorKind::Decode, "expected SQL text")),
        }
    }
}
impl FromSqlValue for i64 {
    /// Decode an exact integer only.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        match value {
            SqlValue::Integer(value) => Ok(*value),
            _ => Err(SqlError::new(ErrorKind::Decode, "expected SQL integer")),
        }
    }
}
impl FromSqlValue for bool {
    /// Accept only native booleans or integer zero/one.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        match value {
            SqlValue::Bool(value) => Ok(*value),
            SqlValue::Integer(0) => Ok(false),
            SqlValue::Integer(1) => Ok(true),
            _ => Err(SqlError::new(ErrorKind::Decode, "expected SQL boolean")),
        }
    }
}

/// Ordered names and values, including duplicate names from joins.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    cells: Vec<(String, SqlValue)>,
}

impl Row {
    /// Construct an owned ordered row in an adapter.
    pub fn new(cells: Vec<(String, SqlValue)>) -> Self {
        Self { cells }
    }

    /// Inspect every ordered column without overwriting duplicate names.
    pub fn cells(&self) -> &[(String, SqlValue)] {
        &self.cells
    }

    /// Decode a unique named cell; missing/ambiguous names are errors.
    pub fn try_get<T: FromSqlValue>(&self, name: &str) -> Result<T> {
        let mut matches = self.cells.iter().filter(|(column, _)| column == name);
        let (_, value) = matches
            .next()
            .ok_or_else(|| SqlError::new(ErrorKind::Decode, "missing SQL column"))?;
        // Ambiguous lookup must not choose a join side by accident.
        if matches.next().is_some() {
            return Err(SqlError::new(ErrorKind::Decode, "ambiguous SQL column"));
        }
        T::from_sql(value)
    }
}

/// One statement's outcome, never shared mutable last-insert state.
#[derive(Debug, Default)]
pub struct QueryResult {
    /// Owned ordered rows.
    pub rows: Vec<Row>,
    /// Native affected-row count for this statement.
    pub affected: u64,
    /// Insert IDs are not yet exposed by the M0 adapter; use RETURNING instead.
    pub insert_id: Option<i64>,
}
