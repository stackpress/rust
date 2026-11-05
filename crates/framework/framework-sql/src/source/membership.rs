//! Source nested filter flattening and backend JSON membership mechanics.
use super::SourceResult as Result;
use stackpress_framework_schema::source::{Record, Value};
use stackpress_sql::{DialectId, Expr, Identifier, JsonPath, JsonScalar, Select, Selector};
/// Source flatten traverses plain nested objects, including array indices as the original flatten(..., true) does.
pub(super) fn flatten(value: &Record) -> Record {
    flatten_values(value, true)
}

/// G53 keeps each has array on its list column; all other filters retain source indices.
pub(super) fn flatten_has(value: &Record) -> Record {
    flatten_values(value, false)
}

/// Objects always supply path segments; only the source mode expands array indices.
fn flatten_values(value: &Record, indexed_arrays: bool) -> Record {
    let mut result = Record::new();
    for (name, value) in value {
        if let Value::Object(object) = value {
            for (child, value) in flatten_values(object, indexed_arrays) {
                result.insert(format!("{name}.{child}"), value);
            }
        } else if indexed_arrays && let Value::Array(values) = value {
            // Source store/select.ts passes flatten(..., true): indices become path segments.
            for (index, value) in values.iter().enumerate() {
                for (child, value) in flatten_values(
                    &Record::from_iter([(index.to_string(), value.clone())]),
                    true,
                ) {
                    result.insert(format!("{name}.{child}"), value);
                }
            }
        } else {
            result.insert(name.clone(), value.clone());
        }
    }
    result
}

/// Convert a JSON path to the SQL crate's checked path representation.
pub(super) fn path(parts: &[String]) -> Result<JsonPath> {
    Ok(JsonPath::new(parts.iter().map(|part| {
        if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) {
            part.parse::<u32>()
                .map(stackpress_sql::JsonSegment::Index)
                .unwrap_or_else(|_| stackpress_sql::JsonSegment::Key(part.clone()))
        } else {
            stackpress_sql::JsonSegment::Key(part.clone())
        }
    })))
}

/// Native JSON membership accepts the source scalar family, with unsupported shapes explicit.
fn json_scalar(value: &Value) -> Result<JsonScalar> {
    Ok(match value {
        Value::Null => JsonScalar::Null,
        Value::Bool(value) => JsonScalar::Bool(*value),
        Value::String(value) => JsonScalar::String(value.clone()),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                JsonScalar::Integer(value)
            } else {
                JsonScalar::Real(value.as_f64().unwrap_or_default())
            }
        }
        _ => {
            return Err(super::ActionError::new(
                500,
                "JSON membership requires a scalar",
                Record::new(),
            ));
        }
    })
}

/// SQLite source json_each compares extracted SQL scalars: null never equals null,
/// and Boolean/number equality follows SQLite, rather than adding a JSON type guard.
pub(super) fn membership(
    identifier: Identifier,
    json: &[String],
    value: &Value,
    dialect: DialectId,
) -> Result<Expr> {
    if dialect != DialectId::Sqlite {
        return Ok(Expr::json_contains(
            identifier,
            path(json)?,
            json_scalar(value)?,
        ));
    }
    use stackpress_sql::Render;
    let rendered = Select::new([Selector::expression(Expr::column(identifier))])
        .from("__source")
        .render(dialect)?;
    // The selected expression is a checked identifier, with no aliases, binds or clauses.
    let column = rendered
        .sql()
        .strip_prefix("SELECT ")
        .and_then(|sql| sql.strip_suffix(" FROM \"__source\""))
        .ok_or_else(|| {
            super::ActionError::new(500, "native column rendering failed", Record::new())
        })?;
    let mut path = String::from("$");
    for part in json {
        if !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) {
            path.push_str(&format!("[{part}]"));
        } else {
            path.push('.');
            path.push_str(
                &serde_json::to_string(part).map_err(|cause| {
                    super::ActionError::new(500, cause.to_string(), Record::new())
                })?,
            );
        }
    }
    let scalar = match value {
        Value::Null => stackpress_sql::SqlValue::Null,
        Value::Bool(value) => stackpress_sql::SqlValue::Bool(*value),
        Value::Number(value) => {
            if let Some(value) = value.as_i64() {
                stackpress_sql::SqlValue::Integer(value)
            } else {
                stackpress_sql::SqlValue::Real(value.as_f64().unwrap_or_default())
            }
        }
        Value::String(value) => stackpress_sql::SqlValue::Text(value.clone()),
        _ => {
            return Err(super::ActionError::new(
                500,
                "JSON membership requires a scalar",
                Record::new(),
            ));
        }
    };
    Ok(Expr::Trusted(
        dialect,
        stackpress_sql::TrustedSql::audited(format!(
            "EXISTS (SELECT 1 FROM json_each({column}, ?) WHERE value = ?)"
        )),
        vec![path.into(), scalar],
    ))
}
