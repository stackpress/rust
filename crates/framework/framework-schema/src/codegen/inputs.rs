//! Generate pure input validation separately from host default execution and stored decoding.

use super::{projection, source, transform_error};
use crate::{InputSchema, Model, Schema, ValueType};
use serde_json::json;
use stackpress_idea_transformer::Result;

/// Construct a sanitized input policy. Published shapes include writable fields only.
fn policy(schema: &Schema, model: &Model, public: bool, patch: bool) -> InputSchema {
    let mut record = projection::record_schema(schema, model);
    for target in std::iter::once(&mut record.model).chain(&mut record.fieldsets) {
        let original = std::iter::once(model)
            .chain(schema.fieldsets())
            .find(|item| item.name == target.name)
            .expect("checked input model");
        target.fields.retain(|field| {
            !field.generated
                && !matches!(field.value_type, ValueType::Model(_))
                && (!patch || target.name != model.name || !field.key)
                && (!public
                    || original
                        .fields
                        .iter()
                        .any(|item| item.name == field.name && item.access.writable))
        });
        for field in &mut target.fields {
            let original = original
                .fields
                .iter()
                .find(|item| item.name == field.name)
                .expect("checked input field");
            if !patch && original.access.publish_default && !original.access.sensitive {
                field.default = original
                    .default
                    .as_ref()
                    .map(|value| projection::public_literal(schema, original, value));
            }
            if patch {
                field.input_required = field.required
                    || field
                        .rules
                        .iter()
                        .any(|rule| matches!(rule.code.as_str(), "required" | "ne"));
            }
        }
    }
    InputSchema { record, patch }
}

/// Emit methods sharing the same business validator without executing providers twice.
pub(super) fn methods(schema: &Schema, model: &Model, public: bool, patch: bool) -> Result<String> {
    let name = source::generate(
        "templates/input-type.rs.tpl",
        include_str!("templates/input-type.rs.tpl"),
        &json!({
            "public":public,"patch":patch,"module":model.module,"name":model.rust_name,
        }),
    )?;
    let descriptor =
        serde_json::to_string(&policy(schema, model, public, patch)).map_err(|cause| {
            transform_error(
                crate::SchemaError::new(
                    crate::SchemaErrorKind::Codec,
                    "input policy encoding failed",
                )
                .with_cause(cause),
            )
        })?;
    source::generate(
        "templates/input-methods.rs.tpl",
        include_str!("templates/input-methods.rs.tpl"),
        &json!({
            "name":name,"descriptor":source::string_literal(&descriptor),
            "method":if patch {"validate_patch"} else {"validate_create"},
            "public":public,"trusted_create":!public&&!patch,
            "public_create":public&&!patch,"module":model.module,"stored":model.rust_name,
        }),
    )
}

/// Emit a public patch shape that cannot name immutable or app-denied fields.
pub(super) fn patch(schema: &Schema, model: &Model) -> Result<String> {
    let mut fields = Vec::new();
    for field in &model.fields {
        if !field.access.writable
            || field.generated
            || field.key
            || matches!(field.value_type, ValueType::Model(_))
        {
            continue;
        }
        let mut field = field.clone();
        field.required = true;
        let ty = super::emit::public_input_type(schema, &field).map_err(transform_error)?;
        fields.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    let methods = methods(schema, model, true, true)?;
    source::generate(
        "templates/public-patch.rs.tpl",
        include_str!("templates/public-patch.rs.tpl"),
        &json!({"name":model.rust_name,"fields":fields,"methods":methods}),
    )
}
