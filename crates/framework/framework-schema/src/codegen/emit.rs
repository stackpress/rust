//! Pure deterministic contract emission into the schema-owned output partitions.

use super::{invalid, transform_error};
use crate::{Field, Schema, SchemaResult, ValueType};
use stackpress_idea_transformer::OutputPlan;

/// Resolve one field's Rust type without introducing native dependencies into contracts.
pub fn rust_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut name = match &field.value_type {
        ValueType::String => "String".into(),
        ValueType::Integer => "i64".into(),
        ValueType::Boolean => "bool".into(),
        ValueType::Number => "stackpress_framework_schema::FiniteNumber".into(),
        ValueType::Object => "stackpress_framework_schema::JsonObject".into(),
        ValueType::Date => "stackpress_framework_schema::DateValue".into(),
        ValueType::Time => "stackpress_framework_schema::TimeValue".into(),
        ValueType::Datetime => "stackpress_framework_schema::DatetimeValue".into(),
        ValueType::Enum(name) => {
            let item = schema
                .enums
                .iter()
                .find(|item| &item.name == name)
                .ok_or_else(|| invalid("unknown enum"))?;
            format!("super::{}::{}", item.module, item.rust_name)
        }
        ValueType::Fieldset(name) => {
            let item = schema
                .fieldsets
                .iter()
                .find(|item| &item.name == name)
                .ok_or_else(|| invalid("unknown fieldset"))?;
            format!("super::{}::{}", item.module, item.rust_name)
        }
        ValueType::Model(_) => return Err(invalid("virtual relation has no scalar Rust type")),
    };
    if field.multiple {
        name = format!("Vec<{name}>");
    }
    if !field.required {
        name = format!("Option<{name}>");
    }
    Ok(name)
}

/// Emit owned contract structs and scalar enums into this transform's exclusive partition.
pub fn emit_contracts(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    for model in schema.fieldsets.iter().chain(&schema.models) {
        let mut text = format!(
            "//! Owned {} contract generated from finalized Idea.\n\n/// Checked-shape owned values; no database handle or lazy relation.\n#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct {} {{\n",
            model.rust_name, model.rust_name
        );
        for field in &model.fields {
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            let value_type = rust_type(schema, field).map_err(transform_error)?;
            text.push_str(&format!(
                "/// Declared stored value.\n#[serde(rename = {:?})]\npub {}: {},\n",
                field.name, field.rust_name, value_type
            ));
        }
        text.push_str("}\n");
        let record = super::projection::record_schema(schema, model);
        let json = serde_json::to_string(&record).map_err(|cause| {
            transform_error(
                crate::SchemaError::new(
                    crate::SchemaErrorKind::Codec,
                    "record descriptor encoding failed",
                )
                .with_cause(cause),
            )
        })?;
        text.push_str(&format!("impl {} {{\n/// Decode the bounded runtime-only record policy; private metadata is absent.\nfn record_schema() -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {{ serde_json::from_str({json:?}).map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, \"record policy decoding failed\").with_cause(cause)) }}\n/// Validate stored values without rerunning time-relative creation rules.\npub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {{ self.validate_with(&stackpress_framework_schema::ValueContext::default(), true) }}\n/// Validate against one explicit clock/provider context.\npub fn validate_with(&self, context: &stackpress_framework_schema::ValueContext<'_>, stored: bool) -> stackpress_framework_schema::SchemaResult<()> {{ let value=stackpress_framework_schema::to_bounded_value(self)?; Self::record_schema()?.validate(&value,context,false,stored)?.into_result() }}\n}}\n",model.rust_name));
        text.push_str(&super::create::emit(schema, model)?);
        output.write(format!("contracts/{}.rs", model.module), text)?;
        output.contribute_module("contracts", &model.module)?;
    }
    for enumeration in &schema.enums {
        let scalar = if enumeration.variants[0].1.is_string() {
            "String"
        } else if enumeration.variants[0].1.is_boolean() {
            "bool"
        } else {
            "i64"
        };
        let mut text = format!(
            "//! Finite scalar enumeration generated from Idea.\n/// Checked wire choices, without coercion.\n#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]\n#[serde(try_from = {scalar:?}, into = {scalar:?})]\npub enum {} {{\n",
            enumeration.rust_name
        );
        for (variant, _) in &enumeration.variants {
            text.push_str(&format!("/// Declared scalar choice.\n{variant},\n"));
        }
        text.push_str("}\n");
        let match_value = if scalar == "String" {
            "value.as_str()"
        } else {
            "value"
        };
        text.push_str(&format!("impl TryFrom<{scalar}> for {} {{ type Error = &'static str; fn try_from(value: {scalar}) -> Result<Self, Self::Error> {{ match {match_value} {{\n",enumeration.rust_name));
        for (variant, value) in &enumeration.variants {
            let literal = if let Some(value) = value.as_str() {
                format!("{value:?}")
            } else {
                value.to_string()
            };
            text.push_str(&format!("{literal} => Ok(Self::{variant}),\n"));
        }
        if scalar != "bool" || enumeration.variants.len() != 2 {
            text.push_str("_ => Err(\"invalid enum choice\"),");
        }
        text.push_str("} } }\n");
        text.push_str(&format!(
            "impl From<{}> for {scalar} {{ fn from(value: {}) -> Self {{ match value {{\n",
            enumeration.rust_name, enumeration.rust_name
        ));
        for (variant, value) in &enumeration.variants {
            let literal = if let Some(value) = value.as_str() {
                format!("{value:?}.into()")
            } else {
                value.to_string()
            };
            text.push_str(&format!(
                "{}::{variant} => {literal},\n",
                enumeration.rust_name
            ));
        }
        text.push_str("} } }\n");
        output.write(format!("contracts/{}.rs", enumeration.module), text)?;
        output.contribute_module("contracts", &enumeration.module)?;
    }
    emit_public(schema, output)?;
    output.write("metadata/schema.rs", format!("//! Compatibility identity shared by generated partitions.\n/// Normalized semantic fingerprint.\npub const FINGERPRINT: &str = {:?};\n/// Required runtime descriptor revision.\npub const CONTRACT_REVISION: u32 = {};\nconst _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);\n", schema.fingerprint().map_err(transform_error)?, crate::CONTRACT_REVISION))?;
    output.contribute_module("metadata", "schema")
}

