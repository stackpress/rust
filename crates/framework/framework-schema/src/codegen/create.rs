//! Typed nested create inputs and a single bounded host-preparation pass.

use super::{rust_type, source, transform_error};
use crate::{Field, Model, Schema, ValueType};
use serde_json::json;
use stackpress_idea_transformer::Result;

/// Use nested create inputs at the wire boundary; stored fieldsets are output values only.
fn input_type(schema: &Schema, field: &Field) -> crate::SchemaResult<String> {
    let mut field = field.clone();
    field.required = true;
    let mut ty = rust_type(schema, &field)?;
    if let ValueType::Fieldset(name) = &field.value_type {
        let model = schema
            .fieldsets()
            .iter()
            .find(|model| model.name == *name)
            .expect("checked fieldset");
        let stored = source::type_fragment(
            "super::[%module%]::[%name%]",
            &[("module", &model.module), ("name", &model.rust_name)],
        )?;
        let create = source::type_fragment(
            "super::[%module%]::Create[%name%]",
            &[("module", &model.module), ("name", &model.rust_name)],
        )?;
        ty = ty.replace(&stored, &create);
    }
    Ok(ty)
}

/// Convert one supplied nonnullable input, charging before a stored value is retained.
fn supplied(field: &Field) -> Result<String> {
    source::generate(
        "templates/create-supplied.rs.tpl",
        include_str!("templates/create-supplied.rs.tpl"),
        &json!({
            "nested":matches!(field.value_type,ValueType::Fieldset(_)),
            "multiple":field.multiple,"wire":source::string_literal(&field.name),
        }),
    )
}

/// Resolve the typed default source without publishing private literals into browser files.
fn missing(schema: &Schema, model: &Model, field: &Field) -> Result<String> {
    let raw = if let Some(value) = &field.default {
        // The existing app policy alone decides whether a literal may be embedded.
        let published = field.access.publish_default
            && !field.access.sensitive
            && super::projection::public_literal(schema, field, value) == *value;
        let literal = if published {
            source::string_literal(&value.to_string())
        } else {
            String::new()
        };
        source::generate(
            "templates/create-default-literal.rs.tpl",
            include_str!("templates/create-default-literal.rs.tpl"),
            &json!({
                "published":published,"literal":literal,
                "provider":source::string_literal(&format!("default:{}.{}",model.name,field.name)),
            }),
        )?
    } else if let Some(rule) = &field.value_rule {
        let rule = serde_json::to_string(rule).map_err(|cause| {
            transform_error(
                crate::SchemaError::new(
                    crate::SchemaErrorKind::Codec,
                    "value rule encoding failed",
                )
                .with_cause(cause),
            )
        })?;
        let kind =
            serde_json::to_string(&field.value_type).expect("checked provider type serialization");
        source::generate(
            "templates/create-default-rule.rs.tpl",
            include_str!("templates/create-default-rule.rs.tpl"),
            &json!({
                "rule":source::string_literal(&rule),"kind":source::string_literal(&kind),
            }),
        )?
    } else {
        return source::generate(
            "templates/create-default-absent.rs.tpl",
            include_str!("templates/create-default-absent.rs.tpl"),
            &json!({"required":field.required}),
        );
    };
    // Decode the selected default through the same required scalar/nested input shape.
    let ty = input_type(schema, field).map_err(transform_error)?;
    let mut scalar = field.clone();
    scalar.required = true;
    let convert = supplied(&scalar)?;
    source::generate(
        "templates/create-default-resolve.rs.tpl",
        include_str!("templates/create-default-resolve.rs.tpl"),
        &json!({
            "raw":raw,"ty":ty,"required":field.required,"convert":convert,
        }),
    )
}

/// Prepare checked view models; the structural template owns declarations and input branches.
pub(super) fn emit(schema: &Schema, model: &Model) -> Result<String> {
    let mut fields = Vec::new();
    for field in &model.fields {
        if field.generated || matches!(field.value_type, ValueType::Model(_)) {
            continue;
        }
        let ty = input_type(schema, field).map_err(transform_error)?;
        fields.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    let mut values = Vec::new();
    for field in &model.fields {
        if matches!(field.value_type, ValueType::Model(_)) {
            continue;
        }
        // Generated fields resolve only Missing; caller-supplied null never consults defaults.
        let missing = missing(schema, model, field)?;
        let convert = if field.generated {
            String::new()
        } else {
            supplied(field)?
        };
        values.push(json!({"member":field.rust_name,"generated":field.generated,
            "missing":missing,"convert":convert,"required":field.required}));
    }
    source::generate(
        "templates/create.rs.tpl",
        include_str!("templates/create.rs.tpl"),
        &json!({
            "name":model.rust_name,"fields":fields,"values":values,
        }),
    )
}
