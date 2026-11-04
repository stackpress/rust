//! Native source assertions and G51's named validation repairs; Typed validation is separate.
use super::{Column, Kind, Value, as_date, string};
use crate::{Field, ValidationRule, ValueType};

/// Assert a column in source item order, preserving its first authored/type diagnostic.
pub(super) fn assert(
    field: &Field,
    value: Option<&Value>,
    codec: &Column,
    now: i64,
    offset: i32,
    catalog: &crate::SourceCatalog,
) -> Option<String> {
    if field.multiple {
        let Some(values) = value.and_then(Value::as_array) else {
            return Some(type_error("array", value));
        };
        // Zod visits one element and its refinements before reporting a later item's type.
        for value in values {
            if let Some(error) = scalar(field, Some(value), codec, now, offset, true, catalog) {
                return Some(error);
            }
        }
        return None;
    }
    scalar(field, value, codec, now, offset, false, catalog)
}

/// Base shape precedes refinements; nullable scalars bypass both, list items do not.
fn scalar(
    field: &Field,
    value: Option<&Value>,
    codec: &Column,
    now: i64,
    offset: i32,
    item: bool,
    catalog: &crate::SourceCatalog,
) -> Option<String> {
    if !item && !field.required && value.is_none_or(Value::is_null) {
        return None;
    }
    if field.value_type == ValueType::Unknown && value.is_none() {
        return field.required.then(|| {
            field
                .rules
                .iter()
                .find(|r| r.code == "required")
                .map(|r| r.message.clone())
                .unwrap_or_else(|| "Value is required.".into())
        });
    }
    if let ValueType::Enum(name) = &field.value_type {
        let Some(enumeration) = catalog.enums.iter().find(|item| item.name == *name) else {
            return Some(format!("Source enum definition is missing: {name}"));
        };
        if !value.is_some_and(|value| {
            enumeration
                .variants
                .iter()
                .any(|(_, option)| equal(value, option))
        }) {
            let choices: Vec<_> = enumeration
                .variants
                .iter()
                .map(|(_, value)| value.to_string())
                .collect();
            return Some(if choices.len() == 1 {
                format!("Invalid input: expected {}", choices[0])
            } else {
                format!("Invalid option: expected one of {}", choices.join("|"))
            });
        }
    } else if matches!(field.value_type, ValueType::Integer | ValueType::Number) {
        let Some(number) = value.and_then(Value::as_f64) else {
            return Some(type_error("number", value));
        };
        if !number.is_finite() {
            return Some(type_error("number", value));
        }
        if let Some(error) = super::number::assert(field, number) {
            return Some(error);
        }
    } else if field.value_type != ValueType::Unknown {
        let mut scalar = codec.clone();
        scalar.nullable = false;
        if scalar.kind == Kind::Strings {
            scalar.kind = Kind::String;
        }
        if let Some(error) = scalar.assert(value) {
            return Some(error);
        }
    }
    let value = value?;
    if !item
        && field.required
        && !field.rules.iter().any(|rule| rule.code == "required")
        && string(value).is_empty()
    {
        return Some("Value is required.".into());
    }
    for rule in &field.rules {
        if !passes(rule, value, now, offset) {
            return Some(message(field, rule, value));
        }
    }
    None
}

