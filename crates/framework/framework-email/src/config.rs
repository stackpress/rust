//! Lossless source email option/string union; adapters own provider defaults.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// All source object variants and connection strings share this extension-preserving carrier.
///
/// Objects include generic, SMTP, pool, JSON, stream, sendmail and SES options.
/// Native callbacks/resources live in the factory, replacing JavaScript object methods.
/// No object field is stripped, defaulted or rejected by this carrier.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum EmailConfig {
    /// Original transport object, including unknown application/provider options.
    Options(Map<String, Value>),
    /// Original connection string, retaining credentials and transport syntax.
    ConnectionString(String),
}

impl EmailConfig {
    /// Decode an optional source section while preserving missing/falsy activation.
    ///
    /// Empty objects are active (JavaScript truthiness); empty strings are inactive.
    /// This is also used at the direct event gate, independent of registration.
    ///
    /// # Errors
    /// Truthy values outside the declared source object/string union fail decoding.
    pub fn from_section(value: Option<&Value>) -> Result<Option<Self>, serde_json::Error> {
        // Source !config disables on missing, null, false, zero and empty string.
        match value {
            None | Some(Value::Null | Value::Bool(false)) => Ok(None),
            Some(Value::Number(number)) if number.as_f64() == Some(0.0) => Ok(None),
            Some(Value::String(text)) if text.is_empty() => Ok(None),
            Some(value) => serde_json::from_value(value.clone()).map(Some),
        }
    }
}
