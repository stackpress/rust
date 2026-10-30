//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "a4c323edabb195fecd5f2bb32df8c98814638a6e19a4e21a873bb06f4cbf499b";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 3;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
