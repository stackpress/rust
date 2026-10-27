//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "01ce48470ea335704f921efa41c4eb215f504f13aa473e093bd2b95b3734522d";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 2;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
