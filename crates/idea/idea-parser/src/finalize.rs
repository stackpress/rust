//! Extracted ordered schema compilation and prior-reference expansion.

use indexmap::IndexMap;
use std::collections::HashMap;

use serde::Serialize;
use serde_json::{Map, Value};

use crate::ast::{
    DataToken, DeclarationToken, IdentifierToken, ImportToken, ObjectToken, PropertyToken,
    SchemaToken, Statement,
};
use crate::error::{ErrorCode, IdeaError};
use crate::types::{
    AttributeValue, ColumnConfig, EnumConfig, ModelConfig, PluginConfig, PropConfig, TypeConfig,
};

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
/// Ordered compiled source, retaining imports and reusable props.
pub struct SchemaConfig {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    /// Import specifiers in source order.
    pub r#use: Vec<String>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Plugin configurations in declared order.
    pub plugin: IndexMap<String, PluginConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Reusable property objects in declared order.
    pub prop: IndexMap<String, PropConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Enumerations in declared order.
    pub r#enum: IndexMap<String, EnumConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Reusable fieldsets in declared order.
    pub r#type: IndexMap<String, TypeConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Models in declared order.
    pub model: IndexMap<String, ModelConfig>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Default)]
/// Final single-source config without props/import directives.
pub struct FinalSchema {
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Plugin configurations in declared order.
    pub plugin: IndexMap<String, PluginConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Enumerations in declared order.
    pub r#enum: IndexMap<String, EnumConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Reusable fieldsets in declared order.
    pub r#type: IndexMap<String, TypeConfig>,
    #[serde(skip_serializing_if = "IndexMap::is_empty", default)]
    /// Models in declared order.
    pub model: IndexMap<String, ModelConfig>,
}

/// Compile declarations in source order with prior-reference visibility.
pub fn compile_schema(
    ast: &SchemaToken,
    finalize: bool,
    limits: crate::ParseLimits,
) -> Result<SchemaConfig, IdeaError> {
    let mut schema = SchemaConfig::default();
    let mut references = References::new(limits);

    // Source order defines imports and which earlier declarations are reference-visible.
    for statement in &ast.body {
        if let Statement::Import(import) = statement {
            schema.r#use.push(compile_use(import)?);
        }
    }

    // Source order defines imports and which earlier declarations are reference-visible.
    for statement in &ast.body {
        let declaration = match statement {
            Statement::Declaration(value) => value,
            Statement::Import(_) => continue,
        };

        // Each branch validates uniqueness before publishing its ordered config entry.
        match declaration.kind {
            "enum" => {
                let (key, value) = compile_enum(declaration)?;
                ensure_unique(&references, &key, declaration.start, declaration.end)?;
                references.insert(key.clone(), serialize(&value, limits.max_depth)?);
                schema.r#enum.insert(key, value);
            }
            "prop" => {
                let (key, value) =
                    compile_prop(declaration, if finalize { Some(&references) } else { None })?;
                ensure_unique(&references, &key, declaration.start, declaration.end)?;
                references.insert(key.clone(), serialize(&value, limits.max_depth)?);
                schema.prop.insert(key, value);
            }
            "type" => {
                let (key, value) = compile_type_like(
                    declaration,
                    if finalize { Some(&references) } else { None },
                )?;
                ensure_unique(&references, &key, declaration.start, declaration.end)?;
                references.insert(key.clone(), serialize(&value, limits.max_depth)?);
                schema.r#type.insert(key, value);
            }
            "model" => {
                let (key, value) = compile_type_like(
                    declaration,
                    if finalize { Some(&references) } else { None },
                )?;
                ensure_unique(&references, &key, declaration.start, declaration.end)?;
                references.insert(key.clone(), serialize(&value, limits.max_depth)?);
                schema.model.insert(key, value);
            }
            "plugin" => {
                let (key, value) = compile_plugin(declaration)?;
                ensure_unique(&references, &key, declaration.start, declaration.end)?;
                references.insert(key.clone(), serialize(&value, limits.max_depth)?);
                schema.plugin.insert(key, value);
            }
            _ => {
                return Err(IdeaError::new(
                    ErrorCode::InvalidDeclaration,
                    format!("Invalid declaration {}", declaration.kind),
                )
                .with_span(declaration.start, declaration.end));
            }
        }
    }

    Ok(schema)
}

