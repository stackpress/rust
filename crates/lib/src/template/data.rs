//! Borrowed input inspection and bounded scalar-map construction.

use super::{TemplateError, TemplateErrorKind, TemplateLimits, TemplateResult};
use serde_json::{Map, Value};
use std::collections::BTreeSet;

/// Check compact JSON bytes and depth without serializing/cloning the object.
pub(super) fn validate(value: &Value, limits: &TemplateLimits) -> TemplateResult<()> {
    if !value.is_object() {
        return Err(TemplateError::new(
            TemplateErrorKind::InvalidData,
            "render data must be an object",
        ));
    }
    inspect(value, 1, limits, &mut 0)
}

/// Count one JSON value, rejecting depth before recursive descent.
fn inspect(
    value: &Value,
    depth: usize,
    limits: &TemplateLimits,
    bytes: &mut usize,
) -> TemplateResult<()> {
    if depth > limits.max_data_depth {
        return Err(TemplateError::new(
            TemplateErrorKind::Limit,
            "data depth limit",
        ));
    }
    match value {
        Value::Null => charge(bytes, 4, limits),
        Value::Bool(value) => charge(bytes, if *value { 4 } else { 5 }, limits),
        Value::Number(value) => charge(bytes, value.to_string().len(), limits),
        Value::String(value) => string_bytes(value, bytes, limits),
        Value::Array(values) => {
            charge(bytes, 2, limits)?;
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    charge(bytes, 1, limits)?;
                }
                inspect(value, depth + 1, limits, bytes)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            charge(bytes, 2, limits)?;
            for (index, (key, value)) in values.iter().enumerate() {
                if index > 0 {
                    charge(bytes, 1, limits)?;
                }
                string_bytes(key, bytes, limits)?;
                charge(bytes, 1, limits)?;
                inspect(value, depth + 1, limits, bytes)?;
            }
            Ok(())
        }
    }
}

/// Count JSON's quotes/escapes; ordinary UTF-8 bytes are emitted unchanged.
fn string_bytes(value: &str, bytes: &mut usize, limits: &TemplateLimits) -> TemplateResult<()> {
    charge(bytes, 2, limits)?;
    for byte in value.bytes() {
        // JSON uses short escapes for these controls and six bytes for other ASCII controls.
        let length = match byte {
            b'"' | b'\\' | b'\n' | b'\r' | b'\t' | 8 | 12 => 2,
            0..=31 => 6,
            _ => 1,
        };
        charge(bytes, length, limits)?;
    }
    Ok(())
}

/// Refuse arithmetic overflow and the next byte before charging it.
fn charge(bytes: &mut usize, amount: usize, limits: &TemplateLimits) -> TemplateResult<()> {
    let next = bytes
        .checked_add(amount)
        .filter(|next| *next <= limits.max_data_bytes)
        .ok_or_else(|| TemplateError::new(TemplateErrorKind::Limit, "data byte limit"))?;
    *bytes = next;
    Ok(())
}

/// Validate pairs before making any engine-owned copies of their strings.
pub(super) fn pairs(pairs: &[(&str, &str)], limits: &TemplateLimits) -> TemplateResult<Value> {
    let mut bytes = 0;
    let mut names = BTreeSet::new();
    charge(&mut bytes, 2, limits)?;
    for (index, (key, value)) in pairs.iter().enumerate() {
        if limits.max_data_depth < 2 {
            return Err(TemplateError::new(
                TemplateErrorKind::Limit,
                "data depth limit",
            ));
        }
        if index > 0 {
            charge(&mut bytes, 1, limits)?;
        }
        string_bytes(key, &mut bytes, limits)?;
        charge(&mut bytes, 1, limits)?;
        string_bytes(value, &mut bytes, limits)?;
        if !names.insert(*key) {
            return Err(TemplateError::new(
                TemplateErrorKind::InvalidData,
                "duplicate pair key",
            ));
        }
    }
    // Every copy is now bounded by the validated compact JSON budget.
    let object: Map<_, _> = pairs
        .iter()
        .map(|(key, value)| ((*key).into(), Value::String((*value).into())))
        .collect();
    Ok(Value::Object(object))
}
