//! Metadata-selected numeric/list/enum/fieldset codecs, preserving the original conversion order.
use super::{Column, Kind, Record, SourceColumn, Value, as_date, string, to_json};
use crate::{SchemaError, SchemaErrorKind, SchemaResult, ValueType};

/// Encode/decode one source column; lists invoke item conversion without a scalar-storage flag.
pub(super) fn convert(
    column: &SourceColumn,
    value: Option<&Value>,
    seed: &str,
    storage: bool,
    offset: i32,
    encode: bool,
) -> SchemaResult<Option<Value>> {
    if column.field.multiple {
        let parsed = if encode {
            None
        } else {
            value
                .and_then(Value::as_str)
                .and_then(|text| serde_json::from_str::<Value>(text).ok())
        };
        let values = value
            .and_then(Value::as_array)
            .or_else(|| parsed.as_ref().and_then(Value::as_array));
        let mut result = Vec::new();
        if let Some(values) = values {
            for value in values {
                if let Some(value) = scalar(column, Some(value), seed, false, offset, encode, true)?
                    && !value.is_null()
                {
                    result.push(value);
                }
            }
        }
        return if encode {
            encode_json(&Value::Array(result)).map(|value| Some(Value::String(value)))
        } else {
            Ok(Some(Value::Array(result)))
        };
    }
    scalar(column, value, seed, storage, offset, encode, false)
}

/// Native scalar conversion; enum String(value) differs from ordinary JSON-object string columns.
fn scalar(
    column: &SourceColumn,
    value: Option<&Value>,
    seed: &str,
    storage: bool,
    offset: i32,
    encode: bool,
    item: bool,
) -> SchemaResult<Option<Value>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if column.field.value_type == ValueType::Unknown {
        return Ok(Some(value.clone()));
    }
    if !item && !column.field.required && value.is_null() {
        return Ok(Some(Value::Null));
    }
    if matches!(
        column.field.value_type,
        ValueType::Integer | ValueType::Number
    ) {
        return Ok(Some(number(value)));
    }
    if column.field.value_type == ValueType::String
        && matches!(value, Value::Number(_) | Value::NonFinite(_))
    {
        let text = Value::String(js_string(value));
        let mut codec = column.codec().clone();
        // Array wrappers call a scalar item serializer, not the outer Strings codec.
        if codec.kind == Kind::Strings {
            codec.kind = Kind::String;
        }
        return if encode {
            codec.serialize(Some(&text), seed, storage, offset)
        } else {
            codec.unserialize(Some(&text), seed, storage, offset)
        };
    }
    if matches!(column.field.value_type, ValueType::Enum(_)) {
        return Ok(Some(Value::String(js_string(value))));
    }
    if let Some(schema) = column.embedded(seed.into(), offset) {
        let parsed = value
            .as_str()
            .and_then(|text| serde_json::from_str::<Value>(text).ok());
        let input = value
            .as_object()
            .or_else(|| parsed.as_ref().and_then(Value::as_object));
        let Some(input) = input else {
            return Ok(Some(if encode {
                Value::String("{}".into())
            } else {
                Value::Object(Record::new())
            }));
        };
        if encode {
            let converted = schema.serialize(input, false)?;
            // Original parseJSONValue parses every valid JSON string, not only object/list text.
            let parsed = converted
                .into_iter()
                .map(|(key, value)| {
                    let value = value
                        .as_str()
                        .and_then(|text| serde_json::from_str::<Value>(text).ok())
                        .unwrap_or(value);
                    (key, value)
                })
                .collect();
            return encode_json(&Value::Object(parsed)).map(|text| Some(Value::String(text)));
        }
        return schema
            .unserialize(input, false)
            .map(|record| Some(Value::Object(record)));
    }
    let mut codec: Column = column.codec().clone();
    if item {
        codec.nullable = false;
        if codec.kind == Kind::Strings {
            codec.kind = Kind::String;
        }
    }
    if encode {
        codec.serialize(Some(value), seed, storage, offset)
    } else {
        codec.unserialize(Some(value), seed, storage, offset)
    }
}

