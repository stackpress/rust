//! Typed nested create inputs and a single bounded host-preparation pass.

use super::{rust_type, transform_error};
use crate::{Field, Model, Schema, ValueType};
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
        ty = ty.replace(
            &format!("super::{}::{}", model.module, model.rust_name),
            &format!("super::{}::Create{}", model.module, model.rust_name),
        );
    }
    Ok(ty)
}
/// Convert one supplied nonnullable input to its stored type, charging before it is retained.
fn supplied(field: &Field) -> String {
    if let ValueType::Fieldset(_) = &field.value_type {
        let prepare = if field.multiple {
            "if value.len()>1024{return Err(SchemaError::new(SchemaErrorKind::Limit,\"list element budget exceeded\"));} value.into_iter().map(|input|input.prepare_bounded(context,budget)).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?"
        } else {
            "value.prepare_bounded(context,budget)?"
        };
        format!("{{budget.container({:?})?;{prepare}}}", field.name)
    } else {
        format!("{{budget.charge({:?},&value)?;value}}", field.name)
    }
}
/// Resolve the typed default source. Private literals stay host-owned rather than entering browser files.
fn missing(schema: &Schema, model: &Model, field: &Field) -> Result<String> {
    let raw = if let Some(value) = &field.default {
        if field.access.publish_default
            && !field.access.sensitive
            && super::projection::public_literal(schema, field, value) == *value
        {
            format!(
                "serde_json::from_str::<serde_json::Value>({:?}).map_err(|cause|SchemaError::new(SchemaErrorKind::Codec,\"literal default decoding failed\").with_cause(cause))?",
                value.to_string()
            )
        } else {
            format!(
                "context.providers.ok_or_else(||SchemaError::new(SchemaErrorKind::Unsupported,\"explicit host defaults are required\"))?.generate({:?},&[])?",
                format!("default:{}.{}", model.name, field.name)
            )
        }
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
        format!(
            "serde_json::from_str::<stackpress_framework_schema::ValueRule>({rule:?}).map_err(|cause|SchemaError::new(SchemaErrorKind::Codec,\"value rule decoding failed\").with_cause(cause))?.resolve_for(&serde_json::from_str::<stackpress_framework_schema::ValueType>({kind:?}).map_err(|cause|SchemaError::new(SchemaErrorKind::Codec,\"provider type decoding failed\").with_cause(cause))?,context)?"
        )
    } else {
        return Ok(if field.required {
            "return Err(SchemaError::new(SchemaErrorKind::Invalid,\"required create field is missing\"))".into()
        } else {
            "None".into()
        });
    };
    let ty = input_type(schema, field).map_err(transform_error)?;
    let mut scalar = field.clone();
    scalar.required = true;
    let convert = supplied(&scalar);
    let resolved = format!(
        "{{let raw={raw};stackpress_framework_schema::check_json(&raw)?;if raw.is_null() {{ {} }} else {{let value:{ty}=serde_json::from_value(raw).map_err(|cause|SchemaError::new(SchemaErrorKind::Codec,\"default value decoding failed\").with_cause(cause))?;{} }} }}",
        if field.required {
            "return Err(SchemaError::new(SchemaErrorKind::Invalid,\"required default is null\"));"
        } else {
            "None"
        },
        if field.required {
            convert
        } else {
            format!("Some({convert})")
        }
    );
    Ok(resolved)
}
/// Emit the same create/default contract for models and all acyclic embedded values.
pub(super) fn emit(schema: &Schema, model: &Model) -> Result<String> {
    let mut text = format!(
        "/// Create input preserving omitted, null and supplied values at every embedded depth.\n#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct Create{} {{\n",
        model.rust_name
    );
    for field in &model.fields {
        if field.generated || matches!(field.value_type, ValueType::Model(_)) {
            continue;
        }
        let ty = input_type(schema, field).map_err(transform_error)?;
        text.push_str(&format!("/// Explicit create input; omission consults the declared host/literal source.\n#[serde(rename={:?},default,skip_serializing_if=\"stackpress_framework_schema::Input::is_missing\")]\npub {}:stackpress_framework_schema::Input<{ty}>,\n",field.name,field.rust_name));
    }
    text.push_str(&format!("}}\nimpl Create{} {{\n/// Prepare using no ambient clock or provider.\npub fn prepare(self)->stackpress_framework_schema::SchemaResult<{}> {{self.prepare_with(&stackpress_framework_schema::ValueContext::default())}}\n/// Fill defaults exactly once and validate all nested business rules once before effects.\npub fn prepare_with(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<{}> {{let mut budget=stackpress_framework_schema::PreparationBudget::default();let value=self.prepare_bounded(context,&mut budget)?;value.validate_with(context,false)?;Ok(value)}}\n/// Internal cross-module composition shares the caller's aggregate budget; validation follows once.\npub(crate) fn prepare_bounded(self,context:&stackpress_framework_schema::ValueContext<'_>,budget:&mut stackpress_framework_schema::PreparationBudget)->stackpress_framework_schema::SchemaResult<{}> {{use stackpress_framework_schema::{{Input,SchemaError,SchemaErrorKind}};let _=context;Ok({} {{\n",model.rust_name,model.rust_name,model.rust_name,model.rust_name,model.rust_name));
    for field in &model.fields {
        if matches!(field.value_type, ValueType::Model(_)) {
            continue;
        }
        let missing = missing(schema, model, field)?;
        if field.generated {
            text.push_str(&format!(
                "// Generated values cannot be supplied by the caller.\n{}:{missing},\n",
                field.rust_name
            ));
            continue;
        }
        let convert = supplied(field);
        let supplied = if field.required {
            convert
        } else {
            format!("Some({convert})")
        };
        let null = if field.required {
            "return Err(SchemaError::new(SchemaErrorKind::Invalid,\"required create field is null\"))"
        } else {
            "None"
        };
        text.push_str(&format!("// Explicit null never falls through to a literal or host default.\n{}:match self.{} {{Input::Missing=>{missing},Input::Null=>{null},Input::Value(value)=>{supplied}}},\n",field.rust_name,field.rust_name));
    }
    text.push_str("}) } }\n");
    // Emit only fully qualified symbols, so empty/all-generated shapes need no lint allowances.
    text = text
        .replace(
            "use stackpress_framework_schema::{Input,SchemaError,SchemaErrorKind};",
            "",
        )
        .replace(
            "SchemaError::",
            "stackpress_framework_schema::SchemaError::",
        )
        .replace(
            "SchemaErrorKind::",
            "stackpress_framework_schema::SchemaErrorKind::",
        );
    for variant in ["Missing", "Null", "Value"] {
        text = text.replace(
            &format!("Input::{variant}"),
            &format!("stackpress_framework_schema::Input::{variant}"),
        );
    }
    Ok(text)
}
