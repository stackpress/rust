//! Finite scalar enumeration generated from Idea.
/// Checked wire choices, without coercion.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "bool", into = "bool")]
pub enum Truth {
    /// Declared scalar choice.
    No,
    /// Declared scalar choice.
    Yes,
}
impl TryFrom<bool> for Truth {
    type Error = &'static str;
    fn try_from(value: bool) -> Result<Self, Self::Error> {
        match value {
            false => Ok(Self::No),
            true => Ok(Self::Yes),
        }
    }
}
impl From<Truth> for bool {
    fn from(value: Truth) -> Self {
        match value {
            Truth::No => false,
            Truth::Yes => true,
        }
    }
}
