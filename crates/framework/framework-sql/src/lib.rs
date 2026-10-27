//! Generated-store support over exact borrowed SQL executors, with no ambient transactions.
//!
//! `DataSession` is exclusive and non-Clone. The caller owns transaction completion;
//! operation/lifecycle success never commits implicitly. No backend or server is selected
//! by default. `sqlite-maintenance` selects a separate operator capability, never a route.
//!
//! ```no_run
//! use stackpress_framework_sql::{DataResult, DataSession};
//! use stackpress_sql::{Engine, Select};
//! async fn inspect(engine: &Engine) -> DataResult<()> {
//!     let mut transaction = engine.begin().await?;
//!     {
//!         // Every generated store receives this same exclusive borrowed executor.
//!         let mut session = DataSession::transaction(&mut transaction);
//!         let _rows = session.fetch(&Select::new(["id"]).from("profile")).await?;
//!     }
//!     transaction.rollback().await?;
//!     Ok(())
//! }
//! ```

use std::error::Error;
use std::fmt;

use serde::{Serialize, de::DeserializeOwned};
use stackpress_sql::{
    DialectId, Engine, QueryResult, Render, Row, SqlError, SqlValue, Transaction,
};

#[cfg(feature = "codegen")]
pub mod codegen;
#[cfg(feature = "migrations")]
pub mod migrations;

/// Runtime contract revision checked by emitted code.
pub const CONTRACT_REVISION: u32 = 1;
/// Fallible domain/storage operation retaining original typed causes.
pub type DataResult<T> = Result<T, DataError>;

/// Stable data-layer categories; private SQL diagnostics are never parsed as strings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataErrorKind {
    /// Invalid domain input or an unsupported shape.
    Validation,
    /// A SQL execution/rendering failure.
    Sql,
    /// Stored data or structured artifacts cannot be decoded faithfully.
    Decode,
    /// The selected M0 capability has no implementation.
    Unsupported,
    /// Plan, ledger or catalog no longer agrees with approved state.
    Stale,
    /// COMMIT was submitted but its final acknowledgement was lost.
    UnknownOutcome,
}

/// Distinct safe context, a concrete cause and optional secondary cleanup failure.
#[derive(Debug)]
pub struct DataError {
    /// Selected by the boundary that understands the operation.
    kind: DataErrorKind,
    /// Never includes input values, SQL, database paths or cause Display.
    context: &'static str,
    /// Original concrete error, available only by deliberate private traversal.
    cause: Option<Box<dyn Error + Send + Sync>>,
    /// Cleanup failure retained separately from the primary cause.
    cleanup: Option<SqlError>,
}

impl DataError {
    /// Describe one failing boundary independently of its cause.
    pub fn new(context: &'static str) -> Self {
        Self {
            kind: DataErrorKind::Validation,
            context,
            cause: None,
            cleanup: None,
        }
    }
    /// Select the stable semantic category.
    pub fn with_kind(mut self, kind: DataErrorKind) -> Self {
        self.kind = kind;
        self
    }
    /// Preserve the original concrete failure without formatting it into context.
    pub fn with_cause(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }
    /// Preserve a secondary native rollback/close failure without replacing the primary error.
    pub fn with_cleanup(mut self, cleanup: SqlError) -> Self {
        self.cleanup = Some(cleanup);
        self
    }
    /// Inspect the semantic category without matching private text.
    pub fn kind(&self) -> DataErrorKind {
        self.kind
    }
    /// Inspect an explicit secondary cleanup failure when one occurred.
    pub fn cleanup_error(&self) -> Option<&SqlError> {
        self.cleanup.as_ref()
    }
}
impl fmt::Display for DataError {
    /// Display one safe explanation only.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.context)
    }
}
impl Error for DataError {
    /// Retain the original typed native or schema error.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as _)
    }
}
impl From<SqlError> for DataError {
    /// Add domain storage context while preserving the original SQL category and source.
    fn from(cause: SqlError) -> Self {
        Self::new("data storage operation failed")
            .with_kind(DataErrorKind::Sql)
            .with_cause(cause)
    }
}

