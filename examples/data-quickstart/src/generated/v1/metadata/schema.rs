//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "86b43263cb802c294272acd970f1569cc91dcf871004c15ab0ca6df3266b9c3f";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 2;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
