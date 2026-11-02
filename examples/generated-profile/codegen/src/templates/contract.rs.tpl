//! Public model fields generated from schema.idea.

/// Public profile data; generated from the declared model.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Profile {
[%#@:fields field%]    /// Public [%field%] field.
    pub [%field%]: String,
[%/@:fields%]}
