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
    for definition in source_components() {
        definitions = definitions.with_component(definition, None)?;
    }
    Ok(definitions)
}

/// Source argument definitions, reusable by a separately selected compiled renderer catalog.
pub fn source_components() -> Vec<ComponentDefinition> {
    let mut definitions = Vec::new();
    // These names describe source props rather than the locale-first Typed UI interface.
    for (name, props) in [
        ("field.input", vec![("type", PropType::String)]),
        ("field.url", vec![("type", PropType::String)]),
        ("field.password", vec![("type", PropType::String)]),
        ("field.metadata", vec![("add", PropType::String)]),
        ("field.stringlist", vec![("add", PropType::String)]),
        // attributes.ts:3406-3644 and frui/form/TextEditor.tsx:17-55. The source
        // dictionary calls style a CSS object while Frui/blog use a Boolean toolbar flag;
        // retain both as JSON so the renderer can follow Frui's truthiness semantics.
        (
            "field.editor",
            vec![
                ("align", PropType::Boolean),
                ("audio", PropType::Boolean),
                ("blockquote", PropType::Boolean),
                ("className", PropType::String),
                ("code", PropType::Boolean),
                ("color", PropType::Boolean),
                ("dir", PropType::Json),
                ("fullscreen", PropType::Boolean),
                ("history", PropType::Boolean),
                ("font", PropType::Boolean),
                ("format", PropType::Boolean),
                ("highlight", PropType::Boolean),
                ("indent", PropType::Boolean),
                ("lineheight", PropType::Boolean),
                ("link", PropType::Boolean),
                ("list", PropType::Boolean),
                ("math", PropType::Boolean),
                ("paragraph", PropType::Boolean),
                ("preview", PropType::Boolean),
                ("remove", PropType::Boolean),
                ("rule", PropType::Boolean),
                ("showblocks", PropType::Boolean),
                ("size", PropType::Boolean),
                ("table", PropType::Boolean),
                ("style", PropType::Json),
                ("text", PropType::Boolean),
                ("textStyle", PropType::Boolean),
                ("video", PropType::Boolean),
                ("image", PropType::Boolean),
                ("imageGallery", PropType::Boolean),
                ("print", PropType::Boolean),
                ("save", PropType::Boolean),
                // Append newly supported named props without moving source positional slots.
                ("template", PropType::Boolean),
                ("defaultValue", PropType::String),
                ("value", PropType::String),
                ("name", PropType::String),
                ("id", PropType::String),
                ("disabled", PropType::Boolean),
                ("required", PropType::Boolean),
                ("error", PropType::Json),
            ],
        ),
        ("view.html", vec![]),
        // Original attributes.ts:2660-2719 and Frui Select's array/object option carrier.
        (
            "field.select",
            vec![
                ("className", PropType::String),
                ("display", PropType::Json),
                ("dropdown", PropType::Json),
                ("option", PropType::Json),
                ("options", PropType::Json),
                ("placeholder", PropType::String),
            ],
        ),
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
            // Original positional order and string-or-number carrier, attributes.ts:5182-5211.
            vec![
                ("length", PropType::Json),
                ("words", PropType::Boolean),
                ("hellip", PropType::Boolean),
            ],
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
        (
            "view.metadata",
            vec![("className", PropType::String), ("style", PropType::Json)],
        ),
        ("view.template", vec![("template", PropType::String)]),
    ] {
        definitions.push(ComponentDefinition {
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
        });
    }
    definitions
}
