//! Explicit integration of reusable Leptos with Stackpress page contracts.
#[cfg(all(feature = "leptos-ssr", not(target_arch = "wasm32")))]
pub mod async_render;
/// Public-value presentation mapping, without persistence acquisition.
pub mod presentation;
#[cfg(all(feature = "leptos-ssr", not(target_arch = "wasm32")))]
pub mod render;
#[cfg(feature = "leptos-async")]
pub mod replay;
pub mod routing;
#[cfg(all(feature = "leptos-stream", not(target_arch = "wasm32")))]
pub mod stream;

/// View-local escaping, guarded against drift by the shared test table.
pub mod safety;
