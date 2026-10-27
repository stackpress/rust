//! Bounded build-host normalization and deterministic Rust contract emission.
//!
//! Both schema and SQL transforms receive the same frozen app-owned definition registry.
//! No mutable cross-transform payload or process-global registration is introduced.

use std::collections::BTreeSet;

use sha2::{Digest, Sha256};
use stackpress_idea_parser::{AttributeValue, SchemaConfig, TypeConfig};

use crate::{
    Enumeration, Field, Model, Relation, Schema, SchemaError, SchemaErrorKind, SchemaResult,
    ValueType,
};

/// Frozen compiled definitions selected by the app before transforms run.
#[derive(Clone, Debug)]
pub struct DefinitionRegistry {
    /// Names and versions are owned by this tool; duplicate registration is rejected.
    definitions: Vec<(String, String)>,
}

impl DefinitionRegistry {
    /// Select the implemented M0 assertion and descriptor-only role vocabulary.
    pub fn standard() -> Self {
        Self {
            definitions: vec![("is.ne".into(), "1".into())],
        }
    }

    /// Construct a frozen catalog; names/versions must be unique and bounded.
    pub fn checked(definitions: Vec<(String, String)>) -> SchemaResult<Self> {
        let mut names = BTreeSet::new();
        // Refuse duplicates before any normalization, callback or filesystem effect.
        if definitions.len() > 256
            || definitions.iter().any(|(name, revision)| {
                name.is_empty()
                    || name.len() > 128
                    || revision.is_empty()
                    || revision.len() > 128
                    || !names.insert(name)
            })
        {
            return Err(invalid("invalid definition registry"));
        }
        Ok(Self { definitions })
    }
}

/// Lowerable direct-config budgets; hard ceilings apply even if callers raise them.
#[derive(Clone, Copy, Debug)]
pub struct NormalizeOptions {
    /// Maximum models/fieldsets/enums together, capped at 128.
    pub declarations: usize,
    /// Maximum fields per declaration, capped at 128.
    pub fields: usize,
}

