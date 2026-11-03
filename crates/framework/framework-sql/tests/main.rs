//! Function plugin contracts over supplied resources, without a native database or listener.
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
#[path = "main/plugin.rs"]
mod plugin;

#[cfg(feature = "codegen")]
#[path = "main/codegen.rs"]
mod codegen;
