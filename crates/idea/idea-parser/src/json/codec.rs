//! Duplicate-aware JSON decoding and bounded serialization, independent of schema meaning.
//!
//! The iterative preflight guards the recursive Serde visitor before allocations. Callers
//! validate config structure separately; this module never evaluates references or env.

use serde::Serialize;
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value};
use std::io::{self, Write};

use super::{limit, validate_value, validation};
use crate::{ErrorCode, IdeaError, ParseLimits};

/// Deserialize structure only after a nonrecursive text pass has bounded depth and bytes.
pub(crate) fn read_json(file: &str, source: &str, limits: ParseLimits) -> Result<Value, IdeaError> {
    preflight(source, limits).map_err(|error| error.locate(file, source))?;
    let mut decoder = serde_json::Deserializer::from_str(source);
    let value = UniqueValue::deserialize(&mut decoder)
        .and_then(|value| {
            decoder.end()?;
            Ok(value.0)
        })
        .map_err(|error| {
            // Serde reports byte columns. Convert them to a UTF-8 source boundary once.
            let start = source
                .split_inclusive('\n')
                .take(error.line().saturating_sub(1))
                .map(str::len)
                .sum::<usize>()
                + error.column().saturating_sub(1);
            let code = if error.to_string().contains("duplicate JSON key") {
                ErrorCode::DuplicateName
            } else {
                ErrorCode::ValidationError
            };
            IdeaError::new(code, "Invalid normalized JSON")
                .with_span(start, start)
                .locate(file, source)
        })?;
    validate_value(&value, limits).map_err(|error| error.locate(file, source))?;
    Ok(value)
}

