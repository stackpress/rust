//! Typed native selectors and executing read functions. Public DTOs remain schema-owned.
use serde_json::json;
use stackpress_framework_schema::{Field, Model, Schema};
use stackpress_idea_transformer::TransformError;

use super::templates::{Templates, literal};
use super::{is_scalar, key_argument, scalar, variant};

/// Emit native selectors and bounded read/maintenance/atomic functions over the runtime catalog.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Resolve every scalar in declaration order before rendering the typed filter variants.
    let filters = stored.iter().filter(|field| is_scalar(field)).map(|field| Ok(json!({
        "name_literal": literal(&field.name), "variant": variant(field), "ty": scalar(schema, field, templates)?,
    }))).collect::<stackpress_idea_transformer::Result<Vec<_>>>()?;
    let sort_fields: Vec<_> = stored
        .iter()
        .filter(|field| is_scalar(field) && !field.multiple && (field.key || field.sortable))
        .map(|field| {
            json!({
                "name_literal": literal(&field.name), "variant": variant(field),
            })
        })
        .collect();
    let (key_type, key_value) = key_argument(model, templates)?;
    let legacy_key = key_type != "&Key";
    // Only legacy selectors inspect one key column; compound keys use the complete identity API.
    let key_column = if legacy_key {
        Some(
            model
                .fields
                .iter()
                .find(|field| field.key)
                .ok_or_else(|| TransformError::new("legacy scalar key missing"))?,
        )
    } else {
        None
    };
    let active = model.fields.iter().find(|field| field.active);
    templates.append(text, "SQL queries template failed", include_str!("templates/queries.rs.tpl"), &json!({
        "model": model.rust_name, "table_literal": literal(&model.module), "filters": filters, "sort_fields": sort_fields,
        "key_type": key_type, "key_value": key_value, "legacy_key": legacy_key,
        "key_column_literal": key_column.map(|field| literal(&field.column)),
        "has_active": active.is_some(), "active_column_literal": active.map(|field| literal(&field.column)),
    }))
}
