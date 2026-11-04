//! Source-native schema emission; Typed contracts and their templates stay independent.
use super::{SourceGraph, emit_source_witnesses, source, transform_error, unsupported};
use crate::{Field, SchemaResult, ValueType};
use serde_json::json;
use stackpress_idea_transformer::{OutputPlan, Result};

/// Emit complete native schema/columns, witnesses and the private source catalog.
///
/// `SourceGraph` comes from `Schema::normalize_source`, never from a Typed projection.
/// The caller's transform must own `source`. Generated modules depend on the schema
/// `source` runtime, not on plugins, Idea, renderers or application IO. Stores/actions
/// and DDL execution are a separate emission stage. No public/browser DTO is implied.
///
/// # Errors
/// Refuses unsupported source codec shapes before contributing any files. Planning
/// ownership/budget errors propagate through the existing transformer writer.
pub fn emit_source_contracts(graph: &SourceGraph, output: &mut OutputPlan) -> Result<()> {
    // Prepare the complete inventory before touching the plan: no early model masks a
    // later unsupported declaration. Metadata values enter only encoded Rust literals.
    let metadata = graph.catalog().map_err(transform_error)?;
    let mut prepared = Vec::new();
    let mut catalog_module = "catalog".to_owned();
    while graph
        .models()
        .iter()
        .chain(graph.fieldsets())
        .any(|model| model.module == catalog_module)
    {
        catalog_module.insert(0, '_');
    }
    for model in graph.models().iter().chain(graph.fieldsets()) {
        let mut columns = Vec::new();
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            columns.push(json!({
                "name": source::string_literal(&field.name),
                "sql": source::string_literal(&field.column),
                "kind": codec(field).map_err(transform_error)?,
                "nullable": !field.required && !field.multiple,
            }));
        }
        let metadata = serde_json::to_string(model).map_err(encoding_error)?;
        let text = source::generate(
            "templates/source-model.rs.tpl",
            include_str!("templates/source-model.rs.tpl"),
            &json!({
                "name": model.rust_name, "model": source::string_literal(&metadata),
                "catalog_module":catalog_module, "columns": columns, "witnesses": emit_source_witnesses(model).map_err(transform_error)?,
            }),
        )?;
        prepared.push((source_module(model).to_owned(), text));
    }
    let models: Vec<_> = graph
        .models()
        .iter()
        .map(|model| json!({"module":source_module(model)}))
        .collect();
    let catalog = source::generate(
        "templates/source-catalog.rs.tpl",
        include_str!("templates/source-catalog.rs.tpl"),
        &json!({
            "models": models,
            "externals": graph.external_models().iter().map(|name| source::string_literal(name)).collect::<Vec<_>>(),
            "catalog": source::string_literal(&serde_json::to_string(&metadata).map_err(encoding_error)?),
        }),
    )?;
    // The existing plan/writer owns module indexes, path validation and atomic installation.
    for (module, text) in prepared {
        output.write(format!("source/{module}.rs"), text)?;
        output.contribute_module("source", &module)?;
    }
    output.write(format!("source/{catalog_module}.rs"), catalog)?;
    output.contribute_module("source", &catalog_module)
}

/// Select codecs from declarations/attributes, never from framework field names.
fn codec(field: &Field) -> SchemaResult<&'static str> {
    if field.multiple && field.value_type == ValueType::String {
        return Ok("Strings");
    }
    Ok(match field.value_type {
        ValueType::String if field.attributes.iter().any(|a| a.name == "encrypted") => "Token",
        ValueType::String if field.attributes.iter().any(|a| a.name == "hashed") => "Secret",
        ValueType::Unknown
        | ValueType::String
        | ValueType::Enum(_)
        | ValueType::Integer
        | ValueType::Number => "String",
        ValueType::Boolean => "Boolean",
        ValueType::Object | ValueType::Fieldset(_) => "Object",
        ValueType::Date | ValueType::Time | ValueType::Datetime => "Date",
        _ => {
            return Err(unsupported(
                "source scalar codec is not implemented for this type",
            ));
        }
    })
}

/// Preserve a structured cause when encoding a trusted normalized descriptor fails.
fn encoding_error(cause: serde_json::Error) -> stackpress_idea_transformer::TransformError {
    transform_error(
        crate::SchemaError::new(
            crate::SchemaErrorKind::Codec,
            "source descriptor encoding failed",
        )
        .with_cause(cause),
    )
}

/// Generated Rust module identity, independent of the SQL table spelling.
/// A model named Source cannot produce source::source under warning-clean app builds.
pub fn source_module(model: &crate::Model) -> &str {
    if model.module == "source" {
        "_source"
    } else {
        &model.module
    }
}
