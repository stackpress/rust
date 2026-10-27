//! Finite scalar enumeration generated from Idea.
/// Checked wire choices, without coercion.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "i64", into = "i64")]
pub enum Count {
    /// Declared scalar choice.
    Zero,
    /// Declared scalar choice.
    One,
}
impl TryFrom<i64> for Count {
    type Error = &'static str;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            1 => Ok(Self::One),
            _ => Err("invalid enum choice"),
        }
    }
}
impl From<Count> for i64 {
    fn from(value: Count) -> Self {
        match value {
            Count::Zero => 0,
            Count::One => 1,
        }
    }
}