impl Default for NormalizeOptions {
    /// Supply finite M0 generation bounds.
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
            if !modules.insert(module.clone()) || choices.is_empty() || choices.len() > 128 {
                return Err(invalid("enum name collision or choice budget"));
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
                let value = value
                    .as_str()
                    .ok_or_else(|| unsupported("M0 requires string-valued enums"))?;
                let variant = type_name(variant)?;
                if value.len() > 4096 || !values.insert(value) || !variants.insert(variant.clone())
                {
                    return Err(invalid("duplicate enum identity"));
                }
                enumeration.variants.push((variant, value.into()));
            }
            schema.enums.push(enumeration);
        }
        // Register identities before resolving relations; model graphs may be cyclic.
        for (embedded, declarations) in [(true, &config.r#type), (false, &config.model)] {
            for (name, declaration) in declarations {
                if name != &declaration.name {
                    return Err(invalid("declaration identity mismatch"));
                }
                let module = portable_name(name)?;
                if !modules.insert(module.clone()) {
                    return Err(invalid("declaration name collision"));
                }
                let model = normalize_model(declaration, config, definitions, options, embedded)?;
                if embedded {
                    schema.fieldsets.push(model);
                } else {
                    schema.models.push(model);
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
        Ok(schema)
    }

    /// Compute a stable semantic fingerprint from framework-owned ordered descriptors.
    pub fn fingerprint(&self) -> SchemaResult<String> {
        let bytes = serde_json::to_vec(self).map_err(|cause| {
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
            return Err(invalid("embedded value cycle or depth limit"));
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
            let target = self
                .models
                .iter()
                .find(|item| &item.name == target)
                .ok_or_else(|| invalid("unknown relation target"))?;
            if let Some(mapping) = &field.relation {
                if field.multiple {
                    return Err(invalid("owning relation must be singular"));
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
                    || !foreign.key
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
                if !field.multiple || candidates != 1 {
                    return Err(invalid("missing or ambiguous reverse relation"));
                }
            }
        }
        Ok(())
    }
}

/// One total accounting owner for all direct-config names and metadata values.
#[derive(Default)]
struct NormalizationBudget {
    /// Total metadata/declaration nodes visited, including individually small defaults.
    nodes: usize,
    /// Logical source-name and scalar/key bytes, independent of output encoding expansion.
    bytes: usize,
}

impl NormalizationBudget {
    /// Charge before any descriptor clone; no counter may wrap or reset between declarations.
    fn charge(&mut self, nodes: usize, bytes: usize) -> SchemaResult<()> {
        self.nodes = self.nodes.checked_add(nodes).ok_or_else(limit)?;
        self.bytes = self.bytes.checked_add(bytes).ok_or_else(limit)?;
        if self.nodes > 100_000 || self.bytes > 1024 * 1024 {
            return Err(limit());
        }
        Ok(())
    }
}

/// Bound every identity and aggregate value before resolving references to later declarations.
fn preflight(config: &SchemaConfig, options: NormalizeOptions) -> SchemaResult<()> {
    let mut budget = NormalizationBudget::default();
    for (name, choices) in &config.r#enum {
        portable_name(name)?;
        budget.charge(1, name.len())?;
        if choices.len() > 128 {
            return Err(limit());
        }
        for (variant, value) in choices {
            portable_name(variant)?;
            // Unsupported enum objects are rejected here without cloning their nested data.
            let value = value
                .as_str()
                .ok_or_else(|| unsupported("M0 requires string-valued enums"))?;
            budget.charge(1, variant.len().checked_add(value.len()).ok_or_else(limit)?)?;
        }
    }
    for declarations in [&config.r#type, &config.model] {
        for (name, declaration) in declarations {
            portable_name(name)?;
            if name != &declaration.name {
                return Err(invalid("declaration identity mismatch"));
            }
            budget.charge(1, name.len())?;
            if declaration.columns.len() > options.fields.min(128)
                || declaration.attributes.len() > 128
            {
                return Err(limit());
            }
            for (name, value) in &declaration.attributes {
                charge_attribute(&mut budget, name, value)?;
            }
            for column in &declaration.columns {
                portable_name(&column.name)?;
                // A reference to an invalid later declaration must not cause an early huge clone.
                if column.column_type.len() > 512 || column.attributes.len() > 128 {
                    return Err(limit());
                }
                budget.charge(1, column.name.len() + column.column_type.len())?;
                for (name, value) in &column.attributes {
                    charge_attribute(&mut budget, name, value)?;
                }
            }
        }
    }
    Ok(())
}

/// Charge one independently bounded metadata tree to the same schema-wide owner.
fn charge_attribute(
    budget: &mut NormalizationBudget,
    name: &str,
    value: &AttributeValue,
) -> SchemaResult<()> {
    if name.len() > 128 {
        return Err(limit());
    }
    let (nodes, bytes) = validate_metadata_value(value)?;
    budget.charge(nodes, name.len().checked_add(bytes).ok_or_else(limit)?)
}

/// Normalize fields only after validating names, flags and selected metadata.
fn normalize_model(
    declaration: &TypeConfig,
    config: &SchemaConfig,
    registry: &DefinitionRegistry,
    options: NormalizeOptions,
    embedded: bool,
) -> SchemaResult<Model> {
    if declaration.columns.is_empty() || declaration.columns.len() > options.fields.min(128) {
        return Err(limit());
    }
    for (attribute, value) in &declaration.attributes {
        validate_metadata_value(value)?;
        if !matches!(
            attribute.as_str(),
            "labels" | "query" | "description" | "label" | "display" | "icon"
        ) {
            return Err(unsupported("M0 model metadata is unsupported"));
        }
    }
    let mut model = Model {
        name: declaration.name.clone(),
        rust_name: type_name(&declaration.name)?,
        module: portable_name(&declaration.name)?,
        fields: Vec::new(),
    };
    let mut columns = BTreeSet::new();
    for column in &declaration.columns {
        let name = portable_name(&column.name)?;
        if !columns.insert(name.clone()) {
            return Err(invalid("field name collision"));
        }
        let value_type = match column.column_type.as_str() {
            "String" | "Text" => ValueType::String,
            "Integer" => ValueType::Integer,
            "Boolean" => ValueType::Boolean,
            "Number" | "Float" | "Json" | "Object" | "Hash" => {
                return Err(unsupported(
                    "M0 bounded numeric/object value wrappers are unavailable",
                ));
            }
            name if config.r#enum.contains_key(name) => ValueType::Enum(name.into()),
            name if config.r#type.contains_key(name) => ValueType::Fieldset(name.into()),
            name if config.model.contains_key(name) => ValueType::Model(name.into()),
            _ => return Err(unsupported("M0 field type is unsupported")),
        };
        let mut field = Field {
            name: column.name.clone(),
            rust_name: member_name(&name),
            column: name,
            value_type,
            required: column.required,
            multiple: column.multiple,
            key: false,
            default: None,
            nonempty: false,
            active: false,
            relation: None,
        };
        for (attribute, value) in &column.attributes {
            validate_metadata_value(value)?;
            match attribute.as_str() {
                "id" => {
                    require_flag(value)?;
                    field.key = true;
                }
                "active" => {
                    require_flag(value)?;
                    field.active = true;
                }
                "is.ne" => {
                    require_flag(value)?;
                    if !registry
                        .definitions
                        .iter()
                        .any(|(name, revision)| name == attribute && revision == "1")
                    {
                        return Err(unsupported("missing assertion definition"));
                    }
                    field.nonempty = true;
                }
                "default" => {
                    let AttributeValue::Array(values) = value else {
                        return Err(invalid("default requires one argument"));
                    };
                    if values.len() != 1 {
                        return Err(invalid("default requires one argument"));
                    }
                    field.default = Some(values[0].clone());
                }
                "relation" => field.relation = Some(relation(value)?),
                // Roles are retained in source only at M0; selecting UI output requires M1 descriptors.
                "searchable" | "sortable" | "field.input" | "view.text" | "list.text"
                | "filter.select" | "filter.checkbox" | "list.yesno" => require_flag(value)?,
                _ => return Err(unsupported("M0 field metadata is unsupported")),
            }
        }
        // Reject invalid state before code generation can emit a misleading stored shape.
        if field.key
            && (!field.required
                || field.multiple
                || !matches!(field.value_type, ValueType::String | ValueType::Integer))
        {
            return Err(invalid("identity must be a required scalar"));
        }
        if field.active
            && (field.value_type != ValueType::Boolean || !field.required || field.multiple)
        {
            return Err(invalid("active must be a required Boolean"));
        }
        if field.nonempty && (field.value_type != ValueType::String || field.multiple) {
            return Err(unsupported("M0 nonempty assertion requires text"));
        }
        // Relation metadata on a scalar would otherwise be discarded by store emission.
        // Only actual virtual model references may declare an owning mapping.
        if field.relation.is_some() && !matches!(field.value_type, ValueType::Model(_)) {
            return Err(invalid("relation mapping requires a model reference"));
        }
        if embedded && matches!(field.value_type, ValueType::Model(_)) {
            return Err(unsupported("embedded model relations are unsupported"));
        }
        // The active role supplies true only when no explicit literal was declared.
        // An explicit false remains false rather than being overwritten by a truthy fallback.
        if field.active && field.default.is_none() {
            field.default = Some(serde_json::Value::Bool(true));
        }
        if let Some(value) = &field.default {
            // Known provider calls are not literal fallback strings. M0 has no provider
            // registry yet, so refuse these selected semantics before generating Rust.
            if value.as_str().is_some_and(|text| {
                matches!(text, "now()" | "cuid()" | "nano()" | "random()")
                    || text.starts_with("nanoid(")
            }) {
                return Err(unsupported(
                    "M0 default provider registration is unavailable",
                ));
            }
            validate_default(&field, value, config)?;
        }
        model.fields.push(field);
    }
    if !embedded
        && (model.fields.iter().filter(|field| field.key).count() != 1
            || model.fields.iter().filter(|field| field.active).count() > 1)
    {
        return Err(unsupported("M0 stores require exactly one scalar identity"));
    }
    Ok(model)
}

/// Validate every metadata node iteratively, before cloning any direct caller-owned value.
fn validate_metadata_value(attribute: &AttributeValue) -> SchemaResult<(usize, usize)> {
    let AttributeValue::Array(values) = attribute else {
        return Ok((1, 1));
    };
    if values.len() > 10_000 {
        return Err(limit());
    }
    let mut pending: Vec<_> = values.iter().map(|value| (value, 0)).collect();
    let mut nodes = 0_usize;
    let mut bytes = 0_usize;
    while let Some((value, depth)) = pending.pop() {
        nodes += 1;
        if nodes > 10_000 || depth > 32 {
            return Err(limit());
        }
        match value {
            serde_json::Value::Array(values) => {
                if values.len() > 10_000 - nodes.min(10_000) {
                    return Err(limit());
                }
                pending.extend(values.iter().map(|value| (value, depth + 1)));
            }
            serde_json::Value::Object(values) => {
                if values.len() > 10_000 - nodes.min(10_000) {
                    return Err(limit());
                }
                for (key, value) in values {
                    bytes += key.len();
                    pending.push((value, depth + 1));
                }
            }
            serde_json::Value::String(value) => bytes += value.len(),
            _ => bytes += 8,
        }
        if pending.len() > 10_000 || bytes > 1024 * 1024 {
            return Err(limit());
        }
    }
    Ok((nodes, bytes))
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
        ValueType::Number => value.as_f64().is_some_and(f64::is_finite),
        ValueType::Object => value.is_object(),
        ValueType::Enum(name) => config.r#enum[name].values().any(|choice| choice == value),
        ValueType::Fieldset(_) | ValueType::Model(_) => false,
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
/// Mark unavailable M0 semantics explicitly, leaving full breadth to M1.
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