/// Reject a repeated top-level name across all declaration kinds.
fn ensure_unique(
    references: &References,
    key: &str,
    start: usize,
    end: usize,
) -> Result<(), IdeaError> {
    if references.contains_key(key) {
        return Err(
            IdeaError::new(ErrorCode::DuplicateName, format!("Duplicate {}", key))
                .with_span(start, end),
        );
    }
    Ok(())
}

/// Recursively compile bounded data while preserving literal/reference distinction.
fn compile_data(token: &DataToken, references: Option<&References>) -> Result<Value, IdeaError> {
    // The private typed enum keeps arrays, objects, literals and references distinguishable.
    match token {
        DataToken::Object(value) => compile_object(value, references),
        DataToken::Array(value) => {
            let mut compiled = Vec::with_capacity(value.elements.len());
            for element in &value.elements {
                compiled.push(compile_data(element, references)?);
            }
            Ok(Value::Array(compiled))
        }
        DataToken::Literal(value) => Ok(value.value.clone()),
        DataToken::Environment(value) => Err(IdeaError::new(
            ErrorCode::ValidationError,
            "Environment node was not compiled",
        )
        .with_span(value.start, value.end)),
        DataToken::Identifier(value) => compile_identifier(value, references),
    }
}

/// Resolve a prior declaration or retain its non-final placeholder.
fn compile_identifier(
    token: &IdentifierToken,
    references: Option<&References>,
) -> Result<Value, IdeaError> {
    // Only final compilation resolves references; ordinary compilation preserves placeholders.
    if let Some(references) = references {
        if let Some(value) = references.get(&token.name) {
            // Charge a bounded value before cloning; repeated props cannot amplify forever.
            let bytes = serde_json::to_vec(value)
                .map_err(|_| {
                    IdeaError::new(ErrorCode::ValidationError, "Reference serialization failed")
                })?
                .len();
            let remaining = references
                .remaining
                .get()
                .checked_sub(bytes)
                .ok_or_else(|| {
                    IdeaError::new(ErrorCode::Limit, "Reference expansion budget exceeded")
                        .with_span(token.start, token.end)
                })?;
            references.remaining.set(remaining);
            return Ok(value.clone());
        }

        return Err(IdeaError::new(
            ErrorCode::UnknownReference,
            format!("Unknown reference {}", token.name),
        )
        .with_span(token.start, token.end));
    }

    Ok(Value::String(format!("${{{}}}", token.name)))
}

/// Compile an ordered unique-key object from private validated syntax.
fn compile_object(
    token: &ObjectToken,
    references: Option<&References>,
) -> Result<Value, IdeaError> {
    let mut object = Map::new();
    // Parser validation already rejected duplicates; insertion now preserves member order.
    for property in &token.properties {
        object.insert(
            property.key.name.clone(),
            compile_data(&property.value, references)?,
        );
    }
    Ok(Value::Object(object))
}

/// Compile scalar enum choices in declaration order.
fn compile_enum(token: &DeclarationToken) -> Result<(String, EnumConfig), IdeaError> {
    let declarator = first_declarator(token)?;
    let mut options = IndexMap::new();
    for property in &declarator.init.properties {
        options.insert(property.key.name.clone(), literal_value(&property.value)?);
    }
    Ok((declarator.id.name.clone(), options))
}

