//! Small compiled inline transformations; block behavior belongs to the renderer.

use super::{TemplateError, TemplateErrorKind, TemplateHelper, TemplateResult};
use serde_json::Value;
use std::collections::BTreeMap;

/// Install immutable built-in function pointers under the original compact names.
pub(super) fn builtins() -> BTreeMap<String, TemplateHelper> {
    [
        ("A", upper as TemplateHelper),
        ("a", lower),
        (".", clip),
        ("|", fallback),
    ]
    .into_iter()
    .map(|(name, helper)| (name.into(), helper))
    .collect()
}

/// Require a single string before applying Unicode uppercase mapping.
fn upper(arguments: &[&Value]) -> TemplateResult<String> {
    Ok(one_string(arguments)?.to_uppercase())
}

/// Require a single string before applying Unicode lowercase mapping.
fn lower(arguments: &[&Value]) -> TemplateResult<String> {
    Ok(one_string(arguments)?.to_lowercase())
}

/// Resolve the common exact-one-string contract without implicit JSON coercion.
fn one_string<'a>(arguments: &[&'a Value]) -> TemplateResult<&'a str> {
    match arguments {
        [value] => value.as_str().ok_or_else(invalid),
        _ => Err(invalid()),
    }
}

/// Clip Unicode scalar values and add a suffix only when another character exists.
fn clip(arguments: &[&Value]) -> TemplateResult<String> {
    let (value, count, suffix) = match arguments {
        [value, count] => (*value, *count, ""),
        [value, count, suffix] => (*value, *count, suffix.as_str().ok_or_else(invalid)?),
        _ => return Err(invalid()),
    };
    let value = value.as_str().ok_or_else(invalid)?;
    let count = count
        .as_u64()
        .and_then(|count| usize::try_from(count).ok())
        .ok_or_else(invalid)?;
    let mut characters = value.chars();
    let mut result: String = characters.by_ref().take(count).collect();
    if characters.next().is_some() {
        result.push_str(suffix);
    }
    Ok(result)
}

/// Select the first explicitly truthy scalar, rejecting collections even after a match.
fn fallback(arguments: &[&Value]) -> TemplateResult<String> {
    let mut selected = None;
    for value in arguments {
        let truthy = match value {
            Value::Null => false,
            Value::Bool(value) => *value,
            Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
            Value::String(value) => !value.is_empty(),
            _ => return Err(invalid()),
        };
        if truthy && selected.is_none() {
            selected = Some(*value);
        }
    }
    selected.map(scalar).unwrap_or_else(|| Ok(String::new()))
}

/// Format only template-supported scalar values, preserving strings literally.
pub(super) fn scalar(value: &Value) -> TemplateResult<String> {
    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(invalid()),
    }
}

/// Avoid disclosing argument values in helper/type failures.
fn invalid() -> TemplateError {
    TemplateError::new(
        TemplateErrorKind::InvalidType,
        "invalid scalar or helper arguments",
    )
}
