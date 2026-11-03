//! Finite scalar enumeration generated from Idea.
/// Checked wire choices, without coercion.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = [%scalar_literal%], into = [%scalar_literal%])]
pub enum [%name%] {
[%#@:variants variant%]/// Declared scalar choice.
[%variant.name%],
[%/@:variants%]}
impl TryFrom<[%scalar%]> for [%name%] {
    type Error = &'static str;
    fn try_from(value: [%scalar%]) -> Result<Self, Self::Error> {
        match [%#?:is_string%]value.as_str()[%|%]value[%/?:is_string%] {
[%#@:variants variant%]            [%variant.literal%] => Ok(Self::[%variant.name%]),
[%/@:variants%][%#?:fallback%]            _ => Err("invalid enum choice"),[%/?:fallback%]
        }
    }
}
impl From<[%name%]> for [%scalar%] {
    fn from(value: [%name%]) -> Self {
        match value {
[%#@:variants variant%]            [%name%]::[%variant.name%] => [%variant.literal%][%#?:is_string%].into()[%/?:is_string%],
[%/@:variants%]        }
    }
}
