//! Emit finite per-model query selectors and values; execution remains a separate milestone.

use super::{projection, rust_type, source, transform_error, type_name};
use crate::{Model, Schema, ValueType};
use serde_json::json;
use stackpress_idea_transformer::Result;

/// Prepare exact typed selectors; the template owns query declarations and validation control flow.
pub(super) fn emit(schema: &Schema, model: &Model) -> Result<String> {
    let filters: Vec<_> = model
        .fields
        .iter()
        .filter(|field| projection::filterable(field))
        .collect();
    let mut predicates = Vec::new();
    for field in &filters {
        let mut scalar = (*field).clone();
        scalar.required = true;
        scalar.multiple = false;
        let ty = rust_type(schema, &scalar).map_err(transform_error)?;
        let variant = type_name(&field.name).map_err(transform_error)?;
        predicates
            .push(json!({"wire":source::string_literal(&field.name),"variant":variant,"ty":ty}));
    }
    // Preserve declared predicate order and lower an enum to its checked scalar kind.
    for (field, predicate) in filters.iter().zip(&mut predicates) {
        let mut kind = field.value_type.clone();
        if let ValueType::Enum(name) = &kind {
            kind = schema
                .enums()
                .iter()
                .find(|item| item.name == *name)
                .and_then(|item| item.variants.first())
                .map(|(_, value)| {
                    if value.is_string() {
                        ValueType::String
                    } else if value.is_boolean() {
                        ValueType::Boolean
                    } else {
                        ValueType::Integer
                    }
                })
                .expect("checked enum");
        }
        predicate["kind"] = json!(source::string_literal(
            &serde_json::to_string(&kind).expect("scalar kind serialization")
        ));
        predicate["multiple"] = json!(field.multiple);
        predicate["nullable"] = json!(!field.required);
    }
    let mut sorts = Vec::new();
    for field in model
        .fields
        .iter()
        .filter(|field| field.access.readable && !field.access.sensitive && field.sortable)
    {
        sorts.push(json!({"wire":source::string_literal(&field.name),"variant":type_name(&field.name).map_err(transform_error)?}));
    }
    let searchable = model.fields.iter().any(|field| {
        field.access.readable
            && !field.access.sensitive
            && field.searchable
            && field.value_type == ValueType::String
            && !field.multiple
    });
    let ui = serde_json::to_string(&schema.project_ui().map_err(transform_error)?)
        .expect("checked UI schema serialization");
    source::generate(
        "templates/query.rs.tpl",
        include_str!("templates/query.rs.tpl"),
        &json!({
            "name":model.rust_name,"model":source::string_literal(&model.name),"filters":predicates,
            "has_filters":!filters.is_empty(),"sorts":sorts,"searchable":searchable,"ui":source::string_literal(&ui),
        }),
    )
}
