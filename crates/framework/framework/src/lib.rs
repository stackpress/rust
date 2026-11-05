//! Explicit compiled composition of the Stackpress framework.
//!
//! The default feature set selects no plugin or native host. `full` selects the nine
//! source stages; each constituent remains independently usable. Selection performs no
//! listener, database or network IO. Command execution needs an application binding.
//! Native bridges preserve source input/skip/order rules over explicit application-owned
//! execution, revision and rendering providers. `codegen` additionally exposes the real
//! existing Rust build-host APIs; it is independent from selecting browser renderers.

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod native;

/// Exact shipped upstream aggregate Idea source; import as `stackpress/stackpress.idea`.
pub const SCHEMA_SOURCE: &str = include_str!("../stackpress.idea");
#[cfg(all(feature = "language", not(target_arch = "wasm32")))]
pub use native::language;
#[cfg(all(feature = "view", not(target_arch = "wasm32")))]
pub use native::view;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use native::{
    CompiledClient, CompiledModel, FrameworkBindings, Register, client, commands, config,
};
#[cfg(all(
    not(target_arch = "wasm32"),
    any(
        feature = "server-commands",
        feature = "schema-client",
        feature = "language",
        feature = "csrf",
        feature = "sql",
        feature = "view",
        feature = "session",
        feature = "api",
        feature = "admin"
    )
))]
pub use native::{register, register_with};
