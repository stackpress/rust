//! Host-free WASM entrypoint with explicit hydration/packaged CSR mode selection.

#[cfg(all(target_arch = "wasm32", any(feature = "hydrate", feature = "csr")))]
mod browser;
