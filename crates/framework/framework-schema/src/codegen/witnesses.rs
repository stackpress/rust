//! Structural emission of core accessors, independent of full source-model emission.
use crate::{Model, SchemaResult, ValueType};

/// Emit typed core accessors for `<Model>Schema` from its actual winning declarations.
///
/// The caller emits that schema implementation separately. An incompatible declaration
/// keeps its declared type, so the family bound fails at compile time. This function emits
/// no models, defaults, tables, actions or plugin registrations.
pub fn emit_source_witnesses(model: &Model) -> SchemaResult<String> {
    let mut source = String::new();
    for field in &model.fields {
        if !CORE_FIELDS.contains(&field.name.as_str()) {
            continue;
        }
        let marker = format!(
            "{}{}",
            field.name[..1].to_ascii_uppercase(),
            &field.name[1..]
        );
        let scalar = match field.value_type {
            ValueType::String => "String",
            ValueType::Boolean => "bool",
            ValueType::Integer => "i64",
            ValueType::Object | ValueType::Model(_) => {
                "stackpress_framework_schema::source::Record"
            }
            ValueType::Date | ValueType::Time | ValueType::Datetime => {
                "stackpress_framework_schema::source::Date"
            }
            ValueType::Number => "f64",
            _ => {
                return Err(super::unsupported(
                    "source core witness type is not implemented",
                ));
            }
        };
        let declaration = match &field.value_type {
            ValueType::String => "StringType",
            ValueType::Integer => "IntegerType",
            ValueType::Number => "NumberType",
            ValueType::Boolean => "BooleanType",
            ValueType::Object => "ObjectType",
            ValueType::Date => "DateType",
            ValueType::Time => "TimeType",
            ValueType::Datetime => "DatetimeType",
            ValueType::Model(name) => match name.as_str() {
                "Profile" => "ProfileType",
                "Auth" => "AuthType",
                "Application" => "ApplicationType",
                "Session" => "SessionType",
                _ => "OtherModelType",
            },
            _ => {
                return Err(super::unsupported(
                    "source core declaration is not implemented",
                ));
            }
        };
        let mut declaration = format!("stackpress_framework_schema::source::{declaration}");
        if field.multiple {
            declaration = format!("stackpress_framework_schema::source::List<{declaration}>");
        }
        if !field.required {
            declaration = format!("stackpress_framework_schema::source::Nullable<{declaration}>");
        }
        let mut kind = scalar.to_owned();
        if field.multiple {
            kind = format!("Vec<{kind}>");
        }
        if !field.required {
            kind = format!("Option<{kind}>");
        }
        // The schema/type/marker identifiers come from checked normalization. Data values
        // never enter this Rust structure; the field key alone is encoded as a Rust literal.
        source.push_str(&super::source::type_fragment(
            include_str!("templates/source-field.rs.tpl"),
            &[
                ("marker", &marker),
                (
                    "schema_type",
                    &format!("{}Schema", super::type_name(&model.name)?),
                ),
                ("field_type", &kind),
                ("declaration", &declaration),
                ("key", &super::source::string_literal(&field.name)),
            ],
        )?);
    }
    Ok(source)
}

/// Every source field currently read or written by the session/API collaborators.
const CORE_FIELDS: &[&str] = &[
    "id",
    "image",
    "name",
    "type",
    "roles",
    "tags",
    "references",
    "active",
    "created",
    "updated",
    "auth",
    "applications",
    "sessions",
    "profileId",
    "token",
    "secret",
    "verified",
    "consumed",
    "profile",
    "logo",
    "website",
    "scopes",
    "expires",
    "session",
    "applicationId",
    "application",
];
