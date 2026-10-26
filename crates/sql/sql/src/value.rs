//! Owned native values and ordered rows; no connection borrow escapes execution.

use crate::{ErrorKind, Result, SqlError};

/// Explicit native scalar values. JSON and dates are explicitly validated text inputs.
///
/// Conversions preserve storage types. Real construction can retain nonfinite
/// input, but rendering rejects it before IO. JSON/date conversion is explicit;
/// ordinary Text is never guessed to be either. u64 conversion is checked against
/// i64::MAX; output decoding never silently truncates or coerces strings/numbers.
///
/// # Examples
///
/// ```
/// use stackpress_sql::SqlValue;
/// assert_eq!(SqlValue::try_from(7_u64)?, SqlValue::Integer(7));
/// assert!(SqlValue::try_from(u64::MAX).is_err());
/// assert_eq!(SqlValue::from("Ada").retained_bytes(), 3);
/// assert!(SqlValue::utc_text("2026-10-26T12:30:00Z").is_ok());
/// assert!(SqlValue::utc_text("2026-02-30T12:30:00Z").is_err());
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    /// SQL NULL, distinct from missing input.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Null;
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Null,
    /// Bound as a boolean; SQLite stores an integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Bool(true);
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Bool(bool),
    /// Exact signed 64-bit integer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Integer(7);
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Integer(i64),
    /// Finite real number; validation rejects NaN and infinity before IO.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Real(1.5);
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Real(f64),
    /// Owned UTF-8 text.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Text("Ada".into());
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Text(String),
    /// Owned binary bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{DialectId, Render, Sql, SqlValue};
    /// let value = SqlValue::Blob(vec![0, 1]);
    /// let query = Sql::literal(DialectId::Sqlite, "SELECT ?")
    ///     .bind(value.clone())
    ///     .render(DialectId::Sqlite)?;
    /// assert_eq!(query.binds(), &[value]);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    Blob(Vec<u8>),
}

impl SqlValue {
    /// Encode a bounded structured JSON document explicitly as native text.
    /// JSON null encodes as `"null"`; it is never converted to SQL NULL.
    ///
    /// # Errors
    ///
    /// Rejects duplicate JSON keys, nonfinite numbers and JSON depth/node/byte budget overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{JsonScalar, JsonValue, SqlValue};
    /// let document = JsonValue::Object(vec![(
    ///     "tags".into(),
    ///     JsonValue::Array(vec![
    ///         JsonValue::Scalar(JsonScalar::String("admin".into())),
    ///         JsonValue::Scalar(JsonScalar::Null),
    ///     ]),
    /// )]);
    /// assert_eq!(document.to_json()?, r#"{"tags":["admin",null]}"#);
    /// assert_ne!(
    ///     SqlValue::json(&JsonValue::Scalar(JsonScalar::Null))?,
    ///     SqlValue::Null
    /// );
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn json(value: &crate::JsonValue) -> Result<Self> {
        Ok(Self::Text(value.to_json()?))
    }

    /// Validate canonical UTC text (`YYYY-MM-DDTHH:MM:SS[.1-9 digits]Z`).
    ///
    /// # Errors
    /// Rejects impossible calendar/time values, timezone offsets and leap seconds.
    /// No ambient timezone or invalid-date epoch fallback is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::SqlValue;
    /// assert_eq!(SqlValue::try_from(7_u64)?, SqlValue::Integer(7));
    /// assert!(SqlValue::try_from(u64::MAX).is_err());
    /// assert_eq!(SqlValue::from("Ada").retained_bytes(), 3);
    /// assert!(SqlValue::utc_text("2026-10-26T12:30:00Z").is_ok());
    /// assert!(SqlValue::utc_text("2026-02-30T12:30:00Z").is_err());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn utc_text(value: &str) -> Result<Self> {
        validate_utc(value)?;
        Ok(Self::Text(value.into()))
    }

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
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::SqlValue;
    /// assert_eq!(SqlValue::try_from(7_u64)?, SqlValue::Integer(7));
    /// assert!(SqlValue::try_from(u64::MAX).is_err());
    /// assert_eq!(SqlValue::from("Ada").retained_bytes(), 3);
    /// assert!(SqlValue::utc_text("2026-10-26T12:30:00Z").is_ok());
    /// assert!(SqlValue::utc_text("2026-02-30T12:30:00Z").is_err());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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

