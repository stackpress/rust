//! Native list-role tables reuse the selected compiled cell adapters and public DTOs.
use super::emit::{control_source, public_type};
use super::templates::{Asset, Renderer};
use stackpress_framework_schema::{
    Role, SchemaResult, UiField, UiModel, codegen::DefinitionRegistry,
};

/// Emit a table only when a model has explicit public list selections.
pub(super) fn source(
    renderer: &mut Renderer,
    model: &UiModel,
    fields: &[UiField],
    definitions: &DefinitionRegistry,
) -> SchemaResult<String> {
    let selected: Vec<_> = fields
        .iter()
        .filter(|field| crate::draft::admitted(field, Role::List))
        .flat_map(|field| {
            field
                .roles
                .iter()
                .filter(|binding| binding.role == Role::List)
                .map(move |binding| (field, binding))
        })
        .collect();
    if selected.is_empty() {
        return Ok(String::new());
    }
    let public = public_type(&model.id)?;
    let module = stackpress_framework_schema::codegen::portable_name(&model.id)?;
    // Preserve source key order and encode Rust literals outside the template engine.
    let identity: Vec<_> = model
        .key
        .iter()
        .map(|name| super::templates::literal(name))
        .collect();
    // Resolve each selected adapter before serializing the minimal table context.
    let controls = selected
        .iter()
        .map(|(field, binding)| {
            control_source(renderer, field, binding, Role::List, definitions, false)
        })
        .collect::<SchemaResult<Vec<_>>>()?;
    let headers: Vec<_> = selected
        .into_iter()
        .map(|(field, binding)| {
            let label = binding
                .props
                .get("label")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(&field.label);
            serde_json::json!({"literal": super::templates::literal(label)})
        })
        .collect();
    renderer.render(
        Asset::Table,
        &serde_json::json!({
            "public_type": public, "controls": controls, "headers": headers,
            "module_literal": super::templates::literal(&module),
            "has_identity": !identity.is_empty(), "identity": identity,
            "caption_literal": super::templates::literal(&model.label),
        }),
    )
}
