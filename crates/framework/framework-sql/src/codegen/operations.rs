//! Explicit operation DTOs/handlers and app-owned registration, with no inferred route table.
use serde_json::{Value, json};
use stackpress_framework_schema::{Model, Schema, ValueType};
use stackpress_idea_transformer::{OutputPlan, TransformError};

use super::store_module;
use super::templates::{Templates, literal, policy_literal};

/// Emit native host-only handler modules when the app tool explicitly selects that output.
pub(super) fn emit(
    schema: &Schema,
    templates: &Templates,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let mut text = templates.render(
        "SQL operations-header template failed",
        include_str!("templates/operations-header.rs.tpl"),
        &json!({}),
    )?;
    for model in schema.models() {
        model_operations(schema, model, templates, &mut text)?;
    }
    output.write("stores/operations.rs", text)?;
    output.contribute_module("stores", "operations")
}

/// Retain native host defaults as named providers when public literal policy refuses them.
fn policy(schema: &Schema, model: &Model) -> stackpress_idea_transformer::Result<String> {
    let mut policy = stackpress_framework_schema::RecordSchema {
        model: model.clone(),
        fieldsets: schema.fieldsets().to_vec(),
        enums: schema.enums().to_vec(),
    };
    let public = schema
        .project_ui()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    // Reuse the authoritative public literal projection. Private literals remain named
    // host defaults even in this native assembly policy, rather than being copied into it.
    for record in std::iter::once(&mut policy.model).chain(&mut policy.fieldsets) {
        let projected = public
            .models
            .iter()
            .chain(&public.fieldsets)
            .find(|item| item.id == record.name);
        record.attributes.clear();
        for field in &mut record.fields {
            field.attributes.clear();
            if let Some(default) = &field.default {
                let safe = projected
                    .and_then(|model| {
                        model
                            .fields
                            .iter()
                            .find(|item| item.wire_name == field.name)
                    })
                    .is_some_and(|field| {
                        matches!(
                            &field.default,
                            stackpress_framework_schema::PublicDefault::Literal(value)
                                if value == default
                        )
                    });
                if !safe {
                    field.default = None;
                    field.value_rule = Some(stackpress_framework_schema::ValueRule {
                        provider: format!("default:{}.{}", record.name, field.name),
                        args: Vec::new(),
                        revision: "1".into(),
                    });
                }
            }
        }
    }
    policy_literal(&policy)
}

/// Resolve mutable fieldset targets without constructing qualified source paths in Rust.
fn patch_fields(schema: &Schema, model: &Model) -> stackpress_idea_transformer::Result<Vec<Value>> {
    model.fields.iter().filter(|field| !field.key && !field.generated).filter_map(|field| {
        // Only fieldsets need preparation; non-fieldset values retain their existing patch path.
        if let ValueType::Fieldset(name) = &field.value_type { Some((field, name)) } else { None }
    }).map(|(field, name)| {
        let target = schema.fieldsets().iter().find(|target| target.name == *name)
            .ok_or_else(|| TransformError::new("patch fieldset missing"))?;
        Ok(json!({"name_literal": literal(&field.name), "module": target.module, "model": target.rust_name, "multiple": field.multiple}))
    }).collect()
}

/// Describe one selected operation with strict booleans; templates never compare/evaluate names.
fn operation(name: &str) -> Value {
    // The caller supplies only the fixed, nonempty ASCII operation names below.
    let variant = name[..1].to_ascii_uppercase() + &name[1..];
    let input = match name {
        "get" | "detail" | "remove" | "restore" | "delete" => "KeyRequest",
        "create" => "CreateRequest",
        "update" => "UpdateRequest",
        "upsert" => "UpsertRequest",
        "batch" => "BatchRequest",
        _ => "",
    };
    let patch = matches!(name, "update" | "upsert");
    let mutation = matches!(name, "remove" | "restore" | "delete");
    json!({
        "name": name, "variant": variant, "input": input,
        "host": matches!(name, "create" | "update" | "upsert" | "batch" | "remove" | "restore"),
        "get": name == "get", "detail": name == "detail", "search": name == "search", "create": name == "create",
        "patch": patch, "upsert": name == "upsert", "batch": name == "batch", "mutation": mutation,
        "context": name != "delete", "project_row": name == "get" || name == "create" || patch || mutation,
    })
}

/// Emit one model's public envelopes, selected handlers and registration in their original order.
fn model_operations(
    schema: &Schema,
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let policy = policy(schema, model)?;
    // Policy serialization precedes fieldset resolution, as in the original emitter.
    let fieldsets = patch_fields(schema, model)?;
    let store = store_module(model);
    templates.append(text, "SQL operations-open template failed", include_str!("templates/operations-open.rs.tpl"), &json!({
        "model": model.rust_name, "module": model.module, "store": store, "policy_literal": policy,
        "has_fieldsets": !fieldsets.is_empty(), "fieldsets": fieldsets,
    }))?;
    let active = model.fields.iter().any(|field| field.active);
    let operations: Vec<_> = [
        "get", "detail", "search", "create", "update", "upsert", "batch", "remove", "restore",
        "delete",
    ]
    .into_iter()
    .filter(|name| active || !matches!(*name, "remove" | "restore"))
    .map(operation)
    .collect();
    // Each handler receives tiny metadata; the potentially large policy literal is not copied here.
    for operation in &operations {
        templates.append(text, "SQL operations-handler template failed", include_str!("templates/operations-handler.rs.tpl"), &json!({
            "model": model.rust_name, "model_literal": literal(&model.rust_name), "store": store, "operation": operation,
        }))?;
    }
    templates.append(text, "SQL operations-registration template failed", include_str!("templates/operations-registration.rs.tpl"), &json!({
        "active": active, "operations": operations,
        "mutable": model.fields.iter().any(|field| !field.key && !field.generated && !matches!(field.value_type, ValueType::Model(_))),
    }))
}
