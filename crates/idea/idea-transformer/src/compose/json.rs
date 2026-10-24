//! Strict normalized JSON import, bounded before recursion and with duplicate-key detection.
//!
//! This internal adapter bridges frozen M0 config, whose types do not implement Deserialize.
//! It can be replaced by the parser's validated JSON API after integration, without changing
//! source acquisition or merge policy. Strings remain literals; placeholders are not inferred.

use std::collections::HashSet;
use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use stackpress_idea_parser::{AttributeValue, ColumnConfig, ParseLimits, SchemaConfig, TypeConfig};

use crate::{LoadedSource, Result, TransformError};

/// Validate complete bounded JSON and then convert only the normalized config shape.
pub(super) fn parse(
    source: &LoadedSource,
    limits: ParseLimits,
    max_nodes: usize,
) -> Result<SchemaConfig> {
    if source.text.len() > limits.max_bytes {
        return Err(TransformError::new("JSON source byte budget exceeded"));
    }
    check_integer_range(&source.text)
        .map_err(|cause| super::context(format!("JSON source {} failed", source.name), cause))?;
    let mut remaining = max_nodes;
    let mut deserializer = serde_json::Deserializer::from_str(&source.text);
    let value = Seed {
        depth: 0,
        max_depth: limits.max_depth.min(ParseLimits::HARD_DEPTH),
        remaining: &mut remaining,
    }
    .deserialize(&mut deserializer)
    .map_err(|cause| super::context(format!("JSON source {} failed", source.name), cause))?;
    deserializer
        .end()
        .map_err(|cause| super::context(format!("JSON source {} failed", source.name), cause))?;
    schema(&value)
        .map_err(|cause| super::context(format!("JSON schema {} failed", source.name), cause))
}

/// A reborrowed node budget and depth bound shared by one deserialization traversal.
struct Seed<'a> {
    /// Current JSON value depth; checked before recursive deserialization.
    depth: usize,
    /// Caller-lowered ceiling, never above the hard native recursion limit.
    max_depth: usize,
    /// Remaining nodes, charged before allocation of each corresponding Value.
    remaining: &'a mut usize,
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;

    /// Check depth and node counts before serde constructs or descends into a value.
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Value, D::Error> {
        if self.depth > self.max_depth || *self.remaining == 0 {
            return Err(de::Error::custom("JSON depth/node budget exceeded"));
        }
        *self.remaining -= 1;
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;

    /// Describe data-only values without including any source contents.
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("bounded normalized schema JSON")
    }

    /// Retain exact boolean data.
    fn visit_bool<E: de::Error>(self, value: bool) -> std::result::Result<Value, E> {
        Ok(Value::Bool(value))
    }

    /// Retain null without a synthetic string or absent-value conversion.
    fn visit_unit<E: de::Error>(self) -> std::result::Result<Value, E> {
        Ok(Value::Null)
    }

    /// Retain exact signed integers, avoiding floating-point coercion.
    fn visit_i64<E: de::Error>(self, value: i64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    /// Retain exact unsigned integers through the full serde_json range.
    fn visit_u64<E: de::Error>(self, value: u64) -> std::result::Result<Value, E> {
        Ok(Value::Number(value.into()))
    }

    /// Reject nonfinite numbers instead of silently publishing null.
    fn visit_f64<E: de::Error>(self, value: f64) -> std::result::Result<Value, E> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| de::Error::custom("nonfinite JSON number"))
    }

    /// Copy one already byte-bounded JSON string without interpreting reference placeholders.
    fn visit_str<E: de::Error>(self, value: &str) -> std::result::Result<Value, E> {
        Ok(Value::String(value.into()))
    }

    /// Deserialize arrays in order, sharing the aggregate node counter with all descendants.
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> std::result::Result<Value, A::Error> {
        let mut values = Vec::new();
        // Do not reserve an untrusted serde size hint; each next node checks its own budget.
        while let Some(value) = sequence.next_element_seed(Seed {
            depth: self.depth + 1,
            max_depth: self.max_depth,
            remaining: self.remaining,
        })? {
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    /// Reject duplicate keys at every depth before their losing values can be erased.
    fn visit_map<A: MapAccess<'de>>(self, mut object: A) -> std::result::Result<Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = object.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            let value = object.next_value_seed(Seed {
                depth: self.depth + 1,
                max_depth: self.max_depth,
                remaining: self.remaining,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

/// Validate allowed root sections, cross-kind uniqueness and ordered normalized declarations.
fn schema(value: &Value) -> Result<SchemaConfig> {
    let root = object(value)?;
    fields(root, &["use", "plugin", "prop", "enum", "type", "model"])?;
    let mut result = SchemaConfig::default();
    let mut names = HashSet::new();
    for (section, value) in root {
        if section == "use" {
            // Imports must be a list of literal source specifiers, never arbitrary data.
            for specifier in array(value)? {
                result.r#use.push(string(specifier)?.into());
            }
            continue;
        }
        for (name, value) in object(value)? {
            if !names.insert(name) || (section != "plugin" && !capital_name(name)) {
                return Err(invalid());
            }
            match section.as_str() {
                "plugin" | "prop" => {
                    let properties = object(value)?;
                    if properties.keys().any(|key| !member_name(key)) {
                        return Err(invalid());
                    }
                    let properties = properties.clone().into_iter().collect();
                    if section == "prop" {
                        result.prop.insert(name.clone(), properties);
                    } else {
                        result.plugin.insert(name.clone(), properties);
                    }
                }
                "enum" => {
                    let options = object(value)?;
                    if options.iter().any(|(name, value)| {
                        !enum_name(name) || matches!(value, Value::Array(_) | Value::Object(_))
                    }) {
                        return Err(invalid());
                    }
                    result
                        .r#enum
                        .insert(name.clone(), options.clone().into_iter().collect());
                }
                "type" => {
                    result
                        .r#type
                        .insert(name.clone(), type_config(name, value)?);
                }
                "model" => {
                    result.model.insert(name.clone(), type_config(name, value)?);
                }
                _ => return Err(invalid()),
            }
        }
    }
    Ok(result)
}

/// Require a named type with complete normalized flags, attributes and unique ordered columns.
fn type_config(name: &str, value: &Value) -> Result<TypeConfig> {
    let value = object(value)?;
    fields(value, &["name", "mutable", "attributes", "columns"])?;
    if string(required(value, "name")?)? != name {
        return Err(invalid());
    }
    let mut names = HashSet::new();
    let mut columns = Vec::new();
    for value in array(required(value, "columns")?)? {
        let value = object(value)?;
        fields(
            value,
            &["name", "type", "required", "multiple", "attributes"],
        )?;
        let name = string(required(value, "name")?)?;
        let column_type = string(required(value, "type")?)?;
        if !member_name(name)
            || name.as_bytes()[0].is_ascii_uppercase()
            || !capital_name(column_type)
            || !names.insert(name.to_owned())
        {
            return Err(invalid());
        }
        columns.push(ColumnConfig {
            name: name.into(),
            column_type: column_type.into(),
            required: boolean(required(value, "required")?)?,
            multiple: boolean(required(value, "multiple")?)?,
            attributes: attributes(required(value, "attributes")?)?,
        });
    }
    Ok(TypeConfig {
        name: name.into(),
        mutable: boolean(required(value, "mutable")?)?,
        attributes: attributes(required(value, "attributes")?)?,
        columns,
    })
}

/// Validate open metadata syntax without supplying later framework semantics.
fn attributes<T: FromIterator<(String, AttributeValue)>>(value: &Value) -> Result<T> {
    let mut attributes = Vec::new();
    for (name, value) in object(value)? {
        if !name.split('.').all(|part| {
            part.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        }) {
            return Err(invalid());
        }
        // Attribute arrays are argument lists; strings are not implicitly made into references.
        let value = match value {
            Value::Bool(value) => AttributeValue::Boolean(*value),
            Value::Array(values) => AttributeValue::Array(values.clone()),
            _ => return Err(invalid()),
        };
        attributes.push((name.clone(), value));
    }
    Ok(attributes.into_iter().collect())
}

/// Require one object boundary; arrays/null cannot masquerade as normalized maps.
fn object(value: &Value) -> Result<&Map<String, Value>> {
    value.as_object().ok_or_else(invalid)
}

/// Require an ordered list boundary.
fn array(value: &Value) -> Result<&Vec<Value>> {
    value.as_array().ok_or_else(invalid)
}

/// Require a string, without disclosing its invalid contents in the public error.
fn string(value: &Value) -> Result<&str> {
    value.as_str().ok_or_else(invalid)
}

/// Require a boolean rather than silently inventing default mutability/modifier flags.
fn boolean(value: &Value) -> Result<bool> {
    value.as_bool().ok_or_else(invalid)
}

/// Require a normalized field instead of conflating absent and empty data.
fn required<'a>(value: &'a Map<String, Value>, key: &str) -> Result<&'a Value> {
    value.get(key).ok_or_else(invalid)
}

