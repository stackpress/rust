//! Leptos integration with explicit SSR, hydrate and CSR compilation modes.
//!
//! M0 supplies the render/ownership boundary. Broader components and routing are
//! later milestones. Native hosts never store an Owner or live reactive state.

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

#[cfg(all(feature = "ssr", target_arch = "wasm32"))]
compile_error!("ssr is a native mode; use hydrate or csr for wasm32");

#[cfg(all(feature = "ssr", not(target_arch = "wasm32")))]
pub mod render;
