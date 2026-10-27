//! Opt-in concrete formatted definitions; their data never carries executable identities.
use super::catalog::{Component, prop};
use stackpress_framework_schema::codegen::{ComponentDefinition, ComponentImport, PropType};

/// Keep the same schema vocabulary and let filter/span/list derive through the frozen registry.
pub(super) fn components() -> Vec<Component> {
    [
        ("field.number", "DecimalField"),
        ("field.price", "DecimalField"),
        ("field.integer", "DecimalField"),
        ("field.date", "TemporalField"),
        ("field.time", "TemporalField"),
        ("field.datetime", "TemporalField"),
        ("field.phone", "PhoneField"),
        ("view.number", "FormattedView"),
        ("view.price", "FormattedView"),
        ("view.date", "FormattedView"),
        ("view.phone", "FormattedView"),
    ]
    .into_iter()
    .map(|(name, symbol)| {
        let mut props = vec![prop("locale", PropType::String)];
        if name.starts_with("field.") {
            props.extend([
                prop("label", PropType::String),
                prop("disabled", PropType::Boolean),
            ]);
        } else {
            props.push(prop("null_label", PropType::String));
        }
        if matches!(
            name,
            "field.number" | "field.price" | "field.integer" | "view.number" | "view.price"
        ) {
            props.extend([
                prop("scale", PropType::Number),
                prop("digits", PropType::Number),
            ]);
        }
        if name == "field.phone" {
            let mut region = prop("region", PropType::String);
            region.required = true;
            props.push(region);
        }
        if matches!(name, "field.date" | "field.time" | "field.datetime") {
            props.push(prop("precision", PropType::Number));
        }
        let key = format!("stackpress.ui.{name}");
        Component {
            definition: ComponentDefinition {
                name: name.into(),
                key: key.clone(),
                revision: "1".into(),
                props,
                fixed: Default::default(),
                available: true,
            },
            import: ComponentImport {
                component: key,
                module: "stackpress_framework_admin::components".into(),
                symbol: symbol.into(),
                default: false,
            },
        }
    })
    .collect()
}
