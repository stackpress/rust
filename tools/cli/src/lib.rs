//! Primitives for an application's compiled `stackpress` executable.
//!
//! This package intentionally has no stock/empty binary. M0 proves native terminal
//! ownership and the compile-time bootstrap seam; command parsing, top-level handling
//! and full prompt/control behavior remain gated later milestones.

pub use stackpress_server::ConfigLayers;

#[cfg(feature = "tty")]
pub mod tty;
