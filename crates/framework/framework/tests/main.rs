//! Real M0 host proofs; no socket, database or native development process is started.
#![cfg(all(feature = "full", not(target_arch = "wasm32")))]

mod cases;
