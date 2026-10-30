//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "45c8b89edfac48dc1ee02ca2f8099a523f38217fd20ebd1f8bbf9708a21ec7b3";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 3;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
