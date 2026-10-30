//! Explicit compiled composition of the Stackpress framework.
//!
//! The default feature set selects no plugin or native host. `full` selects the nine
//! source stages; each constituent remains independently usable. Selection performs no
//! listener, database or network IO. Command execution needs an application binding.
//! M0 provides the compiled-client and buffered locale-dispatch proof; complete command,
//! rendering and generation integrations are later milestones, not implied by `full`.

#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
mod native;
#[cfg(all(feature = "language", not(target_arch = "wasm32")))]
pub use native::language;
#[cfg(all(feature = "view", not(target_arch = "wasm32")))]
pub use native::view;
#[cfg(all(feature = "native", not(target_arch = "wasm32")))]
pub use native::{CompiledClient, CompiledModel, Register, client, commands};
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
