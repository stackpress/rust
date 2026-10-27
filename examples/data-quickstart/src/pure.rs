//! Exact generated contracts and metadata, independently usable on browser targets.

/// Baseline pure value/input/query contracts.
#[cfg(not(feature = "evolved"))]
#[path = "generated/v1/contracts/mod.rs"]
pub mod contracts;
/// Evolved pure value/input/query contracts.
#[cfg(all(feature = "evolved", not(feature = "expanded")))]
#[path = "generated/v2/contracts/mod.rs"]
pub mod contracts;
/// Expanded pure value/input/query contracts.
#[cfg(feature = "expanded")]
#[path = "generated/v3/contracts/mod.rs"]
pub mod contracts;

/// Baseline safe metadata.
#[cfg(not(feature = "evolved"))]
#[path = "generated/v1/metadata/mod.rs"]
pub mod metadata;
/// Evolved safe metadata.
#[cfg(all(feature = "evolved", not(feature = "expanded")))]
#[path = "generated/v2/metadata/mod.rs"]
pub mod metadata;
/// Expanded safe metadata.
#[cfg(feature = "expanded")]
#[path = "generated/v3/metadata/mod.rs"]
pub mod metadata;
