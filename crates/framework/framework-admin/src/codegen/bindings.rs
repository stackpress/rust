//! Import/value compatibility and source diagnostics, independent of Rust source formatting.
use super::emit::role_name;
use stackpress_framework_schema::{
    Role, RoleBinding, SchemaError, SchemaErrorKind, SchemaResult, UiField, UiModel, ValueType,
    codegen::{ComponentImport, DefinitionRegistry, DiagnosticMap, portable_name},
};

/// Find the exact compiled import without interpreting component keys as Rust paths.
pub(super) fn import<'a>(
    binding: &RoleBinding,
    definitions: &'a DefinitionRegistry,
) -> SchemaResult<&'a ComponentImport> {
    if !binding.available {
        return Err(invalid("selected UI component is unavailable"));
    }
    let import = definitions
        .imports()
        .iter()
        .find(|import| import.component == binding.component)
        .ok_or_else(|| invalid("selected UI component has no compiled import"))?;
    // DefinitionRegistry checks lexical paths; this boundary also refuses Rust keywords and
    // default-export conventions, neither of which is a valid component function mapping.
    if import.default || !module_path(&import.module) || !identifier(&import.symbol) {
        return Err(invalid("invalid compiled UI component import"));
    }
    Ok(import)
}

/// Prove the native adapter's accepted value family instead of coercing JSON at runtime.
pub(super) fn validate_binding(
    field: &UiField,
    binding: &RoleBinding,
    definitions: &DefinitionRegistry,
) -> SchemaResult<()> {
    let import = import(binding, definitions)?;
    if import.module != "stackpress_framework_admin::components" {
        return Ok(());
    }
    let scalar = !field.value_type.multiple;
    let valid = match import.symbol.as_str() {
        "TextField" | "TextareaField" => scalar && field.value_type.value == ValueType::String,
        "SlugField" => scalar && field.value_type.value == ValueType::String,
        "TextListField" => !scalar && field.value_type.value == ValueType::String,
        "MetadataField" => scalar && field.value_type.value == ValueType::Object,
        "SliderField" => {
            let number = |name: &str, default| {
                binding
                    .props
                    .get(name)
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(default)
            };
            let min = number("min", 0.0);
            let max = number("max", 100.0);
            let step = number("step", 1.0);
            scalar
                && matches!(
                    field.value_type.value,
                    ValueType::Integer | ValueType::Number
                )
                && crate::components::range_scale(
                    min,
                    max,
                    step,
                    field.value_type.value == ValueType::Integer,
                )
                .is_ok()
        }
        "RatingField" => {
            let max = binding
                .props
                .get("max")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(5.0);
            scalar
                && field.value_type.value == ValueType::Integer
                && crate::components::rating_limit(max).is_ok()
        }
        "CheckboxField" | "SwitchField" | "BooleanView" => {
            scalar && field.value_type.value == ValueType::Boolean
        }
        "SelectField" | "ComboboxField" => {
            scalar
                // Native select has a placeholder; the current combobox API does not.
                // Refuse this combination instead of accepting a silently ignored prop.
                && !(binding.props.get("searchable").and_then(serde_json::Value::as_bool)==Some(true)
                    && binding.props.contains_key("placeholder"))
                && matches!(field.value_type.value, ValueType::Enum(_))
                && !field.value_type.choices.is_empty()
                && field.value_type.choices.len() <= 100
                && field.value_type.choices.iter().all(|choice| {
                    choice.as_str().is_some_and(|choice| {
                        choice.len() <= 1024 && choice != "__stackpress_unselected__"
                    })
                })
        }
        // Virtual relations are absent from Public<Model>; rendering them as empty would
        // hide a missing lookup capability. Embedded public fieldsets remain real data.
        "TextView" | "CaseView" => !matches!(field.value_type.value, ValueType::Model(_)),
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid(
            "UI component is incompatible with public field type",
        ))
    }
}

/// Retain the original alias attribute's span, falling back to its actual field path.
pub(super) fn locate(
    error: SchemaError,
    kind: &str,
    model: &UiModel,
    field: &UiField,
    role: Role,
    diagnostics: &DiagnosticMap,
) -> SchemaError {
    let mut path = vec![
        kind.into(),
        model.id.clone(),
        "columns".into(),
        field.wire_name.clone(),
    ];
    let prefix = format!("{}.", role_name(role));
    if let Some(entry) = diagnostics.entries.iter().find(|entry| {
        entry.path.starts_with(&path) && entry.path.len() == 6 && entry.path[5].starts_with(&prefix)
    }) {
        path = entry.path.clone();
    }
    error.at(path).with_diagnostics(diagnostics)
}

/// Require ordinary Rust identifiers and reject reserved words even after lexical checks.
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value != "_"
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_alphabetic() || byte == b'_' || index > 0 && byte.is_ascii_digit()
        })
        && portable_name(value).is_ok_and(|name| !name.ends_with('_') || value.ends_with('_'))
}

/// Only module-leading crate/self/super are meaningful path qualifiers.
fn module_path(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.split("::").enumerate().all(|(index, part)| {
            identifier(part) || index == 0 && matches!(part, "crate" | "self" | "super")
        })
}

/// Add one safe context without exposing rejected metadata.
fn invalid(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, context)
}
