//! Compatibility identity shared by generated partitions.
/// Normalized semantic fingerprint.
pub const FINGERPRINT: &str = "dbc3f50dcd8942354ff56d52ef513a78f18d1d2e58287306362e6b7c83d46969";
/// Required runtime descriptor revision.
pub const CONTRACT_REVISION: u32 = 2;
const _: () = assert!(CONTRACT_REVISION == stackpress_framework_schema::CONTRACT_REVISION);