/// Bound nesting before recursive decoding, respecting JSON quoting and escaped quotes.
fn preflight(source: &str, limits: ParseLimits) -> Result<(), IdeaError> {
    if source.len() > limits.max_bytes {
        return Err(limit("JSON byte budget exceeded").with_span(0, 0));
    }
    // This iterative pass runs before Serde allocates any recursive container.
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut nodes = 0usize;
    let mut collections: Vec<usize> = Vec::new();
    let bytes = source.as_bytes();
    while index < bytes.len() {
        if matches!(
            bytes[index],
            b'"' | b'[' | b'{' | b'-' | b'0'..=b'9' | b't' | b'f' | b'n'
        ) {
            nodes += 1;
            if nodes > limits.max_nodes() {
                return Err(limit("JSON node budget exceeded").with_span(index, index));
            }
        }
        match bytes[index] {
            b'"' => {
                index += 1;
                // Escapes are JSON syntax, separate from Idea's intentionally raw strings.
                while index < bytes.len() && bytes[index] != b'"' {
                    if bytes[index] == b'\\' {
                        index += 1;
                    }
                    index += 1;
                }
            }
            b'[' | b'{' => {
                // The hard ceiling cannot be raised by caller-supplied JSON limits.
                depth += 1;
                collections.push(1);
                if depth > limits.max_depth.min(ParseLimits::HARD_DEPTH) {
                    return Err(limit("JSON nesting limit exceeded").with_span(index, index + 1));
                }
            }
            b']' | b'}' => {
                depth = depth.saturating_sub(1);
                collections.pop();
            }
            b',' => {
                // Commas inside strings were skipped; each remaining comma starts an entry.
                if let Some(entries) = collections.last_mut() {
                    *entries += 1;
                    if *entries > limits.max_collection() {
                        return Err(
                            limit("JSON collection budget exceeded").with_span(index, index)
                        );
                    }
                }
            }
            b'-' | b'0'..=b'9' => {
                let start = index;
                index += 1;
                while index < bytes.len()
                    && matches!(bytes[index], b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-')
                {
                    index += 1;
                }
                let raw = &source[start..index];
                // JSON's default number visitor otherwise rounds overflowing integers to f64.
                if !raw.contains(['.', 'e', 'E'])
                    && raw.parse::<i64>().is_err()
                    && raw.parse::<u64>().is_err()
                {
                    return Err(IdeaError::new(
                        ErrorCode::InvalidLiteral,
                        "Integer outside 64-bit range",
                    )
                    .with_span(start, index));
                }
                // Decimal/exponent overflow is a literal-range failure, not JSON null data.
                if raw.contains(['.', 'e', 'E'])
                    && raw.parse::<f64>().is_ok_and(|value| !value.is_finite())
                {
                    return Err(
                        IdeaError::new(ErrorCode::InvalidLiteral, "Nonfinite JSON number")
                            .with_span(start, index),
                    );
                }
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    Ok(())
}

/// Ordered value visitor that refuses duplicate keys instead of silently keeping the last.
struct UniqueValue(Value);

impl<'de> Deserialize<'de> for UniqueValue {
    /// Decode through the duplicate-aware visitor for every nested map and sequence.
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        decoder.deserialize_any(UniqueVisitor)
    }
}

/// Recursively used only after the text-level hard depth check.
struct UniqueVisitor;

impl<'de> Visitor<'de> for UniqueVisitor {
    type Value = UniqueValue;
    /// Describe normalized input without echoing confidential data into errors.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("bounded JSON data")
    }
    /// Preserve JSON null.
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueValue(Value::Null))
    }
    /// Preserve boolean data.
    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Preserve signed integer precision.
    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Preserve unsigned integer precision.
    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Accept only finite JSON numbers.
    fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(|value| UniqueValue(Value::Number(value)))
            .ok_or_else(|| E::custom("nonfinite number"))
    }
    /// Own decoded strings only within the already bounded source.
    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(UniqueValue(value.into()))
    }
    /// Retain sequence order.
    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element::<UniqueValue>()? {
            values.push(value.0);
        }
        Ok(UniqueValue(Value::Array(values)))
    }
    /// Retain map order while detecting duplicates at every nesting level.
    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
        let mut values = Map::new();
        while let Some(key) = entries.next_key::<String>()? {
            if values.contains_key(&key) {
                return Err(de::Error::custom("duplicate JSON key"));
            }
            let value = entries.next_value::<UniqueValue>()?;
            values.insert(key, value.0);
        }
        Ok(UniqueValue(Value::Object(values)))
    }
}

/// Bounded serializer sink that can count bytes without allocating an intermediate string.
struct Sink {
    /// Encoded output; kept empty when this sink measures without collecting.
    bytes: Vec<u8>,
    /// Remaining serialized bytes, debited before extending the allocation.
    remaining: usize,
    /// Select allocation only for explicit export, rather than reference-copy accounting.
    collect: bool,
}
impl Write for Sink {
    /// Charge serialized bytes before extending the output allocation.
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.remaining = self
            .remaining
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("JSON byte budget exceeded"))?;
        if self.collect {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(bytes.len())
    }
    /// There is no external IO to flush.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// Measure already validated, depth-bounded data without allocating serialized output.
pub(crate) fn measure<T: Serialize>(value: &T, maximum: usize) -> Result<usize, IdeaError> {
    let mut sink = Sink {
        bytes: Vec::new(),
        remaining: maximum,
        collect: false,
    };
    serde_json::to_writer(&mut sink, value)
        .map_err(|_| limit("Serialized byte budget exceeded"))?;
    Ok(maximum - sink.remaining)
}

/// Serialize bounded data with an allocation limit enforced by the writer.
pub(crate) fn encode<T: Serialize>(value: &T, limits: ParseLimits) -> Result<String, IdeaError> {
    let mut sink = Sink {
        bytes: Vec::new(),
        remaining: limits.max_bytes,
        collect: true,
    };
    serde_json::to_writer(&mut sink, value)
        .map_err(|_| limit("Serialized byte budget exceeded"))?;
    String::from_utf8(sink.bytes).map_err(|_| validation("JSON serialization failed"))
}
