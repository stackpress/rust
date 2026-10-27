//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "d7a224aaa8760cd18b45ca0e575a2459e5fca3f2a519023a6ae7f0a0fa14a9d3";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 3;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
