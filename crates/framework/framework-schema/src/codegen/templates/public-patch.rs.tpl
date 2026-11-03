/// Published partial update; the host supplies private values explicitly.
#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatch[%name%] {
[%#@:fields field%]/// Approved update value; omission never writes null.
#[serde(rename=[%field.wire%],default,skip_serializing_if="stackpress_framework_schema::Patch::is_unchanged")]
pub [%field.member%]:stackpress_framework_schema::Patch<[%field.ty%]>,
[%/@:fields%]}
[%methods%]