impl TryFrom<u64> for SqlValue {
    type Error = SqlError;

    /// Reject unsigned input outside the exact native signed-integer range.
    fn try_from(value: u64) -> Result<Self> {
        i64::try_from(value)
            .map(Self::Integer)
            .map_err(|_| crate::identifier::invalid("SQL integer out of range"))
    }
}

impl From<f64> for SqlValue {
    /// Preserve a real; finite validation occurs at rendering/execution.
    fn from(value: f64) -> Self {
        Self::Real(value)
    }
}

impl From<Vec<u8>> for SqlValue {
    /// Take ownership of binary bytes without UTF-8 conversion.
    fn from(value: Vec<u8>) -> Self {
        Self::Blob(value)
    }
}

/// Implement only integer input conversions that are lossless on every supported target.
macro_rules! integer_inputs {
    ($($kind:ty),+ $(,)?) => { $(
        impl From<$kind> for SqlValue {
            /// Widen an exact integer without parsing or floating-point conversion.
            fn from(value: $kind) -> Self { Self::Integer(i64::from(value)) }
        }
    )+ };
}
integer_inputs!(i8, i16, i32, u8, u16, u32);

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
///
/// Implement this trait for application-specific scalar decoding; use [`crate::Row`]
/// lookups to compose independent DTOs. Returned data must own its contents.
///
/// # Errors
/// Return [`crate::ErrorKind::Decode`] on incompatible storage types or out-of-range values.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{FromSqlValue, SqlValue};
/// assert_eq!(i64::from_sql(&SqlValue::Integer(7))?, 7);
/// assert!(i64::from_sql(&SqlValue::Text("7".into())).is_err());
/// assert_eq!(Option::<i64>::from_sql(&SqlValue::Null)?, None);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
pub trait FromSqlValue: Sized {
    /// Return an owned value or a typed NULL/type/range error.
    ///
    /// # Errors
    /// Returns a decode error for incompatible types/ranges; no implicit string coercion.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{FromSqlValue, SqlValue};
    /// assert_eq!(i64::from_sql(&SqlValue::Integer(7))?, 7);
    /// assert!(i64::from_sql(&SqlValue::Text("7".into())).is_err());
    /// assert_eq!(Option::<i64>::from_sql(&SqlValue::Null)?, None);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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

impl<T: FromSqlValue> FromSqlValue for Option<T> {
    /// Decode SQL NULL as None, delegating every non-NULL value to strict decoding.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        if matches!(value, SqlValue::Null) {
            Ok(None)
        } else {
            T::from_sql(value).map(Some)
        }
    }
}

impl FromSqlValue for f64 {
    /// Decode finite real values only; integer-to-real precision loss is not implicit.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        match value {
            SqlValue::Real(value) if value.is_finite() => Ok(*value),
            _ => Err(SqlError::new(ErrorKind::Decode, "expected finite SQL real")),
        }
    }
}

impl FromSqlValue for Vec<u8> {
    /// Decode native binary storage only.
    fn from_sql(value: &SqlValue) -> Result<Self> {
        match value {
            SqlValue::Blob(value) => Ok(value.clone()),
            _ => Err(SqlError::new(ErrorKind::Decode, "expected SQL blob")),
        }
    }
}

/// Implement range-checked integer decoders without permissive scalar coercion.
macro_rules! integer_decoders {
    ($($kind:ty),+ $(,)?) => { $(
        impl FromSqlValue for $kind {
            /// Decode an exact native integer and reject target-range overflow.
            fn from_sql(value: &SqlValue) -> Result<Self> {
                let integer = i64::from_sql(value)?;
                Self::try_from(integer).map_err(|_| SqlError::new(ErrorKind::Decode, "SQL integer decode out of range"))
            }
        }
    )+ };
}
integer_decoders!(i8, i16, i32, u8, u16, u32, u64, usize);

