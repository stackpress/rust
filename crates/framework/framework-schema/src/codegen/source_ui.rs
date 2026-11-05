//! Source-role projection keeps authored presentation separate from private model metadata.
use super::{DefinitionRegistry, SourceGraph};
use crate::{
    Model, PublicDefault, PublicType, Role, SchemaResult, SourceUiCatalog, UiField, UiModel,
    UiRelation, ValueType,
};
use stackpress_idea_parser::AttributeValue;

impl SourceGraph {
    /// Select compiled roles without passing source models through Typed normalization.
    /// Only role props, shape and literal UI defaults are retained; assertions stay host-owned.
    /// Hash/encryption attributes, provider arguments and native codec state never enter this value.
    /// # Errors
    /// Preserves selected component/prop errors; unavailable roles fail at emission.
    pub fn project_source_ui(
        &self,
        definitions: &DefinitionRegistry,
    ) -> SchemaResult<SourceUiCatalog> {
        let project = |model: &Model| -> SchemaResult<UiModel> {
            let mut fields = Vec::new();
            for field in &model.fields {
                let mut roles = Vec::new();
                for attribute in &field.attributes {
                    if ["field.", "filter.", "span.", "list.", "view."]
                        .iter()
                        .any(|prefix| attribute.name.starts_with(prefix))
                    {
                        let args = match &attribute.value {
                            serde_json::Value::Bool(value) => AttributeValue::Boolean(*value),
                            serde_json::Value::Array(value) => AttributeValue::Array(value.clone()),
                            _ => unreachable!("normalized role arguments"),
                        };
                        roles.push(definitions.role(&attribute.name, &args)?);
                    }
                }
                let has = |role| roles.iter().any(|binding| binding.role == role);
                let readable = has(Role::List) || has(Role::View) || has(Role::Filter) || field.key;
                let writable = has(Role::Field) || has(Role::Span);
                // Source @secret fields may have a password editor, but never gain a display role.
                // The record supplied by a host remains that host's source process responsibility.
                let default = if field.value_rule.is_some() {
                    PublicDefault::HostGenerated
                } else if writable {
                    field
                        .default
                        .clone()
                        .map(PublicDefault::Literal)
                        .unwrap_or(PublicDefault::None)
                } else {
                    PublicDefault::None
                };
                let choices = if let ValueType::Enum(name) = &field.value_type {
                    self.enums()
                        .iter()
                        .find(|item| item.name == *name)
                        .map(|item| {
                            item.variants
                                .iter()
                                .map(|(_, value)| value.clone())
                                .collect()
                        })
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                fields.push(UiField {
                    id: format!("{}.{}", model.name, field.name),
                    wire_name: field.name.clone(),
                    label: field
                        .documentation
                        .label
                        .clone()
                        .unwrap_or_else(|| field.name.clone()),
                    value_type: PublicType {
                        value: field.value_type.clone(),
                        multiple: field.multiple,
                        choices,
                    },
                    required: field.required,
                    input_required: field.input_required,
                    readable,
                    writable,
                    sensitive: false,
                    nullable: !field.required,
                    default,
                    validation: Vec::new(),
                    filterable: has(Role::Filter),
                    searchable: field.searchable,
                    sortable: field.sortable,
                    roles,
                    relation: if let ValueType::Model(target) = &field.value_type {
                        Some(UiRelation {
                            target: target.clone(),
                            many: field.multiple,
                            mapping: field.relation.clone(),
                            lookup: None,
                        })
                    } else {
                        None
                    },
                });
            }
            Ok(UiModel {
                id: model.name.clone(),
                label: model
                    .documentation
                    .label
                    .clone()
                    .or_else(|| model.documentation.labels.first().cloned())
                    .unwrap_or_else(|| model.name.clone()),
                fields,
                key: model
                    .fields
                    .iter()
                    .filter(|field| field.key)
                    .map(|field| field.name.clone())
                    .collect(),
                query_fields: model
                    .fields
                    .iter()
                    .filter(|field| {
                        field
                            .roles
                            .iter()
                            .any(|binding| binding.role == Role::Filter)
                    })
                    .map(|field| field.name.clone())
                    .collect(),
                default_selection: model.query.clone(),
                operations: Vec::new(),
            })
        };
        Ok(SourceUiCatalog {
            models: self
                .models()
                .iter()
                .map(project)
                .collect::<SchemaResult<_>>()?,
            fieldsets: self
                .fieldsets()
                .iter()
                .map(project)
                .collect::<SchemaResult<_>>()?,
        })
    }
}
