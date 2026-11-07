//! Native form field conversion shared by browser transports and native contract tests.
use serde_json::Value;

/// Expand an aggregate draft into source repeated/bracketed HTML field names.
pub fn fields(data: &Value) -> Vec<(String, String)> {
    /// Preserve nested array order, object keys and source scalar string conversion.
    fn append(fields: &mut Vec<(String, String)>, name: &str, value: &Value) {
        match value {
            Value::Array(rows) => {
                for row in rows {
                    append(fields, &format!("{name}[]"), row);
                }
            }
            Value::Object(row) => {
                for (key, value) in row {
                    append(fields, &format!("{name}[{key}]"), value);
                }
            }
            _ => fields.push((
                name.into(),
                value
                    .as_str()
                    .map(str::to_owned)
                    .unwrap_or_else(|| value.to_string()),
            )),
        }
    }

    // Only aggregate objects become forms; empty/non-object drafts retain an empty form.
    let mut fields = Vec::new();
    if let Some(row) = data.as_object() {
        for (key, value) in row {
            append(&mut fields, key, value);
        }
    }
    fields
}
