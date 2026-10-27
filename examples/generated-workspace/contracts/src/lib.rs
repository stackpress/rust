//! Pure consumer of the exact generated contracts, without native runtime/tool dependencies.

/// Pure generated contracts, shared verbatim with the native application.
#[path = "../../app/src/generated/contracts/mod.rs"]
pub mod contracts;

/// Expanded M1 fixture: portable values and approved descriptors compiled for native and WASM.
#[allow(
    missing_docs,
    reason = "Idea aggregate emits module declarations; leaves carry Rustdoc"
)]
pub mod matrix;
