//! Bounded build-host normalization and deterministic Rust contract emission.
//!
//! Both schema and SQL transforms receive the same frozen app-owned definition registry.
//! No mutable cross-transform payload or process-global registration is introduced.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use stackpress_idea_parser::{AttributeValue, SchemaConfig};

use crate::{
    Enumeration, Field, Model, Relation, Schema, SchemaError, SchemaErrorKind, SchemaResult,
    ValueType,
};

mod budgets;
use budgets::preflight;
mod registry;
pub use registry::*;
mod assertions;
mod create;
mod inputs;
mod metadata;
mod projection;
mod query;
/// Lowerable direct-config budgets; hard ceilings apply even if callers raise them.
#[derive(Clone, Copy, Debug)]
pub struct NormalizeOptions {
    /// Maximum models/fieldsets/enums together, capped at 128.
    pub declarations: usize,
    /// Maximum fields per declaration, capped at 128.
    pub fields: usize,
}

impl Default for NormalizeOptions {
    /// Supply finite generation bounds.
    fn default() -> Self {
        Self {
            declarations: 128,
            fields: 128,
        }
    }
}

impl Schema {
    /// Normalize finalized Idea configuration into finite, owned runtime descriptors.
    ///
    /// # Errors
    /// Refuses unresolved imports/props, unsupported selected semantics, invalid types,
    /// collisions, embedded cycles, bad keys/relations/defaults and exceeded budgets.
    pub fn normalize(
        config: &SchemaConfig,
        definitions: &DefinitionRegistry,
        options: NormalizeOptions,
    ) -> SchemaResult<Self> {
        // Imported references must be finalized by the existing typed composer boundary.
        if !config.r#use.is_empty() || !config.prop.is_empty() {
            return Err(invalid("schema must be composed and finalized"));
        }
        if config.model.len() + config.r#type.len() + config.r#enum.len()
            > options.declarations.min(128)
        {
            return Err(limit());
        }
        // Validate all identities and charge one aggregate budget before cloning descriptors.
        // Direct caller-created config cannot rely on parser acquisition limits.
        preflight(config, options)?;
        let mut schema = Self {
            models: Vec::new(),
            fieldsets: Vec::new(),
            enums: Vec::new(),
        };
        let mut modules = BTreeSet::new();
        for (name, choices) in &config.r#enum {
            let module = portable_name(name)?;
            if module == "public"
                || !modules.insert(module.clone())
                || choices.is_empty()
                || choices.len() > 128
            {
                return Err(invalid("enum name collision or choice budget")
                    .at(vec!["enum".into(), name.clone()]));
            }
            let mut values = BTreeSet::new();
            let mut variants = BTreeSet::new();
            let mut enumeration = Enumeration {
                name: name.clone(),
                rust_name: type_name(name)?,
                module,
                variants: Vec::new(),
            };
            for (variant, value) in choices {
                let family = enum_family(value)?;
                if choices
                    .values()
                    .any(|v| enum_family(v).ok() != Some(family))
                {
                    return Err(
                        invalid("mixed enum scalar families").at(vec!["enum".into(), name.clone()])
                    );
                }
                let identity = value.to_string();
                let variant = type_name(variant)?;
                if identity.len() > 4096
                    || !values.insert(identity)
                    || !variants.insert(variant.clone())
                {
                    return Err(
                        invalid("duplicate enum identity").at(vec!["enum".into(), name.clone()])
                    );
                }
                enumeration.variants.push((variant, value.clone()));
            }
            schema.enums.push(enumeration);
        }
        let mut resolved_roles = crate::values::JsonBudget::default();
        // Register identities before resolving relations; model graphs may be cyclic.
        for (embedded, declarations) in [(true, &config.r#type), (false, &config.model)] {
            for (name, declaration) in declarations {
                if name != &declaration.name {
                    return Err(invalid("declaration identity mismatch"));
                }
                let module = definitions
                    .declaration_name(name)
                    .map(str::to_owned)
                    .unwrap_or(portable_name(name)?);
                if module == "public" || !modules.insert(module.clone()) {
                    return Err(invalid("declaration name collision").at(vec![
                        if embedded { "type" } else { "model" }.into(),
                        name.clone(),
                    ]));
                }
                let model = metadata::normalize_model(
                    declaration,
                    config,
                    definitions,
                    options,
                    embedded,
                    &mut resolved_roles,
                )?;
                if embedded {
                    schema.fieldsets.push(model);
                } else {
                    schema.models.push(model);
                }
            }
        }
        // All public DTOs share one module, so helper prefixes must not collide with another source name.
        let mut public_names = BTreeSet::new();
        for model in schema.fieldsets.iter().chain(&schema.models) {
            let mut names = vec![
                format!("Public{}", model.rust_name),
                format!("PublicCreate{}", model.rust_name),
            ];
            let is_model = schema.models.iter().any(|item| item.name == model.name);
            if is_model {
                names.extend([
                    format!("PublicPatch{}", model.rust_name),
                    format!("Patch{}", model.rust_name),
                ]);
                names.extend(
                    ["Key", "Filter", "Sort", "Query"]
                        .map(|suffix| format!("{}{suffix}", model.rust_name)),
                );
            }
            for name in names {
                if !public_names.insert(name) {
                    return Err(invalid("generated public type name collision").at(vec![
                        if is_model { "model" } else { "type" }.into(),
                        model.name.clone(),
                    ]));
                }
            }
        }
        // Embedded values are bounded DFS paths; relations are names and are never traversed here.
        for model in &schema.fieldsets {
            schema.check_embedded(model, &mut Vec::new(), &mut 10_000)?;
        }
        for model in &schema.models {
            schema.check_relations(model)?;
        }
        let fieldsets = schema.fieldsets.clone();
        for model in schema.fieldsets.iter_mut().chain(&mut schema.models) {
            for field in &mut model.fields {
                if let Some(value) = field.default.take() {
                    field.default = Some(metadata::canonical_default(field, value, &fieldsets)?);
                }
                for rule in &mut field.rules {
                    if matches!(
                        field.value_type,
                        ValueType::Date | ValueType::Time | ValueType::Datetime
                    ) && matches!(
                        rule.code.as_str(),
                        "eq" | "neq" | "option" | "gt" | "ge" | "lt" | "le"
                    ) {
                        for value in &mut rule.args {
                            *value = metadata::canonical_temporal(&field.value_type, value)?;
                        }
                    }
                }
            }
        }
        schema.check_defaults()?;
        Ok(schema)
    }

    /// Normalize while preserving an exact winning source coordinate on semantic failure.
    pub fn normalize_with_diagnostics(
        config: &SchemaConfig,
        definitions: &DefinitionRegistry,
        options: NormalizeOptions,
        map: &DiagnosticMap,
    ) -> SchemaResult<Self> {
        Self::normalize(config, definitions, options).map_err(|error| error.with_diagnostics(map))
    }
    /// Validate nested literal defaults against the complete resolved embedding graph.
    fn check_defaults(&self) -> SchemaResult<()> {
        for model in self.fieldsets.iter().chain(&self.models) {
            let record = crate::RecordSchema {
                model: model.clone(),
                fieldsets: self.fieldsets.clone(),
                enums: self.enums.clone(),
            };
            for field in &model.fields {
                if let Some(value) = &field.default {
                    let value = serde_json::json!({field.name.clone():value});
                    record
                        .validate_defaults(&value, &crate::ValueContext::default())?
                        .into_result()
                        .map_err(|error| {
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
                                "attributes".into(),
                                "default".into(),
                            ])
                        })?;
                }
            }
        }
        Ok(())
    }
    /// Compute a stable semantic fingerprint from framework-owned ordered descriptors.
    pub fn fingerprint(&self) -> SchemaResult<String> {
        let mut semantic = self.clone();
        for model in semantic.models.iter_mut().chain(&mut semantic.fieldsets) {
            model.attributes.clear();
            for field in &mut model.fields {
                field.attributes.clear();
            }
        }
        let value =
            serde_json::to_value((crate::CONTRACT_REVISION, semantic)).map_err(|cause| {
                SchemaError::new(
                    SchemaErrorKind::Codec,
                    "schema fingerprint conversion failed",
                )
                .with_cause(cause)
            })?;
        let bytes = serde_json::to_vec(&canonical(value)).map_err(|cause| {
            SchemaError::new(SchemaErrorKind::Codec, "schema fingerprint encoding failed")
                .with_cause(cause)
        })?;
        Ok(format!("{:x}", Sha256::digest(bytes)))
    }

    /// Reject embedded cycles before generating recursive Rust values or JSON codecs.
    fn check_embedded(
        &self,
        model: &Model,
        path: &mut Vec<String>,
        remaining: &mut usize,
    ) -> SchemaResult<()> {
        if *remaining == 0 {
            return Err(limit());
        }
        *remaining -= 1;
        if path.len() >= 32 || path.contains(&model.name) {
            let mut cycle = path.clone();
            cycle.push(model.name.clone());
            return Err(invalid("embedded value cycle or depth limit").at([
                vec!["type".into(), model.name.clone(), "cycle".into()],
                cycle,
            ]
            .concat()));
        }
        path.push(model.name.clone());
        for field in &model.fields {
            if let ValueType::Fieldset(name) = &field.value_type {
                let child = self
                    .fieldsets
                    .iter()
                    .find(|item| &item.name == name)
                    .ok_or_else(|| invalid("unknown embedded type"))?;
                self.check_embedded(child, path, remaining)?;
            }
        }
        path.pop();
        Ok(())
    }

    /// Validate explicit single-key owning mappings and unambiguous reverse collections.
    fn check_relations(&self, model: &Model) -> SchemaResult<()> {
        for field in &model.fields {
            let ValueType::Model(target) = &field.value_type else {
                continue;
            };
            let path = vec![
                "model".into(),
                model.name.clone(),
                "columns".into(),
                field.name.clone(),
            ];
            (|| -> SchemaResult<()> {
                let target = self
                    .models
                    .iter()
                    .find(|item| &item.name == target)
                    .ok_or_else(|| invalid("unknown relation target"))?;
                if let Some(mapping) = &field.relation {
                    if field.multiple {
                        return Err(invalid("owning relation must be singular"));
                    }
                    if let Some(inverse) = &mapping.inverse
                        && !target.fields.iter().any(|other| {
                            other.name == *inverse
                                && other.value_type == ValueType::Model(model.name.clone())
                        })
                    {
                        return Err(invalid("unknown inverse relation"));
                    }
                    let local = model
                        .fields
                        .iter()
                        .find(|item| item.name == mapping.local)
                        .ok_or_else(|| invalid("unknown local relation field"))?;
                    let foreign = target
                        .fields
                        .iter()
                        .find(|item| item.name == mapping.foreign)
                        .ok_or_else(|| invalid("unknown foreign relation field"))?;
                    if local.multiple
                        || foreign.multiple
                        || !(foreign.unique
                            || foreign.key && target.fields.iter().filter(|f| f.key).count() == 1)
                        || local.value_type != foreign.value_type
                        || local.required != field.required
                    {
                        return Err(invalid("incompatible relation mapping"));
                    }
                } else {
                    let candidates = target
                        .fields
                        .iter()
                        .filter(|other| {
                            other.value_type == ValueType::Model(model.name.clone())
                                && other.relation.as_ref().is_some_and(|mapping| {
                                    mapping.inverse.as_deref() == Some(&field.name)
                                })
                        })
                        .count();
                    let reverse_unique = target
                        .fields
                        .iter()
                        .find(|other| {
                            other.value_type == ValueType::Model(model.name.clone())
                                && other.relation.as_ref().is_some_and(|mapping| {
                                    mapping.inverse.as_deref() == Some(&field.name)
                                })
                        })
                        .and_then(|other| other.relation.as_ref())
                        .and_then(|mapping| {
                            target
                                .fields
                                .iter()
                                .find(|other| other.name == mapping.local)
                        })
                        .is_some_and(|local| {
                            local.unique
                                || local.key
                                    && target.fields.iter().filter(|field| field.key).count() == 1
                        });
                    if candidates != 1 || !field.multiple && !reverse_unique {
                        return Err(invalid("missing or ambiguous reverse relation"));
                    }
                }
                Ok(())
            })()
            .map_err(|error| error.at(path))?;
        }
        Ok(())
    }
}

