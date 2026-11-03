//! Source presentation argument definitions, without choosing a renderer or publishing values.
use super::{ComponentDefinition, DefinitionRegistry, PropDefinition, PropType};
use crate::SchemaResult;

/// Complete source role vocabulary plus the original shipped argument shapes.
///
/// Definitions are metadata-only (`available = false`). Native source descriptors retain
/// their full props; renderer selection and browser projection remain separate operations.
/// The Typed catalog is unchanged. Aliases/derived families use the existing resolver.
pub fn source_definitions() -> SchemaResult<DefinitionRegistry> {
    let mut definitions = DefinitionRegistry::standard();
    // These names describe source props rather than the locale-first Typed UI interface.
    for (name, props) in [
        ("field.input", vec![("type", PropType::String)]),
        ("field.url", vec![("type", PropType::String)]),
        ("field.password", vec![("type", PropType::String)]),
        ("field.metadata", vec![("add", PropType::String)]),
        ("field.stringlist", vec![("add", PropType::String)]),
        (
            "field.relation",
            vec![
                ("id", PropType::String),
                ("search", PropType::String),
                ("template", PropType::String),
            ],
        ),
        (
            "view.overflow",
            vec![("length", PropType::Number), ("hellip", PropType::Boolean)],
        ),
        ("view.date", vec![("format", PropType::String)]),
        (
            "view.image",
            vec![("width", PropType::Number), ("height", PropType::Number)],
        ),
        ("view.link", vec![("target", PropType::String)]),
        (
            "view.tags",
            vec![
                ("warning", PropType::Boolean),
                ("pill", PropType::Boolean),
                ("className", PropType::String),
            ],
        ),
        ("view.metadata", vec![("className", PropType::String)]),
        ("view.template", vec![("template", PropType::String)]),
    ] {
        definitions = definitions.with_component(
            ComponentDefinition {
                name: name.into(),
                key: format!("stackpress.source.{name}"),
                revision: "source-1".into(),
                props: props
                    .into_iter()
                    .map(|(name, value_type)| PropDefinition {
                        name: name.into(),
                        value_type,
                        required: false,
                        public: true,
                    })
                    .collect(),
                // Source AttributeComponent.props spreads these definition attributes last.
                fixed: match name {
                    "field.url" => [("type".into(), serde_json::json!("url"))].into(),
                    "field.password" => [("type".into(), serde_json::json!("password"))].into(),
                    _ => Default::default(),
                },
                available: false,
            },
            None,
        )?;
    }
    Ok(definitions)
}
