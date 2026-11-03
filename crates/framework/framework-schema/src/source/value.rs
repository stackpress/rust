//! Source native values distinguish Date from arbitrary JSON without reserving object keys.
use serde::{Deserialize, Serialize, Serializer};
use stackpress_lib::DataMap;
use std::{fmt, ops::Index};

/// A finite source value. JSON decoding never guesses that a string/object is a Date.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Value {
    /// Explicit native Date, skipped by JSON decoding; hosts construct it with `date`.
    #[serde(skip)]
    Date {
        /// Epoch milliseconds.
        millis: i64,
        /// Explicit host Date.toString spelling for nested array coercion.
        display: Option<String>,
    },
    /// JSON null.
    Null,
    /// JSON Boolean.
    Bool(bool),
    /// Finite JSON number.
    Number(serde_json::Number),
    /// JSON text.
    String(String),
    /// Source list, which may contain native Dates.
    Array(Vec<Value>),
    /// Arbitrary object; no key spelling has a hidden native meaning.
    Object(DataMap<String, Value>),
}
impl Serialize for Value {
    /// Match JSON.stringify: native Dates become UTC ISO text; ordinary objects stay objects.
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Date { millis, .. } => super::dates::iso(*millis)
                .map_err(serde::ser::Error::custom)?
                .serialize(serializer),
            Self::Null => serializer.serialize_none(),
            Self::Bool(value) => value.serialize(serializer),
            Self::Number(value) => value.serialize(serializer),
            Self::String(value) => value.serialize(serializer),
            Self::Array(value) => value.serialize(serializer),
            Self::Object(value) => value.serialize(serializer),
        }
    }
}
impl Value {
    /// Borrow text only; Dates remain their own native family.
    pub fn as_str(&self) -> Option<&str> {
        if let Self::String(value) = self {
            Some(value)
        } else {
            None
        }
    }
    /// Borrow a list without converting missing/null to an empty list.
    pub fn as_array(&self) -> Option<&Vec<Self>> {
        if let Self::Array(value) = self {
            Some(value)
        } else {
            None
        }
    }
    /// Borrow an ordinary object, without treating a native Date as a plain object.
    pub fn as_object(&self) -> Option<&DataMap<String, Self>> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }
    /// Exclusively borrow an ordinary object for source projection nesting.
    pub fn as_object_mut(&mut self) -> Option<&mut DataMap<String, Self>> {
        if let Self::Object(value) = self {
            Some(value)
        } else {
            None
        }
    }
    /// Read an exact signed number, without coercing a Date or string.
    pub fn as_i64(&self) -> Option<i64> {
        if let Self::Number(value) = self {
            value.as_i64()
        } else {
            None
        }
    }
    /// Read the finite numeric representation.
    pub fn as_f64(&self) -> Option<f64> {
        if let Self::Number(value) = self {
            value.as_f64()
        } else {
            None
        }
    }
    /// Read a Boolean without source truthiness coercion.
    pub fn as_bool(&self) -> Option<bool> {
        if let Self::Bool(value) = self {
            Some(*value)
        } else {
            None
        }
    }
    /// Read an optional object member.
    pub fn get(&self, key: &str) -> Option<&Self> {
        self.as_object()?.get(key)
    }
    /// Whether this is JSON null.
    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }
    /// Whether this is ordinary text.
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }
    /// Whether this is a native list.
    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
    /// Whether this is an ordinary object.
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }
    /// Whether this is a Boolean.
    pub fn is_boolean(&self) -> bool {
        matches!(self, Self::Bool(_))
    }
}
impl Index<&str> for Value {
    type Output = Self;
    /// Source-style optional nested reads return null for an absent member.
    fn index(&self, key: &str) -> &Self {
        self.get(key).unwrap_or(&Self::Null)
    }
}
impl fmt::Display for Value {
    /// Render finite JSON for source column conversion; Date errors stay fallible in codecs.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match serde_json::to_string(self) {
            Ok(value) => f.write_str(&value),
            Err(_) => Err(fmt::Error),
        }
    }
}
impl From<serde_json::Value> for Value {
    /// Import JSON recursively without recognizing any private marker or special key.
    fn from(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(value) => Self::Bool(value),
            serde_json::Value::Number(value) => Self::Number(value),
            serde_json::Value::String(value) => Self::String(value),
            serde_json::Value::Array(values) => {
                Self::Array(values.into_iter().map(Self::from).collect())
            }
            serde_json::Value::Object(values) => Self::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, Self::from(value)))
                    .collect(),
            ),
        }
    }
}
impl From<String> for Value {
    /// Retain owned text literally.
    fn from(value: String) -> Self {
        Self::String(value)
    }
}
impl From<&str> for Value {
    /// Own borrowed text for a source record.
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}
impl From<bool> for Value {
    /// Preserve a Boolean rather than converting it to a number.
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}
macro_rules! integer {
    ($($kind:ty),*)=>{$(impl From<$kind> for Value {
        /// Preserve an exact JSON integer.
        fn from(value:$kind)->Self{Self::Number(value.into())}
    })*};
}
integer!(i32, i64, u8, u32, u64);
impl From<f64> for Value {
    /// Finite source JSON representation; nonfinite values use JSON.stringify's null.
    fn from(value: f64) -> Self {
        serde_json::Number::from_f64(value)
            .map(Self::Number)
            .unwrap_or(Self::Null)
    }
}
impl PartialEq<serde_json::Value> for Value {
    /// Compare wire values for source-oracle assertions without erasing native type storage.
    fn eq(&self, other: &serde_json::Value) -> bool {
        serde_json::to_value(self).is_ok_and(|value| value == *other)
    }
}
impl PartialEq<str> for Value {
    /// Compare native text without converting other families.
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}
impl PartialEq<&str> for Value {
    /// Compare borrowed string literals.
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}
impl PartialEq<bool> for Value {
    /// Compare actual Boolean values.
    fn eq(&self, other: &bool) -> bool {
        self.as_bool() == Some(*other)
    }
}
impl PartialEq<i32> for Value {
    /// Compare actual integral numbers.
    fn eq(&self, other: &i32) -> bool {
        self.as_i64() == Some(i64::from(*other))
    }
}
