//! Detached source SQL scalar conversion; models supply their actual column metadata.
use super::SourceResult as Result;
use stackpress_framework_schema::source::{Column, Kind, Value, date_millis, iso, string, truthy};
use stackpress_sql::SqlValue;

/// Convert a prepared value using source null, Boolean and Date behavior.
/// Unknown projection names retain their ordinary source string conversion.
pub fn to_sql_value(
    column: Option<&Column>,
    value: &Value,
    offset_minutes: i32,
) -> Result<SqlValue> {
    let Some(column) = column else {
        return Ok(if value.is_null() {
            SqlValue::Null
        } else {
            SqlValue::Text(string(value))
        });
    };
    if column.nullable && value.is_null() {
        return Ok(SqlValue::Null);
    }
    Ok(match column.kind {
        Kind::Boolean => SqlValue::Bool(truthy(value)),
        Kind::Date => SqlValue::Text(iso(date_millis(value, offset_minutes))?),
        _ => SqlValue::Text(if value.is_null() {
            String::new()
        } else {
            string(value)
        }),
    })
}