/// Reject unknown structural fields while leaving metadata/prop contents open.
fn fields(value: &Map<String, Value>, allowed: &[&str]) -> Result<()> {
    if value.keys().any(|key| !allowed.contains(&key.as_str())) {
        return Err(invalid());
    }
    Ok(())
}

/// Match the parser's ASCII declaration-name grammar.
fn capital_name(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_uppercase() || *byte == b'_')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Match ordinary source member names without allowing empty/numeric-leading keys.
fn member_name(name: &str) -> bool {
    name.as_bytes()
        .first()
        .is_some_and(|byte| byte.is_ascii_alphabetic() || *byte == b'_')
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

/// Match uppercase enum keys in the corrected Idea grammar.
fn enum_name(name: &str) -> bool {
    capital_name(name) && !name.bytes().any(|byte| byte.is_ascii_lowercase())
}

/// Report normalized shape failure without echoing arbitrary input values.
fn invalid() -> TransformError {
    TransformError::new("invalid normalized schema JSON")
}

/// Reject integer overflow before serde_json can coerce an out-of-range integer to a float.
fn check_integer_range(text: &str) -> Result<()> {
    let bytes = text.as_bytes();
    let mut index = 0;
    let mut quoted = false;
    while index < bytes.len() {
        let byte = bytes[index];
        // JSON escapes apply only inside strings; skip their next byte without interpreting it.
        if quoted && byte == b'\\' {
            index += 2;
            continue;
        }
        if byte == b'"' {
            quoted = !quoted;
            index += 1;
            continue;
        }
        if !quoted && (byte == b'-' || byte.is_ascii_digit()) {
            let start = index;
            index += 1;
            while index < bytes.len()
                && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
            {
                index += 1;
            }
            // All bytes in this range are ASCII, so the slice endpoints are UTF-8 boundaries.
            let number = &text[start..index];
            if !number.contains(['.', 'e', 'E']) {
                let valid = if number.starts_with('-') {
                    number.parse::<i64>().is_ok()
                } else {
                    number.parse::<u64>().is_ok()
                };
                if !valid {
                    return Err(TransformError::new("JSON integer range exceeded"));
                }
            }
            continue;
        }
        index += 1;
    }
    Ok(())
}
