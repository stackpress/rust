//! Authentication schema and direct actions, preserving the source SQL codec boundary.

use crate::columns::{Record, RecordSchema};
use std::ops::Deref;

#[cfg(feature = "auth-server")]
mod config;
#[cfg(feature = "auth-server")]
pub use config::{
    AuthConfig, AuthEmailConfig, AuthMenuConfig, AuthSection, Environment, RandomFraction,
};
#[cfg(feature = "auth-server")]
mod bindings;
#[cfg(feature = "auth-server")]
pub(crate) mod bridge;
#[cfg(feature = "auth-server")]
pub mod events;
#[cfg(feature = "auth-server")]
mod hooks;
#[cfg(feature = "auth-server")]
pub mod pages;
#[cfg(feature = "auth-server")]
pub use hooks::Hooks;
#[cfg(feature = "auth-server")]
mod plugin;
#[cfg(feature = "auth-server")]
pub use plugin::{register, register_with};
#[cfg(feature = "auth-server")]
pub mod templates;
#[cfg(feature = "auth-server")]
pub mod totp;

/// Source Auth record; token is plaintext after native decode, secret remains its stored hash.
pub type Auth = Record;
/// Partial source input; values are prepared before field assertions.
pub type AuthInput = Record;
/// Source projection including the optional requested profile object.
pub type AuthExtended = Record;

/// Generated authentication columns, with global token uniqueness owned by the store.
#[derive(Clone, Debug)]
pub struct AuthSchema(RecordSchema);
impl AuthSchema {
    /// Direct source constructor uses the caller's database seed, including an empty seed.
    pub fn new(seed: impl Into<String>) -> Self {
        Self(RecordSchema::from_generated::<
            stackpress_framework_models::AuthSchema,
        >(seed.into(), 0))
    }
    /// Select the host's local UTC offset for Date column conversions.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.0 = self.0.with_offset_minutes(offset);
        self
    }
    /// Own the underlying schema for native store composition.
    pub fn into_schema(self) -> RecordSchema {
        self.0
    }
}
impl Default for AuthSchema {
    /// Preserve the original constructor's empty seed.
    fn default() -> Self {
        Self::new("")
    }
}
impl Deref for AuthSchema {
    type Target = RecordSchema;
    /// Borrow the common source schema operations.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(feature = "sql")]
pub use crate::storage::{ActionOptions, AuthActions, AuthStore, PasswordConfig};
