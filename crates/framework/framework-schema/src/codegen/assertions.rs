//! Canonical assertion names, signatures and applicability; no generation or IO.

use super::metadata::rule;
use super::{DefinitionRegistry, invalid, limit, unsupported};
use crate::*;
use serde_json::Value;
use stackpress_idea_parser::AttributeValue;

/// Normalize aliases once, validate applicability/arity and preserve host-only rules explicitly.
pub(super) fn normalize(
    field: &mut Field,
    name: &str,
    value: &AttributeValue,
    registry: &DefinitionRegistry,
) -> SchemaResult<()> {
    let code = match name {
        "notempty" => "ne",
        "pattern" => "regex",
        "gte" => "ge",
        "lte" => "le",
        "cgte" => "cge",
        "clte" => "cle",
        "wgte" => "wge",
        "wlte" => "wle",
        name => name,
    };
    let mut args = match value {
        AttributeValue::Boolean(true) => Vec::new(),
        AttributeValue::Array(args) => args.clone(),
        _ => return Err(invalid("assertion must be enabled")),
    };
    let text = field.value_type == ValueType::String && !field.multiple;
    let numeric =
        matches!(field.value_type, ValueType::Integer | ValueType::Number) && !field.multiple;
    let date = matches!(
        field.value_type,
        ValueType::Date | ValueType::Time | ValueType::Datetime
    ) && !field.multiple;
    let (valid, arity, client) = match code {
        "required" | "ne" => (true, 0, true),
        "string" => (field.value_type == ValueType::String, 0, true),
        "boolean" => (field.value_type == ValueType::Boolean, 0, true),
        "integer" => (field.value_type == ValueType::Integer, 0, true),
        "number" | "float" => (
            matches!(field.value_type, ValueType::Integer | ValueType::Number),
            0,
            true,
        ),
        "object" => (
            matches!(field.value_type, ValueType::Object | ValueType::Fieldset(_)),
            0,
            true,
        ),
        "unique" => {
            field.unique = true;
            (true, 0, false)
        }
        "eq" | "neq" => (!field.multiple, 1, true),
        "option" => (!field.multiple, usize::MAX, true),
        "gt" | "ge" | "lt" | "le" => (numeric || date, 1, true),
        "ceq" | "cgt" | "cge" | "clt" | "cle" | "weq" | "wgt" | "wge" | "wlt" | "wle" => {
            (text, 1, true)
        }
        "lowercase" | "uppercase" | "slug" | "hex" | "url" => (text, 0, true),
        "regex" => (text, 1, true),
        "date" => (text || date, 0, true),
        "future" | "past" | "present" => (date, 0, true),
        name => {
            let definition = registry
                .validators
                .iter()
                .find(|v| v.name == name)
                .ok_or_else(|| unsupported("assertion definition is not registered"))?;
            (
                definition.types.contains(&field.value_type) && !field.multiple,
                0,
                false,
            )
        }
    };
    // Optional messages are data, never derived from rejected values. Variadic option
    // uses an explicit array to disambiguate its optional trailing message.
    if code == "option" && args.first().is_some_and(Value::is_array) {
        let mut choices = args[0].as_array().expect("array checked").clone();
        choices.extend(args.into_iter().skip(1));
        args = choices;
    }
    let mut message = None;
    if arity != usize::MAX && args.len() == arity + 1 {
        let text = args
            .last()
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("assertion message must be text"))?;
        if text.is_empty() || text.len() > 256 || text.chars().any(char::is_control) {
            return Err(invalid("invalid assertion message"));
        }
        message = Some(text.to_owned());
        args.pop();
    }
    if !valid
        || if arity == usize::MAX {
            args.is_empty()
        } else {
            args.len() != arity
        }
    {
        return Err(invalid("assertion arguments or type are incompatible"));
    }
    if (numeric && matches!(code, "gt" | "ge" | "lt" | "le")
        || code.starts_with('c') && code != "cc"
        || code.starts_with('w'))
        && args.iter().any(|v| !v.as_f64().is_some_and(f64::is_finite))
    {
        return Err(invalid("comparison requires a finite number"));
    }
    if code == "regex" {
        let pattern = args[0]
            .as_str()
            .ok_or_else(|| invalid("regex requires text"))?;
        if pattern.len() > 4096 {
            return Err(limit());
        }
        regex::RegexBuilder::new(pattern)
            .size_limit(1024 * 1024)
            .dfa_size_limit(1024 * 1024)
            .build()
            .map_err(|cause| invalid("invalid bounded regex").with_cause(cause))?;
    }
    if matches!(code, "eq" | "neq" | "option") {
        for value in &args {
            if !crate::value_matches(&field.value_type, value) {
                return Err(invalid("assertion literal type mismatch"));
            }
        }
    }
    if date
        && matches!(code, "gt" | "ge" | "lt" | "le")
        && !crate::value_matches(&field.value_type, &args[0])
    {
        return Err(invalid("invalid date comparison"));
    }
    if code == "ne"
        && !registry
            .definitions
            .iter()
            .any(|(name, revision)| name == "is.ne" && revision == "1")
    {
        return Err(unsupported("missing assertion definition"));
    }
    field.nonempty |= code == "ne";
    let mut rule = rule(code, args, client);
    if let Some(message) = message {
        rule.message = message;
    }
    if let Some(definition) = registry.validators.iter().find(|v| v.name == code) {
        rule.definition_revision = definition.revision.clone();
    }
    field.rules.push(rule);
    Ok(())
}

/// Reserve canonical built-ins and aliases so app extensions cannot silently redefine them.
pub(super) fn builtin(name: &str) -> bool {
    matches!(
        name,
        "required"
            | "ne"
            | "notempty"
            | "unique"
            | "eq"
            | "neq"
            | "option"
            | "regex"
            | "pattern"
            | "date"
            | "future"
            | "past"
            | "present"
            | "gt"
            | "ge"
            | "gte"
            | "lt"
            | "le"
            | "lte"
            | "ceq"
            | "cgt"
            | "cge"
            | "cgte"
            | "clt"
            | "cle"
            | "clte"
            | "weq"
            | "wgt"
            | "wge"
            | "wgte"
            | "wlt"
            | "wle"
            | "wlte"
            | "lowercase"
            | "uppercase"
            | "slug"
            | "hex"
            | "url"
            | "string"
            | "boolean"
            | "number"
            | "float"
            | "integer"
            | "object"
    )
}
