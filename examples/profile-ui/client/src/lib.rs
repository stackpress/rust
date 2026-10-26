//! Host-free WASM entrypoint with explicit hydration/packaged CSR mode selection.

#[cfg(any(all(target_arch = "wasm32", feature = "hydrate"), test))]
mod assets_compat;

#[cfg(any(target_arch = "wasm32", test))]
mod seed;

#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
mod browser;