/// Named rules retain source character/word definitions; G51 makes cgt/clt exclusive.
fn passes(rule: &ValidationRule, value: &Value, now: i64, offset: i32) -> bool {
    let text = value.as_str();
    let arg = rule.args.first();
    let number = value.as_f64();
    let bound = arg.and_then(serde_json::Value::as_f64);
    let compare = |a: Option<f64>, kind: &str| {
        a.zip(bound).is_some_and(|(a, b)| match kind {
            "eq" => a == b,
            "gt" => a > b,
            "ge" => a >= b,
            "lt" => a < b,
            "le" => a <= b,
            _ => false,
        })
    };
    let pattern = |pattern: &str| {
        text.is_some_and(|text| regex::Regex::new(pattern).is_ok_and(|regex| regex.is_match(text)))
    };
    match rule.code.as_str() {
        "required" => !value.is_null() && !string(value).is_empty(),
        "ne" => match value {
            Value::String(text) => !text.is_empty(),
            Value::Array(values) => !values.is_empty(),
            _ => !value.is_null(),
        },
        "string" => value.is_string(),
        "boolean" => value.is_boolean(),
        "number" | "float" => number.is_some_and(f64::is_finite),
        "integer" => number.is_some_and(|number| number.is_finite() && number.fract() == 0.0),
        "object" => value.is_object(),
        "eq" => arg.is_some_and(|arg| equal(value, arg)),
        "neq" => arg.is_some_and(|arg| !equal(value, arg)),
        "option" => rule
            .args
            .iter()
            .flat_map(|arg| {
                arg.as_array()
                    .map(Vec::as_slice)
                    .unwrap_or(std::slice::from_ref(arg))
            })
            .any(|arg| equal(value, arg)),
        "gt" | "ge" | "lt" | "le" => compare(
            number.or_else(|| as_date(value).map(|date| date as f64)),
            &rule.code,
        ),
        "ceq" | "cgt" | "cge" | "clt" | "cle" => compare(
            text.map(|text| text.encode_utf16().count() as f64),
            &rule.code[1..],
        ),
        "weq" | "wgt" | "wge" | "wlt" | "wle" => compare(
            text.map(|text| text.split(' ').count() as f64),
            &rule.code[1..],
        ),
        "lowercase" => pattern("^[a-z]+$"),
        "uppercase" => pattern("^[A-Z]+$"),
        "slug" => pattern("^[a-z0-9_-]+$"),
        "hex" => pattern("^[a-f0-9]+$"),
        "regex" => arg.and_then(serde_json::Value::as_str).is_some_and(pattern),
        "url" => text.is_some_and(|text| url::Url::parse(text).is_ok()),
        "email" => {
            text.is_some_and(|text| !text.starts_with('.') && !text.contains(".."))
                && pattern(
                    "^[A-Za-z0-9_'+.\\-]*[A-Za-z0-9_+\\-]@([A-Za-z0-9][A-Za-z0-9\\-]*\\.)+[A-Za-z]{2,}$",
                )
        }
        "color" => pattern("^#?([a-fA-F0-9]{6}|[a-fA-F0-9]{3})$"),
        "cc" => pattern(
            "^(4[0-9]{12}([0-9]{3})?|5[1-5][0-9]{14}|6(011|5[0-9][0-9])[0-9]{12}|3[47][0-9]{13}|3(0[0-5]|[68][0-9])[0-9]{11}|(2131|1800|35[0-9]{3})[0-9]{11})$",
        ),
        "price" => regex::Regex::new("^-?[0-9]+(\\.[0-9]{2})?$")
            .is_ok_and(|regex| regex.is_match(&string(value))),
        "date" => super::dates::parsed_date(value, offset).is_some(),
        "future" => super::dates::parsed_date(value, offset).is_some_and(|date| date > now),
        "past" => super::dates::parsed_date(value, offset).is_some_and(|date| date < now),
        "present" => super::dates::parsed_date(value, offset)
            .is_some_and(|date| local_day(date, offset) == local_day(now, offset)),
        // Source schema validation cannot establish database uniqueness; actions enforce it.
        "unique" => true,
        _ => false,
    }
}

/// Source strict equality compares primitive values, without integer/float representation drift.
fn equal(value: &Value, arg: &serde_json::Value) -> bool {
    if let Some((a, b)) = value.as_f64().zip(arg.as_f64()) {
        return a == b;
    }
    match (value, arg) {
        (Value::Null, serde_json::Value::Null) => true,
        (Value::Bool(a), serde_json::Value::Bool(b)) => a == b,
        (Value::String(a), serde_json::Value::String(b)) => a == b,
        _ => false,
    }
}

/// Source "present" compares the host's calendar day, not equality with the current instant.
fn local_day(millis: i64, offset: i32) -> i128 {
    (i128::from(millis) + i128::from(offset) * 60_000).div_euclid(86_400_000)
}

/// Preserve authored interpolation, including the original rejected-value placeholder.
pub(super) fn message(field: &Field, rule: &ValidationRule, value: &Value) -> String {
    let mut message = rule
        .message
        .replace("{{name}}", &field.name)
        .replace(
            "{{label}}",
            field.documentation.label.as_deref().unwrap_or(&field.name),
        )
        .replace("{{value}}", &super::coercion::js_string(value));
    for index in 0..3 {
        let arg = rule
            .args
            .get(index)
            .map(|arg| string(&Value::from(arg.clone())))
            .unwrap_or_else(|| "undefined".into());
        if index == 0 {
            message = message.replace("{{arg}}", &arg);
        }
        message = message.replace(&format!("{{{{arg{}}}}}", index + 1), &arg);
    }
    message
}

/// Match the original first Zod type error without coercing the rejected input.
fn type_error(expected: &str, value: Option<&Value>) -> String {
    let received = match value {
        None => "undefined",
        Some(Value::Null) => "null",
        Some(Value::Bool(_)) => "boolean",
        Some(Value::Number(_) | Value::NonFinite(_)) => "number",
        Some(Value::String(_)) => "string",
        Some(Value::Array(_)) => "array",
        Some(Value::Date { .. }) => "Date",
        Some(Value::Object(_)) => "object",
    };
    format!("Invalid input: expected {expected}, received {received}")
}
