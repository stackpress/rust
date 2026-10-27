//! Checked portable values. Parsing never consults a machine timezone or clock.

use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use time::{Date, OffsetDateTime, Time, UtcOffset, format_description};

/// Reject malformed values without placing the rejected data in an error message.
fn invalid() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "invalid canonical value")
}

/// A finite IEEE-754 number with no implicit lossy JSON integer conversion.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FiniteNumber(f64);
impl FiniteNumber {
    /// Check finiteness before retaining a floating-point value.
    pub fn new(value: f64) -> SchemaResult<Self> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(invalid())
        }
    }
    /// Read the already checked finite number.
    pub fn get(self) -> f64 {
        self.0
    }
    /// Decode a JSON number, rejecting integer-to-real precision loss.
    pub fn from_json(value: &Value) -> SchemaResult<Self> {
        let number = value.as_f64().ok_or_else(invalid)?;
        if let Some(integer) = value.as_i64() {
            if number as i128 != i128::from(integer) {
                return Err(invalid());
            }
        } else if let Some(integer) = value.as_u64()
            && number as i128 != i128::from(integer)
        {
            return Err(invalid());
        }
        Self::new(number)
    }
}
impl<'de> Deserialize<'de> for FiniteNumber {
    /// Use the same strict conversion for direct and transport input.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Self::from_json(&Value::deserialize(decoder)?).map_err(serde::de::Error::custom)
    }
}

/// Bound a JSON tree iteratively before cloning or embedding it in a descriptor.
/// # Errors
/// Refuses more than 10,000 nodes, depth 32, or one MiB of logical text/scalars.
pub fn check_json(value: &Value) -> SchemaResult<()> {
    JsonBudget::default().charge(value)
}

/// One aggregate owner for existing values plus every default inserted during preparation.
#[derive(Default)]
pub(crate) struct JsonBudget {
    /// Cumulative visited/container values across all values owned by this preparation.
    nodes: usize,
    /// Cumulative logical scalar/key bytes; never reset between inserted defaults.
    bytes: usize,
}
impl JsonBudget {
    /// Charge borrowed data before any clone or insertion can increase retained memory.
    pub(crate) fn charge(&mut self, value: &Value) -> SchemaResult<()> {
        let mut pending = vec![(value, 0usize)];
        while let Some((value, depth)) = pending.pop() {
            self.nodes = self.nodes.saturating_add(1);
            if self.nodes > 10_000 || depth > 32 {
                return Err(json_limit());
            }
            match value {
                Value::Array(values) => {
                    if values.len() + pending.len() > 10_000 {
                        return Err(json_limit());
                    }
                    pending.extend(values.iter().map(|value| (value, depth + 1)));
                }
                Value::Object(values) => {
                    if values.len() + pending.len() > 10_000 {
                        return Err(json_limit());
                    }
                    for (key, value) in values {
                        self.charge_bytes(key.len())?;
                        pending.push((value, depth + 1));
                    }
                }
                Value::String(value) => self.charge_bytes(value.len())?,
                _ => self.charge_bytes(8)?,
            }
        }
        Ok(())
    }
    /// Account for newly inserted object keys independently of their value subtree.
    pub(crate) fn charge_bytes(&mut self, bytes: usize) -> SchemaResult<()> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > 1024 * 1024 {
            Err(json_limit())
        } else {
            Ok(())
        }
    }
}
/// Report a finite JSON resource boundary without reflecting input data.
fn json_limit() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Limit, "JSON value budget exceeded")
}

/// Bounded JSON object; nested values may contain arrays, scalar values and JSON null.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(transparent)]
pub struct JsonObject(serde_json::Map<String, Value>);
impl JsonObject {
    /// Check the complete tree and require an object at its root.
    pub fn new(value: Value) -> SchemaResult<Self> {
        check_json(&value)?;
        match value {
            Value::Object(value) => Ok(Self(value)),
            _ => Err(invalid()),
        }
    }
    /// Borrow checked object members without allowing unchecked mutation.
    pub fn members(&self) -> &serde_json::Map<String, Value> {
        &self.0
    }
}
impl<'de> Deserialize<'de> for JsonObject {
    /// Enforce the same root and tree budgets at the wire boundary.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Self::new(Value::deserialize(decoder)?).map_err(serde::de::Error::custom)
    }
}

/// Canonical calendar date, in years 0001 through 9999, exactly YYYY-MM-DD.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct DateValue(String);
/// Canonical local time, seconds required, with up to nine fractional digits and trailing zeros removed.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct TimeValue(String);
/// Canonical UTC RFC3339 instant, seconds required, with up to nine fractional digits.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DatetimeValue(String);

