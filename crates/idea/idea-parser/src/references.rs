//! Structural reference validation and bounded dependency resolution after composition.

mod collect;
pub(crate) use collect::collect;

use serde_json::Value;
use std::collections::{HashMap, HashSet};

use crate::json::{measure, schema_from_value, validate_value, validation};
use crate::parser::{is_camel_identifier, is_capital_identifier, valid_attribute};
use crate::{
    CompileInputs, DeclarationKind, ErrorCode, FinalSchema, IdeaError, ParseLimits, Reference,
    SchemaConfig, ValuePath,
};

/// Finalize composed data by resolving exactly the supplied typed reference sites.
///
/// Composition owns declaration precedence and reference-site selection. Every surviving
/// site must still point to its unresolved placeholder. Other strings remain literal data.
/// Imports must already be removed by the composer. Source-local forward references are
/// rejected; imported targets may be supplied by any composed section. No IO occurs.
///
/// # Errors
/// Returns missing targets/cycles with original source coordinates, stale or duplicate sites,
/// outstanding imports, invalid config and exhausted expansion/depth/byte limits.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::*;
/// let ast = parse_ast(&SourceFile::new("a", concat!("prop P { x 1 } ",
///     "model A @x(P \"${P}\") { id String }")),
///     ParseLimits::default())?;
/// let compiled = compile_ast_with_references(&ast, &CompileInputs::default())?;
/// let schema = finalize_with_references(&compiled.config, &compiled.references,
///     &CompileInputs::default())?;
/// assert!(schema.model.contains_key("A"));
/// # Ok::<(), IdeaError>(())
/// ```
pub fn finalize_with_references(
    schema: &SchemaConfig,
    references: &[Reference],
    inputs: &CompileInputs,
) -> Result<FinalSchema, IdeaError> {
    schema.validate(inputs.limits)?;
    validate_references(schema, references, inputs.limits)?;
    if !schema.r#use.is_empty() {
        return Err(validation("Compose imports before finalization"));
    }
    // Only validated bounded config reaches recursive serialization and owned cloning.
    let mut targets = HashMap::new();
    for (name, value) in &schema.prop {
        targets.insert(name.clone(), serde_json::to_value(value));
    }
    for (name, value) in &schema.r#enum {
        targets.insert(name.clone(), serde_json::to_value(value));
    }
    for (name, value) in &schema.plugin {
        targets.insert(name.clone(), serde_json::to_value(value));
    }
    for (name, value) in schema.r#type.iter().chain(&schema.model) {
        targets.insert(name.clone(), serde_json::to_value(value));
    }
    let targets: HashMap<_, _> = targets
        .into_iter()
        .map(|(name, value)| {
            value
                .map(|value| (name, value))
                .map_err(|_| validation("Config serialization failed"))
        })
        .collect::<Result<_, _>>()?;
    let mut resolver = Resolver {
        targets,
        references,
        resolved: HashMap::new(),
        active: HashSet::new(),
        remaining: inputs.limits.max_bytes,
        limits: inputs.limits,
    };
    // Resolve in surviving schema order. Dependencies use the same resolver/cache.
    let mut config =
        serde_json::to_value(schema).map_err(|_| validation("Config serialization failed"))?;
    for (kind, declarations) in [("type", &schema.r#type), ("model", &schema.model)] {
        for name in declarations.keys() {
            let value = resolver.resolve(name, 0, None)?;
            config[kind][name] = value;
        }
    }
    validate_value(&config, inputs.limits)?;
    measure(&config, inputs.limits.max_bytes)?;
    Ok(crate::stages::into_final(schema_from_value(&config)?))
}

/// Check site coordinates and actual destinations before any resolving mutation.
pub(crate) fn validate_references(
    schema: &SchemaConfig,
    references: &[Reference],
    limits: ParseLimits,
) -> Result<(), IdeaError> {
    if references.len() > limits.max_nodes() {
        return Err(validation("Too many reference sites"));
    }
    let mut sites = HashSet::new();
    let mut bytes = 0usize;
    for reference in references {
        let fail = || {
            reference
                .span
                .error(ErrorCode::ValidationError, "Invalid typed reference site")
        };
        if reference.span.start > reference.span.end
            || reference.span.end > limits.max_bytes
            || reference.span.line == 0
            || reference.span.column == 0
            || reference.path.is_empty()
            || reference.path.len() > limits.max_depth.min(ParseLimits::HARD_DEPTH)
            || !is_capital_identifier(&reference.declaration)
            || !is_capital_identifier(&reference.target)
            || !valid_attribute(&reference.attribute)
            || reference
                .column
                .as_ref()
                .is_some_and(|name| !is_camel_identifier(name))
        {
            return Err(fail());
        }
        // Structural identity, rather than spans, detects sites duplicated by a faulty merge.
        let identity = (
            &reference.kind,
            &reference.declaration,
            &reference.column,
            &reference.attribute,
            &reference.path,
        );
        if !sites.insert(identity) {
            return Err(fail());
        }
        bytes = bytes.saturating_add(measure(reference, limits.max_bytes)?);
        if bytes > limits.max_bytes {
            return Err(reference
                .span
                .error(ErrorCode::Limit, "Reference metadata byte budget exceeded"));
        }
        let declarations = match reference.kind {
            DeclarationKind::Type => &schema.r#type,
            DeclarationKind::Model => &schema.model,
        };
        let declaration = declarations.get(&reference.declaration).ok_or_else(fail)?;
        let attributes = if let Some(name) = &reference.column {
            &declaration
                .columns
                .iter()
                .find(|column| &column.name == name)
                .ok_or_else(fail)?
                .attributes
        } else {
            &declaration.attributes
        };
        let Some(crate::AttributeValue::Array(arguments)) = attributes.get(&reference.attribute)
        else {
            return Err(fail());
        };
        let Some(ValuePath::Index(index)) = reference.path.first() else {
            return Err(fail());
        };
        let mut value = arguments.get(*index).ok_or_else(fail)?;
        for step in &reference.path[1..] {
            value = match step {
                ValuePath::Index(index) => value.as_array().and_then(|values| values.get(*index)),
                ValuePath::Key(key) => value.as_object().and_then(|values| values.get(key)),
            }
            .ok_or_else(fail)?;
        }
        if value.as_str() != Some(&format!("${{{}}}", reference.target)) {
            return Err(fail());
        }
    }
    Ok(())
}

