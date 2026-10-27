//! Explicit safe projection; neither component roles nor schema existence grant public access.

use crate::*;

impl Schema {
    /// Build a versioned public descriptor from app-approved fields only.
    /// # Errors
    /// Returns an encoding failure while computing the semantic fingerprint.
    pub fn project_ui(&self) -> SchemaResult<UiSchema> {
        let mut models = Vec::new();
        for model in self.models.iter().chain(&self.fieldsets) {
            let mut fields = Vec::new();
            for field in &model.fields {
                if !field.access.readable && !field.access.writable {
                    continue;
                }
                let choices = if let ValueType::Enum(name) = &field.value_type {
                    self.enums
                        .iter()
                        .find(|v| v.name == *name)
                        .map(|v| v.variants.iter().map(|(_, v)| v.clone()).collect())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                let default = if field.value_rule.is_some()
                    || field.generated
                    || field.default.is_some()
                        && (!field.access.publish_default || field.access.sensitive)
                {
                    PublicDefault::HostGenerated
                } else if field.access.publish_default && !field.access.sensitive {
                    field
                        .default
                        .as_ref()
                        .map(|value| PublicDefault::Literal(public_literal(self, field, value)))
                        .unwrap_or(PublicDefault::None)
                } else {
                    PublicDefault::None
                };
                let relation = if let ValueType::Model(target) = &field.value_type {
                    Some(UiRelation {
                        target: target.clone(),
                        many: field.multiple,
                        mapping: field.relation.clone(),
                        lookup: None,
                    })
                } else {
                    None
                };
                // Every emitted literal must retain exact meaning in the browser wire format.
                let check = || -> SchemaResult<()> {
                    for choice in &choices {
                        check_browser_json(choice)?;
                    }
                    if let PublicDefault::Literal(value) = &default {
                        check_browser_json(value)?;
                    }
                    for role in &field.roles {
                        for value in role.props.values() {
                            check_browser_json(value)?;
                        }
                    }
                    Ok(())
                };
                check().map_err(|error| {
                    error.at(vec![
                        if self.models.iter().any(|item| item.name == model.name) {
                            "model"
                        } else {
                            "type"
                        }
                        .into(),
                        model.name.clone(),
                        "columns".into(),
                        field.name.clone(),
                    ])
                })?;
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
                    input_required: field.access.writable && field.input_required,
                    readable: field.access.readable && !field.access.sensitive,
                    writable: field.access.writable && !field.generated,
                    sensitive: field.access.sensitive,
                    nullable: !field.required,
                    default,
                    validation: public_rules(field),
                    filterable: filterable(field),
                    searchable: field.access.readable
                        && !field.access.sensitive
                        && field.searchable,
                    sortable: field.access.readable && !field.access.sensitive && field.sortable,
                    roles: field.roles.clone(),
                    relation,
                });
            }
            if fields.is_empty() {
                continue;
            }
            let keys: Vec<_> = model.fields.iter().filter(|v| v.key).collect();
            let key = if keys
                .iter()
                .all(|f| f.access.readable && !f.access.sensitive)
            {
                keys.iter().map(|f| f.name.clone()).collect()
            } else {
                Vec::new()
            };
            let query_fields = model
                .fields
                .iter()
                .filter(|f| filterable(f))
                .map(|f| f.name.clone())
                .collect();
            models.push(UiModel {
                id: model.name.clone(),
                label: model
                    .documentation
                    .label
                    .clone()
                    .or_else(|| model.documentation.labels.first().cloned())
                    .unwrap_or_else(|| model.name.clone()),
                fields,
                key,
                query_fields,
                default_selection: model
                    .query
                    .iter()
                    .filter(|path| approved_path(self, model, path))
                    .cloned()
                    .collect(),
                operations: Vec::new(),
            });
        }
        let (models, fieldsets) = models
            .into_iter()
            .partition(|model| self.models.iter().any(|item| item.name == model.id));
        Ok(UiSchema {
            contract_revision: CONTRACT_REVISION,
            schema_fingerprint: self.fingerprint()?,
            models,
            fieldsets,
        })
    }
    /// Refuse selecting unavailable metadata for actual UI output; descriptor emission is independent.
    pub fn require_available_roles(&self) -> SchemaResult<()> {
        for model in self.models.iter().chain(&self.fieldsets) {
            for field in &model.fields {
                if !(field.access.readable || field.access.writable) {
                    continue;
                }
                if field.roles.iter().any(|role| !role.available) {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Unsupported,
                        "selected UI component is unavailable",
                    )
                    .at(vec![
                        if self.models.iter().any(|item| item.name == model.name) {
                            "model"
                        } else {
                            "type"
                        }
                        .into(),
                        model.name.clone(),
                        "columns".into(),
                        field.name.clone(),
                    ]));
                }
            }
        }
        Ok(())
    }
}
/// Remove tool/source/private metadata from the runtime record validator snapshot.
pub(super) fn record_schema(schema: &Schema, model: &Model) -> RecordSchema {
    fn clean(model: &Model) -> Model {
        let mut model = model.clone();
        model.attributes.clear();
        model.documentation = Documentation::default();
        model.query.clear();
        for field in &mut model.fields {
            for (index, rule) in field.rules.iter_mut().enumerate() {
                if (!field.access.readable && !field.access.writable || field.access.sensitive)
                    && !rule.args.is_empty()
                {
                    rule.code = format!("private:{}.{}.{}", model.name, field.name, index);
                    rule.args.clear();
                    rule.client_checkable = false;
                    rule.message = "field validation failed".into();
                }
            }
            field.attributes.clear();
            field.documentation = Documentation::default();
            field.roles.clear();
            field.default = None;
            field.value_rule = None;
            field.access = FieldAccess::default();
        }
        model
    }
    RecordSchema {
        model: clean(model),
        fieldsets: schema.fieldsets.iter().map(clean).collect(),
        enums: schema.enums.clone(),
    }
}