impl DateValue {
    /// Parse an exact calendar date without timezone inference.
    pub fn parse(value: &str) -> SchemaResult<Self> {
        if value.len() != 10 {
            return Err(invalid());
        }
        let format = format_description::parse_borrowed::<2>("[year]-[month]-[day]")
            .map_err(|_| invalid())?;
        let date = Date::parse(value, &format).map_err(|cause| invalid().with_cause(cause))?;
        if !(1..=9999).contains(&date.year()) {
            return Err(invalid());
        }
        Ok(Self(date.to_string()))
    }
    /// Borrow the canonical date text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TimeValue {
    /// Parse local time at nanosecond precision; no offset or missing seconds is accepted.
    pub fn parse(value: &str) -> SchemaResult<Self> {
        let format = if value.len() == 8 {
            "[hour]:[minute]:[second]"
        } else {
            "[hour]:[minute]:[second].[subsecond digits:1+]"
        };
        if value.len() > 18 {
            return Err(invalid());
        }
        let format = format_description::parse_borrowed::<2>(format).map_err(|_| invalid())?;
        let time = Time::parse(value, &format).map_err(|cause| invalid().with_cause(cause))?;
        let mut text = format!(
            "{:02}:{:02}:{:02}",
            time.hour(),
            time.minute(),
            time.second()
        );
        if time.nanosecond() != 0 {
            text.push('.');
            text.push_str(format!("{:09}", time.nanosecond()).trim_end_matches('0'));
        }
        Ok(Self(text))
    }
    /// Borrow canonical local time text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl DatetimeValue {
    /// Parse an explicit RFC3339 offset and normalize to UTC; leap seconds are refused.
    pub fn parse(value: &str) -> SchemaResult<Self> {
        // RFC3339 parsers may discard excess fractional precision. Refuse it explicitly.
        let fraction = value
            .get(19..)
            .and_then(|tail| tail.strip_prefix('.'))
            .map(|tail| tail.bytes().take_while(u8::is_ascii_digit).count());
        if value.len() > 40
            || value.get(17..19) == Some("60")
            || fraction.is_some_and(|digits| digits == 0 || digits > 9)
        {
            return Err(invalid());
        }
        let instant = OffsetDateTime::parse(value, &format_description::well_known::Rfc3339)
            .map_err(|cause| invalid().with_cause(cause))?
            .to_offset(UtcOffset::UTC);
        if !(1..=9999).contains(&instant.year()) {
            return Err(invalid());
        }
        let text = instant
            .format(&format_description::well_known::Rfc3339)
            .map_err(|_| invalid())?;
        Ok(Self(text))
    }
    /// Borrow the canonical UTC representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Return nanoseconds since the Unix epoch for explicit-clock comparisons.
    pub fn unix_nanos(&self) -> i128 {
        OffsetDateTime::parse(&self.0, &format_description::well_known::Rfc3339)
            .expect("checked datetime invariant")
            .unix_timestamp_nanos()
    }
}

// All string wrappers validate at deserialization and never retain unparsed text.
macro_rules! decode_string {
    ($name:ty) => {
        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
                Self::parse(&String::deserialize(decoder)?).map_err(serde::de::Error::custom)
            }
        }
    };
}
decode_string!(DateValue);
decode_string!(TimeValue);
decode_string!(DatetimeValue);

/// Check whether an integer can cross a JavaScript-number wire boundary exactly.
/// # Errors
/// Refuses integers outside the interoperable signed 53-bit range.
pub fn browser_integer(value: i64) -> SchemaResult<i64> {
    if (-9_007_199_254_740_991..=9_007_199_254_740_991).contains(&value) {
        Ok(value)
    } else {
        Err(invalid())
    }
}

/// Serialize within a one-MiB byte budget before allocating a JSON value tree.
/// # Errors
/// Returns Limit before any later serializer work when the next write exceeds the budget;
/// otherwise retains the original typed codec cause beneath distinct context.
pub fn to_bounded_value<T: Serialize>(value: &T) -> SchemaResult<Value> {
    use std::io::Write;
    struct Buffer {
        bytes: Vec<u8>,
        exceeded: bool,
    }
    impl Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > 1024 * 1024 - self.bytes.len() {
                self.exceeded = true;
                return Err(std::io::Error::other("value byte budget exceeded"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        exceeded: false,
    };
    serde_json::to_writer(&mut buffer, value).map_err(|cause| {
        SchemaError::new(
            if buffer.exceeded {
                SchemaErrorKind::Limit
            } else {
                SchemaErrorKind::Codec
            },
            "bounded value encoding failed",
        )
        .with_cause(cause)
    })?;
    let value = serde_json::from_slice(&buffer.bytes).map_err(|cause| {
        SchemaError::new(SchemaErrorKind::Codec, "bounded value decoding failed").with_cause(cause)
    })?;
    check_json(&value)?;
    Ok(value)
}

/// Check all numeric values in an approved browser projection without silently rounding integers.
/// # Errors
/// Refuses JSON budgets or any integer outside the interoperable signed 53-bit range.
pub fn check_browser_json(value: &Value) -> SchemaResult<()> {
    check_json(value)?;
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            Value::Number(number) => {
                if let Some(value) = number.as_i64() {
                    browser_integer(value)?;
                } else if number.as_u64().is_some() {
                    return Err(invalid());
                }
            }
            Value::Array(values) => pending.extend(values),
            Value::Object(values) => pending.extend(values.values()),
            _ => {}
        }
    }
    Ok(())
}

/// One creation-wide value budget, shared by every generated embedded input.
#[derive(Default)]
pub struct PreparationBudget {
    budget: JsonBudget,
}
impl PreparationBudget {
    /// Charge one owned scalar/list/object before retaining it in a prepared record.
    /// # Errors
    /// Refuses the aggregate byte/node budget or malformed nested JSON before another clone/append.
    pub fn charge<T: Serialize>(&mut self, name: &str, value: &T) -> SchemaResult<()> {
        self.budget.charge_bytes(name.len())?;
        let value = to_bounded_value(value)?;
        self.budget.charge(&value)
    }
    /// Charge an embedded record/list container whose members are charged individually.
    pub fn container(&mut self, name: &str) -> SchemaResult<()> {
        self.budget.charge_bytes(name.len())?;
        self.budget.charge(&Value::Null)
    }
}
