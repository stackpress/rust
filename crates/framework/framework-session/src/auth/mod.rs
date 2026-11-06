//! Authentication schema and direct actions, preserving the source SQL codec boundary.

use crate::columns::Record;

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

#[cfg(feature = "sql")]
pub use crate::storage::{AuthActions, AuthStore, PasswordConfig};
/// Generated default Auth schema, with token uniqueness owned by its generated store.
pub use stackpress_framework_models::AuthSchema;