/// Public rules expose only client-safe arguments; host-only checks are marked without private data.
fn public_rules(field: &Field) -> Vec<PublicRule> {
    field
        .rules
        .iter()
        .map(|rule| {
            let mut rule = rule.clone();
            if !rule.client_checkable
                || field.access.sensitive && !rule.args.is_empty()
                || rule
                    .args
                    .iter()
                    .any(|value| check_browser_json(value).is_err())
            {
                rule.args.clear();
                rule.client_checkable = false;
                rule.message = "field validation failed".into();
            }
            rule
        })
        .collect()
}
/// Literal fieldset defaults honor the same nested app policy as output projection.
pub(super) fn public_literal(
    schema: &Schema,
    field: &Field,
    value: &serde_json::Value,
) -> serde_json::Value {
    if field.multiple
        && let Some(values) = value.as_array()
    {
        let mut scalar = field.clone();
        scalar.multiple = false;
        return serde_json::Value::Array(
            values
                .iter()
                .map(|value| public_literal(schema, &scalar, value))
                .collect(),
        );
    }
    if let ValueType::Fieldset(name) = &field.value_type
        && let (Some(model), Some(values)) = (
            schema.fieldsets.iter().find(|model| model.name == *name),
            value.as_object(),
        )
    {
        return serde_json::Value::Object(
            model
                .fields
                .iter()
                .filter(|field| {
                    (field.access.readable || field.access.writable) && !field.access.sensitive
                })
                .filter_map(|field| {
                    values
                        .get(&field.name)
                        .map(|value| (field.name.clone(), public_literal(schema, field, value)))
                })
                .collect(),
        );
    }
    value.clone()
}
/// Query eligibility is separate from presentation; only app-readable scalar candidates participate.
pub(super) fn filterable(field: &Field) -> bool {
    field.access.readable
        && !field.access.sensitive
        && !matches!(
            field.value_type,
            ValueType::Fieldset(_) | ValueType::Model(_) | ValueType::Object
        )
        && (field.key || field.roles.iter().any(|role| role.role == Role::Filter))
}
/// Resolve selected projection paths while applying app policy at every segment.
fn approved_path(schema: &Schema, model: &Model, path: &str) -> bool {
    let mut model = model;
    let parts: Vec<_> = path.split('.').collect();
    for (index, part) in parts.iter().enumerate() {
        let Some(field) = model
            .fields
            .iter()
            .find(|field| field.name == *part && field.access.readable && !field.access.sensitive)
        else {
            return false;
        };
        if index + 1 < parts.len() {
            let (ValueType::Model(name) | ValueType::Fieldset(name)) = &field.value_type else {
                return false;
            };
            let Some(next) = schema
                .models
                .iter()
                .chain(&schema.fieldsets)
                .find(|model| model.name == *name)
            else {
                return false;
            };
            model = next;
        }
    }
    true
}
