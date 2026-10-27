//! Pure consumer of the exact generated contracts, without native runtime/tool dependencies.
//!
//! ```
//! use generated_workspace_contracts::contracts::public::ProfileKey;
//! let key = ProfileKey { id: "u1".into() };
//! assert_eq!(serde_json::to_value(&key)?, serde_json::json!({"id": "u1"}));
//! # Ok::<(), serde_json::Error>(())
//! ```

/// Pure generated contracts, shared verbatim with the native application.
#[path = "../../app/src/generated/contracts/mod.rs"]
pub mod contracts;

/// Expanded M1 fixture: portable values and approved descriptors compiled for native and WASM.
#[allow(
    missing_docs,
    reason = "Idea aggregate emits module declarations; leaves carry Rustdoc"
)]
pub mod matrix;
