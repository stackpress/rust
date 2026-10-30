//! Source parity and owned-provider proofs, including finite real SQLite; no listener or watcher.
#![cfg(all(feature = "full", not(target_arch = "wasm32")))]

mod cases;
