//! Source column preparation, distinct from SQL encryption and hashing.
//!
//! Records retain omitted members separately from JSON null. Rust represents Date values
//! as distinct native Date values after decoding; `serialize_date` handles a Date in a text column.
//! Hosts supply their local UTC offset explicitly rather than consulting process globals.

mod dates;
mod value;
pub use value::Value;
mod model;
mod schema;
#[cfg(feature = "sql")]
pub(crate) use dates::iso;
pub use dates::{date_millis, date_text};
#[cfg(feature = "sql")]
pub(crate) use schema::now;
pub use schema::{Record, RecordSchema};

/// Construct a native Date, keeping it distinct from numeric input and arbitrary objects.
/// This representation belongs to direct native records, not automatic browser publication.
pub fn date(millis: i64) -> Value {
    Value::Date {
        millis,
        display: None,
    }
}

/// Carry a host's Date.toString spelling for nested array coercion.
pub fn date_with_display(millis: i64, display: impl Into<String>) -> Value {
    Value::Date {
        millis,
        display: Some(display.into()),
    }
}

/// Read a native Date, leaving every ordinary JSON object unchanged.
pub fn as_date(value: &Value) -> Option<i64> {
    if let Value::Date { millis, .. } = value {
        Some(*millis)
    } else {
        None
    }
}

use stackpress_framework_schema::{SchemaResult, providers};

/// One generated column's source codec family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    /// String conversion, with optional nullable image handling.
    String,
    /// Arrays convert every present element to a string, including null.
    Strings,
    /// Nullable object whose invalid encodings decode to an empty object.
    Object,
    /// Source Boolean truthiness, including special serialized strings.
    Boolean,
    /// Source Date, retained as a distinct native value rather than an ordinary number.
    Date,
    /// Auth token, encrypted only when the caller selects storage encoding.
    Token,
    /// Auth secret, hashed only when the caller selects storage encoding.
    Secret,
}

/// Immutable column metadata and its source conversion behavior.
#[derive(Clone, Debug)]
pub struct Column {
    /// Source field spelling.
    pub name: &'static str,
    /// SQL spelling, including profile_id.
    pub sql: &'static str,
    /// Conversion family.
    pub kind: Kind,
    /// Source optional/nullable column.
    pub nullable: bool,
    /// First refine message for empty text, absent for unrestricted strings.
    pub required_message: Option<&'static str>,
    /// Source string minimum measured in JavaScript UTF-16 code units.
    pub minimum: Option<usize>,
    /// Source literal default, excluding generated CUID and clock defaults.
    pub default: Option<serde_json::Value>,
}

impl Column {
    /// Source per-column default getter: fresh identity/Date, literal value or undefined.
    pub fn defaults_at(&self, now: i64) -> SchemaResult<Option<Value>> {
        Ok(match self.name {
            "id" => Some(Value::String(providers::cuid()?)),
            "created" | "updated" | "consumed" => Some(date(now)),
            _ => self.default.clone().map(Value::from),
        })
    }
    /// Resolve the source getter at the native clock.
    pub fn defaults(&self) -> SchemaResult<Option<Value>> {
        self.defaults_at(schema::now())
    }