/// Per-run resolver state. No cache or mutation is shared across clones or invocations.
struct Resolver<'a> {
    /// Validated composed declarations, keyed by the globally unique declaration name.
    targets: HashMap<String, Value>,
    /// Surviving structural sites selected by composition.
    references: &'a [Reference],
    /// Independently owned completed dependency snapshots.
    resolved: HashMap<String, Value>,
    /// Current recursion stack membership; completed diamonds are not cycles.
    active: HashSet<String>,
    /// Aggregate remaining serialized copy bytes, charged before materialization.
    remaining: usize,
    /// Caller policy including the non-raiseable hard depth ceiling.
    limits: ParseLimits,
}

impl Resolver<'_> {
    /// Resolve one bounded declaration and recursively materialize its dependencies.
    fn resolve(
        &mut self,
        name: &str,
        depth: usize,
        origin: Option<&Reference>,
    ) -> Result<Value, IdeaError> {
        let error = |code, message| match origin {
            Some(reference) => reference.span.error(code, message),
            None => IdeaError::new(code, message),
        };
        if depth > self.limits.max_depth.min(ParseLimits::HARD_DEPTH) {
            return Err(error(
                ErrorCode::Limit,
                "Reference dependency depth exceeded",
            ));
        }
        if let Some(value) = self.resolved.get(name) {
            // Even cached hits spend bytes before copying the materialized tree.
            charge(value, &mut self.remaining)
                .map_err(|_| error(ErrorCode::Limit, "Reference expansion budget exceeded"))?;
            return Ok(value.clone());
        }
        if !self.active.insert(name.into()) {
            return Err(error(
                ErrorCode::UnknownReference,
                "Cyclic reference dependency",
            ));
        }
        let target = self
            .targets
            .get(name)
            .ok_or_else(|| error(ErrorCode::UnknownReference, "Unknown reference target"))?;
        charge(target, &mut self.remaining)
            .map_err(|_| error(ErrorCode::Limit, "Reference expansion budget exceeded"))?;
        let mut value = target.clone();
        for reference in self
            .references
            .iter()
            .filter(|reference| reference.declaration == name)
        {
            if reference.forward {
                return Err(reference.span.error(
                    ErrorCode::UnknownReference,
                    "Reference target is not a prior declaration",
                ));
            }
            let resolved = self.resolve(&reference.target, depth + 1, Some(reference))?;
            let site = site_mut(&mut value, reference)?;
            *site = resolved;
            // Validate each graft before it can become another clone/serialization source.
            validate_value(&value, self.limits).map_err(|_| {
                reference.span.error(
                    ErrorCode::Limit,
                    "Resolved reference nesting limit exceeded",
                )
            })?;
        }
        self.active.remove(name);
        measure(&value, self.limits.max_bytes)?;
        // Cached materialization and the returned copy both count toward aggregate work.
        charge(&value, &mut self.remaining)
            .map_err(|_| error(ErrorCode::Limit, "Reference expansion budget exceeded"))?;
        self.resolved.insert(name.into(), value.clone());
        Ok(value)
    }
}

/// Charge work before cloning an already depth-bounded value.
fn charge(value: &Value, remaining: &mut usize) -> Result<(), IdeaError> {
    let bytes = measure(value, *remaining)?;
    *remaining -= bytes;
    Ok(())
}

/// Locate a checked structural reference path in a normalized declaration value.
fn site_mut<'a>(value: &'a mut Value, reference: &Reference) -> Result<&'a mut Value, IdeaError> {
    let fail = || {
        reference
            .span
            .error(ErrorCode::ValidationError, "Stale typed reference site")
    };
    let value = if let Some(name) = &reference.column {
        value
            .get_mut("columns")
            .and_then(Value::as_array_mut)
            .and_then(|columns| {
                columns
                    .iter_mut()
                    .find(|column| column["name"].as_str() == Some(name))
            })
            .ok_or_else(fail)?
    } else {
        value
    };
    let mut value = value
        .get_mut("attributes")
        .and_then(|values| values.get_mut(&reference.attribute))
        .ok_or_else(fail)?;
    for step in &reference.path {
        value = match step {
            ValuePath::Index(index) => value
                .as_array_mut()
                .and_then(|values| values.get_mut(*index)),
            ValuePath::Key(key) => value.as_object_mut().and_then(|values| values.get_mut(key)),
        }
        .ok_or_else(fail)?;
    }
    Ok(value)
}
