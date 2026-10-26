//! Compact, recursively sorted JSON preserving numeric wire tokens without global serde features.
use serde::Deserialize;
use serde_json::value::RawValue;
use stackpress_sql::{ErrorKind, Result, SqlError};
use std::collections::BTreeMap;
use std::io::{self, Write};

/// Bound parsing input and retained output before exposing a complete canonical document.
/// Numeric tokens retain their spelling/digits; this is not RFC 8785 normalization.
pub(crate) fn canonical(raw: &RawValue, remaining: usize) -> Result<String> {
    // Native packet allocation predates this cap; first-party parsing does not.
    if raw.get().len() > remaining {
        return Err(limit());
    }
    let mut output = BoundedJson {
        bytes: Vec::new(),
        remaining,
        exceeded: false,
    };
    if let Err(error) = emit(raw, &mut output, 0) {
        // Only our writer identifies a retained-output limit; preserve other typed causes.
        return if output.exceeded {
            Err(limit())
        } else {
            Err(error)
        };
    }
    String::from_utf8(output.bytes).map_err(|cause| {
        SqlError::with_source(ErrorKind::Decode, "native JSON encoding failed", cause)
    })
}
/// Validate duplicate keys/depth without allocating a second complete output for a JSON bind.
pub(crate) fn validate(raw: &RawValue) -> Result<()> {
    emit(raw, &mut io::sink(), 0)
}
/// Traverse borrowed raw subtrees, normalizing strings/keys while preserving all numeric digits.
fn emit(raw: &RawValue, output: &mut impl Write, depth: usize) -> Result<()> {
    // RawValue validates JSON grammar; explicit depth also bounds recursive raw-subtree traversal.
    if depth >= 128 {
        return Err(SqlError::new(
            ErrorKind::Decode,
            "JSON nesting exceeds adapter depth",
        ));
    }
    let source = raw.get().trim();
    match source.as_bytes().first() {
        Some(b'{') => {
            let Object(entries) = serde_json::from_str(source).map_err(json_error)?;
            bytes(output, b"{")?;
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index != 0 {
                    bytes(output, b",")?;
                }
                serde_json::to_writer(&mut *output, &key).map_err(json_error)?;
                bytes(output, b":")?;
                emit(value, output, depth + 1)?;
            }
            bytes(output, b"}")?;
        }
        Some(b'[') => {
            let elements: Vec<&RawValue> = serde_json::from_str(source).map_err(json_error)?;
            bytes(output, b"[")?;
            for (index, value) in elements.into_iter().enumerate() {
                if index != 0 {
                    bytes(output, b",")?;
                }
                emit(value, output, depth + 1)?;
            }
            bytes(output, b"]")?;
        }
        Some(b'"') => {
            // Normalize equivalent JSON string escaping without guessing a SQL text value's type.
            let value: String = serde_json::from_str(source).map_err(json_error)?;
            serde_json::to_writer(output, &value).map_err(json_error)?;
        }
        _ => {
            // A grammar-validated number/bool/null needs no f64 conversion or owned tree.
            bytes(output, source.as_bytes())?;
        }
    }
    Ok(())
}
/// Preserve serialization/validation causes behind a payload-independent public context.
fn json_error(cause: serde_json::Error) -> SqlError {
    SqlError::with_source(
        ErrorKind::Decode,
        "invalid or duplicate-key native JSON",
        cause,
    )
}
/// Write raw validated syntax through the same bounded output as normalized strings/keys.
fn bytes(output: &mut impl Write, value: &[u8]) -> Result<()> {
    output.write_all(value).map_err(|cause| {
        SqlError::with_source(ErrorKind::Decode, "native JSON serialization failed", cause)
    })
}
/// Sorted borrowed object entries; duplicates must fail before a map could overwrite them.
struct Object<'a>(BTreeMap<String, &'a RawValue>);
impl<'de> Deserialize<'de> for Object<'de> {
    /// Retain only this object's keys and raw borrowed children; recursion is adapter-owned.
    fn deserialize<D: serde::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        deserializer.deserialize_map(ObjectVisitor)
    }
}
/// Object-specific visitor avoids serde_json::Value's numeric coercion and feature unification.
struct ObjectVisitor;
impl<'de> serde::de::Visitor<'de> for ObjectVisitor {
    type Value = Object<'de>;
    /// Keep parser diagnostics generic; no native data is copied into public SQL messages.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("JSON object with unique keys")
    }
    /// Reject duplicate decoded keys, including equivalent Unicode escape spellings.
    fn visit_map<A: serde::de::MapAccess<'de>>(
        self,
        mut object: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut entries = BTreeMap::new();
        while let Some(key) = object.next_key::<String>()? {
            if entries.contains_key(&key) {
                return Err(serde::de::Error::custom("duplicate JSON object key"));
            }
            entries.insert(key, object.next_value::<&RawValue>()?);
        }
        Ok(Object(entries))
    }
}
/// Output ownership grows only after each serialization write fits the finite byte cap.
struct BoundedJson {
    /// Admitted UTF-8 bytes convert into String without another complete allocation.
    bytes: Vec<u8>,
    /// Whole-statement name/value capacity remaining when this cell starts.
    remaining: usize,
    /// Distinguishes output budget failure from another serialization error.
    exceeded: bool,
}
impl Write for BoundedJson {
    /// Reject the complete write before copying on overflow or a whole-result overrun.
    fn write(&mut self, value: &[u8]) -> io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(value.len())
            .is_none_or(|next| next > self.remaining)
        {
            self.exceeded = true;
            return Err(io::Error::other("JSON output budget exceeded"));
        }
        self.bytes.extend_from_slice(value);
        Ok(value.len())
    }
    /// No native IO is buffered by this in-memory writer.
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Keep budget diagnostics independent of document contents.
fn limit() -> SqlError {
    SqlError::new(ErrorKind::Limit, "SQL JSON parsing/output limit exceeded")
}