/// Ordered native column metadata. Duplicate names remain separate at distinct ordinals.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
/// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
///     .with_metadata(vec![
///         ColumnMetadata {
///             name: "id".into(),
///             ordinal: 0,
///             declared_type: Some("INTEGER".into()),
///         },
///         ColumnMetadata {
///             name: "name".into(),
///             ordinal: 1,
///             declared_type: None,
///         },
///     ])?;
/// assert_eq!(row.try_get::<i64>("id")?, 7);
/// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
/// assert_eq!(row.cells().len(), 2);
/// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
/// assert_eq!(
///     (column.name, column.ordinal, column.declared_type),
///     ("id", 0, Some("INTEGER"))
/// );
/// assert_eq!(row.retained_bytes()?, 24);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ColumnMetadata {
    /// Result-column name or explicit projection alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::ColumnMetadata;
    /// let value = ColumnMetadata {
    ///     name: "id".into(),
    ///     ordinal: 0,
    ///     declared_type: Some("INTEGER".into()),
    /// };
    /// assert_eq!(value.name, "id");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub name: String,
    /// Zero-based ordinal in this result row.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::ColumnMetadata;
    /// let value = ColumnMetadata {
    ///     name: "id".into(),
    ///     ordinal: 0,
    ///     declared_type: Some("INTEGER".into()),
    /// };
    /// assert_eq!(value.ordinal, 0);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub ordinal: usize,
    /// Driver-reported declaration, when known; never used to coerce storage values.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::ColumnMetadata;
    /// let value = ColumnMetadata {
    ///     name: "id".into(),
    ///     ordinal: 0,
    ///     declared_type: Some("INTEGER".into()),
    /// };
    /// assert_eq!(value.declared_type.as_deref(), Some("INTEGER"));
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub declared_type: Option<String>,
}

/// Borrowed result metadata; names borrow row cells instead of duplicating retained bytes.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
/// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
///     .with_metadata(vec![
///         ColumnMetadata {
///             name: "id".into(),
///             ordinal: 0,
///             declared_type: Some("INTEGER".into()),
///         },
///         ColumnMetadata {
///             name: "name".into(),
///             ordinal: 1,
///             declared_type: None,
///         },
///     ])?;
/// assert_eq!(row.try_get::<i64>("id")?, 7);
/// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
/// assert_eq!(row.cells().len(), 2);
/// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
/// assert_eq!(
///     (column.name, column.ordinal, column.declared_type),
///     ("id", 0, Some("INTEGER"))
/// );
/// assert_eq!(row.retained_bytes()?, 24);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColumnMetadataRef<'a> {
    /// Result-column name or projection alias.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Row;
    /// let row = Row::new(vec![("id".into(), 7.into())]);
    /// let value = row.columns().next().unwrap();
    /// assert_eq!(value.name, "id");
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub name: &'a str,
    /// Zero-based ordinal.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Row;
    /// let row = Row::new(vec![("id".into(), 7.into())]);
    /// let value = row.columns().next().unwrap();
    /// assert_eq!(value.ordinal, 0);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub ordinal: usize,
    /// Optional native declaration, independent from the actual value's storage type.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::Row;
    /// let row = Row::new(vec![("id".into(), 7.into())]);
    /// let value = row.columns().next().unwrap();
    /// assert_eq!(value.declared_type, None);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub declared_type: Option<&'a str>,
}

/// Ordered names and values, including duplicate names from joins.
///
/// Names need not be unique. [`Self::try_get`] errors on duplicate/missing names;
/// [`Self::try_get_at`] selects the exact ordinal. Native declarations are diagnostic
/// metadata and never force value coercion. Rows own all data after leases release.
/// Built-in decoding supports strict String/i64/finite f64/bytes, checked integer
/// ranges, bool from Bool or integer 0/1, and `Option<T>` for SQL NULL.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
/// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
///     .with_metadata(vec![
///         ColumnMetadata {
///             name: "id".into(),
///             ordinal: 0,
///             declared_type: Some("INTEGER".into()),
///         },
///         ColumnMetadata {
///             name: "name".into(),
///             ordinal: 1,
///             declared_type: None,
///         },
///     ])?;
/// assert_eq!(row.try_get::<i64>("id")?, 7);
/// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
/// assert_eq!(row.cells().len(), 2);
/// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
/// assert_eq!(
///     (column.name, column.ordinal, column.declared_type),
///     ("id", 0, Some("INTEGER"))
/// );
/// assert_eq!(row.retained_bytes()?, 24);
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    /// Ordered names/values retained for compatibility with native adapter collection.
    cells: Vec<(String, SqlValue)>,
    /// Optional native declarations aligned with cells; names/ordinals are derived as views.
    declared_types: Vec<Option<String>>,
}

