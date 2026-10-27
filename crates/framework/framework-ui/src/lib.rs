//! Leptos integration with explicit SSR, hydrate and CSR compilation modes.
//!
//! Native buffered rendering and compiled route descriptors share checked public
//! contracts. Synchronous staging exposes only owned buffers. The explicit async
//! renderer owns a request-local root in its complete buffered future.

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

#[cfg(all(feature = "ssr", target_arch = "wasm32"))]
compile_error!("ssr is a native mode; use hydrate or csr for wasm32");

#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod render;

/// Opt-in complete asynchronous pages with cooperative resource ownership.
#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod async_render;

/// Opt-in in-order SSR with a bounded, supervised native bridge.
#[cfg(all(feature = "stream", not(target_arch = "wasm32")))]
pub mod stream;

/// Checked route context and Leptos path matching over a compiled route catalog.
pub mod routing;
