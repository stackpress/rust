//! Typed explicit primary/single-field-unique conflict targets, independent of public request policy.
use serde_json::json;
use stackpress_framework_schema::{Model, Schema};

use super::templates::{Templates, key_records, literal};
use super::{scalar, variant};

/// Emit every schema-supported unique target; nullable unique fields require a nonnull scalar key.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Wire tags remain lexical names; type and conflict-expression syntax belongs to the template.
    let unique_fields = model.fields.iter().filter(|field| field.unique).map(|field| Ok(json!({
        "name_literal": literal(&field.name), "tag_literal": literal(&format!("unique.{}", field.name)),
        "column_literal": literal(&field.column), "variant": variant(field), "ty": scalar(schema, field, templates)?,
    }))).collect::<stackpress_idea_transformer::Result<Vec<_>>>()?;
    templates.append(
        text,
        "SQL conflicts template failed",
        include_str!("templates/conflicts.rs.tpl"),
        &json!({
            "model": model.rust_name, "unique_fields": unique_fields, "keys": key_records(model),
        }),
    )
}
