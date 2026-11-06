//! Source prop, CSS, presence and native-name adaptation, independent of view components.
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::source::SourceAppearance;

/// Selected source argument, preserving fixed attributes from the schema registry.
pub(super) fn prop<'a>(context: &'a RoleContext, name: &str) -> Option<&'a Value> {
    context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role)
        .and_then(|binding| binding.props.get(name))
}

/// CSS-object keys use kebab case; values remain source-owned without admission policy.
pub(super) fn style(value: Option<&Value>) -> String {
    let Some(Value::Object(values)) = value else {
        return String::new();
    };
    values
        .iter()
        .map(|(name, value)| {
            let key = if name.starts_with("--") {
                name.clone()
            } else {
                name.chars()
                    .flat_map(|c| {
                        if c.is_ascii_uppercase() {
                            vec!['-', c.to_ascii_lowercase()]
                        } else {
                            vec![c]
                        }
                    })
                    .collect()
            };
            // React adds px for numeric non-unitless CSS properties, including zero without px.
            let unitless = matches!(
                name.as_str(),
                "opacity"
                    | "zIndex"
                    | "lineHeight"
                    | "fontWeight"
                    | "flex"
                    | "flexGrow"
                    | "flexShrink"
                    | "order"
                    | "zoom"
                    | "gridRow"
                    | "gridColumn"
                    | "fillOpacity"
                    | "strokeOpacity"
                    | "strokeWidth"
            );
            let text = super::metadata_string(value);
            let suffix = if value.is_number()
                && value.as_f64() != Some(0.0)
                && !unitless
                && !name.starts_with("--")
            {
                "px"
            } else {
                ""
            };
            format!("{key}:{text}{suffix}")
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// Ordinary classes and style object for one source component.
pub(super) fn appearance(context: &RoleContext) -> SourceAppearance {
    SourceAppearance {
        class_name: context.text_prop("className", ""),
        style: style(prop(context, "style")),
    }
}

/// Missing/null content is empty rather than Typed text's dash placeholder.
pub(super) fn content(context: &RoleContext) -> String {
    match context.value.get() {
        Presence::Missing | Presence::Null => String::new(),
        Presence::Value(value) => super::metadata_string(&value),
    }
}

/// Selected canonical component key, including source fixed attributes.
pub(super) fn component(context: &RoleContext) -> String {
    context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role)
        .map(|binding| binding.component.clone())
        .unwrap_or_default()
}

/// Original field/filter successful-name grammar, including repeated-array suffixes.
pub(super) fn form_name(context: &RoleContext) -> String {
    let base = context.field.wire_name.clone();
    let name = if context.role == stackpress_framework_schema::Role::Filter
        && use_context::<super::fieldset::SourceFormPrefix>().is_none()
    {
        format!("eq[{base}]")
    } else {
        base
    };
    if context.field.value_type.multiple {
        format!("{name}[]")
    } else {
        name
    }
}

/// Reuse the existing Rust text/input conversion for G55 structured code/JSON values.
/// Strings pass through; structured values become compact text without a parse/validation pass.
pub(super) fn input_content(context: &RoleContext) -> String {
    match context.value.get() {
        Presence::Missing | Presence::Null => String::new(),
        Presence::Value(value) => value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string()),
    }
}
