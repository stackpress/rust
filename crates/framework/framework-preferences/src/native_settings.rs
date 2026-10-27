//! Narrow installed-app command DTOs, independent of any filesystem or IPC runtime.
use crate::Preferences;
use serde::{Deserialize, Serialize};

/// Public initial settings, including visible recovery information without a path.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadedPreferences {
    /// Fixed command revision, separate from existing search wire contracts.
    pub revision: u32,
    /// Canonical checked settings loaded before the first CSR root.
    pub preferences: Preferences,
    /// Invalid/unreadable storage used defaults; announce this to the user.
    pub recovered: bool,
}
/// Fixed update command fields; no path, event, filesystem or shell selector.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NativePreferenceUpdate {
    /// Only revision one is currently admitted by the host.
    pub revision: u32,
    /// Requested public locale/theme, revalidated by native storage.
    pub preferences: Preferences,
}
