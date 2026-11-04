//! Explicit native providers shared by account and API stores.
//!
//! Enable `providers`; default/browser schema graphs retain no native entropy or codecs.
//! `cuid` is CUID2-compatible, while encrypt/decrypt/hash preserve the original schema
//! helper byte formats. These codecs add no authentication envelope or format migration.

mod crypto;
mod cuid;
mod nanoid;
use crate::{SchemaError, SchemaErrorKind, SchemaResult, ValueProviders};
pub use crypto::{decrypt, encrypt, hash};
pub use cuid::{CuidGenerator, cuid};
pub use nanoid::{NANOID_ALPHABET, nanoid, nanoid_with_length};
use serde_json::Value;

/// Opt-in built-in provider for the schema `cuid()` default, without implicit host policy.
#[derive(Default)]
pub struct BuiltinProviders;
impl ValueProviders for BuiltinProviders {
    /// Generate source CUID/nanoid defaults with optional lengths; unknown names remain errors.
    fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value> {
        match (name, args) {
            ("cuid", []) => return cuid().map(Value::String),
            ("nanoid", []) => return nanoid().map(Value::String),
            ("cuid" | "nanoid", [length]) => {
                if let Some(length) = length
                    .as_u64()
                    .and_then(|length| usize::try_from(length).ok())
                {
                    return if name == "cuid" {
                        CuidGenerator::with_source_length(length)?.generate()
                    } else {
                        nanoid_with_length(length)
                    }
                    .map(Value::String);
                }
            }
            _ => {}
        }
        Err(SchemaError::new(
            SchemaErrorKind::Unsupported,
            "unknown built-in value provider or arguments",
        ))
    }
    /// No remote or host validators are selected by choosing the value provider.
    fn validate(&self, _: &str, _: &Value, _: &[Value]) -> SchemaResult<bool> {
        Err(SchemaError::new(
            SchemaErrorKind::Unsupported,
            "no built-in host validator",
        ))
    }
}
#[cfg(test)]
mod tests;

/// Build-host definition for the same no-argument runtime `cuid()` provider.
/// Add it with `DefinitionRegistry::with_provider` before normalization; registration stays
/// explicit and the existing registry still rejects duplicate providers.
#[cfg(feature = "codegen")]
pub fn cuid_definition() -> crate::codegen::ProviderDefinition {
    crate::codegen::ProviderDefinition {
        name: "cuid".into(),
        revision: "cuid2-3.3.0".into(),
        output: crate::ValueType::String,
    }
}