    /// Convert a supplied field; None represents JavaScript undefined.
    ///
    /// `storage` enables only the source's token encryption/secret hashing. SQL scalar
    /// conversion remains the store's responsibility. Encryption errors propagate.
    pub fn serialize(
        &self,
        value: Option<&Value>,
        seed: &str,
        storage: bool,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        // List columns deliberately produce [] even for an omitted source value.
        if self.kind == Kind::Strings {
            let values = value
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .map(|value| text(value, offset))
                        .collect::<SchemaResult<Vec<_>>>()
                })
                .transpose()?
                .unwrap_or_default();
            return Ok(Some(Value::String(
                serde_json::to_string(&values).map_err(json_error)?,
            )));
        }
        let Some(value) = value else { return Ok(None) };
        // Nullable image/references retain explicit null rather than String(null).
        if self.nullable && value.is_null() {
            return Ok(Some(Value::Null));
        }
        let result = match self.kind {
            Kind::Boolean => Value::Bool(match value.as_str() {
                Some("false" | "0") => false,
                Some("true" | "1") => true,
                _ => truthy(value),
            }),
            Kind::Object => Value::String(match value.as_str() {
                Some(text) if object(text).is_some() => text.to_owned(),
                Some(_) => "{}".into(),
                None => serde_json::to_string(&to_json(value)?).map_err(json_error)?,
            }),
            Kind::Date => Value::String(date_text(date_millis(value, offset), offset)?),
            _ => {
                let text = text(value, offset)?;
                // Source empty token/secret bypasses their storage codec.
                let text = if storage && !text.is_empty() {
                    match self.kind {
                        Kind::Token => providers::encrypt(&text, seed),
                        Kind::Secret => providers::hash(&text),
                        _ => text,
                    }
                } else {
                    text
                };
                Value::String(text)
            }
        };
        Ok(Some(result))
    }

    /// Decode storage or preparation values; no validation is added to source coercion.
    pub fn unserialize(
        &self,
        value: Option<&Value>,
        seed: &str,
        storage: bool,
        offset: i32,
    ) -> SchemaResult<Option<Value>> {
        if self.kind == Kind::Strings {
            let parsed = value
                .and_then(Value::as_str)
                .and_then(|text| serde_json::from_str::<Value>(text).ok());
            let values = value
                .and_then(Value::as_array)
                .or_else(|| parsed.as_ref().and_then(Value::as_array));
            return Ok(Some(Value::Array(
                values
                    .map(|values| {
                        values
                            .iter()
                            .map(|value| text(value, offset).map(Value::String))
                            .collect::<SchemaResult<Vec<_>>>()
                    })
                    .transpose()?
                    .unwrap_or_default(),
            )));
        }
        let Some(value) = value else { return Ok(None) };
        if self.nullable && value.is_null() {
            return Ok(Some(Value::Null));
        }
        // Source Date branch returns before token decryption, including storage=true.
        if as_date(value).is_some()
            && matches!(self.kind, Kind::String | Kind::Token | Kind::Secret)
        {
            return Ok(Some(Value::String(text(value, offset)?)));
        }
        let result = match self.kind {
            Kind::Boolean => Value::Bool(truthy(value)),
            Kind::Object => value
                .as_str()
                .and_then(object)
                .or_else(|| {
                    if as_date(value).is_none() {
                        value.as_object().cloned()
                    } else {
                        None
                    }
                })
                .map(Value::Object)
                .unwrap_or_else(|| Value::Object(Record::new())),
            Kind::Date => date(date_millis(value, offset)),
            _ => {
                let text = text(value, offset)?;
                Value::String(if storage && self.kind == Kind::Token && !text.is_empty() {
                    providers::decrypt(&text, seed)?
                } else {
                    text
                })
            }
        };
        Ok(Some(result))
    }

    /// Convert a native Date input to the source's mixed UTC-date/local-time spelling.
    pub fn serialize_date(&self, millis: i64, offset: i32) -> SchemaResult<Value> {
        Ok(Value::String(date_text(millis, offset)?))
    }

    /// Return the source's first assertion message, without coercing the tested value.
    pub fn assert(&self, value: Option<&Value>) -> Option<String> {
        if self.nullable && value.is_none_or(Value::is_null) {
            return None;
        }
        let expected = match self.kind {
            Kind::Strings => "array",
            Kind::Object => "object",
            Kind::Boolean => "boolean",
            Kind::Date => "date",
            _ => "string",
        };
        let valid = value.is_some_and(|value| match self.kind {
            Kind::Strings => value
                .as_array()
                .is_some_and(|values| values.iter().all(Value::is_string)),
            Kind::Object => value.is_object() || as_date(value).is_some(),
            Kind::Boolean => value.is_boolean(),
            Kind::Date => as_date(value).is_some(),
            _ => value.is_string(),
        });
        if !valid {
            if self.kind == Kind::Strings
                && let Some(values) = value.and_then(Value::as_array)
                && let Some(value) = values.iter().find(|value| !value.is_string())
            {
                let received = match value {
                    value if as_date(value).is_some() => "Date",
                    Value::Null => "null",
                    Value::Number(_) => "number",
                    Value::Bool(_) => "boolean",
                    Value::Array(_) => "array",
                    _ => "object",
                };
                return Some(format!(
                    "Invalid input: expected string, received {received}"
                ));
            }
            let received = match value {
                None => "undefined",
                Some(Value::Null) => "null",
                Some(Value::Array(_)) => "array",
                Some(value) if as_date(value).is_some() => "Date",
                Some(Value::Object(_)) => "object",
                Some(Value::String(_)) => "string",
                Some(Value::Bool(_)) => "boolean",
                _ => "number",
            };
            return Some(format!(
                "Invalid input: expected {expected}, received {received}"
            ));
        }
        let text = value.and_then(Value::as_str).unwrap_or_default();
        if let Some(message) = self
            .required_message
            .filter(|_| text.is_empty() && expected == "string")
        {
            return Some(message.into());
        }
        if self
            .minimum
            .is_some_and(|minimum| text.encode_utf16().count() < minimum)
        {
            return Some("Must be at least 5 characters".into());
        }
        None
    }
}

