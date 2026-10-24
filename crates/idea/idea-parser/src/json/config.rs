//! Ordered normalized-schema validation after bounded JSON decoding.
//!
//! Structural fields are closed and metadata stays open. This module converts validated
//! data without source loading, environment evaluation or executable plugin activation.

use indexmap::IndexMap;
use serde_json::{Map, Value};
use std::collections::HashSet;

use super::validation;
use crate::parser::{
    is_any_identifier, is_camel_identifier, is_capital_identifier, is_upper_identifier,
    valid_attribute,
};
use crate::{AttributeValue, ColumnConfig, ErrorCode, IdeaError, SchemaConfig, TypeConfig};

/// Validate known sections and convert normalized values without interpreting metadata.
pub(crate) fn schema_from_value(value: &Value) -> Result<SchemaConfig, IdeaError> {
    // Structural sections are closed; plugin/prop/attribute payloads remain open data.
    let root = object(value)?;
    fields(root, &["use", "plugin", "prop", "enum", "type", "model"])?;
    let mut schema = SchemaConfig::default();
    let mut names = HashSet::new();
    for (section, value) in root {
        if section == "use" {
            // Import order is preserved and no source is loaded by this data-only boundary.
            schema.r#use = value
                .as_array()
                .ok_or_else(|| validation("Expected import array"))?
                .iter()
                .map(|value| string(value).map(Into::into))
                .collect::<Result<_, _>>()?;
            continue;
        }
        for (name, value) in object(value)? {
            // Cross-kind name collisions are errors, even if both payloads are valid.
            if !names.insert(name) {
                return Err(IdeaError::new(
                    ErrorCode::DuplicateName,
                    "Duplicate declaration",
                ));
            }
            if section != "plugin" && !is_capital_identifier(name) {
                return Err(validation("Invalid declaration name"));
            }
            match section.as_str() {
                "type" => {
                    // Reuse the same fieldset validation for models and reusable types.
                    schema.r#type.insert(name.clone(), fieldset(name, value)?);
                }
                "model" => {
                    schema.model.insert(name.clone(), fieldset(name, value)?);
                }
                "plugin" | "prop" | "enum" => {
                    // Preserve ordinary data verbatim after validating the member-name grammar.
                    let mut data = IndexMap::new();
                    for (key, value) in object(value)? {
                        if !is_any_identifier(key)
                            || (section == "enum" && !is_upper_identifier(key))
                        {
                            return Err(validation("Invalid member name"));
                        }
                        if section == "enum" && matches!(value, Value::Array(_) | Value::Object(_))
                        {
                            return Err(validation("Enum values must be scalar"));
                        }
                        data.insert(key.clone(), value.clone());
                    }
                    match section.as_str() {
                        "plugin" => {
                            schema.plugin.insert(name.clone(), data);
                        }
                        "prop" => {
                            schema.prop.insert(name.clone(), data);
                        }
                        _ => {
                            schema.r#enum.insert(name.clone(), data);
                        }
                    }
                }
                _ => return Err(validation("Unknown schema section")),
            }
        }
    }
    Ok(schema)
}

/// Validate a model/type's exact normalized fields and ordered unique columns.
pub(super) fn fieldset(name: &str, value: &Value) -> Result<TypeConfig, IdeaError> {
    // Required normalized fields cannot be defaulted without hiding malformed imports.
    let value = object(value)?;
    fields(value, &["name", "mutable", "attributes", "columns"])?;
    if string(required(value, "name")?)? != name {
        return Err(validation("Declaration name mismatch"));
    }
    let mutable = required(value, "mutable")?
        .as_bool()
        .ok_or_else(|| validation("Expected mutable boolean"))?;
    let attributes = attributes(required(value, "attributes")?)?;
    let mut columns = Vec::new();
    let mut names = HashSet::new();
    for column in required(value, "columns")?
        .as_array()
        .ok_or_else(|| validation("Expected columns array"))?
    {
        // Columns are ordered arrays with unique names; their types are already normalized.
        let column = object(column)?;
        fields(
            column,
            &["name", "type", "required", "multiple", "attributes"],
        )?;
        let name = string(required(column, "name")?)?;
        if !is_camel_identifier(name) {
            return Err(validation("Invalid column name"));
        }
        if !names.insert(name) {
            return Err(IdeaError::new(ErrorCode::DuplicateName, "Duplicate column"));
        }
        let column_type = string(required(column, "type")?)?;
        if !is_capital_identifier(column_type) {
            return Err(validation("Expected unmodified base type"));
        }
        // Preserve caller-supplied flags only after verifying each is a real JSON boolean.
        columns.push(ColumnConfig {
            name: name.into(),
            column_type: column_type.into(),
            attributes: attributes_from_column(column)?,
            required: required(column, "required")?
                .as_bool()
                .ok_or_else(|| validation("Expected required boolean"))?,
            multiple: required(column, "multiple")?
                .as_bool()
                .ok_or_else(|| validation("Expected multiple boolean"))?,
        });
    }
    Ok(TypeConfig {
        name: name.into(),
        mutable,
        attributes,
        columns,
    })
}

/// Extract column metadata without shadowing the parent fieldset's metadata binding.
fn attributes_from_column(
    column: &Map<String, Value>,
) -> Result<IndexMap<String, AttributeValue>, IdeaError> {
    attributes(required(column, "attributes")?)
}

/// Require lowercase metadata namespaces and boolean/argument-array values.
fn attributes(value: &Value) -> Result<IndexMap<String, AttributeValue>, IdeaError> {
    let mut result = IndexMap::new();
    for (name, value) in object(value)? {
        if !valid_attribute(name) {
            return Err(validation("Invalid attribute namespace"));
        }
        // Flags and method arguments are the only normalized metadata shapes.
        let attribute = match value {
            Value::Bool(value) => AttributeValue::Boolean(*value),
            Value::Array(values) => AttributeValue::Array(values.clone()),
            _ => return Err(validation("Expected attribute boolean or argument array")),
        };
        result.insert(name.clone(), attribute);
    }
    Ok(result)
}

/// Require an object at a normalized structural boundary.
pub(super) fn object(value: &Value) -> Result<&Map<String, Value>, IdeaError> {
    value
        .as_object()
        .ok_or_else(|| validation("Expected normalized object"))
}

/// Require a string without coercing other scalar types.
fn string(value: &Value) -> Result<&str, IdeaError> {
    value.as_str().ok_or_else(|| validation("Expected string"))
}

/// Require a known structural field; metadata objects themselves stay open.
pub(super) fn required<'a>(
    object: &'a Map<String, Value>,
    key: &str,
) -> Result<&'a Value, IdeaError> {
    object
        .get(key)
        .ok_or_else(|| IdeaError::new(ErrorCode::MissingProperty, "Missing normalized property"))
}

/// Reject unknown normalized fields rather than silently dropping incorrect input.
pub(super) fn fields(object: &Map<String, Value>, allowed: &[&str]) -> Result<(), IdeaError> {
    if object.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(validation("Unknown normalized property"));
    }
    Ok(())
}