impl Row {
    /// Construct an owned ordered row in an adapter.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn new(cells: Vec<(String, SqlValue)>) -> Self {
        // Even adapters without declared types retain complete names and column order.
        let declared_types = vec![None; cells.len()];
        Self {
            cells,
            declared_types,
        }
    }

    /// Attach explicit adapter metadata, checking row/name/ordinal consistency.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ErrorKind::Decode`] if count, names or zero-based ordinals disagree with cells.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn with_metadata(mut self, columns: Vec<ColumnMetadata>) -> Result<Self> {
        if columns.len() != self.cells.len()
            || columns
                .iter()
                .zip(&self.cells)
                .enumerate()
                .any(|(index, (column, (name, _)))| column.ordinal != index || &column.name != name)
        {
            return Err(SqlError::new(
                ErrorKind::Decode,
                "SQL row metadata mismatch",
            ));
        }
        // Retain declarations only: the same name must not consume the output budget twice.
        self.declared_types = columns
            .into_iter()
            .map(|column| column.declared_type)
            .collect();
        Ok(self)
    }

    /// Inspect every ordered metadata entry, including duplicate names.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn columns(&self) -> impl ExactSizeIterator<Item = ColumnMetadataRef<'_>> {
        self.cells.iter().zip(&self.declared_types).enumerate().map(
            |(ordinal, ((name, _), declared_type))| ColumnMetadataRef {
                name,
                ordinal,
                declared_type: declared_type.as_deref(),
            },
        )
    }

    /// Count retained names, values and native type text with checked arithmetic.
    /// Adapters attaching declarations should include this complete value in their budgets.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ErrorKind::Limit`] if complete byte accounting overflows usize.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn retained_bytes(&self) -> Result<usize> {
        let mut bytes = 0_usize;
        for ((name, value), declaration) in self.cells.iter().zip(&self.declared_types) {
            bytes = bytes
                .checked_add(name.len())
                .and_then(|bytes| bytes.checked_add(value.retained_bytes()))
                .and_then(|bytes| bytes.checked_add(declaration.as_ref().map_or(0, String::len)))
                .ok_or_else(|| SqlError::new(ErrorKind::Limit, "SQL row byte count overflow"))?;
        }
        Ok(bytes)
    }

    /// Decode by ordinal, including otherwise ambiguous duplicate named columns.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ErrorKind::Decode`] for an absent ordinal or incompatible value types/ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn try_get_at<T: FromSqlValue>(&self, index: usize) -> Result<T> {
        let (_, value) = self
            .cells
            .get(index)
            .ok_or_else(|| SqlError::new(ErrorKind::Decode, "SQL column ordinal out of range"))?;
        T::from_sql(value)
    }

    /// Inspect every ordered column without overwriting duplicate names.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub fn cells(&self) -> &[(String, SqlValue)] {
        &self.cells
    }

    /// Decode a unique named cell; missing/ambiguous names are errors.
    ///
    /// # Errors
    ///
    /// Returns [`crate::ErrorKind::Decode`] for missing/duplicate names or incompatible value types/ranges.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::{ColumnMetadata, ColumnMetadataRef, Row};
    /// let row = Row::new(vec![("id".into(), 7.into()), ("name".into(), "Ada".into())])
    ///     .with_metadata(vec![
    ///         ColumnMetadata {
    ///             name: "id".into(),
    ///             ordinal: 0,
    ///             declared_type: Some("INTEGER".into()),
    ///         },
    ///         ColumnMetadata {
    ///             name: "name".into(),
    ///             ordinal: 1,
    ///             declared_type: None,
    ///         },
    ///     ])?;
    /// assert_eq!(row.try_get::<i64>("id")?, 7);
    /// assert_eq!(row.try_get_at::<String>(1)?, "Ada");
    /// assert_eq!(row.cells().len(), 2);
    /// let column: ColumnMetadataRef<'_> = row.columns().next().unwrap();
    /// assert_eq!(
    ///     (column.name, column.ordinal, column.declared_type),
    ///     ("id", 0, Some("INTEGER"))
    /// );
    /// assert_eq!(row.retained_bytes()?, 24);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
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

