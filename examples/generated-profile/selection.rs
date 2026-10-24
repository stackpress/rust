//! Shared example selection source; neither consumer depends on generated data.

/// Whether this executable selects the demo runtime contribution.
pub const ENABLED: bool = cfg!(feature = "profiles");
/// Generator/runtime source contract revision, checked during application compilation.
pub const REVISION: u32 = 1;
