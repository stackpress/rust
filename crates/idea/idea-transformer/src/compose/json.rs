//! Strict normalized JSON import, bounded before recursion and with duplicate-key detection.
//!
//! The composer owns its explicit node budget; the parser owns normalized config and typed
//! reference validation. Plain strings remain literals. Only a checked CompiledSchema envelope
//! supplies reference metadata, so no string-based discovery can reinterpret user data.

use std::fmt;

use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};
use stackpress_idea_parser::{CompiledSchema, ParseLimits, SchemaConfig};

use crate::{LoadedSource, Result, TransformError};

/// Apply composition budgets, then use parser-owned plain/typed normalized JSON validation.
pub(super) fn parse(
    source: &LoadedSource,
    limits: ParseLimits,
    max_nodes: usize,
) -> Result<CompiledSchema> {
    if source.text.len() > limits.max_bytes {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Limit,
            "JSON source byte budget exceeded",
        ));
    }
    check_integer_range(&source.text)
        .map_err(|cause| super::context(format!("JSON source {} failed", source.name), cause))?;
    let mut remaining = max_nodes;
    let mut exhausted = false;
    let mut deserializer = serde_json::Deserializer::from_str(&source.text);
    let value = Seed {
        depth: 0,
        max_depth: limits.max_depth.min(ParseLimits::HARD_DEPTH),
        remaining: &mut remaining,
        exhausted: &mut exhausted,
    }
    .deserialize(&mut deserializer)
    .map_err(|cause| {
        // The traversal records budget exhaustion directly; never classify serde message text.
        let kind = if exhausted {
            crate::TransformErrorKind::Limit
        } else {
            crate::TransformErrorKind::Parse
        };
        TransformError::caused(kind, format!("JSON source {} failed", source.name), cause)
    })?;
    deserializer
        .end()
        .map_err(|cause| super::context(format!("JSON source {} failed", source.name), cause))?;
    // Typed envelopes preserve explicit sites; plain-schema strings never discover references.
    if value
        .as_object()
        .is_some_and(|root| root.contains_key("config") || root.contains_key("references"))
    {
        return CompiledSchema::from_json(&source.text, limits)
            .map_err(|cause| super::context(format!("JSON schema {} failed", source.name), cause));
    }
    let config = SchemaConfig::from_json_named(&source.name, &source.text, limits)
        .map_err(|cause| super::context(format!("JSON schema {} failed", source.name), cause))?;
    Ok(CompiledSchema {
        config,
        references: Vec::new(),
    })
}

/// A reborrowed node budget and depth bound shared by one deserialization traversal.
struct Seed<'a> {
    /// Current JSON value depth; checked before recursive deserialization.
    depth: usize,
    /// Caller-lowered ceiling, never above the hard native recursion limit.
    max_depth: usize,
    /// Remaining nodes, charged before allocation of each corresponding Value.
    remaining: &'a mut usize,
    /// Records a policy rejection independently of serde's erased custom error text.
    exhausted: &'a mut bool,
}

impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;

    /// Check depth and node counts before serde constructs or descends into a value.
    fn deserialize<D: de::Deserializer<'de>>(
        self,
        deserializer: D,
    ) -> std::result::Result<Value, D::Error> {
        if self.depth > self.max_depth || *self.remaining == 0 {
            *self.exhausted = true;
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
            exhausted: self.exhausted,
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
                exhausted: self.exhausted,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
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
                    return Err(TransformError::classified(
                        crate::TransformErrorKind::Parse,
                        "JSON integer range exceeded",
                    ));
                }
            }
            continue;
        }
        index += 1;
    }
    Ok(())
}