/// JavaScript Number conversion for the finite native value family, including radix literals.
fn number(value: &Value) -> Value {
    let number = if let Some(date) = as_date(value) {
        date as f64
    } else {
        match value {
            Value::Null => 0.0,
            Value::Bool(value) => {
                if *value {
                    1.0
                } else {
                    0.0
                }
            }
            Value::Number(value) => value.as_f64().unwrap_or(0.0),
            Value::NonFinite(value) => {
                if value.is_nan() {
                    0.0
                } else {
                    *value
                }
            }
            Value::Object(_) => 0.0,
            _ => {
                let text = js_string(value);
                let text = super::trim(&text);
                if text.is_empty() {
                    0.0
                } else if let Some(hex) =
                    text.strip_prefix("0x").or_else(|| text.strip_prefix("0X"))
                {
                    radix_number(hex, 16)
                } else if let Some(binary) =
                    text.strip_prefix("0b").or_else(|| text.strip_prefix("0B"))
                {
                    radix_number(binary, 2)
                } else if let Some(octal) =
                    text.strip_prefix("0o").or_else(|| text.strip_prefix("0O"))
                {
                    radix_number(octal, 8)
                } else if matches!(text, "Infinity" | "+Infinity" | "-Infinity")
                    || text
                        .bytes()
                        .all(|b| b.is_ascii_digit() || b"+-eE.".contains(&b))
                {
                    text.parse::<f64>()
                        .ok()
                        .filter(|value| !value.is_nan())
                        .unwrap_or(0.0)
                } else {
                    0.0
                }
            }
        }
    };
    if !number.is_finite() {
        Value::NonFinite(number)
    } else if number == 0.0 {
        Value::from(0)
    } else if number.fract() == 0.0 && number >= i64::MIN as f64 && number < i64::MAX as f64 {
        Value::from(number as i64)
    } else {
        Value::from(number)
    }
}

/// Source String coercion for enum values and assertion placeholders.
pub(super) fn js_string(value: &Value) -> String {
    match value {
        Value::Object(_) => "[object Object]".into(),
        Value::Number(value) => value
            .as_f64()
            .map(super::number_text)
            .unwrap_or_else(|| value.to_string()),
        Value::NonFinite(value) => super::number_text(*value),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    String::new()
                } else {
                    js_string(value)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        _ => string(value),
    }
}

/// Encode nested native Dates through the same source JSON stringify behavior.
fn encode_json(value: &Value) -> SchemaResult<String> {
    serde_json::to_string(&to_json(value)?).map_err(|cause| {
        SchemaError::new(SchemaErrorKind::Codec, "source column JSON encoding failed")
            .with_cause(cause)
    })
}

/// Parse power-of-two radix literals with one IEEE-754 ties-to-even rounding step.
/// A u64 parser would silently turn valid larger literals into zero; repeated f64
/// multiplication would instead round at each digit. Keep 53 bits plus guard/sticky bits.
fn radix_number(text: &str, radix: u32) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let width = radix.trailing_zeros();
    let mut count = 0usize;
    let mut top = 0u64;
    let mut guard = false;
    let mut sticky = false;
    for c in text.chars() {
        let Some(digit) = c.to_digit(radix) else {
            return 0.0;
        };
        for shift in (0..width).rev() {
            let bit = (digit >> shift) & 1;
            if count == 0 && bit == 0 {
                continue;
            }
            if count < 53 {
                top = (top << 1) | u64::from(bit)
            } else if count == 53 {
                guard = bit != 0
            } else {
                sticky |= bit != 0
            }
            count += 1;
        }
    }
    if guard && (sticky || top & 1 != 0) {
        top += 1;
    }
    top as f64 * 2_f64.powi(count.saturating_sub(53).min(i32::MAX as usize) as i32)
}
