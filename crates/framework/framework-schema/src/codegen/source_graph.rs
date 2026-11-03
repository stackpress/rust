//! Source normalization owns the original metadata vocabulary, independently of Typed policy.
use super::{
    NormalizeOptions, invalid, limit, member_name, metadata, portable_name, relation, type_name,
};
use crate::*;
use stackpress_idea_parser::{AttributeValue, SchemaConfig, TypeConfig};
use std::collections::BTreeSet;

/// Source-normalized metadata, kept distinct from the Typed emitter's Schema input.
/// This prevents an unimplemented source emitter from silently emitting strict DTOs.
///
/// ```compile_fail,E0308
/// use stackpress_framework_schema::{Schema,codegen::NormalizeOptions};
/// use stackpress_idea_parser::{parse,CompileInputs};
/// let config=parse("model.idea","model Entry { id String @id }",&CompileInputs::default()).unwrap();
/// let source=Schema::normalize_source(&config,NormalizeOptions::default(),&[]).unwrap();
/// fn typed_only(_: &Schema) {}
/// typed_only(&source);
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct SourceGraph {
    models: Vec<Model>,
    external_models: Vec<String>,
}
impl SourceGraph {
    /// Complete source models in composed catalog order, including every private attribute.
    pub fn models(&self) -> &[Model] {
        &self.models
    }
    /// Explicit virtual targets; these are never synthesized as models/tables.
    pub fn external_models(&self) -> &[String] {
        &self.external_models
    }
}

impl Schema {
    /// Normalize a composed source graph without rewriting its attributes or query paths.
    ///
    /// External model names are explicitly declared virtual references, never synthesized
    /// tables. Source role arguments stay exact in `attributes`; they are not resolved through
    /// Typed component definitions. Source emission is still separately refused.
    ///
    /// ```
    /// use stackpress_framework_schema::{Schema, codegen::NormalizeOptions};
    /// use stackpress_idea_parser::{parse, CompileInputs};
    /// let config = parse("model.idea", "model Auth { id String @id token String @encrypted }", &CompileInputs::default()).unwrap();
    /// let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[]).unwrap();
    /// assert!(graph.models()[0].fields[1].attributes.iter().any(|a| a.name == "encrypted"));
    /// ```
    /// # Errors
    /// Rejects unresolved inputs, unknown types/targets, invalid mappings and bounded budgets.
    pub fn normalize_source(
        config: &SchemaConfig,
        options: NormalizeOptions,
        external_models: &[&str],
    ) -> SchemaResult<SourceGraph> {
        if !config.r#use.is_empty() || !config.prop.is_empty() {
            return Err(invalid("schema must be composed and finalized"));
        }
        super::preflight(config, options)?;
        if config.model.len() + config.r#type.len() + config.r#enum.len()
            > options.declarations.min(128)
        {
            return Err(limit());
        }
        let external: BTreeSet<_> = external_models.iter().copied().collect();
        if external.len() != external_models.len()
            || external.iter().any(|name| config.model.contains_key(*name))
        {
            return Err(invalid("external model must be distinct and undeclared"));
        }
        // No enum/fieldset approximation: each source construct needs its own native emission.
        if !config.r#enum.is_empty() || !config.r#type.is_empty() {
            return Err(super::unsupported(
                "source enum/fieldset normalization is not implemented",
            ));
        }
        let mut schema = Self {
            models: Vec::new(),
            fieldsets: Vec::new(),
            enums: Vec::new(),
        };
        let mut modules = BTreeSet::new();
        for declaration in config.model.values() {
            let model = normalize_model(declaration, config, &external)?;
            if !modules.insert(model.module.clone()) {
                return Err(invalid("declaration name collision"));
            }
            schema.models.push(model);
        }
        // Check real owning mappings; reverse lists derive their sole inverse without mutating it.
        for model in &schema.models {
            check_relations(&schema, model, &external)?;
        }
        Ok(SourceGraph {
            models: schema.models,
            external_models: external_models
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        })
    }
}

/// Preserve every attribute verbatim while resolving the source storage contract.
fn normalize_model(
    declaration: &TypeConfig,
    config: &SchemaConfig,
    external: &BTreeSet<&str>,
) -> SchemaResult<Model> {
    let mut model = Model {
        name: declaration.name.clone(),
        rust_name: type_name(&declaration.name)?,
        module: portable_name(&declaration.name)?,
        fields: Vec::new(),
        documentation: Default::default(),
        query: Vec::new(),
        attributes: Vec::new(),
    };
    for (name, value) in &declaration.attributes {
        model.attributes.push(metadata::attribute(name, value));
        metadata::document(&mut model.documentation, name, value)?;
        if name == "query" {
            model.query = arguments(value)?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| invalid("query requires text paths"))
                })
                .collect::<SchemaResult<_>>()?;
        }
    }
    let mut names = BTreeSet::new();
    for column in &declaration.columns {
        let field = normalize_field(column, config, external)?;
        if !names.insert(field.column.clone()) {
            return Err(invalid("field name collision"));
        }
        model.fields.push(field);
    }
    // Wildcards retain their spelling; validate only the traversed declaration prefix.
    for path in &model.query {
        let mut current = declaration;
        let segments: Vec<_> = path.split('.').collect();
        if segments.len() > 8 {
            return Err(limit());
        }
        for (index, segment) in segments.iter().enumerate() {
            if *segment == "*" && index + 1 == segments.len() {
                break;
            }
            let column = current
                .columns
                .iter()
                .find(|f| f.name == *segment)
                .ok_or_else(|| invalid("unknown query path"))?;
            if index + 1 < segments.len() {
                current = config
                    .model
                    .get(&column.column_type)
                    .ok_or_else(|| invalid("query path traverses unresolved target"))?;
            }
        }
    }
    Ok(model)
}