/// Exclusive borrowed executor. Store functions never own an Engine or acquire a hidden pool.
pub enum DataSession<'a> {
    /// Independent single-statement calls; multi-statement promises require a transaction.
    Pooled(&'a Engine),
    /// Exact caller-owned transaction, retained across awaits without cloning or widening.
    Transaction(&'a mut Transaction),
}

impl<'a> DataSession<'a> {
    /// Borrow an app-owned Engine for independent statement execution.
    pub fn pooled(engine: &'a Engine) -> Self {
        Self::Pooled(engine)
    }
    /// Exclusively borrow the exact transaction selected by the caller.
    pub fn transaction(transaction: &'a mut Transaction) -> Self {
        Self::Transaction(transaction)
    }
    /// Return the underlying renderer identity without another acquisition.
    pub fn dialect(&self) -> DialectId {
        match self {
            Self::Pooled(engine) => engine.dialect(),
            Self::Transaction(transaction) => transaction.dialect(),
        }
    }
    /// Execute one typed query on the selected executor, with a future bounded by this borrow.
    pub async fn execute(&mut self, query: &(impl Render + Sync)) -> DataResult<QueryResult> {
        match self {
            Self::Pooled(engine) => Ok(engine.execute(query).await?),
            Self::Transaction(transaction) => Ok(transaction.execute(query).await?),
        }
    }
    /// Fetch owned rows; no lazy fetch or connection reference escapes the borrow.
    pub async fn fetch(&mut self, query: &(impl Render + Sync)) -> DataResult<Vec<Row>> {
        Ok(self.execute(query).await?.rows)
    }
}

/// A strict storage codec selected by generated typed field metadata.
#[derive(Clone, Copy, Debug)]
pub enum StorageCodec {
    /// Unchanged UTF-8 or a checked string enum.
    Text,
    /// Exact signed 64-bit integer.
    Integer,
    /// Finite floating-point number.
    Real,
    /// Boolean; SQL NULL and truthy text are never coerced.
    Boolean,
    /// Explicitly encoded bounded JSON.
    Json,
}

/// One field's wire/storage mapping, emitted as immutable compile-time data.
#[derive(Clone, Copy, Debug)]
pub struct StorageColumn {
    /// Exact external field spelling.
    pub wire: &'static str,
    /// Checked SQL column name.
    pub column: &'static str,
    /// Strict application codec.
    pub codec: StorageCodec,
    /// Whether SQL NULL is forbidden.
    pub required: bool,
    /// Nonempty text validation, applied on create and stored decode.
    pub nonempty: bool,
}

/// Encode a checked owned contract without interpreting arbitrary wire keys as SQL.
/// Serialization stops at 1 MiB before allocating the decoded JSON object.
pub fn encode<T: Serialize>(value: &T, columns: &[StorageColumn]) -> DataResult<Vec<SqlValue>> {
    if columns.len() > 128 {
        return Err(DataError::new("record shape does not match storage"));
    }
    let mut buffer = RecordBuffer::default();
    if let Err(cause) = serde_json::to_writer(&mut buffer, value) {
        // Distinguish the finite input ceiling from an unrelated custom serialization failure.
        if buffer.exceeded {
            return Err(DataError::new("record byte limit exceeded")
                .with_kind(DataErrorKind::Validation)
                .with_cause(cause));
        }
        return Err(codec_error(cause));
    }
    // Only a bounded complete JSON document can become a retained per-field value tree.
    let value: serde_json::Value = serde_json::from_slice(&buffer.bytes).map_err(codec_error)?;
    let object = value
        .as_object()
        .ok_or_else(|| DataError::new("record must be an object"))?;
    if object.len() != columns.len() {
        return Err(DataError::new("record shape does not match storage"));
    }
    columns
        .iter()
        .map(|column| {
            let value = object
                .get(column.wire)
                .ok_or_else(|| DataError::new("record field is missing"))?;
            encode_value(value, column)
        })
        .collect()
}

/// A finite in-memory serialization sink; it never writes a partial record to a database.
#[derive(Default)]
struct RecordBuffer {
    /// Logical encoded bytes retained only until input encoding finishes.
    bytes: Vec<u8>,
    /// Set before rejecting a write that would cross the aggregate ceiling.
    exceeded: bool,
}

impl std::io::Write for RecordBuffer {
    /// Refuse the entire next chunk before extending the buffer beyond its logical ceiling.
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if bytes.len() > (1024 * 1024_usize).saturating_sub(self.bytes.len()) {
            self.exceeded = true;
            return Err(std::io::Error::other("record byte limit exceeded"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    /// No external sink or pending IO exists for this owned byte buffer.
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Decode strict native values through the generated contract's checked Serde representation.
pub fn decode<T: DeserializeOwned>(row: Row, columns: &[StorageColumn]) -> DataResult<T> {
    let mut object = serde_json::Map::new();
    for column in columns {
        let native = row
            .cells()
            .iter()
            .find(|(name, _)| name == column.column)
            .map(|(_, value)| value)
            .ok_or_else(|| {
                DataError::new("stored field is missing").with_kind(DataErrorKind::Decode)
            })?;
        let value = if *native == SqlValue::Null {
            if column.required {
                return Err(DataError::new("stored required field is null")
                    .with_kind(DataErrorKind::Decode));
            }
            serde_json::Value::Null
        } else {
            match column.codec {
                StorageCodec::Text => {
                    serde_json::Value::String(row.try_get::<String>(column.column)?)
                }
                StorageCodec::Integer => {
                    serde_json::Value::from(row.try_get::<i64>(column.column)?)
                }
                StorageCodec::Boolean => {
                    serde_json::Value::Bool(row.try_get::<bool>(column.column)?)
                }
                StorageCodec::Real => {
                    let number = serde_json::Number::from_f64(row.try_get::<f64>(column.column)?)
                        .ok_or_else(|| {
                        DataError::new("stored number is not finite")
                            .with_kind(DataErrorKind::Decode)
                    })?;
                    serde_json::Value::Number(number)
                }
                StorageCodec::Json => {
                    let text: String = row.try_get(column.column)?;
                    if text.len() > 1024 * 1024 {
                        return Err(DataError::new("stored JSON byte limit exceeded")
                            .with_kind(DataErrorKind::Decode));
                    }
                    serde_json::from_str(&text).map_err(codec_error)?
                }
            }
        };
        // Validate the codec before exposing the decoded contract, retaining null distinctly.
        encode_value(&value, column)?;
        object.insert(column.wire.into(), value);
    }
    serde_json::from_value(serde_json::Value::Object(object)).map_err(codec_error)
}

/// Encode one scalar or explicit JSON value without loss or truthiness conversion.
fn encode_value(value: &serde_json::Value, column: &StorageColumn) -> DataResult<SqlValue> {
    if value.is_null() {
        return if column.required {
            Err(DataError::new("required field cannot be null"))
        } else {
            Ok(SqlValue::Null)
        };
    }
    let encoded = match column.codec {
        StorageCodec::Text => {
            let value = value
                .as_str()
                .ok_or_else(|| DataError::new("field requires text"))?;
            if value.len() > 1024 * 1024 || (column.nonempty && value.is_empty()) {
                return Err(DataError::new("text validation failed"));
            }
            SqlValue::Text(value.into())
        }
        StorageCodec::Integer => SqlValue::Integer(
            value
                .as_i64()
                .ok_or_else(|| DataError::new("field requires an exact integer"))?,
        ),
        StorageCodec::Boolean => SqlValue::Bool(
            value
                .as_bool()
                .ok_or_else(|| DataError::new("field requires Boolean"))?,
        ),
        StorageCodec::Real => {
            let number = value
                .as_f64()
                .filter(|number| number.is_finite())
                .ok_or_else(|| DataError::new("field requires a finite number"))?;
            // A JSON integer is accepted only when its exact value survives the real codec.
            let exact_signed = value
                .as_i64()
                .is_none_or(|integer| number as i128 == i128::from(integer));
            let exact_unsigned = value
                .as_u64()
                .is_none_or(|integer| number as i128 == i128::from(integer));
            if !exact_signed || !exact_unsigned {
                return Err(DataError::new(
                    "integer cannot be represented exactly as real",
                ));
            }
            SqlValue::Real(number)
        }
        StorageCodec::Json => {
            let text = serde_json::to_string(value).map_err(codec_error)?;
            if text.len() > 1024 * 1024 {
                return Err(DataError::new("JSON byte limit exceeded"));
            }
            SqlValue::Text(text)
        }
    };
    Ok(encoded)
}

/// Preserve a concrete JSON error below safe storage codec context.
pub fn codec_error(cause: serde_json::Error) -> DataError {
    DataError::new("data codec operation failed")
        .with_kind(DataErrorKind::Decode)
        .with_cause(cause)
}
