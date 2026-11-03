//! Owned [%model%] contract.
pub struct [%model%] {
[%#@:fields field%]    #[serde(rename = [%field.wire_literal%])]
    pub [%field.member%]: [%field.ty%],
[%/@:fields%]}
// Runtime format syntax remains literal: format!("{{{}}}", value).