/// Validate ASCII UTC timestamps without a runtime/codec dependency or timezone guesses.
fn validate_utc(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    let invalid = || crate::identifier::invalid("invalid canonical UTC date text");
    if !(20..=30).contains(&bytes.len())
        || bytes.get(4) != Some(&b'-')
        || bytes.get(7) != Some(&b'-')
        || bytes.get(10) != Some(&b'T')
        || bytes.get(13) != Some(&b':')
        || bytes.get(16) != Some(&b':')
        || bytes.last() != Some(&b'Z')
    {
        return Err(invalid());
    }
    // Every date/time component is ASCII digits, making the fixed byte slices safe.
    for (start, end) in [(0, 4), (5, 7), (8, 10), (11, 13), (14, 16), (17, 19)] {
        if !bytes[start..end].iter().all(u8::is_ascii_digit) {
            return Err(invalid());
        }
    }
    if bytes.len() > 20
        && (bytes[19] != b'.'
            || bytes.len() < 22
            || !bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit))
    {
        return Err(invalid());
    }
    let number = |start: usize, end: usize| {
        bytes[start..end]
            .iter()
            .fold(0_u32, |number, digit| number * 10 + u32::from(digit - b'0'))
    };
    let year = number(0, 4);
    let month = number(5, 7);
    let day = number(8, 10);
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return Err(invalid()),
    };
    if year == 0
        || day == 0
        || day > days
        || number(11, 13) > 23
        || number(14, 16) > 59
        || number(17, 19) > 59
    {
        return Err(invalid());
    }
    Ok(())
}

/// One statement's outcome, never shared mutable last-insert state.
///
/// All values are owned and statement-local. An absent insert_id differs from zero;
/// there is no shared engine last-ID slot. Driver-specific RETURNING/affected-row
/// semantics remain explicit, rather than predicting IDs for bulk inserts.
///
/// # Examples
///
/// ```
/// use stackpress_sql::{QueryResult, Row};
/// let result = QueryResult {
///     rows: vec![Row::new(vec![("id".into(), 7.into())])],
///     affected: 1,
///     insert_id: Some(7),
/// };
/// assert_eq!(result.rows[0].try_get::<i64>("id")?, 7);
/// assert_eq!((result.affected, result.insert_id), (1, Some(7)));
/// # Ok::<(), stackpress_sql::SqlError>(())
/// ```
#[derive(Debug, Default)]
pub struct QueryResult {
    /// Owned ordered rows.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryResult;
    /// let value = QueryResult {
    ///     rows: vec![],
    ///     affected: 0,
    ///     insert_id: None,
    /// };
    /// assert!(value.rows.is_empty());
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub rows: Vec<Row>,
    /// Native affected-row count for this statement.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryResult;
    /// let value = QueryResult {
    ///     rows: vec![],
    ///     affected: 0,
    ///     insert_id: None,
    /// };
    /// assert_eq!(value.affected, 0);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub affected: u64,
    /// Insert IDs are not yet exposed by the M0 adapter; use RETURNING instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use stackpress_sql::QueryResult;
    /// let value = QueryResult {
    ///     rows: vec![],
    ///     affected: 0,
    ///     insert_id: None,
    /// };
    /// assert_eq!(value.insert_id, None);
    /// # Ok::<(), stackpress_sql::SqlError>(())
    /// ```
    pub insert_id: Option<i64>,
}
