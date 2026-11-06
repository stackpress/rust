//! Native partitions from the one application emit; portable UI is consumed by blog-shared.

/// Static framework client bound to this application's exact generated family.
#[path = "../../../.build/app-generated/client/mod.rs"]
pub mod client;
/// Complete source metadata stays in the native graph.
#[path = "../../../.build/app-generated/source/mod.rs"]
pub mod source;
/// Actual generated store family, actions and registration functions.
#[path = "../../../.build/app-generated/stores/mod.rs"]
pub mod stores;
