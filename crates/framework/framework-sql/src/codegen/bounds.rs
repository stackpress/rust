//! Host-only length guards retain private declaration arguments outside browser contracts.
use serde_json::json;
use stackpress_framework_schema::Model;

use super::templates::{Templates, literal};

/// Emit guards only for annotated models; null and unchanged patches retain their semantics.
pub(super) fn emit(
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Use ordered metadata and explicit flags; absent declarations render no source at all.
    let fields: Vec<_> = model
        .fields
        .iter()
        .filter_map(|field| {
            field.storage_length.map(|length| {
                json!({
                    "member": field.rust_name, "required": field.required, "length": length,
                })
            })
        })
        .collect();
    let patch_fields: Vec<_> = model
        .fields
        .iter()
        .filter(|field| !field.key && !field.generated)
        .filter_map(|field| {
            field.storage_length.map(|length| {
                json!({
                    "column_literal": literal(&field.column), "length": length,
                })
            })
        })
        .collect();
    templates.append(text, "SQL bounds template failed", include_str!("templates/bounds.rs.tpl"), &json!({
        "model": model.rust_name, "bounded": !fields.is_empty(), "patch_bounded": !patch_fields.is_empty(),
        "fields": fields, "patch_fields": patch_fields,
    }))
}
