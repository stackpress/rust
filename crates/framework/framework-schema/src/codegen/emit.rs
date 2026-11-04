//! Pure deterministic contract emission into the schema-owned output partitions.

use super::{invalid, source, transform_error};
use crate::{Field, Model, Schema, SchemaResult, ValueType};
use serde_json::{Value, json};
use stackpress_idea_transformer::{OutputPlan, Result};

/// Resolve one field's Rust type without introducing native dependencies into contracts.
pub fn rust_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut name = match &field.value_type {
        ValueType::Unknown => {
            return Err(invalid("Typed emission does not support source Unknown"));
        }
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
            source::type_fragment(
                "super::[%module%]::[%name%]",
                &[("module", &item.module), ("name", &item.rust_name)],
            )?
        }
        ValueType::Fieldset(name) => {
            let item = schema
                .fieldsets
                .iter()
                .find(|item| &item.name == name)
                .ok_or_else(|| invalid("unknown fieldset"))?;
            source::type_fragment(
                "super::[%module%]::[%name%]",
                &[("module", &item.module), ("name", &item.rust_name)],
            )?
        }
        ValueType::Model(_) => return Err(invalid("virtual relation has no scalar Rust type")),
    };
    // Compose actual Rust generic syntax through templates, preserving wrapper order.
    if field.multiple {
        name = source::type_fragment("Vec<[%inner%]>", &[("inner", &name)])?;
    }
    if !field.required {
        name = source::type_fragment("Option<[%inner%]>", &[("inner", &name)])?;
    }
    Ok(name)
}

/// Emit owned contract structs and scalar enums into this transform's exclusive partition.
pub fn emit_contracts(schema: &Schema, output: &mut OutputPlan) -> Result<()> {
    for model in schema.fieldsets.iter().chain(&schema.models) {
        let mut fields = Vec::new();
        for field in &model.fields {
            // Virtual model relations carry no stored member or independent scalar Rust type.
            if matches!(field.value_type, ValueType::Model(_)) {
                continue;
            }
            let ty = rust_type(schema, field).map_err(transform_error)?;
            fields.push(json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}));
        }
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
        let create = super::create::emit(schema, model)?;
        let text = source::generate(
            "templates/contract.rs.tpl",
            include_str!("templates/contract.rs.tpl"),
            &json!({
                "name":model.rust_name,"fields":fields,"record":source::string_literal(&json),"create":create,
            }),
        )?;
        output.write(format!("contracts/{}.rs", model.module), text)?;
        output.contribute_module("contracts", &model.module)?;
    }
    for enumeration in &schema.enums {
        // Normalization guarantees a nonempty single-family enum before source preparation.
        let scalar = if enumeration.variants[0].1.is_string() {
            "String"
        } else if enumeration.variants[0].1.is_boolean() {
            "bool"
        } else {
            "i64"
        };
        let variants: Vec<_> = enumeration
            .variants
            .iter()
            .map(|(name, value)| {
                json!({
                    "name":name,"literal":enum_literal(value),
                })
            })
            .collect();
        let text = source::generate(
            "templates/enum.rs.tpl",
            include_str!("templates/enum.rs.tpl"),
            &json!({
                "name":enumeration.rust_name,"scalar":scalar,"scalar_literal":source::string_literal(scalar),
                "is_string":scalar=="String","variants":variants,"fallback":scalar!="bool"||enumeration.variants.len()!=2,
            }),
        )?;
        output.write(format!("contracts/{}.rs", enumeration.module), text)?;
        output.contribute_module("contracts", &enumeration.module)?;
    }
    emit_public(schema, output)?;
    let fingerprint = schema.fingerprint().map_err(transform_error)?;
    output.write("metadata/schema.rs",source::generate("templates/schema-metadata.rs.tpl",include_str!("templates/schema-metadata.rs.tpl"),&json!({
        "fingerprint":source::string_literal(&fingerprint),"revision":crate::CONTRACT_REVISION,
    }))?)?;
    output.contribute_module("metadata", "schema")
}

