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
    for model in graph.models() {
        if model.module == "catalog" {
            return Err(transform_error(super::invalid(
                "source catalog name collision",
            )));
        }
        let mut columns = Vec::new();
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            if field.multiple && !field.rules.is_empty() {
                return Err(transform_error(unsupported(
                    "source list-item assertion emission is not implemented",
                )));
            }
            if field
                .default
                .as_ref()
                .and_then(serde_json::Value::as_str)
                .is_some_and(unsupported_default_call)
            {
                return Err(transform_error(unsupported(
                    "source default call emission is not implemented for this provider",
                )));
            }
            // M1 implements the complete shipped assertion set; never approximate another
            // selected source assertion with the independent Typed validator's behavior.
            if field
                .rules
                .iter()
                .any(|rule| !matches!(rule.code.as_str(), "required" | "cge"))
            {
                return Err(transform_error(unsupported(
                    "source assertion emission is not implemented for this rule",
                )));
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
                "columns": columns, "witnesses": emit_source_witnesses(model).map_err(transform_error)?,
            }),
        )?;
        prepared.push((model.module.clone(), text));
    }
    let models: Vec<_> = graph
        .models()
        .iter()
        .map(|model| json!({"module": model.module}))
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
    output.write("source/catalog.rs", catalog)?;
    output.contribute_module("source", "catalog")
}

/// Recognize only the original generator's remaining built-in call spellings, not expressions.
fn unsupported_default_call(value: &str) -> bool {
    if value == "nanoid()" {
        return true;
    }
    ["cuid(", "nanoid("].iter().any(|prefix| {
        value
            .strip_prefix(prefix)
            .and_then(|value| value.strip_suffix(')'))
            .is_some_and(|length| {
                !length.is_empty() && length.bytes().all(|digit| digit.is_ascii_digit())
            })
    })
}

/// Select codecs from declarations/attributes, never from framework field names.
fn codec(field: &Field) -> SchemaResult<&'static str> {
    if field.multiple {
        return if field.value_type == ValueType::String {
            Ok("Strings")
        } else {
            Err(unsupported(
                "source list codec is not implemented for this type",
            ))
        };
    }
    Ok(match field.value_type {
        ValueType::String if field.attributes.iter().any(|a| a.name == "encrypted") => "Token",
        ValueType::String if field.attributes.iter().any(|a| a.name == "hashed") => "Secret",
        ValueType::String => "String",
        ValueType::Boolean => "Boolean",
        ValueType::Object => "Object",
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
