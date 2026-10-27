//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "7d0ce9d40c9988eecd7e1869f740c6f9af360554e99ba7c206217f55be1abcd4";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 3;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