/// Emit explicit public DTOs, key/patch/query shapes and the safe UI metadata partition.
fn emit_public(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let mut text = String::from(
        "//! Explicit application projections. Trusted stored rows must use project() before public output.\n",
    );
    for model in schema.fieldsets.iter().chain(&schema.models) {
        let fields: Vec<_> = model
            .fields
            .iter()
            .filter(|f| {
                f.access.readable
                    && !f.access.sensitive
                    && !matches!(f.value_type, ValueType::Model(_))
            })
            .collect();
        text.push_str(&format!("/// Application-approved owned output.\n#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct Public{} {{\n",model.rust_name));
        for field in &fields {
            let ty = public_type(schema, field).map_err(transform_error)?;
            text.push_str(&format!(
                "/// Approved public field.\n#[serde(rename={:?})]\npub {}: {},\n",
                field.name, field.rust_name, ty
            ));
        }
        text.push_str("}\n");
        text.push_str(&format!("impl super::{}::{} {{\n/// Project only fields explicitly approved by the compiled application policy.\npub fn project(&self)->stackpress_framework_schema::SchemaResult<Public{}> {{ self.validate()?; let public=Public{} {{\n",model.module,model.rust_name,model.rust_name,model.rust_name));
        for field in &fields {
            let value = if matches!(field.value_type, ValueType::Fieldset(_)) {
                if field.multiple {
                    if field.required {
                        format!(
                            "self.{}.iter().map(|value|value.project()).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?",
                            field.rust_name
                        )
                    } else {
                        format!(
                            "self.{}.as_ref().map(|values|values.iter().map(|value|value.project()).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()).transpose()?",
                            field.rust_name
                        )
                    }
                } else if field.required {
                    format!("self.{}.project()?", field.rust_name)
                } else {
                    format!(
                        "self.{}.as_ref().map(|value|value.project()).transpose()?",
                        field.rust_name
                    )
                }
            } else if !field.multiple
                && matches!(
                    field.value_type,
                    ValueType::Integer | ValueType::Boolean | ValueType::Number
                )
            {
                format!("self.{}", field.rust_name)
            } else {
                format!("self.{}.clone()", field.rust_name)
            };
            text.push_str(&format!("{}: {},\n", field.rust_name, value));
        }
        text.push_str("}; let wire=stackpress_framework_schema::to_bounded_value(&public)?;stackpress_framework_schema::check_browser_json(&wire)?;Ok(public) } }\n");
        // Public inputs carry writable fields only, independently of read projection.
        text.push_str(&format!("/// Public create input; generated/read-only/relation members cannot be supplied.\n#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct PublicCreate{} {{\n",model.rust_name));
        for field in &model.fields {
            if !field.access.writable
                || field.generated
                || matches!(field.value_type, ValueType::Model(_))
            {
                continue;
            }
            let mut field = field.clone();
            field.required = true;
            let ty = public_input_type(schema, &field).map_err(transform_error)?;
            text.push_str(&format!("/// Writable public input.\n#[serde(rename={:?},default,skip_serializing_if=\"stackpress_framework_schema::Input::is_missing\")]\npub {}: stackpress_framework_schema::Input<{}>,\n",field.name,field.rust_name,ty));
        }
        text.push_str("}\n");
        text.push_str(&super::inputs::methods(schema, model, true, false)?);
        text.push_str(&super::inputs::methods(schema, model, false, false)?);
        if schema.models.iter().any(|item| item.name == model.name) {
            text.push_str(&super::inputs::patch(schema, model)?);
        }
        if !schema.models.iter().any(|m| m.name == model.name) {
            continue;
        }
        text.push_str(&format!("/// Ordered model key, with exact scalar types.\n#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct {}Key {{\n",model.rust_name));
        for field in model.fields.iter().filter(|f| f.key) {
            let ty = rust_type(schema, field).map_err(transform_error)?;
            text.push_str(&format!(
                "/// Key member.\n#[serde(rename={:?})]\npub {}: {},\n",
                field.name, field.rust_name, ty
            ));
        }
        text.push_str("}\n");
        text.push_str(&format!("/// Trusted partial update input; Unchanged never writes null.\n#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]\n#[serde(deny_unknown_fields)]\npub struct Patch{} {{\n",model.rust_name));
        let writable: Vec<_> = model
            .fields
            .iter()
            .filter(|f| !f.generated && !f.key && !matches!(f.value_type, ValueType::Model(_)))
            .collect();
        for field in &writable {
            let mut field = (*field).clone();
            field.required = true;
            let ty = rust_type(schema, &field).map_err(transform_error)?;
            text.push_str(&format!("/// Omitted, supplied, or explicit null patch value.\n#[serde(rename={:?},default,skip_serializing_if=\"stackpress_framework_schema::Patch::is_unchanged\")]\npub {}: stackpress_framework_schema::Patch<{}>,\n",field.name,field.rust_name,ty));
        }
        text.push_str("}\n");
        let mut record = super::projection::record_schema(schema, model);
        record
            .model
            .fields
            .retain(|f| writable.iter().any(|v| v.name == f.name));
        let json = serde_json::to_string(&record).map_err(|cause| {
            transform_error(
                crate::SchemaError::new(
                    crate::SchemaErrorKind::Codec,
                    "patch descriptor encoding failed",
                )
                .with_cause(cause),
            )
        })?;
        text.push_str(&format!("impl Patch{} {{\n/// Reject empty updates and validate supplied fields without applying create defaults.\npub fn validate(&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<()> {{ let value=stackpress_framework_schema::to_bounded_value(self)?; if value.as_object().is_none_or(|v|v.is_empty()) {{return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid,\"empty patch is invalid\"));}} let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str({json:?}).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,\"patch policy decoding failed\").with_cause(cause))?;policy.validate(&value,context,true,false)?.into_result() }}\n}}\n",model.rust_name));
        text.push_str(&super::inputs::methods(schema, model, false, true)?);
        text.push_str(&super::query::emit(schema, model)?);
    }
    output.write("contracts/public.rs", text)?;
    output.contribute_module("contracts", "public")?;
    let ui = schema.project_ui().map_err(transform_error)?;
    let json = serde_json::to_string(&ui).map_err(|cause| {
        transform_error(
            crate::SchemaError::new(
                crate::SchemaErrorKind::Codec,
                "public descriptor encoding failed",
            )
            .with_cause(cause),
        )
    })?;
    output.write("metadata/ui.rs",format!("//! Approved public UI descriptor, containing no imports, source paths or private defaults.\n/// Decode the versioned application-owned projection.\npub fn schema()->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::UiSchema> {{serde_json::from_str({json:?}).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,\"UI descriptor decoding failed\").with_cause(cause))}}\n"))?;
    output.contribute_module("metadata", "ui")
}
/// Replace embedded stored values with nested safe public projections.
fn public_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut ty = rust_type(schema, field)?;
    if let ValueType::Fieldset(name) = &field.value_type {
        let model = schema
            .fieldsets
            .iter()
            .find(|m| m.name == *name)
            .ok_or_else(|| invalid("missing public fieldset"))?;
        ty = ty.replace(
            &format!("super::{}::{}", model.module, model.rust_name),
            &format!("Public{}", model.rust_name),
        );
    }
    Ok(ty)
}
/// Public nested inputs use their own writable shape rather than an entire stored fieldset.
pub(super) fn public_input_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut ty = public_type(schema, field)?;
    if let ValueType::Fieldset(name) = &field.value_type {
        let model = schema
            .fieldsets
            .iter()
            .find(|m| m.name == *name)
            .ok_or_else(|| invalid("missing public fieldset"))?;
        ty = ty.replace(
            &format!("Public{}", model.rust_name),
            &format!("PublicCreate{}", model.rust_name),
        );
    }
    Ok(ty)
}
