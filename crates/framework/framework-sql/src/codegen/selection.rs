//! Default selected projections have a distinct checked type and never widen field exposure.
use serde_json::json;
use stackpress_framework_schema::{Model, Schema, ValueType};
use stackpress_idea_transformer::TransformError;

use super::templates::{Templates, key_records, literals};
/// Compile approved default selection and its exact required relation prefixes before output publication.
pub(super) fn paths(
    schema: &Schema,
    model: &Model,
) -> stackpress_idea_transformer::Result<(Vec<String>, Vec<String>)> {
    let ui = schema
        .project_ui()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    let selected = ui
        .models
        .iter()
        .find(|item| item.id == model.name)
        .map(|item| item.default_selection.clone())
        .unwrap_or_default();
    let mut includes = std::collections::BTreeSet::new();
    for path in &selected {
        let mut fields = model.fields.as_slice();
        let mut prefix = Vec::new();
        for part in path.split('.') {
            let field = fields
                .iter()
                .find(|field| field.name == part)
                .ok_or_else(|| TransformError::new("selected field missing"))?;
            prefix.push(part);
            fields = match &field.value_type {
                ValueType::Model(name) => {
                    includes.insert(prefix.join("."));
                    schema
                        .models()
                        .iter()
                        .find(|model| model.name == *name)
                        .ok_or_else(|| TransformError::new("selected relation target missing"))?
                        .fields
                        .as_slice()
                }
                ValueType::Fieldset(name) => schema
                    .fieldsets()
                    .iter()
                    .find(|model| model.name == *name)
                    .ok_or_else(|| TransformError::new("selected fieldset missing"))?
                    .fields
                    .as_slice(),
                _ => &[],
            };
        }
    }
    let includes = includes
        .iter()
        .filter(|path| {
            !includes
                .iter()
                .any(|other| other.len() > path.len() && other.starts_with(&format!("{path}.")))
        })
        .cloned()
        .collect::<Vec<_>>();
    if includes.len() > 16 || includes.iter().any(|path| path.split('.').count() > 3) {
        return Err(TransformError::new(
            "default detail includes exceed the supported sixteen-path/depth-three budget",
        ));
    }
    Ok((selected, includes))
}
/// Emit one model-specific opaque projection type; only a checked public view can construct it.
pub(super) fn emit(
    schema: &Schema,
    model: &Model,
    templates: &Templates,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    // Path checking remains the same schema-owned step, before source rendering.
    let (selected, includes) = paths(schema, model)?;
    templates.append(text, "SQL selection template failed", include_str!("templates/selection.rs.tpl"), &json!({
        "model": model.rust_name, "selected": literals(selected.iter().map(String::as_str)),
        "includes": literals(includes.iter().map(String::as_str)), "fallback": model.query.is_empty(), "keys": key_records(model),
    }))
}
