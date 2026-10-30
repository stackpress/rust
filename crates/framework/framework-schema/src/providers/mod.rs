//! Explicit native providers shared by account and API stores.
//!
//! Enable `providers`; default/browser schema graphs retain no native entropy or codecs.
//! `cuid` is CUID2-compatible, while encrypt/decrypt/hash preserve the original schema
//! helper byte formats. These codecs add no authentication envelope or format migration.

mod crypto;
mod cuid;
use crate::{SchemaError, SchemaErrorKind, SchemaResult, ValueProviders};
pub use crypto::{decrypt, encrypt, hash};
pub use cuid::{CuidGenerator, cuid};
use serde_json::Value;

/// Opt-in built-in provider for the schema `cuid()` default, without implicit host policy.
#[derive(Default)]
pub struct BuiltinProviders;
impl ValueProviders for BuiltinProviders {
    /// Generate exactly the source no-argument cuid default; unknown names stay explicit errors.
    fn generate(&self, name: &str, args: &[Value]) -> SchemaResult<Value> {
        // A source cuid() is not a generic option bag; reject arguments rather than ignore them.
        if name == "cuid" && args.is_empty() {
            return cuid().map(Value::String);
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
