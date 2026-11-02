//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = [%fingerprint%];
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = [%revision%];
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