/// Encode a checked scalar enum atom; the template supplies all surrounding Rust syntax.
fn enum_literal(value: &Value) -> String {
    if let Some(value) = value.as_str() {
        source::string_literal(value)
    } else {
        value.to_string()
    }
}

/// Emit explicit public DTOs, key/patch/query shapes and the safe UI metadata partition.
fn emit_public(schema: &Schema, output: &mut OutputPlan) -> Result<()> {
    let mut models = Vec::new();
    for model in schema.fieldsets.iter().chain(&schema.models) {
        let fields = public_fields(schema, model)?;
        let inputs = public_inputs(schema, model)?;
        let public_methods = super::inputs::methods(schema, model, true, false)?;
        let trusted_methods = super::inputs::methods(schema, model, false, false)?;
        let is_model = schema.models.iter().any(|item| item.name == model.name);
        let public_patch = if is_model {
            super::inputs::patch(schema, model)?
        } else {
            String::new()
        };
        // Only stored models receive key, trusted patch and query declarations.
        let operations = if is_model {
            model_operations(schema, model)?
        } else {
            ModelOperations::default()
        };
        models.push(json!({
            "name":model.rust_name,"module":model.module,"fields":fields,"inputs":inputs,
            "public_methods":public_methods,"trusted_methods":trusted_methods,"is_model":is_model,
            "public_patch":public_patch,"keys":operations.keys,"patches":operations.patches,
            "patch_policy":operations.patch_policy,"patch_methods":operations.patch_methods,"query":operations.query,
        }));
    }
    output.write(
        "contracts/public.rs",
        source::generate(
            "templates/public.rs.tpl",
            include_str!("templates/public.rs.tpl"),
            &json!({"models":models}),
        )?,
    )?;
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
    output.write(
        "metadata/ui.rs",
        source::generate(
            "templates/ui-metadata.rs.tpl",
            include_str!("templates/ui-metadata.rs.tpl"),
            &json!({"descriptor":source::string_literal(&json)}),
        )?,
    )?;
    output.contribute_module("metadata", "ui")
}

