//! Explicit integration of reusable Leptos with Stackpress page contracts.
//!
//! Browser routing/replay/presentation consume checked public values. Native adapters
//! retain page admission, framing and error types while delegating execution to the
//! generic library. None of these imports registers a plugin or acquires preferences.
/// Complete native page adapter retaining Resource/replay/reactive Head ownership.
#[cfg(all(feature = "leptos-ssr", not(target_arch = "wasm32")))]
pub mod async_render;
/// Public-value presentation mapping, without persistence acquisition.
pub mod presentation;
/// Synchronous native page adapter with eager owned Head and checked assets.
#[cfg(all(feature = "leptos-ssr", not(target_arch = "wasm32")))]
pub mod render;
/// Checked page-policy wrapper around generic Resource replay.
#[cfg(feature = "leptos-async")]
pub mod replay;
/// Leptos matching and root mount context over view's compiled route contracts.
pub mod routing;
/// Opt-in page stream framing, completion/recovery identity and mapped errors.
#[cfg(all(feature = "leptos-stream", not(target_arch = "wasm32")))]
pub mod stream;

/// View-local escaping, guarded against drift by the shared test table.
pub mod safety;
