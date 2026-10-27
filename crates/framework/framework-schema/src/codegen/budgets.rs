//! Aggregate direct-config accounting, checked before any normalization clone.

use super::{NormalizeOptions, enum_family, invalid, limit, portable_name};
use crate::SchemaResult;
use stackpress_idea_parser::{AttributeValue, SchemaConfig};

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
pub(super) fn preflight(config: &SchemaConfig, options: NormalizeOptions) -> SchemaResult<()> {
    let mut budget = NormalizationBudget::default();
    for (name, choices) in &config.r#enum {
        if name.len() > 512 {
            return Err(limit().at(vec!["enum".into()]));
        }
        let root = vec!["enum".into(), name.clone()];
        portable_name(name).map_err(|error| error.at(root.clone()))?;
        budget
            .charge(1, name.len())
            .map_err(|error| error.at(root.clone()))?;
        if choices.len() > 128 {
            return Err(limit().at(root));
        }
        for (variant, value) in choices {
            if variant.len() > 512 {
                return Err(limit().at(root.clone()));
            }
            let path = [root.clone(), vec![variant.clone()]].concat();
            portable_name(variant).map_err(|error| error.at(path.clone()))?;
            enum_family(value).map_err(|error| error.at(path.clone()))?;
            budget
                .charge(
                    1,
                    variant
                        .len()
                        .checked_add(value.as_str().map(str::len).unwrap_or(8))
                        .ok_or_else(limit)?,
                )
                .map_err(|error| error.at(path))?;
        }
    }
    for (kind, declarations) in [("type", &config.r#type), ("model", &config.model)] {
        for (name, declaration) in declarations {
            if name.len() > 512 {
                return Err(limit().at(vec![kind.into()]));
            }
            let root = vec![kind.into(), name.clone()];
            portable_name(name).map_err(|error| error.at(root.clone()))?;
            if name != &declaration.name {
                return Err(invalid("declaration identity mismatch").at(root));
            }
            budget
                .charge(1, name.len())
                .map_err(|error| error.at(root.clone()))?;
            if declaration.columns.len() > options.fields.min(128)
                || declaration.attributes.len() > 128
            {
                return Err(limit().at(root));
            }
            for (name, value) in &declaration.attributes {
                if name.len() > 128 {
                    return Err(limit().at(root.clone()));
                }
                charge_attribute(&mut budget, name, value).map_err(|error| {
                    error.at([root.clone(), vec!["attributes".into(), name.clone()]].concat())
                })?;
            }
            for column in &declaration.columns {
                if column.name.len() > 512 {
                    return Err(limit().at(root.clone()));
                }
                let path = [root.clone(), vec!["columns".into(), column.name.clone()]].concat();
                portable_name(&column.name).map_err(|error| error.at(path.clone()))?;
                if column.column_type.len() > 512 || column.attributes.len() > 128 {
                    return Err(limit().at(path));
                }
                budget
                    .charge(1, column.name.len() + column.column_type.len())
                    .map_err(|error| error.at(path.clone()))?;
                for (name, value) in &column.attributes {
                    if name.len() > 128 {
                        return Err(limit().at(path.clone()));
                    }
                    charge_attribute(&mut budget, name, value).map_err(|error| {
                        error.at([path.clone(), vec!["attributes".into(), name.clone()]].concat())
                    })?;
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
