//! Page-policy mapping for the independent library's inert resource replay.
use crate::{PageLimits, ViewError};
pub use stackpress_leptos::replay::{MAX_RESOURCE_ID, MAX_RESOURCES};
/// Checked public page replay, preserving view's existing failure categories.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceReplay(stackpress_leptos::replay::ResourceReplay);
/// Translate only generic replay categories; never expose encoded input in errors.
pub(super) fn map_error(error: stackpress_leptos::replay::ReplayError) -> ViewError {
    use stackpress_leptos::replay::ReplayError;
    match error {
        ReplayError::InvalidPolicy => ViewError::InvalidPolicy,
        ReplayError::InvalidSnapshot => ViewError::InvalidSnapshot,
        ReplayError::SnapshotLimit => ViewError::SnapshotLimit,
        ReplayError::Failure(cause) => ViewError::Failure(cause),
    }
}
impl ResourceReplay {
    /// Validate view limits before delegating identity/count/byte admission.
    /// # Errors
    /// Returns the existing page policy/data/limit errors.
    pub fn new(values: Vec<(usize, String)>, byte_limit: usize) -> Result<Self, ViewError> {
        PageLimits {
            snapshot_bytes: byte_limit,
            ..Default::default()
        }
        .validate()?;
        stackpress_leptos::replay::ResourceReplay::new(values, byte_limit)
            .map(Self)
            .map_err(map_error)
    }
    /// Decode inert script text under the same logical and expanded limits.
    /// # Errors
    /// Rejects malformed, duplicate or excessive replay before browser publication.
    pub fn decode(text: &str, byte_limit: usize) -> Result<Self, ViewError> {
        PageLimits {
            snapshot_bytes: byte_limit,
            ..Default::default()
        }
        .validate()?;
        stackpress_leptos::replay::ResourceReplay::decode(text, byte_limit)
            .map(Self)
            .map_err(map_error)
    }
    /// Escape for an inert application/json script element.
    /// # Errors
    /// Retains concrete serializer diagnostics only through native cause traversal.
    pub fn script_json(&self) -> Result<String, ViewError> {
        self.0.script_json().map_err(map_error)
    }
    /// Borrow admitted pairs; unchecked mutation is unavailable.
    pub fn entries(&self) -> &[(usize, String)] {
        self.0.entries()
    }
}
