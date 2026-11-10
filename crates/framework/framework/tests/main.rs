//! Source parity and owned-provider proofs, including finite real SQLite; no listener or watcher.
#![cfg(all(feature = "full", not(target_arch = "wasm32")))]

mod cases;
#[cfg(feature = "codegen")]
#[path = "cases/generated_family.rs"]
mod generated_family;

#[path = "fixtures/generation.rs"]
mod fixture_generation;
#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "fixtures/models/mod.rs"]
mod models;
/// Local output is regenerated independently from the shipped Idea sources.
#[test]
fn generated_models_are_fresh() {
    fixture_generation::fixtures(true).unwrap();
}
