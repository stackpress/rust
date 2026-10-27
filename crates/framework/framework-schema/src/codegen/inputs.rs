//! Generate pure input validation separately from host default execution and stored decoding.

use super::{projection, transform_error};
use crate::{InputSchema, Model, Schema, ValueType};
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
/// Emit methods that share the same business validator without executing providers twice.
pub(super) fn methods(schema: &Schema, model: &Model, public: bool, patch: bool) -> Result<String> {
    let name = match (public, patch) {
        (true, true) => format!("PublicPatch{}", model.rust_name),
        (true, false) => format!("PublicCreate{}", model.rust_name),
        (false, true) => format!("Patch{}", model.rust_name),
        (false, false) => format!("super::{}::Create{}", model.module, model.rust_name),
    };
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
    let method = if patch {
        "validate_patch"
    } else {
        "validate_create"
    };
    let browser = if public {
        "stackpress_framework_schema::check_browser_json(&value)?;"
    } else {
        ""
    };
    let mut text = format!(
        "impl {name} {{\n/// Decode the bounded input policy; no private literal defaults or imports are embedded.\nfn input_schema()->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {{serde_json::from_str({descriptor:?}).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,\"input policy decoding failed\").with_cause(cause))}}\n/// Validate supplied inputs without generating values or executing database work.\npub fn {method}(&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors> {{let value=stackpress_framework_schema::to_bounded_value(self)?;{browser}Self::input_schema()?.validate(&value,context,false)}}\n"
    );
    if !public && !patch {
        text.push_str(&format!("/// Resolve host defaults once and return the fully validated trusted value.\npub fn prepare_create(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<super::{}::{}> {{self.prepare_with(context)}}\n",model.module,model.rust_name));
    }
    if public {
        text.push_str(&format!("/// Evaluate client-checkable rules only; host rules and authorization remain required.\npub fn validate_client(&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors> {{let value=stackpress_framework_schema::to_bounded_value(self)?;{browser}Self::input_schema()?.validate(&value,context,true)}}\n"));
        if !patch {
            text.push_str(&format!("/// Prepare approved literal defaults in public input only; host-generated values remain omitted.\npub fn prepare_create(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<serde_json::Value> {{let value=stackpress_framework_schema::to_bounded_value(&self)?;{browser}Self::input_schema()?.prepare(value,context)}}\n"));
        }
    }
    text.push_str("}\n");
    Ok(text)
}
/// Emit a public patch shape that cannot name immutable or app-denied fields.
pub(super) fn patch(schema: &Schema, model: &Model) -> Result<String> {
    let mut text = format!(
        "/// Published partial update; the host supplies private values explicitly.\n#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct PublicPatch{} {{\n",
        model.rust_name
    );
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
        text.push_str(&format!("/// Approved update value; omission never writes null.\n#[serde(rename={:?},default,skip_serializing_if=\"stackpress_framework_schema::Patch::is_unchanged\")]\npub {}:stackpress_framework_schema::Patch<{ty}>,\n",field.name,field.rust_name));
    }
    text.push_str("}\n");
    text.push_str(&methods(schema, model, true, true)?);
    Ok(text)
}
