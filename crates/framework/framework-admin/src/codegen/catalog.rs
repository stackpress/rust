//! Finite implemented role catalog; absent controls remain explicitly unavailable.

use std::collections::{BTreeMap, BTreeSet};

use stackpress_framework_schema::{
    SchemaError, SchemaErrorKind, SchemaResult,
    codegen::{ComponentDefinition, ComponentImport, DefinitionRegistry, PropDefinition, PropType},
};

/// One app-selected compiled adapter and its checked prop signature.
#[derive(Clone, Debug)]
pub struct Component {
    /// Canonical schema definition; aliases and derived roles stay schema-owned.
    pub definition: ComponentDefinition,
    /// Tool-only Rust identity, never copied into a public descriptor.
    pub import: ComponentImport,
}

/// Return concrete adapters supplied by this crate; unsupported vocabulary stays unavailable.
pub fn components() -> Vec<Component> {
    let mut result = Vec::new();
    for (name, symbol) in [
        ("field.input", "TextField"),
        ("field.textarea", "TextareaField"),
        ("field.checkbox", "CheckboxField"),
        ("field.switch", "SwitchField"),
        ("field.select", "SelectField"),
        ("field.stringlist", "TextListField"),
        ("field.textlist", "TextListField"),
        ("field.tags", "TextListField"),
        ("field.metadata", "MetadataField"),
        ("field.slider", "SliderField"),
        ("field.rating", "RatingField"),
        ("field.slug", "SlugField"),
        ("view.text", "TextView"),
        ("view.yesno", "BooleanView"),
        ("view.uppercase", "CaseView"),
        ("view.lowercase", "CaseView"),
        ("view.capitalize", "CaseView"),
    ] {
        let key = format!("stackpress.ui.{name}");
        let mut props = Vec::new();
        let mut fixed = BTreeMap::new();
        if name.starts_with("field.") {
            props.extend([
                prop("label", PropType::String),
                prop("disabled", PropType::Boolean),
            ]);
            if matches!(symbol, "TextField" | "SelectField") {
                props.push(prop("placeholder", PropType::String));
            }
            if matches!(symbol, "TextField" | "TextareaField") {
                props.push(prop("read_only", PropType::Boolean));
            }
            if symbol == "SelectField" {
                props.push(prop("searchable", PropType::Boolean));
            }
            if symbol == "TextListField" {
                props.push(prop("tags", PropType::Boolean));
                fixed.insert("tags".into(), (name == "field.tags").into());
            }
            if symbol == "SliderField" {
                props.extend([
                    prop("min", PropType::Number),
                    prop("max", PropType::Number),
                    prop("step", PropType::Number),
                ]);
            }
            if symbol == "RatingField" {
                props.push(prop("max", PropType::Number));
            }
        } else {
            props.push(prop("null_label", PropType::String));
            if name == "view.yesno" {
                props.extend([
                    prop("yes_label", PropType::String),
                    prop("no_label", PropType::String),
                ]);
            } else if symbol == "CaseView" {
                props.push(prop("case", PropType::String));
                fixed.insert("case".into(), name[5..].into());
            }
        }
        result.push(Component {
            definition: ComponentDefinition {
                name: name.into(),
                key: key.clone(),
                revision: "1".into(),
                props,
                fixed,
                available: true,
            },
            import: ComponentImport {
                component: key,
                module: "stackpress_framework_admin::components".into(),
                symbol: symbol.into(),
                default: false,
            },
        });
    }
    #[cfg(feature = "format")]
    result.extend(super::formatted::components());
    result
}

/// Register framework definitions with app replacements chosen before normalization.
///
/// Replacements may use any compiled adapter implementing the `RoleContext` interface.
/// # Errors
/// Rejects duplicate replacements or invalid schema definitions/imports. No partial registry
/// escapes because registration consumes and returns the app-owned value.
pub fn register_components(
    mut registry: DefinitionRegistry,
    replacements: Vec<Component>,
) -> SchemaResult<DefinitionRegistry> {
    let mut names = BTreeSet::new();
    for component in &replacements {
        if !names.insert(component.definition.name.clone()) {
            return Err(SchemaError::new(
                SchemaErrorKind::Invalid,
                "duplicate UI replacement",
            ));
        }
    }
    // Choose the app mapping first; never rely on last-writer-wins registration.
    for component in components()
        .into_iter()
        .filter(|component| !names.contains(&component.definition.name))
        .chain(replacements)
    {
        registry = registry.with_component(component.definition, Some(component.import))?;
    }
    Ok(registry)
}

/// Build one optional, public literal argument in stable positional order.
pub(super) fn prop(name: &str, value_type: PropType) -> PropDefinition {
    PropDefinition {
        name: name.into(),
        value_type,
        required: false,
        public: true,
    }
}