/// JavaScript truthiness for finite JSON values; objects and empty arrays remain true.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64() != Some(0.0),
        Value::String(value) => !value.is_empty(),
        _ => true,
    }
}

/// Source String conversion, using JSON.stringify for plain objects and array toString.
pub(crate) fn string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Date { millis, display } => display.clone().unwrap_or_else(|| {
            crate::codecs::date_string(*millis, 0, "Coordinated Universal Time")
                .unwrap_or_else(|_| "Invalid Date".into())
        }),
        Value::Object(_) => value.to_string(),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    String::new()
                } else if value.is_object() {
                    "[object Object]".into()
                } else {
                    string(value)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        _ => value.to_string(),
    }
}

/// Parse only plain JSON objects; arrays, null and malformed JSON become no object.
fn object(text: &str) -> Option<Record> {
    serde_json::from_str::<Value>(text)
        .ok()?
        .as_object()
        .cloned()
}

/// Preserve JSON encoding causes under the schema's existing error contract.
fn json_error(cause: serde_json::Error) -> stackpress_framework_schema::SchemaError {
    stackpress_framework_schema::SchemaError::new(
        stackpress_framework_schema::SchemaErrorKind::Codec,
        "column JSON encoding failed",
    )
    .with_cause(cause)
}

/// Date-aware column String conversion; ordinary JSON objects still use JSON.stringify.
fn text(value: &Value, offset: i32) -> SchemaResult<String> {
    if let Some(millis) = as_date(value) {
        return date_text(millis, offset);
    }
    match value {
        Value::Array(values) => array_string(values, offset),
        Value::Object(_) => Ok(to_json(value)?.to_string()),
        _ => Ok(string(value)),
    }
}

/// Array.toString uses Date.toString, unlike a Date passed directly to a column.
fn array_string(values: &[Value], offset: i32) -> SchemaResult<String> {
    values
        .iter()
        .map(|value| {
            if let Some(millis) = as_date(value) {
                // Native hosts can attach their Date.toString spelling explicitly. UTC is the
                // default native Date context, matching the rest of the direct schema defaults.
                if let Value::Date {
                    display: Some(display),
                    ..
                } = value
                {
                    return Ok(display.to_owned());
                }
                return crate::codecs::date_string(
                    millis,
                    offset,
                    if offset == 0 {
                        "Coordinated Universal Time"
                    } else {
                        ""
                    },
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Unsupported,
                        "native Date display failed",
                    )
                    .with_cause(cause)
                });
            }
            match value {
                Value::Null => Ok(String::new()),
                Value::Array(values) => array_string(values, offset),
                Value::Object(_) => Ok("[object Object]".into()),
                _ => Ok(string(value)),
            }
        })
        .collect::<SchemaResult<Vec<_>>>()
        .map(|values| values.join(","))
}

/// JSON.stringify turns native Date values into UTC ISO text, even when nested.
pub fn to_json(value: &Value) -> SchemaResult<serde_json::Value> {
    serde_json::to_value(value).map_err(json_error)
}
