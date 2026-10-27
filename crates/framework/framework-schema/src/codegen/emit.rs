//! Pure deterministic contract emission into the schema-owned output partitions.

use super::{invalid, transform_error};
use crate::{Field, Schema, SchemaResult, ValueType};
use stackpress_idea_transformer::OutputPlan;

/// Resolve one field's Rust type without introducing native dependencies into contracts.
pub fn rust_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut name = match &field.value_type {
        ValueType::String => "String".into(),
        ValueType::Integer => "i64".into(),
        ValueType::Boolean => "bool".into(),
        ValueType::Number => "f64".into(),
        ValueType::Object => "serde_json::Map<String, serde_json::Value>".into(),
        ValueType::Enum(name) => {
            let item = schema
                .enums
                .iter()
                .find(|item| &item.name == name)
                .ok_or_else(|| invalid("unknown enum"))?;
            format!("super::{}::{}", item.module, item.rust_name)
        }
        ValueType::Fieldset(name) => {
            let item = schema
                .fieldsets
                .iter()
                .find(|item| &item.name == name)
                .ok_or_else(|| invalid("unknown fieldset"))?;
            format!("super::{}::{}", item.module, item.rust_name)
        }
        ValueType::Model(_) => return Err(invalid("virtual relation has no scalar Rust type")),
    };
    if field.multiple {
        name = format!("Vec<{name}>");
    }
    if !field.required {
        name = format!("Option<{name}>");
    }
    Ok(name)
}

/// Emit owned contract structs and scalar enums into this transform's exclusive partition.
pub fn emit_contracts(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    for model in schema.fieldsets.iter().chain(&schema.models) {
        let mut text = format!(
            "//! Owned {} contract generated from finalized Idea.\n\n/// Checked-shape owned values; no database handle or lazy relation.\n#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct {} {{\n",
            model.rust_name, model.rust_name
        );
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            let value_type = rust_type(schema, field).map_err(transform_error)?;
            text.push_str(&format!(
                "/// Declared stored value.\n#[serde(rename = {:?})]\npub {}: {},\n",
                field.name, field.rust_name, value_type
            ));
        }
        text.push_str("}\n");
        text.push_str(&format!("impl {} {{\n/// Validate finite owned values before storage or projection.\npub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {{\n", model.rust_name));
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            let access = format!("&self.{}", field.rust_name);
            if !field.required {
                text.push_str(&format!("if let Some(value) = {} {{\n", access));
            } else {
                text.push_str(&format!("{{ let value = {};\n", access));
            }
            if field.multiple {
                text.push_str("// Lists have an explicit finite element budget.\nif value.len() > 1024 { return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Limit, \"list element budget exceeded\")); }\nfor value in value {\n");
            }
            match field.value_type {
                ValueType::String => {
                    text.push_str(&format!("// Text is unchanged UTF-8; size and declared presence are checked.\nif value.len() > 1024 * 1024 {} {{ return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid, \"text validation failed\")); }}\n", if field.nonempty { "|| value.is_empty()" } else { "" }));
                }
                ValueType::Fieldset(_) => text.push_str("// Embedded validation uses the checked acyclic schema graph.\nvalue.validate()?;\n"),
                _ => text.push_str("let _ = value;\n"),
            }
            if field.multiple {
                text.push_str("}\n");
            }
            text.push_str("}\n");
        }
        text.push_str("Ok(())\n}\n}\n");
        if schema.models.iter().any(|item| item.name == model.name) {
            text.push_str(&format!("/// Create input preserving omitted, supplied and null values.\n#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct Create{} {{\n", model.rust_name));
            for field in &model.fields {
                if matches!(field.value_type, ValueType::Model(_)) {
                    continue;
                }
                let mut required = field.clone();
                required.required = true;
                let value_type = rust_type(schema, &required).map_err(transform_error)?;
                text.push_str(&format!("/// Explicit create value or omission.\n#[serde(rename = {:?}, default, skip_serializing_if = \"stackpress_framework_schema::Input::is_missing\")]\npub {}: stackpress_framework_schema::Input<{}>,\n", field.name, field.rust_name, value_type));
            }
            text.push_str("}\n");
            text.push_str(&format!("impl Create{} {{\n/// Prepare literal defaults once; reject missing/null required fields.\npub fn prepare(self) -> stackpress_framework_schema::SchemaResult<{}> {{\nuse stackpress_framework_schema::{{Input, SchemaError, SchemaErrorKind}};\nlet value = {} {{\n", model.rust_name, model.rust_name, model.rust_name));
            for field in &model.fields {
                if matches!(field.value_type, ValueType::Model(_)) {
                    continue;
                }
                let missing = match &field.default {
                    Some(value) => format!("serde_json::from_str({:?}).map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, \"default decoding failed\").with_cause(cause))?", value.to_string()),
                    None if !field.required => "None".into(),
                    None => "return Err(SchemaError::new(SchemaErrorKind::Invalid, \"required create field is missing\"))".into(),
                };
                let null = if field.required {
                    "return Err(SchemaError::new(SchemaErrorKind::Invalid, \"required create field is null\"))"
                } else {
                    "None"
                };
                let supplied = if field.required {
                    "value"
                } else {
                    "Some(value)"
                };
                text.push_str(&format!("// Omission consults only the declared literal default.\n{}: match self.{} {{ Input::Missing => {}, Input::Null => {}, Input::Value(value) => {} }},\n", field.rust_name, field.rust_name, missing, null, supplied));
            }
            text.push_str("};\n// Validate nested values and finite collections before any IO.\nvalue.validate()?;\nOk(value)\n}\n}\n");
        }
        output.write(format!("contracts/{}.rs", model.module), text)?;
        output.contribute_module("contracts", &model.module)?;
    }
    for enumeration in &schema.enums {
        let mut text = format!(
            "//! Scalar enumeration generated from Idea.\n\n/// Checked finite wire choices.\n#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]\npub enum {} {{\n",
            enumeration.rust_name
        );
        for (variant, value) in &enumeration.variants {
            text.push_str(&format!(
                "/// Declared wire choice.\n#[serde(rename = {value:?})]\n{variant},\n"
            ));
        }
        text.push_str("}\n");
        output.write(format!("contracts/{}.rs", enumeration.module), text)?;
        output.contribute_module("contracts", &enumeration.module)?;
    }
    output.write("metadata/schema.rs", format!("//! Compatibility identity shared by generated partitions.\n/// Normalized semantic fingerprint.\npub const FINGERPRINT: &str = {:?};\n/// Required runtime descriptor revision.\npub const CONTRACT_REVISION: u32 = {};\nconst _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);\n", schema.fingerprint().map_err(transform_error)?, crate::CONTRACT_REVISION))?;
    output.contribute_module("metadata", "schema")
}
