//! Explicit presentation adapters; enabling a feature registers nothing.
#[cfg(all(feature = "native-file", unix, not(target_arch = "wasm32")))]
pub mod native;
#[cfg(feature = "cookies")]
pub mod web;
