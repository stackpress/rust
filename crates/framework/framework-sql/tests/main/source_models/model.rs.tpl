//! Generated M0 contract probe, not the production source-model emitter.
use crate::support;
use stackpress_framework_sql::source::*;

[%#@:models model%]/// Metadata comes from this probe's real composed and normalized declarations.
pub enum [%model.name%]Spec {}
impl support::Spec for [%model.name%]Spec {
    fn describe() -> stackpress_framework_schema::Model {
        serde_json::from_str([%model.json%]).expect("validated probe metadata")
    }
}
/// Each generated model associates its own schema, exact store and action implementation.
pub type [%model.name%] = support::Model<[%model.name%]Spec, [%model.role%]>;
pub type [%model.name%]Schema = support::Schema<[%model.name%]Spec>;
[%model.witnesses%]
[%/@:models%]
[%#?:family%]/// A single inherited Profile/Auth family with explicit cross-model native equality.
pub struct Family;
impl SessionModels for Family { type Profile=Profile; type Auth=Auth; }
[%#?:api%]impl ApiModels for Family { type Application=Application; type Session=Session; }
[%/?:api%][%/?:family%]