/// Compile a reusable object and its prior references.
fn compile_prop(
    token: &DeclarationToken,
    references: Option<&References>,
) -> Result<(String, PropConfig), IdeaError> {
    let declarator = first_declarator(token)?;
    let mut config = IndexMap::new();
    for property in &declarator.init.properties {
        config.insert(
            property.key.name.clone(),
            compile_data(&property.value, references)?,
        );
    }
    Ok((declarator.id.name.clone(), config))
}

/// Compile data-only plugin configuration without resolving executable modules.
fn compile_plugin(token: &DeclarationToken) -> Result<(String, PluginConfig), IdeaError> {
    let declarator = first_declarator(token)?;
    let mut config = IndexMap::new();
    for property in &declarator.init.properties {
        config.insert(
            property.key.name.clone(),
            compile_data(&property.value, None)?,
        );
    }
    Ok((declarator.id.name.clone(), config))
}

/// Compile type/model attributes and ordered field configurations.
fn compile_type_like(
    token: &DeclarationToken,
    references: Option<&References>,
) -> Result<(String, TypeConfig), IdeaError> {
    let declarator = first_declarator(token)?;
    let mut attributes = IndexMap::new();
    let mut columns_source: Option<&ObjectToken> = None;

    for property in &declarator.init.properties {
        match property.key.name.as_str() {
            "attributes" => {
                attributes = compile_attributes(expect_object(&property.value)?, references)?;
            }
            "columns" => {
                columns_source = Some(expect_object(&property.value)?);
            }
            _ => {}
        }
    }

    let columns_source = columns_source.ok_or_else(|| {
        IdeaError::new(ErrorCode::MissingProperty, "Expecting a columns property")
            .with_span(token.start, token.end)
    })?;

    let mut columns = Vec::with_capacity(columns_source.properties.len());
    for column in &columns_source.properties {
        columns.push(compile_column(column, references)?);
    }

    let name = declarator.id.name.clone();
    Ok((
        name.clone(),
        TypeConfig {
            name,
            mutable: token.mutable.unwrap_or(true),
            attributes,
            columns,
        },
    ))
}

/// Normalize validated single-list and optional modifiers into explicit flags.
fn compile_column(
    property: &PropertyToken,
    references: Option<&References>,
) -> Result<ColumnConfig, IdeaError> {
    let source = expect_object(&property.value)?;
    let mut type_name = String::new();
    let mut attributes = IndexMap::new();

    for inner in &source.properties {
        match inner.key.name.as_str() {
            "type" => {
                type_name = literal_string(&inner.value)?;
            }
            "attributes" => {
                attributes = compile_attributes(expect_object(&inner.value)?, references)?;
            }
            _ => {}
        }
    }

    // The parser accepted exactly one optional marker and one list marker, in that order.
    let required = !type_name.ends_with('?');
    let normalized = type_name.trim_end_matches('?').to_string();
    let multiple = normalized.ends_with("[]");
    let normalized = normalized.trim_end_matches("[]").to_string();

    Ok(ColumnConfig {
        name: property.key.name.clone(),
        column_type: normalized,
        attributes,
        required,
        multiple,
    })
}

/// Normalize flag and method syntax without interpreting framework semantics.
fn compile_attributes(
    object: &ObjectToken,
    references: Option<&References>,
) -> Result<IndexMap<String, AttributeValue>, IdeaError> {
    let mut attributes = IndexMap::new();
    for property in &object.properties {
        // Empty methods normalize to flags for compiled-schema compatibility.
        let value = match &property.value {
            DataToken::Literal(literal) if literal.value == Value::Bool(true) => {
                AttributeValue::Boolean(true)
            }
            DataToken::Array(array) => {
                if array.elements.is_empty() {
                    attributes.insert(property.key.name.clone(), AttributeValue::Boolean(true));
                    continue;
                }

                let mut compiled = Vec::with_capacity(array.elements.len());
                for item in &array.elements {
                    compiled.push(compile_data(item, references)?);
                }
                AttributeValue::Array(compiled)
            }
            _ => {
                let compiled = compile_data(&property.value, references)?;
                AttributeValue::Array(vec![compiled])
            }
        };
        attributes.insert(property.key.name.clone(), value);
    }
    Ok(attributes)
}

