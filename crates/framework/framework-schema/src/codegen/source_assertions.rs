//! Original assertion argument/message dictionary, separate from the unchanged Typed policy.
use super::{invalid, unsupported};
use crate::{Field, SchemaResult, ValidationRule};
use serde::Deserialize;
use stackpress_idea_parser::AttributeValue;

/// Captured source definition; raw attribute names stay in the field's source attributes.
#[derive(Deserialize)]
struct Definition {
    name: String,
    canonical: String,
    message: String,
    args: Vec<Argument>,
}
/// Required source arguments, including the optional final authored diagnostic.
#[derive(Deserialize)]
struct Argument {
    required: bool,
    types: Vec<String>,
}

/// Normalize all original dictionary rules, including G51's repaired/previously omitted rules.
pub(super) fn normalize(field: &mut Field, name: &str, value: &AttributeValue) -> SchemaResult<()> {
    // Dictionary data is captured from the original, not derived from Typed error defaults.
    let definitions: Vec<Definition> = serde_json::from_str(include_str!("source-assertions.json"))
        .map_err(|cause| invalid("source assertion dictionary decode failed").with_cause(cause))?;
    let definition = definitions
        .into_iter()
        .find(|definition| definition.name == name)
        .ok_or_else(|| unsupported("unknown source assertion"))?;
    let mut args = match value {
        AttributeValue::Boolean(true) => Vec::new(),
        AttributeValue::Array(args) => args.clone(),
        _ => return Err(invalid("source assertion must be enabled")),
    };
    let mut message = definition.message;
    // ColumnAssertion.ts removes a custom message only when the full reference arity
    // is supplied. Empty messages remain data; source mode adds no Typed message policy.
    if args.len() == definition.args.len() && args.last().is_some_and(serde_json::Value::is_string)
    {
        message = args
            .pop()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_default();
    }
    if args.len() < definition.args.iter().filter(|arg| arg.required).count()
        || args.len() > definition.args.len()
    {
        return Err(invalid("source assertion argument count is invalid"));
    }
    for (value, argument) in args.iter().zip(&definition.args) {
        if !argument
            .types
            .iter()
            .any(|kind| argument_matches(kind, value))
        {
            return Err(invalid("source assertion argument type is invalid"));
        }
    }
    if definition.canonical == "regex" {
        let pattern = args
            .first()
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| invalid("source regex requires text"))?;
        regex::Regex::new(pattern)
            .map_err(|cause| invalid("source regex compilation failed").with_cause(cause))?;
    }
    // G51's is.unique is an action-owned assertion. Only the separate source @unique
    // attribute owns SQL constraints and upsert selectors; do not merge their meanings.
    field.nonempty |= definition.canonical == "ne";
    field.rules.push(ValidationRule {
        client_checkable: definition.canonical != "unique",
        code: definition.canonical,
        args,
        message,
        definition_revision: "1".into(),
    });
    Ok(())
}

/// Original dictionary argument families, retaining arrays as one positional argument.
fn argument_matches(kind: &str, value: &serde_json::Value) -> bool {
    if let Some(item) = kind.strip_suffix("[]") {
        return value
            .as_array()
            .is_some_and(|values| values.iter().all(|value| argument_matches(item, value)));
    }
    match kind {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "boolean" => value.is_boolean(),
        "null" => value.is_null(),
        _ => false,
    }
}
