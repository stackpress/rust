//! Reusable Leptos components, root-scoped state and explicit rendering modes.
//!
//! Independent consumers supply values, callbacks and trusted document framing.
//! This crate has no Stackpress view, server, page protocol or persistence dependency.
//! Native synchronous staging returns owned eager Head/HTML after Owner cleanup;
//! buffered async keeps its root through complete Resource/replay/reactive Head assembly.
//! Opt-in streaming owns a single supervised worker and bounded Send byte bridge.
//! Build SSR, hydrate and CSR separately. Importing a library registers nothing.

#[cfg(any(
    all(feature = "ssr", feature = "hydrate"),
    all(feature = "ssr", feature = "csr"),
    all(feature = "hydrate", feature = "csr")
))]
compile_error!("select exactly one of ssr, hydrate or csr in a separate build");

/// Pinned Leptos APIs for consumers implementing shared components.
pub use leptos;
/// Metadata has one context per render/root.
pub use leptos_meta;
/// Client routing remains distinct from native Stackpress route selection.
pub use leptos_router;

/// Reusable shared UI components and layouts.
pub mod components;
/// Scoped locale and safe phrase/node translation.
pub mod language;
/// Scoped theme and notification state.
pub mod state;

/// Cooperative cancellation for component-owned asynchronous loaders.
#[cfg(feature = "async")]
pub mod resources;

/// Checked inert Resource replay shared by SSR and browser admission.
#[cfg(feature = "async")]
pub mod replay;

/// Optional bounded rich content under one closed sanitizer policy.
#[cfg(feature = "rich")]
pub mod rich;

/// Optional deterministic decimal, ISO date/time and E.164 presentation.
#[cfg(feature = "format")]
pub mod format;

#[cfg(all(feature = "ssr", target_arch = "wasm32"))]
compile_error!("ssr is a native mode; use hydrate or csr for wasm32");

#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
mod owner;

/// Synchronous native SSR with eager owned Head metadata.
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod render;

/// Opt-in complete asynchronous pages with cooperative resource ownership.
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod async_render;

/// Opt-in in-order SSR with a bounded, supervised native bridge.
#[cfg(all(feature = "stream", not(target_arch = "wasm32")))]
pub mod stream;

/// Strict local location values; broad navigation remains components::safe_href.
pub mod navigation;

/// Library-local escaping for inert JSON and ordinary HTML.
pub mod safety;

#[cfg(feature = "async")]
mod replay_error;
