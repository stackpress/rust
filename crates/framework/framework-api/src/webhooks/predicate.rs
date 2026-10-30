//! Source subset/list predicates with explicit Arc container identity.

use serde_json::Value as Json;
use stackpress_lib::{Error, Result, Timestamp, Value};
use std::{collections::BTreeMap, sync::Arc};

/// Source predicate object with explicit shared-container identity.
pub type PredicateObject = Arc<BTreeMap<String, PredicateValue>>;

/// JavaScript-compatible predicate values; cloning a container keeps its identity.
///
/// Separately decoded JSON creates distinct Arc containers. This carrier is intentionally
/// separate from ordinary JSON equality and from low-level Request/Response identity.
#[derive(Clone, Debug)]
pub enum PredicateValue {
    /// A source undefined value; an expected key whose actual value is undefined fails.
    Undefined,
    /// Explicit null, distinct from an absent/undefined member.
    Null,
    /// Source Boolean, never truthiness-coerced for equality.
    Boolean(bool),
    /// Source numeric equality, including SameValueZero for array membership.
    Number(f64),
    /// Source string, distinct from a numeric or Boolean representation.
    String(String),
    /// Array identity is compared by pointer when it is an includes element.
    Array(Arc<Vec<PredicateValue>>),
    /// Object subset checks recurse, but includes elements compare pointer identity.
    Object(PredicateObject),
    /// A Date is not a plain object; strict comparison retains its explicit Arc identity.
    Date(Arc<Timestamp>),
}

impl PredicateValue {
    /// Convert bounded JSON, assigning fresh identity to every decoded container.
    ///
    /// # Errors
    /// Retains the existing schema JSON representation limits; adds no API-specific policy.
    pub fn from_json(value: &Json) -> Result<Self> {
        // Existing finite depth/node checks precede recursive carrier construction.
        stackpress_framework_schema::check_json(value)
            .map_err(stackpress_lib::Error::from_source)?;
        Self::decoded(value)
    }

    /// Convert an already bounded JSON subtree, preserving only explicitly shared Arc identity.
    fn decoded(value: &Json) -> Result<Self> {
        Ok(match value {
            Json::Null => Self::Null,
            Json::Bool(value) => Self::Boolean(*value),
            // Additive arbitrary_precision can retain a numeric lexeme outside finite f64.
            // Parsing its JSON spelling retains source double rounding/overflow without panic.
            Json::Number(value) => Self::Number(match value.as_f64() {
                Some(number) => number,
                None => value
                    .to_string()
                    .parse()
                    .map_err(stackpress_lib::Error::from_source)?,
            }),
            Json::String(value) => Self::String(value.clone()),
            Json::Array(values) => Self::Array(Arc::new(
                values.iter().map(Self::decoded).collect::<Result<_>>()?,
            )),
            Json::Object(values) => Self::Object(Arc::new(
                values
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), Self::decoded(value)?)))
                    .collect::<Result<_>>()?,
            )),
        })
    }
}

/// Match source object subsets and array includes without replacing reference identity.
///
/// The work stack avoids native recursion during matching. It adds no scope, identity or
/// data-projection rule. Empty expected objects match any supplied object.
pub fn valid_data(assertion: &PredicateObject, data: &PredicateObject) -> bool {
    let mut pending = vec![(assertion, data)];
    while let Some((assertion, data)) = pending.pop() {
        for (key, expected) in assertion.iter() {
            // Source checks typeof actual === undefined before selecting the comparison branch.
            let Some(actual) = data
                .get(key)
                .filter(|value| !matches!(value, PredicateValue::Undefined))
            else {
                return false;
            };
            match (expected, actual) {
                (PredicateValue::Array(expected), PredicateValue::Array(actual)) => {
                    // includes is SameValueZero for scalars and identity for container elements.
                    if !expected
                        .iter()
                        .all(|item| actual.iter().any(|other| same_value(item, other, true)))
                    {
                        return false;
                    }
                }
                (PredicateValue::Object(expected), PredicateValue::Object(actual)) => {
                    pending.push((expected, actual));
                }
                // A container expected against another type cannot fall through to scalar equality.
                (PredicateValue::Array(_) | PredicateValue::Object(_), _) => return false,
                _ if !same_value(expected, actual, false) => return false,
                _ => {}
            }
        }
    }
    true
}