/// Retain the literal import specifier in non-final configuration.
fn compile_use(token: &ImportToken) -> Result<String, IdeaError> {
    if token.node_type != "ImportDeclaration" {
        return Err(IdeaError::new(ErrorCode::ValidationError, "Invalid Import")
            .with_span(token.start, token.end));
    }
    token
        .source
        .value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| {
            IdeaError::new(ErrorCode::ValidationError, "Invalid Import")
                .with_span(token.source.start, token.source.end)
        })
}

/// Borrow the validated declaration payload or report missing structure.
fn first_declarator(token: &DeclarationToken) -> Result<&crate::ast::DeclaratorToken, IdeaError> {
    token.declarations.first().ok_or_else(|| {
        IdeaError::new(ErrorCode::ValidationError, "Missing declaration")
            .with_span(token.start, token.end)
    })
}

/// Require object structure at an internal normalized syntax boundary.
fn expect_object(token: &DataToken) -> Result<&ObjectToken, IdeaError> {
    if let DataToken::Object(value) = token {
        Ok(value)
    } else {
        Err(IdeaError::new(
            ErrorCode::ValidationError,
            "Expected object",
        ))
    }
}

/// Borrow the scalar value from the private syntax representation.
fn literal_value(token: &DataToken) -> Result<Value, IdeaError> {
    if let DataToken::Literal(literal) = token {
        Ok(literal.value.clone())
    } else {
        Err(IdeaError::new(
            ErrorCode::ValidationError,
            "Expected literal",
        ))
    }
}

/// Require a string literal for an import/type representation.
fn literal_string(token: &DataToken) -> Result<String, IdeaError> {
    let value = literal_value(token)?;
    value
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| IdeaError::new(ErrorCode::ValidationError, "Expected string literal"))
}

/// Preserve a serialization error rather than silently replacing metadata with null.
fn serialize<T: Serialize>(value: &T, max_depth: usize) -> Result<Value, IdeaError> {
    let value = serde_json::to_value(value)
        .map_err(|_| IdeaError::new(ErrorCode::ValidationError, "Schema serialization failed"))?;
    // A chain of valid prior references can deepen data beyond its original syntax depth.
    // Check before publishing it as another clone/serialization source for the next declaration.
    let mut pending = vec![(&value, 1usize)];
    while let Some((value, depth)) = pending.pop() {
        if depth > max_depth.min(crate::ParseLimits::HARD_DEPTH)
            && matches!(value, Value::Array(_) | Value::Object(_))
        {
            return Err(IdeaError::new(
                ErrorCode::Limit,
                "Compiled reference nesting limit exceeded",
            ));
        }
        match value {
            Value::Array(values) => pending.extend(values.iter().map(|value| (value, depth + 1))),
            Value::Object(values) => {
                pending.extend(values.values().map(|value| (value, depth + 1)))
            }
            _ => {}
        }
    }
    Ok(value)
}

/// Prior declarations plus a finite aggregate reference-copy budget.
struct References {
    /// Owned finite values available to subsequent declarations.
    values: HashMap<String, Value>,
    /// Remaining serialized bytes allowed for reference expansions.
    remaining: std::cell::Cell<usize>,
}

impl References {
    /// Bound repeated references independently of source-text size.
    fn new(limits: crate::ParseLimits) -> Self {
        Self {
            values: HashMap::new(),
            remaining: std::cell::Cell::new(limits.max_bytes),
        }
    }
}

impl std::ops::Deref for References {
    type Target = HashMap<String, Value>;
    /// Borrow the private lookup table without exposing it outside this compiler.
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl std::ops::DerefMut for References {
    /// Insert only values assembled by the bounded parser/compiler.
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