/// Prepare readable field types first, then projection expressions, preserving error precedence.
fn public_fields(schema: &Schema, model: &Model) -> Result<Vec<Value>> {
    let selected: Vec<_> = model
        .fields
        .iter()
        .filter(|field| {
            field.access.readable
                && !field.access.sensitive
                && !matches!(field.value_type, ValueType::Model(_))
        })
        .collect();
    let mut fields = Vec::new();
    // Complete all type checks before rendering projections, retaining the original first error.
    for field in &selected {
        let ty = public_type(schema, field).map_err(transform_error)?;
        fields.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    // Fieldset projection uses checked public shapes; scalar ownership follows the original rules.
    for (field, value) in selected.iter().zip(&mut fields) {
        value["value"] = json!(source::generate(
            "templates/project-value.rs.tpl",
            include_str!("templates/project-value.rs.tpl"),
            &json!({
                "member":field.rust_name,"fieldset":matches!(field.value_type,ValueType::Fieldset(_)),
                "multiple":field.multiple,"required":field.required,
                "copy":!field.multiple&&matches!(field.value_type,ValueType::Integer|ValueType::Boolean|ValueType::Number),
            })
        )?);
    }
    Ok(fields)
}

/// Prepare writable public input members without generated fields or virtual relations.
fn public_inputs(schema: &Schema, model: &Model) -> Result<Vec<Value>> {
    let mut inputs = Vec::new();
    for field in &model.fields {
        if !field.access.writable
            || field.generated
            || matches!(field.value_type, ValueType::Model(_))
        {
            continue;
        }
        let mut field = field.clone();
        // The Input wrapper represents omission/null; its inner value has a required Rust type.
        field.required = true;
        let ty = public_input_type(schema, &field).map_err(transform_error)?;
        inputs.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    Ok(inputs)
}

/// Stored-model operations stay empty for fieldsets, which have no independent identity.
#[derive(Default)]
struct ModelOperations {
    /// Checked key member descriptors, in declaration order.
    keys: Vec<Value>,
    /// Trusted patch member descriptors, excluding keys and generated values.
    patches: Vec<Value>,
    /// Encoded policy and methods for the same selected patch fields.
    patch_policy: String,
    /// Matching trusted input methods, rendered only after policy encoding succeeds.
    patch_methods: String,
    /// Pre-rendered typed query declarations, inserted inertly.
    query: String,
}

/// Prepare key, patch policy/methods and query source in their existing fallible order.
fn model_operations(schema: &Schema, model: &Model) -> Result<ModelOperations> {
    let mut keys = Vec::new();
    let mut patches = Vec::new();
    // Identity order is declaration order, independently of sorted output file paths.
    for field in model.fields.iter().filter(|field| field.key) {
        let ty = rust_type(schema, field).map_err(transform_error)?;
        keys.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    let writable: Vec<_> = model
        .fields
        .iter()
        .filter(|field| {
            !field.generated && !field.key && !matches!(field.value_type, ValueType::Model(_))
        })
        .collect();
    for field in &writable {
        let mut field = (*field).clone();
        // Patch owns its tri-state; nullable storage does not make the inner value optional.
        field.required = true;
        let ty = rust_type(schema, &field).map_err(transform_error)?;
        patches.push(
            json!({"wire":source::string_literal(&field.name),"member":field.rust_name,"ty":ty}),
        );
    }
    let mut record = super::projection::record_schema(schema, model);
    // Use the identical trusted writable set for both emitted members and policy validation.
    record
        .model
        .fields
        .retain(|field| writable.iter().any(|value| value.name == field.name));
    let json = serde_json::to_string(&record).map_err(|cause| {
        transform_error(
            crate::SchemaError::new(
                crate::SchemaErrorKind::Codec,
                "patch descriptor encoding failed",
            )
            .with_cause(cause),
        )
    })?;
    let patch_policy = source::string_literal(&json);
    // Preparation remains ordered: policy, patch methods, then query descriptors and source.
    let patch_methods = super::inputs::methods(schema, model, false, true)?;
    let query = super::query::emit(schema, model)?;
    Ok(ModelOperations {
        keys,
        patches,
        patch_policy,
        patch_methods,
        query,
    })
}

/// Replace embedded stored values with nested safe public projections.
fn public_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut ty = rust_type(schema, field)?;
    if let ValueType::Fieldset(name) = &field.value_type {
        let model = schema
            .fieldsets
            .iter()
            .find(|model| &model.name == name)
            .ok_or_else(|| invalid("missing public fieldset"))?;
        let stored = source::type_fragment(
            "super::[%module%]::[%name%]",
            &[("module", &model.module), ("name", &model.rust_name)],
        )?;
        let public = source::type_fragment("Public[%name%]", &[("name", &model.rust_name)])?;
        ty = ty.replace(&stored, &public);
    }
    Ok(ty)
}

/// Public nested inputs use writable shapes rather than an entire stored fieldset.
pub(super) fn public_input_type(schema: &Schema, field: &Field) -> SchemaResult<String> {
    let mut ty = public_type(schema, field)?;
    if let ValueType::Fieldset(name) = &field.value_type {
        let model = schema
            .fieldsets
            .iter()
            .find(|model| &model.name == name)
            .ok_or_else(|| invalid("missing public fieldset"))?;
        let public = source::type_fragment("Public[%name%]", &[("name", &model.rust_name)])?;
        let input = source::type_fragment("PublicCreate[%name%]", &[("name", &model.rust_name)])?;
        ty = ty.replace(&public, &input);
    }
    Ok(ty)
}