/// Compare scalar strict equality or includes SameValueZero, retaining container pointer identity.
fn same_value(left: &PredicateValue, right: &PredicateValue, includes: bool) -> bool {
    match (left, right) {
        (PredicateValue::Undefined, PredicateValue::Undefined)
        | (PredicateValue::Null, PredicateValue::Null) => true,
        (PredicateValue::Boolean(left), PredicateValue::Boolean(right)) => left == right,
        (PredicateValue::Number(left), PredicateValue::Number(right)) => {
            left == right || (includes && left.is_nan() && right.is_nan())
        }
        (PredicateValue::String(left), PredicateValue::String(right)) => left == right,
        (PredicateValue::Array(left), PredicateValue::Array(right)) => Arc::ptr_eq(left, right),
        (PredicateValue::Object(left), PredicateValue::Object(right)) => Arc::ptr_eq(left, right),
        (PredicateValue::Date(left), PredicateValue::Date(right)) => Arc::ptr_eq(left, right),
        _ => false,
    }
}

/// Match source root property access before reusing the original nested subset predicate.
///
/// Empty assertions access no property, even on null. A nonempty assertion against null
/// fails like source `data[key]`; other primitives are boxed and usually have no members.
///
/// # Errors
/// Returns source property-access or inherited representation/depth errors.
pub fn matches(assertion: &PredicateObject, data: Option<&Value>) -> Result<bool> {
    if assertion.is_empty() {
        return Ok(true);
    }
    let actual = data.map(|data| carrier(data, 0)).transpose()?;
    let properties = match actual {
        None | Some(PredicateValue::Null | PredicateValue::Undefined) => {
            return Err(Error::new(
                "webhook validity cannot read null or undefined results",
            ));
        }
        Some(PredicateValue::Object(properties)) => properties,
        Some(PredicateValue::Array(values)) => {
            let mut properties: BTreeMap<_, _> = values
                .iter()
                .cloned()
                .enumerate()
                .map(|(index, value)| (index.to_string(), value))
                .collect();
            properties.insert("length".into(), PredicateValue::Number(values.len() as f64));
            Arc::new(properties)
        }
        Some(PredicateValue::String(value)) => {
            let mut properties = BTreeMap::new();
            // JS boxes strings as UTF-16 indexed properties. Isolated surrogate values have
            // no Rust String representation, so report that mechanics failure explicitly.
            for (index, unit) in value.encode_utf16().enumerate() {
                if assertion.contains_key(&index.to_string()) {
                    let character = char::from_u32(u32::from(unit))
                        .ok_or_else(|| Error::new("isolated UTF-16 property has no Rust string"))?;
                    properties.insert(index.to_string(), PredicateValue::String(character.into()));
                }
            }
            properties.insert(
                "length".into(),
                PredicateValue::Number(value.encode_utf16().count() as f64),
            );
            Arc::new(properties)
        }
        Some(_) => Arc::new(BTreeMap::new()),
    };
    Ok(valid_data(assertion, &properties))
}

/// Convert rich values before JSON serialization, retaining explicitly shared predicates.
fn carrier(value: &Value, depth: usize) -> Result<PredicateValue> {
    if depth > 64 {
        return Err(Error::limit("webhook predicate depth exceeded"));
    }
    Ok(match value {
        Value::Array(values) => PredicateValue::Array(Arc::new(
            values
                .iter()
                .map(|value| carrier(value, depth + 1))
                .collect::<Result<_>>()?,
        )),
        Value::Object(values) => PredicateValue::Object(Arc::new(
            values
                .iter()
                .map(|(key, value)| Ok((key.clone(), carrier(value, depth + 1)?)))
                .collect::<Result<_>>()?,
        )),
        // Source isObject excludes Date; no separately decoded JSON predicate equals it.
        Value::DateTime(value) => PredicateValue::Date(Arc::new(*value)),
        Value::Shared(value) => value
            .downcast_ref::<PredicateValue>()
            .cloned()
            .ok_or_else(|| Error::new("unsupported native webhook predicate value"))?,
        _ => PredicateValue::from_json(&value.to_json()?)?,
    })
}

/// Explicit source JSON.stringify conversion for request/body values, including dates.
///
/// # Errors
/// Returns inherited depth/unsupported native representation errors.
pub fn wire(value: &Value) -> Result<Json> {
    Ok(wire_optional(value)?.unwrap_or(Json::Null))
}

