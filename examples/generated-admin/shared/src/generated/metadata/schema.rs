//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "0cbe400c604318e1ad0ac49f732d835138be1a72b0df9a63bd4c836d6c71189a";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 2;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
