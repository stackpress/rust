//! Email/session/SQL/SSR integration lives after its dependencies in the publication order.
#![cfg(all(feature = "auth-server", not(target_arch = "wasm32")))]

#[path = "email/fixtures/generation.rs"]
mod fixture_generation;
#[path = "email/integration/mod.rs"]
mod integration;
#[allow(dead_code, missing_docs, unexpected_cfgs)]
#[path = "email/fixtures/models/mod.rs"]
mod models;

/// The moved family is checked with the same public generator, without repairing stale output.
#[test]
fn generated_models_are_fresh() {
    fixture_generation::fixtures(true).unwrap();
}
