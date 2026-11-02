//! Explicit application projections. Trusted stored rows must use project() before public output.
[%#@:models model%]/// Application-approved owned output.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Public[%model.name%] {
[%#@:model.fields field%]/// Approved public field.
#[serde(rename=[%field.wire%])]
pub [%field.member%]: [%field.ty%],
[%/@:model.fields%]}
impl super::[%model.module%]::[%model.name%] {
/// Project only fields explicitly approved by the compiled application policy.
pub fn project(&self)->stackpress_framework_schema::SchemaResult<Public[%model.name%]> {
    self.validate()?;
    let public=Public[%model.name%] {
[%#@:model.fields field%]        [%field.member%]: [%field.value%],
[%/@:model.fields%]    };
    let wire=stackpress_framework_schema::to_bounded_value(&public)?;
    stackpress_framework_schema::check_browser_json(&wire)?;
    Ok(public)
}
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreate[%model.name%] {
[%#@:model.inputs field%]/// Writable public input.
#[serde(rename=[%field.wire%],default,skip_serializing_if="stackpress_framework_schema::Input::is_missing")]
pub [%field.member%]: stackpress_framework_schema::Input<[%field.ty%]>,
[%/@:model.inputs%]}
[%model.public_methods%][%model.trusted_methods%][%model.public_patch%][%#?:model.is_model%]/// Ordered model key, with exact scalar types.
#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct [%model.name%]Key {
[%#@:model.keys field%]/// Key member.
#[serde(rename=[%field.wire%])]
pub [%field.member%]: [%field.ty%],
[%/@:model.keys%]}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Patch[%model.name%] {
[%#@:model.patches field%]/// Omitted, supplied, or explicit null patch value.
#[serde(rename=[%field.wire%],default,skip_serializing_if="stackpress_framework_schema::Patch::is_unchanged")]
pub [%field.member%]: stackpress_framework_schema::Patch<[%field.ty%]>,
[%/@:model.patches%]}
impl Patch[%model.name%] {
/// Reject empty updates and validate supplied fields without applying create defaults.
pub fn validate(&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<()> {
    let value=stackpress_framework_schema::to_bounded_value(self)?;
    if value.as_object().is_none_or(|v|v.is_empty()) {
        return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid,"empty patch is invalid"));
    }
    let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str([%model.patch_policy%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
    policy.validate(&value,context,true,false)?.into_result()
}
}
[%model.patch_methods%][%model.query%][%/?:model.is_model%][%/@:models%]