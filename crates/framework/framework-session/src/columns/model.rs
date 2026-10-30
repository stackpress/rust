//! Owned shared descriptors for API schema composition, independent of a SQL executor.
use super::{Kind, RecordSchema};
use stackpress_framework_schema::{Field, FieldAccess, Model, Relation, ValueRule, ValueType};

impl RecordSchema {
    /// Describe the source Idea fields for an API/catalog consumer. Reverse Profile edges
    /// are metadata; the original generated ProfileStore itself exposes no reverse joins.
    /// No DB seed or credential value enters this descriptor.
    pub fn model(&self) -> Model {
        let mut fields = self
            .columns
            .iter()
            .map(|column| {
                let date = column.kind == Kind::Date;
                let value_rule = if column.name == "id" {
                    Some(ValueRule {
                        provider: "cuid".into(),
                        args: Vec::new(),
                        revision: "cuid2-3.3.0".into(),
                    })
                } else if date {
                    Some(ValueRule {
                        provider: "now".into(),
                        args: Vec::new(),
                        revision: "1".into(),
                    })
                } else {
                    None
                };
                let secret = column.kind == Kind::Secret;
                Field {
                    name: column.name.into(),
                    rust_name: if column.name == "type" {
                        "type_"
                    } else {
                        column.sql
                    }
                    .into(),
                    column: column.sql.into(),
                    value_type: match column.kind {
                        Kind::Boolean => ValueType::Boolean,
                        Kind::Object => ValueType::Object,
                        Kind::Date => ValueType::Datetime,
                        _ => ValueType::String,
                    },
                    storage_length: None,
                    required: !column.nullable,
                    multiple: column.kind == Kind::Strings,
                    key: column.name == "id",
                    default: column.default.clone(),
                    nonempty: column.required_message.is_some(),
                    active: column.name == "active",
                    relation: None,
                    unique: column.name == "token",
                    searchable: column.name == "name",
                    sortable: date,
                    timestamp: column.name == "updated",
                    generated: false,
                    input_required: !column.nullable
                        && column.default.is_none()
                        && value_rule.is_none(),
                    value_rule,
                    rules: Vec::new(),
                    documentation: Default::default(),
                    roles: Vec::new(),
                    access: FieldAccess {
                        readable: !secret,
                        writable: true,
                        sensitive: secret,
                        publish_default: !secret,
                    },
                    attributes: Vec::new(),
                }
            })
            .collect::<Vec<_>>();
        let edges: &[(&str, &str, bool)] = if self.name == "Profile" {
            &[
                ("auth", "Auth", true),
                ("applications", "Application", true),
                ("sessions", "Session", true),
            ]
        } else {
            &[("profile", "Profile", false)]
        };
        for (name, target, multiple) in edges {
            fields.push(Field {
                name: (*name).into(),
                rust_name: (*name).into(),
                column: (*name).into(),
                value_type: ValueType::Model((*target).into()),
                storage_length: None,
                required: !*multiple,
                multiple: *multiple,
                key: false,
                default: None,
                nonempty: false,
                active: false,
                relation: (!*multiple).then(|| Relation {
                    local: "profileId".into(),
                    foreign: "id".into(),
                    inverse: None,
                }),
                unique: false,
                searchable: false,
                sortable: false,
                timestamp: false,
                generated: false,
                input_required: false,
                value_rule: None,
                rules: Vec::new(),
                documentation: Default::default(),
                roles: Vec::new(),
                access: Default::default(),
                attributes: Vec::new(),
            });
        }
        Model {
            name: self.name.into(),
            rust_name: self.name.into(),
            module: self.name.to_ascii_lowercase(),
            fields,
            documentation: Default::default(),
            query: if self.name == "Auth" {
                vec!["*".into(), "profile.*".into()]
            } else {
                Vec::new()
            },
            attributes: Vec::new(),
        }
    }
}
