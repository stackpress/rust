//! Native list-role tables reuse the selected compiled cell adapters and public DTOs.
use super::emit::{control_source, public_type};
use stackpress_framework_schema::{
    Role, SchemaResult, UiField, UiModel, codegen::DefinitionRegistry,
};

/// Emit a table only when a model has explicit public list selections.
pub(super) fn source(
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
    let mut source = format!(
        "/// Render at most 100 public rows through selected list adapters.\n/// Caller-supplied unique nonempty keys (at most 128 bytes) retain row identity.\n/// Invalid row identities fail before constructing any controls.\npub fn table(rows: &[(String,super::super::contracts::public::{public})], instance: &str) -> SchemaResult<AnyView> {{\nuse stackpress_leptos::components::{{Table,TableHead,TableGroup,TableRow,TableHeader,TableCell}};\nlet mut keys = std::collections::BTreeSet::new();\nif rows.len() > 100 || rows.iter().any(|(key,_)| key.is_empty() || key.len() > 128 || !keys.insert(key)) {{ return Err(SchemaError::new(SchemaErrorKind::Invalid, \"invalid public table row identities\")); }}\nlet fields = descriptors()?;\nlet mut rendered = Vec::new();\nfor (key,value) in rows {{\nlet seed = stackpress_framework_schema::to_bounded_value(value)?;\nlet draft = Draft::new(&fields,&seed,Role::List)?;\n// Hex encoding is injective and cannot collide with role/field delimiters.\nlet identity: String = key.as_bytes().iter().map(|byte| format!(\"{{byte:02x}}\")).collect();\nlet instance = format!(\"{{instance}}-row-{{identity}}\");\nlet mut controls = Vec::new();\n"
    );
    for (field, binding) in &selected {
        source.push_str(&control_source(
            field,
            binding,
            Role::List,
            definitions,
            false,
        )?);
    }
    source.push_str("let cells = controls.into_iter().map(|control| view! { <TableCell>{control}</TableCell> }).collect_view();\nrendered.push(view! { <TableRow row_key=key.clone()>{cells}</TableRow> });\n}\nlet headers = vec![\n");
    for (field, binding) in selected {
        let label = binding
            .props
            .get("label")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(&field.label);
        source.push_str(&format!(
            "view! {{ <TableHeader>{{{label:?}}}</TableHeader> }},\n"
        ));
    }
    source.push_str(&format!("].collect_view();\nlet rows = rendered.collect_view();\nOk(view! {{ <Table caption={:?}><TableHead><TableRow row_key=\"header\">{{headers}}</TableRow></TableHead><TableGroup>{{rows}}</TableGroup></Table> }}.into_any())\n}}\n",model.label));
    Ok(source)
}
