//! Retain owned rows only after checking each name/value against the statement budget.
//!
//! SQLx may already have allocated a native row/cell. This boundary prevents a second,
//! first-party text/blob copy from exceeding the configured retained-value budget.

use sqlx::sqlite::SqliteRow;
use sqlx::{Column, Row as NativeRow, TypeInfo, ValueRef};
use stackpress_sql::{ErrorKind, QueryLimits, Result, Row, SqlError, SqlValue};

use crate::execute::driver_error;

/// One statement's monotonic retained payload budget; failed reservations do not advance it.
pub(crate) struct OutputBudget {
    /// Caller-selected finite caps, independent of native VM/allocator usage.
    limits: QueryLimits,
    /// Rows admitted so far, never greater than the configured maximum.
    rows: usize,
    /// Name/value bytes admitted so far, never greater than the configured maximum.
    bytes: usize,
}

impl OutputBudget {
    /// Start a statement with no retained rows or payload.
    pub(crate) fn new(limits: QueryLimits) -> Self {
        Self {
            limits,
            rows: 0,
            bytes: 0,
        }
    }

    /// Reserve a row before decoding; the cap also bounds per-row container overhead.
    pub(crate) fn start_row(&mut self) -> Result<()> {
        if self.rows >= self.limits.rows {
            return Err(output_limit());
        }
        // The preceding comparison proves this increment cannot overflow.
        self.rows += 1;
        Ok(())
    }

    /// Reserve bytes before ownership/copying, leaving the previous count intact on error.
    pub(crate) fn charge(&mut self, bytes: usize) -> Result<()> {
        let next = self.bytes.checked_add(bytes).ok_or_else(output_limit)?;
        if next > self.limits.bytes {
            return Err(output_limit());
        }
        self.bytes = next;
        Ok(())
    }

    /// Decode one owned row using runtime storage types and ordered native column names.
    pub(crate) fn decode(&mut self, native: SqliteRow) -> Result<Row> {
        self.start_row()?;
        let mut cells = Vec::with_capacity(native.len());
        for (index, column) in native.columns().iter().enumerate() {
            // Reserve the borrowed name before creating any owned metadata string.
            self.charge(column.name().len())?;
            let value = self.decode_cell(&native, index)?;
            cells.push((column.name().to_owned(), value));
        }
        Ok(Row::new(cells))
    }

    /// Borrow variable-size values for sizing, then copy only after the reservation succeeds.
    fn decode_cell(&mut self, native: &SqliteRow, index: usize) -> Result<SqlValue> {
        let raw = native.try_get_raw(index).map_err(driver_error)?;
        if raw.is_null() {
            return Ok(SqlValue::Null);
        }
        match raw.type_info().name() {
            "INTEGER" => {
                self.charge(8)?;
                Ok(SqlValue::Integer(
                    native.try_get(index).map_err(driver_error)?,
                ))
            }
            "REAL" => {
                self.charge(8)?;
                let value: f64 = native.try_get(index).map_err(driver_error)?;
                // SQLite calculations can overflow even when every input bind is finite.
                if !value.is_finite() {
                    return Err(SqlError::new(
                        ErrorKind::Decode,
                        "SQLite real must be finite",
                    ));
                }
                Ok(SqlValue::Real(value))
            }
            "TEXT" => {
                let value: &str = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                Ok(SqlValue::Text(value.to_owned()))
            }
            "BLOB" => {
                let value: &[u8] = native.try_get(index).map_err(driver_error)?;
                self.charge(value.len())?;
                Ok(SqlValue::Blob(value.to_owned()))
            }
            _ => Err(SqlError::new(
                ErrorKind::Decode,
                "unsupported SQLite storage type",
            )),
        }
    }
}

/// Report an overrun instead of returning a silently truncated/partially decoded result.
fn output_limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL retained output limit exceeded")
}
