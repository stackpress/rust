//! Native immutable catalogs and finite include-view types, separate from frozen pure DTOs.
use serde_json::{Value, json};
use stackpress_framework_schema::{Field, Model, Schema, ValueType};
use stackpress_idea_transformer::TransformError;

use super::templates::{Templates, column, literal, literals};
use super::{codec, is_scalar, store_module};

/// Resolve the exact owning/inverse scalar mapping already checked by schema normalization.
fn mapping<'a>(
    schema: &'a Schema,
    model: &'a Model,
    field: &'a Field,
) -> stackpress_idea_transformer::Result<(&'a Model, &'a Field, &'a Field)> {
    let ValueType::Model(name) = &field.value_type else {
        return Err(TransformError::new("relation field expected"));
    };
    let target = schema
        .models()
        .iter()
        .find(|model| model.name == *name)
        .ok_or_else(|| TransformError::new("relation target missing"))?;
    let (local, foreign) = if let Some(relation) = &field.relation {
        (relation.local.as_str(), relation.foreign.as_str())
    } else {
        let relation = target
            .fields
            .iter()
            .filter(|other| other.value_type == ValueType::Model(model.name.clone()))
            .filter_map(|other| other.relation.as_ref())
            .find(|relation| relation.inverse.as_deref() == Some(&field.name))
            .ok_or_else(|| TransformError::new("checked inverse relation missing"))?;
        (relation.foreign.as_str(), relation.local.as_str())
    };
    let local = model
        .fields
        .iter()
        .find(|field| field.name == local)
        .ok_or_else(|| TransformError::new("relation local scalar missing"))?;
    let foreign = target
        .fields
        .iter()
        .find(|field| field.name == foreign)
        .ok_or_else(|| TransformError::new("relation foreign scalar missing"))?;
    Ok((target, local, foreign))
}
/// Emit exact field access and normalized relations, without a mutable engine/identity cache.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    public: &stackpress_framework_schema::UiSchema,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Preserve field/relation checks before catalog rendering, then resolve finite views.
    let fields = query_fields(schema, model, stored, public)?;
    let relations = relation_specs(schema, model, public)?;
    let active = model.fields.iter().find(|field| field.active);
    templates.append(text, "SQL catalog template failed", include_str!("templates/catalog.rs.tpl"), &json!({
        "model": model.rust_name, "table_literal": literal(&model.module),
        "keys": literals(stored.iter().filter(|field| field.key).map(|field| field.name.as_str())),
        "has_active": active.is_some(), "active_literal": active.map(|field| literal(&field.name)),
        "fields": fields, "relations": relations,
        "bounded": model.fields.iter().any(|field| field.storage_length.is_some()),
    }))?;
    let depths = view_depths(schema, model)?;
    templates.append(
        text,
        "SQL views template failed",
        include_str!("templates/views.rs.tpl"),
        &json!({"model": model.rust_name, "depths": depths}),
    )
}

/// Build query/codec flags from stored fields and the exact approved public model.
fn query_fields(
    schema: &Schema,
    model: &Model,
    stored: &[&Field],
    public: &stackpress_framework_schema::UiSchema,
) -> stackpress_idea_transformer::Result<Vec<Value>> {
    let public_model = public.models.iter().find(|item| item.id == model.name);
    // Serialize enum choices before interpolation, retaining the original serialization failures.
    let mut fields = Vec::new();
    for field in stored.iter().filter(|field| is_scalar(field)) {
        let choices = if let ValueType::Enum(name) = &field.value_type {
            schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .map(|item| {
                    item.variants
                        .iter()
                        .map(|(_, value)| value.clone())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        } else {
            Vec::new()
        };
        let choices = serde_json::to_string(&choices)
            .map_err(|cause| TransformError::from(std::io::Error::other(cause)))?;
        let approved = public_model.and_then(|model| {
            model
                .fields
                .iter()
                .find(|item| item.wire_name == field.name)
        });
        let mut scalar_field = (*field).clone();
        scalar_field.multiple = false;
        // Start with the same column atoms; this known object receives only scalar metadata.
        let mut data = column(schema, field);
        data["multiple"] = json!(field.multiple);
        data["scalar_codec"] = json!(codec(schema, &scalar_field));
        data["choices_literal"] = json!(literal(&choices));
        data["searchable"] = json!(field.searchable && field.value_type == ValueType::String);
        data["sortable"] = json!(field.sortable || field.key);
        data["public_filter"] = json!(approved.is_some_and(|field| field.filterable));
        data["public_sort"] = json!(approved.is_some_and(|field| field.sortable));
        data["public_search"] = json!(approved.is_some_and(|field| field.searchable));
        fields.push(data);
    }
    Ok(fields)
}

/// Resolve catalog relation atoms independently of finite-view type construction.
fn relation_specs(
    schema: &Schema,
    model: &Model,
    public: &stackpress_framework_schema::UiSchema,
) -> stackpress_idea_transformer::Result<Vec<Value>> {
    let public_model = public.models.iter().find(|item| item.id == model.name);
    let mut relations = Vec::new();
    for field in model
        .fields
        .iter()
        .filter(|field| matches!(field.value_type, ValueType::Model(_)))
    {
        let (target, local, foreign) = mapping(schema, model, field)?;
        relations.push(json!({
            "name_literal": literal(&field.name), "target_literal": literal(&target.module),
            "local_literal": literal(&local.name), "foreign_literal": literal(&foreign.name),
            "many": field.multiple, "required": field.required,
            "public": public_model.is_some_and(|model| model.fields.iter().any(|item| item.wire_name == field.name && item.readable)),
        }));
    }
    Ok(relations)
}

/// Prepare four bounded depth groups; templates own all path and container syntax.
fn view_depths(schema: &Schema, model: &Model) -> stackpress_idea_transformer::Result<Vec<Value>> {
    // Finite types keep the original depth order and trusted/public alternation.
    let mut depths = Vec::new();
    for depth in 0usize..=3 {
        let mut views = Vec::new();
        for is_public in [false, true] {
            let mut fields = Vec::new();
            if depth > 0 {
                for field in model.fields.iter().filter(|field| {
                    matches!(field.value_type, ValueType::Model(_))
                        && (!is_public || field.access.readable && !field.access.sensitive)
                }) {
                    let (target, _, _) = mapping(schema, model, field)?;
                    fields.push(json!({
                        "name_literal": literal(&field.name), "relation_literal": literal(crate::stores::relation_member(&field.name)),
                        "member": view_field(field), "store": store_module(target), "multiple": field.multiple, "optional": !field.required,
                    }));
                }
            }
            views.push(json!({
                "public": is_public, "prefix": if is_public { "Public" } else { "" },
                "audience": if is_public { "projected" } else { "trusted" },
                "row_description": if is_public { "approved public" } else { "stored" }, "fields": fields,
            }));
        }
        let projection_fields = if depth > 0 {
            model.fields.iter().filter(|field| matches!(field.value_type, ValueType::Model(_)) && field.access.readable && !field.access.sensitive)
                .map(|field| json!({"member": view_field(field), "multiple": field.multiple, "optional": !field.required})).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        depths.push(json!({"value": depth, "previous": depth.saturating_sub(1), "views": views, "projection_fields": projection_fields}));
    }
    Ok(depths)
}

/// The base record owns `row`; an underscore-prefixed Rust member cannot collide with source names.
fn view_field(field: &Field) -> &str {
    if field.rust_name == "row" {
        "_relation_row"
    } else {
        &field.rust_name
    }
}
