//! Build-host composition of the API Idea input and the public external Profile catalog.
//!
//! The existing external Profile projection remains until the G50 generated-model release.
//! Package-source acquisition is available independently through the Idea PackageLoader.
use stackpress_framework_schema::{
    Schema, SchemaResult, ValueType,
    codegen::{
        ComponentDefinition, DefinitionRegistry, NormalizeOptions, PropDefinition, PropType,
    },
    providers::cuid_definition,
};
use stackpress_framework_session::profile::ProfileSchema;
use stackpress_idea_parser::{CompileInputs, SchemaConfig};

mod source_compat;

/// Exact owned upstream API schema, including its management metadata.
pub const SOURCE: &str = crate::SCHEMA_SOURCE;
/// Register the shared approved CUID definition in an otherwise app-owned catalog.
pub fn definitions() -> SchemaResult<DefinitionRegistry> {
    let mut registry = DefinitionRegistry::standard().with_provider(cuid_definition())?;
    // Select shared canonical controls, then fill only the source-specific role shapes.
    // The shared URL field adds nullable presence controls absent from the source API form;
    // keep its local mapping, like input/switch/date, without registering the name twice.
    for component in stackpress_framework_admin::codegen::components()
        .into_iter()
        .filter(|component| {
            !matches!(
                component.definition.name.as_str(),
                "field.input" | "field.switch" | "field.url" | "view.date"
            )
        })
    {
        registry = registry.with_component(component.definition, Some(component.import))?;
    }
    for (name, props) in [
        (
            "field.relation",
            vec![
                ("id", PropType::String),
                ("search", PropType::String),
                ("template", PropType::String),
            ],
        ),
        ("field.input", vec![("type", PropType::String)]),
        ("field.switch", Vec::new()),
        ("field.url", vec![("type", PropType::String)]),
        (
            "view.image",
            vec![("width", PropType::Number), ("height", PropType::Number)],
        ),
        ("view.template", vec![("template", PropType::String)]),
        ("view.link", vec![("target", PropType::String)]),
        (
            "view.tags",
            vec![
                ("warning", PropType::Boolean),
                ("pill", PropType::Boolean),
                ("className", PropType::String),
            ],
        ),
        ("view.date", vec![("format", PropType::String)]),
    ] {
        registry = registry.with_component(
            ComponentDefinition {
                name: name.into(),
                key: format!("api.source.{name}"),
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
                fixed: if name == "field.url" {
                    std::collections::BTreeMap::from([("type".into(), serde_json::json!("url"))])
                } else {
                    Default::default()
                },
                available: true,
            },
            Some(stackpress_framework_schema::codegen::ComponentImport {
                component: format!("api.source.{name}"),
                module: "stackpress_framework_api::views::roles".into(),
                symbol: match name {
                    "field.relation" => "RelationField",
                    "field.switch" => "SwitchField",
                    "field.input" | "field.url" => "InputField",
                    "view.image" => "ImageView",
                    "view.template" => "TemplateView",
                    "view.link" => "LinkView",
                    "view.tags" => "TagsView",
                    "view.date" => "DateView",
                    _ => unreachable!(),
                }
                .into(),
                default: false,
            }),
        )?;
    }
    // API storage/admin publishes its plaintext secrets as the source does. Runtime
    // actions do not use generated strict operation DTOs or a global SQL Policy.
    for model in [
        crate::models::ApplicationSchema::new().model(),
        crate::models::SessionSchema::new().model(),
    ] {
        for field in model.fields {
            registry = registry.with_field_policy(
                &model.name,
                &field.name,
                stackpress_framework_schema::FieldAccess {
                    readable: true,
                    writable: !matches!(field.value_type, ValueType::Model(_)),
                    sensitive: false,
                    publish_default: true,
                },
            )?;
        }
    }
    // Joined Profile columns retain their owner's publication description. The API
    // does not invent a second Profile schema or expose its unrelated Auth reverse edge.
    for field in ProfileSchema::default()
        .model()
        .fields
        .into_iter()
        .filter(|field| !matches!(field.value_type, ValueType::Model(_)))
    {
        registry = registry.with_field_policy("Profile", &field.name, field.access)?;
    }
    Ok(registry)
}
/// Resolve the source package import through Profile's public descriptor.
/// No source tree outside the selected Cargo package is needed at generation time.
pub fn config() -> Result<SchemaConfig, stackpress_idea_parser::IdeaError> {
    let profile = ProfileSchema::default().model();
    let mut source = SOURCE.replacen("use \"stackpress-session/schema.idea\"", "", 1);
    source.push_str("\nmodel Profile {\n");
    for field in profile.fields.iter().filter(|field| {
        field.relation.is_none() && !matches!(field.value_type, ValueType::Model(_))
    }) {
        let kind = match field.value_type {
            ValueType::Boolean => "Boolean",
            ValueType::Object => "Hash",
            ValueType::Datetime => "Datetime",
            _ => "String",
        };
        source.push_str(&format!(
            "{} {}{}{}",
            field.name,
            kind,
            if field.multiple { "[]" } else { "" },
            if field.required { "" } else { "?" }
        ));
        if field.key {
            source.push_str(" @id");
        }
        if field.active {
            source.push_str(" @active");
        }
        if field.timestamp {
            source.push_str(" @timestamp");
        }
        if field.searchable {
            source.push_str(" @searchable");
        }
        if field.sortable {
            source.push_str(" @sortable");
        }
        if let Some(rule) = &field.value_rule {
            source.push_str(&format!(" @default(\"{}()\")", rule.provider));
        } else if let Some(value) = &field.default {
            source.push_str(&format!(" @default({value})"));
        }
        source.push('\n');
    }
    source.push_str("}\n");
    let mut config = stackpress_idea_parser::parse(
        "framework-api/schema.idea",
        &source,
        &CompileInputs::default(),
    )?;
    source_compat::apply(&mut config);
    Ok(config)
}
/// Normalize the full source management data with the caller's frozen component registry.
pub fn normalize(definitions: &DefinitionRegistry) -> SchemaResult<Schema> {
    let config = config().map_err(|cause| {
        stackpress_framework_schema::SchemaError::new(
            stackpress_framework_schema::SchemaErrorKind::Invalid,
            "API Idea composition failed",
        )
        .with_cause(cause)
    })?;
    Schema::normalize(&config, definitions, NormalizeOptions::default())
}
