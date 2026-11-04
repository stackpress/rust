//! Source query/computed-key interpolation, separate from JSON column encoding.
use stackpress_framework_schema::source::{Value, number_text};
/// JS query interpolation/computed-property keys differ from the column JSON codec.
/// Arrays join, null entries are empty and objects use their ordinary object spelling.
pub(crate) fn source_text(value: &Value, offset: i32) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Object(_) => "[object Object]".into(),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    String::new()
                } else {
                    source_text(value, offset)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Date { millis, display } => display.clone().unwrap_or_else(|| {
            stackpress_framework_schema::source::date_string(
                *millis,
                offset,
                "Coordinated Universal Time",
            )
            .unwrap_or_else(|_| "Invalid Date".into())
        }),
        Value::NonFinite(value) => number_text(*value),
        Value::Number(value) => value
            .as_f64()
            .map(number_text)
            .unwrap_or_else(|| value.to_string()),
        _ => value.to_string(),
    }
}
