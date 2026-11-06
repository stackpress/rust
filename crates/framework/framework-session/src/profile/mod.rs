//! Profile schema shared with account and API storage adapters.
//!
//! A Profile may describe a person, company or another entity. It is not restricted to
//! an authenticated user. The generated default has ten stored columns and no
//! reverse store relations; Idea's auth/applications/sessions edges remain model metadata.

use crate::columns::Record;

/// Source Profile record, including partial projection/create fallback results.
pub type Profile = Record;
/// Source Profile input; omission, explicit null and empty string remain distinguishable.
pub type ProfileInput = Record;
/// Source extended profile; the generated store defines no eager reverse relations.
pub type ProfileExtended = Record;

/// Generated default Profile schema. Use the `SourceSchema` trait for construction and metadata.
pub use stackpress_framework_models::ProfileSchema;

#[cfg(feature = "sql")]
pub use crate::storage::{ProfileActions, ProfileStore};
