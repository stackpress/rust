//! Buffered body forms without native streams or implicit wire conversions.

use crate::{ParseLimits, Result, Value};

/// Deeply clonable buffered data, retaining empty versus present null.
#[derive(Clone, Debug, Default)]
pub enum Body {
    /// No supplied body; distinct from JSON null.
    #[default]
    Empty,
    /// Text, including serialized JSON text when explicitly selected.
    Text(String),
    /// Exact raw bytes.
    Bytes(Vec<u8>),
    /// Structured rich in-process data.
    Value(Value),
}
impl Body {
    /// Describe the actual representation without JS type inference.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, ParseLimits};
    /// let body = Body::Bytes(vec![0, 255]);
    /// assert_eq!(body.type_name(), "bytes");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn type_name(&self) -> &'static str {
        // Report buffered representation without inspecting or formatting opaque native payloads.
        match self {
            Self::Empty => "empty",
            Self::Text(_) => "string",
            Self::Bytes(_) => "bytes",
            Self::Value(value) => match value {
                Value::Null => "null",
                Value::Bool(_) => "boolean",
                Value::Number(_) => "number",
                Value::String(_) => "string",
                Value::Array(_) => "array",
                Value::Object(_) => "object",
                Value::DateTime(_) => "date",
                Value::Bytes(_) => "byte-value",
                Value::File(_) => "file",
                Value::Shared(_) => "shared",
            },
        }
    }
    /// Validate data budgets without erasing its representation.
    ///
    /// # Errors
    /// Returns exceeded depth/entry/byte budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{Body, ParseLimits};
    /// let body = Body::Bytes(vec![0, 255]);
    /// body.validate(&ParseLimits::default())?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn validate(&self, limits: &ParseLimits) -> Result<()> {
        // Text/raw bytes use byte limits; rich values additionally enforce recursive node/depth
        // budgets.
        match self {
            Self::Empty => Ok(()),
            Self::Text(text) => crate::data::check_input(text.len(), limits),
            Self::Bytes(bytes) => crate::data::check_input(bytes.len(), limits),
            Self::Value(value) => value.validate(limits),
        }
    }
    /// Move a result view out; text maps to a string and raw bytes stay bytes.
    pub(crate) fn into_value(self) -> Option<Value> {
        // Absence remains None; present empty strings, raw bytes and null retain distinct values.
        match self {
            Self::Empty => None,
            Self::Text(value) => Some(value.into()),
            Self::Bytes(value) => Some(Value::Bytes(value)),
            Self::Value(value) => Some(value),
        }
    }
}
