//! Charge ordered native names/values before first-party ownership; never return partial rows.

use sqlx::postgres::PgRow;
use sqlx::{Column, Row as NativeRow, ValueRef};

use crate::execute::driver_error;
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
    pub(crate) fn decode(&mut self, native: &PgRow) -> Result<Row> {
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
    fn cell(&mut self, native: &PgRow, index: usize) -> Result<SqlValue> {
        let raw = native.try_get_raw(index).map_err(driver_error)?;
        let info = raw.type_info();
        // Native type names can collide with built-ins; reject unknown OIDs even for NULL.
        let name = crate::execute::scalar_type(&info).ok_or_else(unsupported)?;
        if raw.is_null() {
            return Ok(SqlValue::Null);
        }
        Ok(match name {
            "BOOL" => {
                self.charge(1)?;
                SqlValue::Bool(native.try_get(index).map_err(driver_error)?)
            }
            "INT2" => {
                self.charge(8)?;
                SqlValue::Integer(i64::from(
                    native.try_get::<i16, _>(index).map_err(driver_error)?,
                ))
            }
            "INT4" => {
                self.charge(8)?;
                SqlValue::Integer(i64::from(
                    native.try_get::<i32, _>(index).map_err(driver_error)?,
                ))
            }
            "INT8" => {
                self.charge(8)?;
                SqlValue::Integer(native.try_get(index).map_err(driver_error)?)
            }
            "FLOAT4" => self.real(f64::from(
                native.try_get::<f32, _>(index).map_err(driver_error)?,
            ))?,
            "FLOAT8" => self.real(native.try_get(index).map_err(driver_error)?)?,
            "JSON" | "JSONB" => {
                // Borrow and validate through SQLx's explicit JSON codec before copying.
                let sqlx::types::Json(raw): sqlx::types::Json<&serde_json::value::RawValue> =
                    native.try_get(index).map_err(driver_error)?;
                let value = crate::json::canonical(raw, self.limits.bytes - self.bytes)?;
                self.charge(value.len())?;
                SqlValue::Text(value)
            }
            "TEXT" | "VARCHAR" => {
                let value: &str = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                SqlValue::Text(value.to_owned())
            }
            "BYTEA" => {
                let value: &[u8] = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                SqlValue::Blob(value.to_owned())
            }
            _ => return Err(unsupported()),
        })
    }
    /// Reject nonfinite native arithmetic before exposing the stable strict value shape.
    pub(crate) fn real(&mut self, value: f64) -> Result<SqlValue> {
        if !value.is_finite() {
            return Err(SqlError::new(
                ErrorKind::Decode,
                "PostgreSQL real must be finite",
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
        "unsupported PostgreSQL native output type",
    )
}