/// Reject defaults that cannot be represented by the emitted input and storage codec.
fn validate_default(
    field: &Field,
    value: &serde_json::Value,
    config: &SchemaConfig,
) -> SchemaResult<()> {
    if value.is_null() {
        return if field.required {
            Err(invalid("required default cannot be null"))
        } else {
            Ok(())
        };
    }
    if field.multiple {
        let values = value
            .as_array()
            .ok_or_else(|| invalid("list default requires array"))?;
        let mut scalar = field.clone();
        scalar.multiple = false;
        for value in values {
            validate_default(&scalar, value, config)?;
        }
        return Ok(());
    }
    let valid = match &field.value_type {
        ValueType::String => {
            value.is_string()
                && (!field.nonempty || value.as_str().is_some_and(|value| !value.is_empty()))
        }
        ValueType::Integer => value.as_i64().is_some(),
        ValueType::Boolean => value.is_boolean(),
        ValueType::Number => crate::FiniteNumber::from_json(value).is_ok(),
        ValueType::Object => crate::JsonObject::new(value.clone()).is_ok(),
        ValueType::Date => value
            .as_str()
            .is_some_and(|v| crate::DateValue::parse(v).is_ok()),
        ValueType::Time => value
            .as_str()
            .is_some_and(|v| crate::TimeValue::parse(v).is_ok()),
        ValueType::Datetime => value
            .as_str()
            .is_some_and(|v| crate::DatetimeValue::parse(v).is_ok()),
        ValueType::Enum(name) => config.r#enum[name].values().any(|choice| choice == value),
        ValueType::Fieldset(_) => value.is_object(),
        ValueType::Model(_) => false,
    };
    if valid {
        Ok(())
    } else {
        Err(invalid("default does not match field type"))
    }
}

