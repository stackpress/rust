//! Prepared key-scoped writes, active-state transitions and validate-before-effects batches.
use serde_json::json;
use stackpress_framework_schema::{Field, Model, Schema};

use super::key_argument;
use super::templates::{Templates, column, literal, policy_literal};

/// Emit one explicit timestamp refresh; no ambient clock or caller patch field can supply it.
fn timestamp(
    schema: &Schema,
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let field = model.fields.iter().find(|field| field.timestamp);
    // Build policy only for the selected branch; private literals are never expanded by the engine.
    let data = if let Some(field) = field {
        let mut policy_model = model.clone();
        policy_model.fields = vec![field.clone()];
        let policy = stackpress_framework_schema::RecordSchema {
            model: policy_model,
            fieldsets: Vec::new(),
            enums: schema.enums().to_vec(),
        };
        json!({"timestamped": true, "policy_literal": policy_literal(&policy)?, "wire_literal": literal(&field.name), "column_literal": literal(&field.column)})
    } else {
        json!({"timestamped": false})
    };
    templates.append(
        text,
        "SQL timestamp template failed",
        include_str!("templates/timestamp.rs.tpl"),
        &data,
    )
}

/// Emit native writes with complete-key helpers and legacy single-key convenience calls.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let columns: Vec<_> = stored
        .iter()
        .filter(|field| !field.key && !field.generated)
        .map(|field| column(schema, field))
        .collect();
    templates.append(
        text,
        "SQL mutable-columns template failed",
        include_str!("templates/mutable-columns.rs.tpl"),
        &json!({"columns": columns}),
    )?;
    timestamp(schema, model, templates, text)?;
    let (key_type, key_value) = key_argument(model, templates)?;
    // Preserve preparation and write ordering; the body contains no executor or provider effects here.
    templates.append(text, "SQL writes template failed", include_str!("templates/writes.rs.tpl"), &json!({
        "model": model.rust_name, "table_literal": literal(&model.module), "key_type": key_type, "key_value": key_value,
        "bounded": model.fields.iter().any(|field| field.storage_length.is_some()),
        "patch_bounded": model.fields.iter().any(|field| field.storage_length.is_some() && !field.key && !field.generated),
    }))?;
    if let Some(active) = model.fields.iter().find(|field| field.active) {
        let mut active_model = model.clone();
        active_model.fields = vec![active.clone()];
        let policy = stackpress_framework_schema::RecordSchema {
            model: active_model,
            fieldsets: Vec::new(),
            enums: Vec::new(),
        };
        templates.append(text, "SQL active template failed", include_str!("templates/active.rs.tpl"), &json!({
            "active": true, "model": model.rust_name, "table_literal": literal(&model.module),
            "policy_literal": policy_literal(&policy)?, "wire_literal": literal(&active.name), "column_literal": literal(&active.column),
        }))?;
    }
    Ok(())
}
