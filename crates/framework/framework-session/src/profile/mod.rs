//! Profile schema shared with account and API storage adapters.
//!
//! A Profile may describe a person, company or another entity. It is not restricted to
//! an authenticated user. The source generated schema has ten stored columns and no
//! reverse store relations; Idea's auth/applications/sessions edges remain model metadata.

use crate::columns::{Record, RecordSchema};
use std::ops::Deref;

/// Source Profile record, including partial projection/create fallback results.
pub type Profile = Record;
/// Source Profile input; omission, explicit null and empty string remain distinguishable.
pub type ProfileInput = Record;
/// Source extended profile; the generated store defines no eager reverse relations.
pub type ProfileExtended = Record;

/// Stable public Profile schema: defaults, assertions, filtering and column codecs.
///
/// ```
/// use stackpress_framework_session::profile::ProfileSchema;
/// let schema = ProfileSchema::new("");
/// let defaults = schema.defaults_at(0)?;
/// assert_eq!(defaults["roles"], serde_json::json!(["GUEST"]));
/// assert!(schema.assert(&defaults, true).unwrap().contains_key("name"));
/// # Ok::<(), stackpress_framework_schema::SchemaError>(())
/// ```
#[derive(Clone, Debug)]
pub struct ProfileSchema(RecordSchema);
impl ProfileSchema {
    /// Retain source seed and columns without opening a database or generating defaults.
    pub fn new(seed: impl Into<String>) -> Self {
        Self(RecordSchema::new("Profile", seed.into()))
    }
    /// Select the host's local UTC offset for Date column conversions.
    pub fn with_offset_minutes(mut self, offset: i32) -> Self {
        self.0.offset = offset;
        self
    }
    /// Own the underlying schema for native store composition.
    pub fn into_schema(self) -> RecordSchema {
        self.0
    }
}
impl Default for ProfileSchema {
    /// Direct source construction uses an empty seed.
    fn default() -> Self {
        Self::new("")
    }
}
impl Deref for ProfileSchema {
    type Target = RecordSchema;
    /// Borrow source operations without duplicating ten generated column implementations.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(feature = "sql")]
pub use crate::storage::{ProfileActions, ProfileStore};
