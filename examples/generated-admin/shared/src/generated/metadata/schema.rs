//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "bf1ffb1dcad0a0fa5a1cc6a8df13652e29f1febbc94915fc48a4ee1235520b03";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 2;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