/// Parse exactly one literal relation object; no caller expression is evaluated.
fn relation(value: &AttributeValue) -> SchemaResult<Relation> {
    let AttributeValue::Array(values) = value else {
        return Err(invalid("relation requires one mapping"));
    };
    if values.len() != 1 {
        return Err(invalid("relation requires one mapping"));
    }
    let object = values[0]
        .as_object()
        .ok_or_else(|| invalid("invalid relation mapping"))?;
    if object
        .keys()
        .any(|key| !matches!(key.as_str(), "local" | "foreign" | "name"))
    {
        return Err(invalid("unknown relation mapping member"));
    }
    let local = object
        .get("local")
        .and_then(|value| value.as_str())
        .ok_or_else(|| invalid("missing local relation field"))?;
    let foreign = object
        .get("foreign")
        .and_then(|value| value.as_str())
        .ok_or_else(|| invalid("missing foreign relation field"))?;
    let inverse = object
        .get("name")
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("invalid inverse name"))
        })
        .transpose()?;
    Ok(Relation {
        local: local.into(),
        foreign: foreign.into(),
        inverse,
    })
}

/// Require an actual true flag, rather than ignoring supplied arguments or false.
fn require_flag(value: &AttributeValue) -> SchemaResult<()> {
    if *value == AttributeValue::Boolean(true) {
        Ok(())
    } else {
        Err(invalid("metadata requires a true flag"))
    }
}

