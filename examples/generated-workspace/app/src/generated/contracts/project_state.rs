//! Scalar enumeration generated from Idea.

/// Checked finite wire choices.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProjectState {
    /// Declared wire choice.
    #[serde(rename = "open")]
    Open,
    /// Declared wire choice.
    #[serde(rename = "closed")]
    Closed,
}