/// Object members with an explicit Undefined carrier are omitted; array entries become null.
pub(crate) fn wire_optional(value: &Value) -> Result<Option<Json>> {
    encode(value, 0)
}

/// Walk only API's selected wire forms; never stringify arbitrary native/debug content.
fn encode(value: &Value, depth: usize) -> Result<Option<Json>> {
    if depth > 64 {
        return Err(Error::limit("webhook JSON depth exceeded"));
    }
    Ok(Some(match value {
        Value::Array(values) => Json::Array(
            values
                .iter()
                .map(|value| Ok(encode(value, depth + 1)?.unwrap_or(Json::Null)))
                .collect::<Result<_>>()?,
        ),
        Value::Object(values) => {
            let mut object = serde_json::Map::new();
            for (key, value) in values {
                if let Some(value) = encode(value, depth + 1)? {
                    object.insert(key.clone(), value);
                }
            }
            Json::Object(object)
        }
        Value::DateTime(value) => date_json(*value),
        Value::Shared(value) => {
            let value = value
                .downcast_ref::<PredicateValue>()
                .ok_or_else(|| Error::new("unsupported native webhook JSON value"))?;
            return encode_carrier(value, depth);
        }
        _ => value.to_json()?,
    }))
}

/// Serialize the explicit Arc carrier with JS undefined and nonfinite-number behavior.
fn encode_carrier(value: &PredicateValue, depth: usize) -> Result<Option<Json>> {
    if depth > 64 {
        return Err(Error::limit("webhook carrier JSON depth exceeded"));
    }
    Ok(Some(match value {
        PredicateValue::Undefined => return Ok(None),
        PredicateValue::Null => Json::Null,
        PredicateValue::Boolean(value) => (*value).into(),
        PredicateValue::Number(value) => {
            serde_json::Number::from_f64(*value).map_or(Json::Null, Json::Number)
        }
        PredicateValue::String(value) => value.clone().into(),
        PredicateValue::Date(value) => date_json(**value),
        PredicateValue::Array(values) => Json::Array(
            values
                .iter()
                .map(|value| Ok(encode_carrier(value, depth + 1)?.unwrap_or(Json::Null)))
                .collect::<Result<_>>()?,
        ),
        PredicateValue::Object(values) => {
            let mut object = serde_json::Map::new();
            for (key, value) in values.iter() {
                if let Some(value) = encode_carrier(value, depth + 1)? {
                    object.insert(key.clone(), value);
                }
            }
            Json::Object(object)
        }
    }))
}

/// Format Date.toJSON's UTC milliseconds, including TimeClip's null outside its range.
fn date_json(value: Timestamp) -> Json {
    // Timestamp uses floor seconds plus positive nanoseconds. Divide the complete instant
    // to truncate toward zero as JS TimeClip does, including instants just before the epoch.
    let millis =
        (i128::from(value.seconds) * 1_000_000_000 + i128::from(value.nanoseconds())) / 1_000_000;
    if millis.abs() > 8_640_000_000_000_000 {
        return Json::Null;
    }
    let days = millis.div_euclid(86_400_000);
    let clock = millis.rem_euclid(86_400_000);
    // Gregorian civil-from-days arithmetic uses 400-year eras. Euclidean division keeps
    // the same calendar for BCE and extended years without a new date-library dependency.
    let shifted = days + 719_468;
    let era = shifted.div_euclid(146_097);
    let day = shifted - era * 146_097;
    let year_in_era = (day - day / 1460 + day / 36524 - day / 146096) / 365;
    let ordinal = day - (365 * year_in_era + year_in_era / 4 - year_in_era / 100);
    let month = (5 * ordinal + 2) / 153;
    let date = ordinal - (153 * month + 2) / 5 + 1;
    let month_number = month + if month < 10 { 3 } else { -9 };
    let year = year_in_era + era * 400 + i128::from(month_number <= 2);
    let year = if (0..=9999).contains(&year) {
        format!("{year:04}")
    } else {
        format!(
            "{}{absolute:06}",
            if year < 0 { "-" } else { "+" },
            absolute = year.abs()
        )
    };
    format!(
        "{year}-{month_number:02}-{date:02}T{:02}:{:02}:{:02}.{:03}Z",
        clock / 3_600_000,
        clock / 60_000 % 60,
        clock / 1000 % 60,
        clock % 1000
    )
    .into()
}
