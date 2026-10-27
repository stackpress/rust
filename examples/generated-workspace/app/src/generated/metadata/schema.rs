//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "d9ce06448438231d01dd291c1466547b999985876a95c9329a341dd170750ac4";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 1;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
