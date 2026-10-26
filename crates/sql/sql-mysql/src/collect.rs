//! Charge ordered native names/values before first-party ownership; never return partial rows.

use crate::execute::driver_error;
use sqlx::Type;
use sqlx::mysql::MySqlRow;
use sqlx::{Column, Row as NativeRow, TypeInfo, ValueRef};
use stackpress_sql::{ErrorKind, QueryLimits, Result, Row, SqlError, SqlValue};

/// Monotonic statement-local retained payload accounting, independent from driver packets.
pub(crate) struct OutputBudget {
    /// Caller selected finite row and retained-byte bounds.
    limits: QueryLimits,
    /// Number of rows already admitted; zero caps admit no row.
    rows: usize,
    /// Reserved name/value bytes; failed reservations leave it unchanged.
    bytes: usize,
}
impl OutputBudget {
    /// Start an empty accounting scope for one native statement.
    pub(crate) fn new(limits: QueryLimits) -> Self {
        Self {
            limits,
            rows: 0,
            bytes: 0,
        }
    }
    /// Reserve bytes before copying, rejecting arithmetic overflow and whole-result overruns.
    pub(crate) fn charge(&mut self, bytes: usize) -> Result<()> {
        let next = self.bytes.checked_add(bytes).ok_or_else(output_limit)?;
        if next > self.limits.bytes {
            return Err(output_limit());
        }
        self.bytes = next;
        Ok(())
    }
    /// Preserve native ordering/duplicate names, charging each borrowed cell before copying.
    pub(crate) fn decode(&mut self, native: &MySqlRow) -> Result<Row> {
        if self.rows >= self.limits.rows {
            return Err(output_limit());
        }
        self.rows += 1;
        let mut cells = Vec::with_capacity(native.len());
        for (index, column) in native.columns().iter().enumerate() {
            self.charge(column.name().len())?;
            let value = self.cell(native, index)?;
            cells.push((column.name().to_owned(), value));
        }
        Ok(Row::new(cells))
    }
    /// Decode supported scalar storage without lossy date/decimal/custom-type fallback.
    fn cell(&mut self, native: &MySqlRow, index: usize) -> Result<SqlValue> {
        let raw = native.try_get_raw(index).map_err(driver_error)?;
        let info = raw.type_info();
        let name = info.name();
        if !matches!(
            name,
            "BOOLEAN"
                | "TINYINT"
                | "SMALLINT"
                | "INT"
                | "MEDIUMINT"
                | "BIGINT"
                | "TINYINT UNSIGNED"
                | "SMALLINT UNSIGNED"
                | "INT UNSIGNED"
                | "MEDIUMINT UNSIGNED"
                | "BIGINT UNSIGNED"
                | "FLOAT"
                | "DOUBLE"
                | "CHAR"
                | "VARCHAR"
                | "TINYTEXT"
                | "TEXT"
                | "MEDIUMTEXT"
                | "LONGTEXT"
                | "BINARY"
                | "VARBINARY"
                | "TINYBLOB"
                | "BLOB"
                | "MEDIUMBLOB"
                | "LONGBLOB"
                | "NULL"
        ) {
            return Err(unsupported());
        }
        if raw.is_null() {
            return Ok(SqlValue::Null);
        }
        Ok(match name {
            "BOOLEAN" | "TINYINT" | "SMALLINT" | "INT" | "MEDIUMINT" | "BIGINT"
            | "TINYINT UNSIGNED" | "SMALLINT UNSIGNED" | "INT UNSIGNED" | "MEDIUMINT UNSIGNED"
            | "BIGINT UNSIGNED" => {
                self.charge(8)?;
                // SQLx's public compatibility check uses native unsigned flags, including
                // TINYINT(1), whose display name alone would lose signedness.
                let value = if <u64 as Type<sqlx::MySql>>::compatible(&info) {
                    let value: u64 = native.try_get(index).map_err(driver_error)?;
                    i64::try_from(value).map_err(|cause| {
                        SqlError::with_source(
                            ErrorKind::Decode,
                            "MySQL unsigned integer exceeds i64",
                            cause,
                        )
                    })?
                } else {
                    native.try_get::<i64, _>(index).map_err(driver_error)?
                };
                // BOOL is a native TINYINT: preserve Integer, including values other than 0/1.
                SqlValue::Integer(value)
            }
            "FLOAT" => self.real(f64::from(
                native.try_get::<f32, _>(index).map_err(driver_error)?,
            ))?,
            "DOUBLE" => self.real(native.try_get(index).map_err(driver_error)?)?,
            "CHAR" | "VARCHAR" | "TINYTEXT" | "TEXT" | "MEDIUMTEXT" | "LONGTEXT" => {
                let value: &str = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                SqlValue::Text(value.to_owned())
            }
            "BINARY" | "VARBINARY" | "TINYBLOB" | "BLOB" | "MEDIUMBLOB" | "LONGBLOB" => {
                let value: &[u8] = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                SqlValue::Blob(value.to_owned())
            }
            _ => return Err(unsupported()),
        })
    }
    /// Reject nonfinite native arithmetic before exposing the stable strict value shape.
    fn real(&mut self, value: f64) -> Result<SqlValue> {
        if !value.is_finite() {
            return Err(SqlError::new(
                ErrorKind::Decode,
                "MySQL real must be finite",
            ));
        }
        self.charge(8)?;
        Ok(SqlValue::Real(value))
    }
}
/// Reject a whole result rather than exposing a silently truncated collection.
fn output_limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL retained output limit exceeded")
}
/// Keep unsupported native type details out of public error strings.
fn unsupported() -> SqlError {
    SqlError::new(
        ErrorKind::Decode,
        "unsupported MySQL native output type in M0",
    )
}
