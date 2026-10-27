//! Normalize selected semantic attributes in source order, retaining exact logical error paths.

use super::{
    DefinitionRegistry, NormalizeOptions, invalid, limit, member_name, portable_name, relation,
    require_flag, type_name, unsupported, validate_default,
};
use crate::*;
use serde_json::Value;
use stackpress_idea_parser::{AttributeValue, SchemaConfig, TypeConfig};
use std::collections::BTreeSet;

/// Normalize one checked declaration. Model cycles are resolved in a later relation pass.
pub(super) fn normalize_model(
    declaration: &TypeConfig,
    config: &SchemaConfig,
    registry: &DefinitionRegistry,
    options: NormalizeOptions,
    embedded: bool,
    resolved_roles: &mut crate::values::JsonBudget,
) -> SchemaResult<Model> {
    let root = vec![
        if embedded { "type" } else { "model" }.into(),
        declaration.name.clone(),
    ];
    if declaration.columns.is_empty() || declaration.columns.len() > options.fields.min(128) {
        return Err(limit().at(root));
    }
    let mut model = Model {
        name: declaration.name.clone(),
        rust_name: type_name(&declaration.name)?,
        module: registry
            .declaration_name(&declaration.name)
            .map(str::to_owned)
            .unwrap_or(portable_name(&declaration.name)?),
        fields: Vec::new(),
        documentation: Documentation::default(),
        query: Vec::new(),
        attributes: Vec::new(),
    };
    // Preserve private source attributes while interpreting only this owner's selected vocabulary.
    for (name, value) in &declaration.attributes {
        let result = (|| {
            model.attributes.push(attribute(name, value));
            if document(&mut model.documentation, name, value)? {
                return Ok(());
            }
            if name == "query" {
                model.query = strings(value)?;
                Ok(())
            } else if foreign(name) {
                Ok(())
            } else {
                Err(unsupported("unknown selected model attribute"))
            }
        })();
        result.map_err(|error: SchemaError| {
            error.at([root.clone(), vec!["attributes".into(), name.clone()]].concat())
        })?;
    }
    // Resolve every column identity/type before its attributes and final cross-attribute invariants.
    let mut names = BTreeSet::new();
    for column in &declaration.columns {
        let path = [root.clone(), vec!["columns".into(), column.name.clone()]].concat();
        let name = registry
            .name(&declaration.name, &column.name)
            .map(str::to_owned)
            .unwrap_or(portable_name(&column.name)?);
        if !names.insert(name.clone()) {
            return Err(invalid("field name collision").at(path));
        }
        let value_type = match column.column_type.as_str() {
            "String" | "Text" => ValueType::String,
            "Integer" => ValueType::Integer,
            "Number" | "Float" => ValueType::Number,
            "Boolean" => ValueType::Boolean,
            "Date" => ValueType::Date,
            "Time" => ValueType::Time,
            "Datetime" => ValueType::Datetime,
            "Json" | "Object" | "Hash" => ValueType::Object,
            name if config.r#enum.contains_key(name) => ValueType::Enum(name.into()),
            name if config.r#type.contains_key(name) => ValueType::Fieldset(name.into()),
            name if config.model.contains_key(name) => ValueType::Model(name.into()),
            _ => return Err(unsupported("unknown selected field type").at(path)),
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
            unique: false,
            searchable: false,
            sortable: false,
            timestamp: false,
            generated: false,
            input_required: false,
            value_rule: None,
            rules: Vec::new(),
            documentation: Documentation::default(),
            roles: Vec::new(),
            access: registry.access(&declaration.name, &column.name),
            attributes: Vec::new(),
        };
        for (name, value) in &column.attributes {
            field.attributes.push(attribute(name, value));
            normalize_attribute(&mut field, name, value, registry, resolved_roles).map_err(
                |error| error.at([path.clone(), vec!["attributes".into(), name.clone()]].concat()),
            )?;
        }
        // Recognized default calls become explicit providers before generated/input requirements settle.
        provider(&mut field, registry).map_err(|error| {
            error.at([path.clone(), vec!["attributes".into(), "default".into()]].concat())
        })?;
        finish(&mut field, embedded).map_err(|error| error.at(path.clone()))?;
        if let Some(value) = &field.default {
            validate_default(&field, value, config).map_err(|error| {
                error.at([path, vec!["attributes".into(), "default".into()]].concat())
            })?;
        }
        model.fields.push(field);
    }
    if !embedded
        && (model.fields.iter().filter(|v| v.key).count() == 0
            || model.fields.iter().filter(|v| v.active).count() > 1
            || model.fields.iter().filter(|v| v.timestamp).count() > 1)
    {
        return Err(invalid("invalid model identity or state roles").at(root));
    }
    // Resolve every intermediate segment; a scalar must never restart lookup at the root.
    for query in &model.query {
        let mut declaration = declaration;
        let segments: Vec<_> = query.split('.').collect();
        if segments.len() > 8 {
            return Err(limit().at(root.clone()));
        }
        for (index, segment) in segments.iter().enumerate() {
            let column = declaration
                .columns
                .iter()
                .find(|v| v.name == *segment)
                .ok_or_else(|| invalid("unknown query path").at(root.clone()))?;
            if index + 1 < segments.len() {
                declaration = config
                    .model
                    .get(&column.column_type)
                    .or_else(|| config.r#type.get(&column.column_type))
                    .ok_or_else(|| invalid("query path traverses a scalar").at(root.clone()))?;
            }
        }
    }
    Ok(model)
}
/// Interpret a single selected attribute; foreign owners remain private and inert.
fn normalize_attribute(
    field: &mut Field,
    name: &str,
    value: &AttributeValue,
    registry: &DefinitionRegistry,
    resolved_roles: &mut crate::values::JsonBudget,
) -> SchemaResult<()> {
    if document(&mut field.documentation, name, value)? {
        return Ok(());
    }
    match name {
        "id" | "unique" | "active" | "timestamp" | "generated" | "searchable" | "sortable" => {
            require_flag(value)?;
            match name {
                "id" => field.key = true,
                "unique" => field.unique = true,
                "active" => field.active = true,
                "timestamp" => field.timestamp = true,
                "generated" => field.generated = true,
                "searchable" => field.searchable = true,
                _ => field.sortable = true,
            };
        }
        "relation" => field.relation = Some(relation(value)?),
        "default" => {
            let args = args(value)?;
            if args.len() != 1 {
                return Err(invalid("default requires one argument"));
            }
            field.default = Some(args[0].clone());
        }
        "min" | "max" | "step" => {
            let args = args(value)?;
            if args.len() != 1 || !args[0].as_f64().is_some_and(f64::is_finite) {
                return Err(invalid("invalid bound argument"));
            }
            let text = field.value_type == ValueType::String && !field.multiple;
            if field.multiple
                || !matches!(
                    field.value_type,
                    ValueType::Integer | ValueType::Number | ValueType::String
                )
                || name == "step" && (text || args[0].as_f64().unwrap_or(0.0) <= 0.0)
            {
                return Err(invalid("bound is incompatible with field type"));
            }
            // Integer steps stay exact over the full i64 range; fractional steps belong to Number.
            if name == "step"
                && field.value_type == ValueType::Integer
                && args[0].as_i64().is_none()
            {
                return Err(invalid("Integer step requires a positive integer"));
            }
            let code = match (name, text) {
                ("min", true) => "cge",
                ("max", true) => "cle",
                ("min", false) => "ge",
                ("max", false) => "le",
                _ => "step",
            };
            field.rules.push(rule(code, args, true));
        }
        "hashed" | "encrypted" => {
            return Err(unsupported("security transformation is not registered"));
        }
        name if name.starts_with("is.") => {
            super::assertions::normalize(field, &name[3..], value, registry)?
        }
        name if ["field.", "filter.", "span.", "list.", "view."]
            .iter()
            .any(|prefix| name.starts_with(prefix)) =>
        {
            let binding = registry.role(name, value)?;
            // Charge the resolved definition props across the entire schema, not per flag.
            resolved_roles.charge(&crate::to_bounded_value(&binding)?)?;
            if field.roles.iter().any(|v| v.role == binding.role) {
                return Err(invalid("multiple bindings for one field role"));
            }
            field.roles.push(binding);
        }
        name if foreign(name) => {}
        _ => return Err(unsupported("unknown selected field attribute")),
    }
    Ok(())
}
/// Derive state/default invariants only after all attributes have been interpreted.
fn finish(field: &mut Field, embedded: bool) -> SchemaResult<()> {
    // Reject storage-only semantics on embeddings and input semantics on virtual relations first.
    if embedded
        && (field.key
            || field.unique
            || field.searchable
            || field.sortable
            || field.active
            || field.timestamp)
    {
        return Err(unsupported(
            "embedded storage attributes require explicit JSON storage support",
        ));
    }
    if matches!(field.value_type, ValueType::Model(_))
        && (!field.rules.is_empty()
            || field.generated
            || field.default.is_some()
            || field.value_rule.is_some())
    {
        return Err(unsupported(
            "virtual relation values cannot declare input defaults or assertions",
        ));
    }
    // Keys, uniqueness and native indexing require an unambiguous scalar storage representation.
    let scalar = !field.multiple
        && !matches!(
            field.value_type,
            ValueType::Object | ValueType::Fieldset(_) | ValueType::Model(_)
        );
    if field.key && (!field.required || !scalar) {
        return Err(invalid("identity requires a nonnullable scalar"));
    }
    if field.unique && !scalar {
        return Err(invalid("unique requires a scalar"));
    }
    if (field.searchable || field.sortable) && !scalar {
        return Err(unsupported("query indexing requires a supported scalar"));
    }
    if field.active && (field.value_type != ValueType::Boolean || !field.required || field.multiple)
    {
        return Err(invalid("active requires a nonnullable Boolean"));
    }
    if field.timestamp
        && (field.value_type != ValueType::Datetime || !field.required || field.multiple)
    {
        return Err(invalid("timestamp requires a nonnullable Datetime"));
    }
    if field.relation.is_some() && !matches!(field.value_type, ValueType::Model(_)) {
        return Err(invalid("relation requires a model reference"));
    }
    if embedded && matches!(field.value_type, ValueType::Model(_)) {
        return Err(unsupported("embedded relations are unsupported"));
    }
    // Derive defaults only after applicability checks, preserving an explicitly supplied false value.
    if field.active && field.default.is_none() {
        field.default = Some(Value::Bool(true));
    }
    if field.timestamp {
        field.generated = true;
        if field.default.is_none() {
            field.value_rule = Some(ValueRule {
                provider: "now".into(),
                args: Vec::new(),
                revision: "1".into(),
            });
        }
    }
    // Omission is valid only when a literal/provider or generated source can supply the stored value.
    field.input_required = (field.required
        || field
            .rules
            .iter()
            .any(|rule| matches!(rule.code.as_str(), "required" | "ne")))
        && field.default.is_none()
        && field.value_rule.is_none()
        && !field.generated;
    field.roles.sort_by_key(|v| v.role);
    if field.generated && field.value_rule.is_none() {
        return Err(unsupported(
            "generated field requires an explicit typed provider",
        ));
    }
    Ok(())
}
/// Require a finite literal method argument list.
fn args(value: &AttributeValue) -> SchemaResult<Vec<Value>> {
    match value {
        AttributeValue::Array(args) => Ok(args.clone()),
        _ => Err(invalid("attribute requires method arguments")),
    }
}
/// Read plain descriptive values as data only.
fn document(doc: &mut Documentation, name: &str, value: &AttributeValue) -> SchemaResult<bool> {
    if !matches!(
        name,
        "label" | "description" | "examples" | "display" | "icon" | "labels"
    ) {
        return Ok(false);
    }
    let args = args(value)?;
    if name == "examples" {
        doc.examples = args;
    } else if name == "labels" {
        if args.len() != 2 {
            return Err(invalid("labels require singular and plural text"));
        }
        doc.labels = args
            .iter()
            .map(|v| {
                v.as_str()
                    .map(str::to_owned)
                    .ok_or_else(|| invalid("labels require text"))
            })
            .collect::<SchemaResult<_>>()?;
    } else {
        if args.len() != 1 {
            return Err(invalid("documentation requires one argument"));
        }
        let value = args[0]
            .as_str()
            .ok_or_else(|| invalid("documentation requires text"))?
            .into();
        match name {
            "label" => doc.label = Some(value),
            "description" => doc.description = Some(value),
            "display" => doc.display = Some(value),
            _ => doc.icon = Some(value),
        };
    }
    Ok(true)
}
/// Accept string lists for query configuration, preserving order.
fn strings(value: &AttributeValue) -> SchemaResult<Vec<String>> {
    args(value)?
        .iter()
        .map(|v| {
            v.as_str()
                .map(str::to_owned)
                .ok_or_else(|| invalid("query requires text paths"))
        })
        .collect()
}
/// Retain flags and method arguments without evaluating foreign metadata.
fn attribute(name: &str, value: &AttributeValue) -> Attribute {
    Attribute {
        name: name.into(),
        value: match value {
            AttributeValue::Boolean(v) => Value::Bool(*v),
            AttributeValue::Array(v) => Value::Array(v.clone()),
        },
    }
}
/// Only namespaced foreign attributes are inert; selected owner misspellings fail.
fn foreign(name: &str) -> bool {
    name.contains('.')
        && !name.starts_with("is.")
        && !matches!(
            name.split('.').next(),
            Some("field" | "filter" | "span" | "view" | "list" | "store")
        )
}
/// Author stable bounded rule messages independently of rejected input.
pub(super) fn rule(code: &str, args: Vec<Value>, client_checkable: bool) -> ValidationRule {
    ValidationRule {
        code: code.into(),
        args,
        client_checkable,
        message: "field validation failed".into(),
        definition_revision: "1".into(),
    }
}

/// Interpret only the finite recognized default-call table; other strings remain literal data.
fn provider(field: &mut Field, registry: &DefinitionRegistry) -> SchemaResult<()> {
    let Some(text) = field.default.as_ref().and_then(Value::as_str) else {
        return Ok(());
    };
    let (name, args) = match text {
        "now()" => ("now", Vec::new()),
        "cuid()" => ("cuid", Vec::new()),
        "nano()" => ("nano", Vec::new()),
        "random()" => ("random", Vec::new()),
        text if text.starts_with("nanoid(") => {
            let length = text
                .strip_prefix("nanoid(")
                .and_then(|s| s.strip_suffix(')'))
                .and_then(|s| s.parse::<u16>().ok())
                .filter(|n| (1..=1024).contains(n))
                .ok_or_else(|| invalid("invalid nanoid provider length"))?;
            ("nanoid", vec![Value::from(length)])
        }
        _ => return Ok(()),
    };
    if field.multiple {
        return Err(invalid("provider requires a scalar field"));
    }
    let revision = if name == "now" {
        if field.value_type != ValueType::Datetime {
            return Err(invalid("now requires Datetime"));
        }
        "1".into()
    } else {
        let definition = registry
            .providers
            .iter()
            .find(|v| v.name == name)
            .ok_or_else(|| unsupported("default provider is not registered"))?;
        if definition.output != field.value_type {
            return Err(invalid("provider output type mismatch"));
        }
        definition.revision.clone()
    };
    field.value_rule = Some(ValueRule {
        provider: name.into(),
        args,
        revision,
    });
    field.default = None;
    Ok(())
}

/// Normalize date/time literals recursively after the complete acyclic type graph is available.
pub(super) fn canonical_default(
    field: &Field,
    value: Value,
    fieldsets: &[Model],
) -> SchemaResult<Value> {
    if value.is_null() {
        return Ok(value);
    }
    if field.multiple {
        let Value::Array(values) = value else {
            return Err(invalid("list default requires array"));
        };
        let mut scalar = field.clone();
        scalar.multiple = false;
        return values
            .into_iter()
            .map(|value| canonical_default(&scalar, value, fieldsets))
            .collect::<SchemaResult<Vec<_>>>()
            .map(Value::Array);
    }
    let text = || {
        value
            .as_str()
            .ok_or_else(|| invalid("canonical default requires text"))
    };
    match &field.value_type {
        ValueType::Date => Ok(Value::String(DateValue::parse(text()?)?.as_str().into())),
        ValueType::Time => Ok(Value::String(TimeValue::parse(text()?)?.as_str().into())),
        ValueType::Datetime => Ok(Value::String(
            DatetimeValue::parse(text()?)?.as_str().into(),
        )),
        ValueType::Fieldset(name) => {
            let model = fieldsets
                .iter()
                .find(|model| model.name == *name)
                .ok_or_else(|| invalid("unknown default fieldset"))?;
            let Value::Object(mut values) = value else {
                return Err(invalid("fieldset default requires object"));
            };
            for field in &model.fields {
                if let Some(value) = values.remove(&field.name) {
                    values.insert(
                        field.name.clone(),
                        canonical_default(field, value, fieldsets)?,
                    );
                }
            }
            Ok(Value::Object(values))
        }
        _ => Ok(value),
    }
}

/// Canonicalize one temporal comparison/default without cloning unrelated field metadata.
pub(super) fn canonical_temporal(kind: &ValueType, value: &Value) -> SchemaResult<Value> {
    let text = value
        .as_str()
        .ok_or_else(|| invalid("canonical temporal literal requires text"))?;
    let text = match kind {
        ValueType::Date => DateValue::parse(text)?.as_str().to_owned(),
        ValueType::Time => TimeValue::parse(text)?.as_str().to_owned(),
        ValueType::Datetime => DatetimeValue::parse(text)?.as_str().to_owned(),
        _ => return Err(invalid("temporal type required")),
    };
    Ok(Value::String(text))
}
