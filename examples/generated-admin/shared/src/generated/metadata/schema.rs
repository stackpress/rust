//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "6fd7b32e8beb5934f1b3849c0980fe073093aa29d7ed1f222a0165e986627b43";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 3;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
