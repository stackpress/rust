//! Finite scalar enumeration generated from Idea.
/// Checked wire choices, without coercion.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum ProjectState {
    /// Declared scalar choice.
    Open,
    /// Declared scalar choice.
    Closed,
}
impl TryFrom<String> for ProjectState {
    type Error = &'static str;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "open" => Ok(Self::Open),
            "closed" => Ok(Self::Closed),
            _ => Err("invalid enum choice"),
        }
    }
}
impl From<ProjectState> for String {
    fn from(value: ProjectState) -> Self {
        match value {
            ProjectState::Open => "open".into(),
            ProjectState::Closed => "closed".into(),
        }
    }
}
