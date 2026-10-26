//! Native JSON expressions for PostgreSQL/MySQL, retaining typed key/index paths.

use crate::expr::RenderContext;
use crate::{
    DialectId, Identifier, JsonPath, JsonScalar, JsonSegment, JsonValue, Result, SqlValue,
};

/// Extract native JSON (JSONB on PostgreSQL), preserving JSON null versus SQL NULL.
pub(crate) fn extract(
    context: &mut RenderContext,
    column: &Identifier,
    path: &JsonPath,
) -> Result<String> {
    path.sqlite_path()?;
    let column = column.quoted(context.dialect)?;
    if context.dialect == DialectId::Mysql {
        let marker = context.bind(path.sqlite_path()?.into())?;
        return Ok(format!("JSON_EXTRACT({column}, {marker})"));
    }
    let mut expression = format!("CAST({column} AS JSONB)");
    for segment in path.segments() {
        // PostgreSQL overloads -> by text/integer, retaining key '01' versus array index 1.
        let (value, cast) = match segment {
            JsonSegment::Key(key) => (SqlValue::Text(key.clone()), "text"),
            JsonSegment::Index(index) => (SqlValue::Integer(i64::from(*index)), "integer"),
        };
        let marker = context.bind(value)?;
        expression = format!("({expression} -> {marker}::{cast})");
    }
    Ok(expression)
}

/// Compare native JSON values. Network JSON numbers use numeric JSON equality;
/// unlike SQLite storage types, 1 and 1.0 are the same JSON number on these dialects.
pub(crate) fn equal(
    context: &mut RenderContext,
    column: &Identifier,
    path: &JsonPath,
    value: &JsonScalar,
) -> Result<String> {
    let extracted = extract(context, column, path)?;
    let marker = scalar(context, value)?;
    Ok(format!("({extracted} = {marker})"))
}

/// Test only direct scalar array membership, including nested-array nonmatches.
pub(crate) fn contains(
    context: &mut RenderContext,
    column: &Identifier,
    path: &JsonPath,
    value: &JsonScalar,
) -> Result<String> {
    let extracted = extract(context, column, path)?;
    let repeated = extract(context, column, path)?;
    let marker = scalar(context, value)?;
    if context.dialect == DialectId::Mysql {
        // JSON_OVERLAPS compares immediate elements; JSON_CONTAINS would recurse into arrays.
        return Ok(format!(
            "(JSON_TYPE({extracted}) = 'ARRAY' AND JSON_OVERLAPS({repeated}, JSON_ARRAY({marker})))"
        ));
    }
    Ok(format!(
        "EXISTS (SELECT 1 FROM jsonb_array_elements(CASE WHEN jsonb_typeof({extracted}) = 'array' THEN {repeated} ELSE '[]'::jsonb END) AS \"__sp_json\"(\"value\") WHERE \"__sp_json\".\"value\" = {marker})"
    ))
}

/// Bind canonical scalar JSON and cast explicitly so bool/number/text/null never coerce.
fn scalar(context: &mut RenderContext, value: &JsonScalar) -> Result<String> {
    let value = JsonValue::Scalar(value.clone());
    if context.dialect == DialectId::Postgres {
        crate::json::validate_postgres_json(&value)?;
    }
    let marker = context.bind(value.to_json()?.into())?;
    Ok(format!(
        "CAST({marker} AS {})",
        if context.dialect == DialectId::Postgres {
            "JSONB"
        } else {
            "JSON"
        }
    ))
}