/// Preserve the concrete schema diagnostic at the build-host callback boundary.
pub fn transform_error(cause: SchemaError) -> stackpress_idea_transformer::TransformError {
    // This fixed safe context satisfies the checked constructor independently of rejected data.
    stackpress_idea_transformer::TransformError::with_source(
        stackpress_idea_transformer::TransformErrorKind::Callback,
        "framework schema generation failed",
        cause,
    )
    .unwrap_or_else(|error| error)
}

/// Construct an invalid-schema explanation without rejected values.
fn invalid(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, context)
}
/// Mark unsupported selected semantics explicitly before any output can be published.
fn unsupported(context: &'static str) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Unsupported, context)
}
/// Construct one bounded-resource refusal.
fn limit() -> SchemaError {
    SchemaError::new(
        SchemaErrorKind::Limit,
        "schema normalization budget exceeded",
    )
}

mod diagnostics;
mod emit;
mod names;
pub use diagnostics::{DiagnosticLocation, DiagnosticMap};
pub use emit::{emit_contracts, rust_type};
pub use names::portable_name;
use names::{member_name, type_name};

/// Accept exactly homogeneous string/integer/Boolean enum scalar families.
fn enum_family(value: &serde_json::Value) -> SchemaResult<u8> {
    if value.is_string() {
        Ok(0)
    } else if value.as_i64().is_some() {
        Ok(1)
    } else if value.is_boolean() {
        Ok(2)
    } else {
        Err(unsupported("unsupported enum scalar family"))
    }
}

/// Canonicalize nested maps independently of additive serde_json map-order features.
fn canonical(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(values) => {
            let ordered: std::collections::BTreeMap<_, _> = values.into_iter().collect();
            serde_json::Value::Object(
                ordered
                    .into_iter()
                    .map(|(key, value)| (key, canonical(value)))
                    .collect(),
            )
        }
        serde_json::Value::Array(values) => {
            serde_json::Value::Array(values.into_iter().map(canonical).collect())
        }
        value => value,
    }
}