/// Source reverse collections choose the sole declared owning edge (no inserted inverse metadata).
fn check_relations(schema: &Schema, model: &Model, external: &BTreeSet<&str>) -> SchemaResult<()> {
    for field in &model.fields {
        let ValueType::Model(name) = &field.value_type else {
            continue;
        };
        if external.contains(name.as_str()) {
            continue;
        }
        let target = schema
            .models
            .iter()
            .find(|m| m.name == *name)
            .ok_or_else(|| invalid("unknown relation target"))?;
        if let Some(mapping) = &field.relation {
            let local = model
                .fields
                .iter()
                .find(|f| f.name == mapping.local)
                .ok_or_else(|| invalid("unknown local relation field"))?;
            let foreign = target
                .fields
                .iter()
                .find(|f| f.name == mapping.foreign)
                .ok_or_else(|| invalid("unknown foreign relation field"))?;
            if field.multiple
                || local.value_type != foreign.value_type
                || local.multiple
                || foreign.multiple
                || !(foreign.key || foreign.unique)
            {
                return Err(invalid("incompatible relation mapping"));
            }
        } else if target
            .fields
            .iter()
            .filter(|f| {
                f.value_type == ValueType::Model(model.name.clone()) && f.relation.is_some()
            })
            .count()
            != 1
        {
            return Err(invalid("missing or ambiguous reverse relation"));
        }
    }
    Ok(())
}
/// Read source method arguments without changing positional/object spellings.
fn arguments(value: &AttributeValue) -> SchemaResult<&[serde_json::Value]> {
    if let AttributeValue::Array(values) = value {
        Ok(values)
    } else {
        Err(invalid("attribute requires method arguments"))
    }
}

/// Normalize one source column while retaining its declared type and all exact attributes.
fn normalize_field(
    column: &stackpress_idea_parser::ColumnConfig,
    config: &SchemaConfig,
    external: &BTreeSet<&str>,
) -> SchemaResult<Field> {
    let column_name = portable_name(&column.name)?;
    let value_type = match column.column_type.as_str() {
        "String" | "Text" => ValueType::String,
        "Integer" => ValueType::Integer,
        "Boolean" => ValueType::Boolean,
        "Number" | "Float" => ValueType::Number,
        "Hash" | "Object" | "Json" => ValueType::Object,
        "Date" => ValueType::Date,
        "Time" => ValueType::Time,
        "Datetime" => ValueType::Datetime,
        name if config.model.contains_key(name) || external.contains(name) => {
            ValueType::Model(name.into())
        }
        _ => return Err(super::unsupported("unknown source field type")),
    };
    let mut field = Field {
        name: column.name.clone(),
        rust_name: member_name(&column_name),
        column: column_name,
        value_type,
        storage_length: None,
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
        documentation: Default::default(),
        roles: Vec::new(),
        access: Default::default(),
        attributes: Vec::new(),
    };
    for (name, value) in &column.attributes {
        field.attributes.push(metadata::attribute(name, value));
        if metadata::document(&mut field.documentation, name, value)? {
            continue;
        }
        normalize_attribute(&mut field, name, value)?;
    }
    field.input_required = field.required
        && field.default.is_none()
        && field.value_rule.is_none()
        && !matches!(field.value_type, ValueType::Model(_));
    Ok(field)
}

/// Interpret storage/assertion metadata; presentation arguments remain exact source attributes.
fn normalize_attribute(field: &mut Field, name: &str, value: &AttributeValue) -> SchemaResult<()> {
    match name {
        "id" => field.key = true,
        "unique" => field.unique = true,
        "active" => field.active = true,
        "timestamp" => field.timestamp = true,
        "searchable" => field.searchable = true,
        "sortable" => field.sortable = true,
        "relation" => field.relation = Some(relation(value)?),
        "secret" => field.access.sensitive = true,
        // Native codecs consume the exact retained attributes at their storage boundary.
        "hashed" | "encrypted" => {}
        "default" => {
            let args = arguments(value)?;
            if args.len() != 1 {
                return Err(invalid("default requires one argument"));
            }
            match args[0].as_str() {
                Some("cuid()") => {
                    field.value_rule = Some(ValueRule {
                        provider: "cuid".into(),
                        args: Vec::new(),
                        revision: "cuid2-3.3.0".into(),
                    })
                }
                Some("now()") => {
                    field.value_rule = Some(ValueRule {
                        provider: "now".into(),
                        args: Vec::new(),
                        revision: "1".into(),
                    })
                }
                _ => field.default = Some(args[0].clone()),
            }
        }
        name if name.starts_with("is.") => super::assertions::normalize(
            field,
            &name[3..],
            value,
            &super::DefinitionRegistry::standard(),
        )?,
        // Source presentation arguments deliberately remain in their exact private
        // attribute form; these are not Typed/public resolved component bindings.
        _ => {}
    }
    Ok(())
}
