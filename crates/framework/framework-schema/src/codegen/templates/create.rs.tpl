/// Create input preserving omitted, null and supplied values at every embedded depth.
#[derive(Clone,Debug,Default,serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create[%name%] {
[%#@:fields field%]/// Explicit create input; omission consults the declared host/literal source.
#[serde(rename=[%field.wire%],default,skip_serializing_if="stackpress_framework_schema::Input::is_missing")]
pub [%field.member%]:stackpress_framework_schema::Input<[%field.ty%]>,
[%/@:fields%]}
impl Create[%name%] {
/// Prepare using no ambient clock or provider.
pub fn prepare(self)->stackpress_framework_schema::SchemaResult<[%name%]> {
    self.prepare_with(&stackpress_framework_schema::ValueContext::default())
}
/// Fill defaults exactly once and validate all nested business rules once before effects.
pub fn prepare_with(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<[%name%]> {
    let mut budget=stackpress_framework_schema::PreparationBudget::default();
    let value=self.prepare_bounded(context,&mut budget)?;
    value.validate_with(context,false)?;
    Ok(value)
}
/// Internal cross-module composition shares the caller's aggregate budget; validation follows once.
pub(crate) fn prepare_bounded(self,context:&stackpress_framework_schema::ValueContext<'_>,budget:&mut stackpress_framework_schema::PreparationBudget)->stackpress_framework_schema::SchemaResult<[%name%]> {
    let _=context;
    Ok([%name%] {
[%#@:values field%][%#?:field.generated%]// Generated values cannot be supplied by the caller.
        [%field.member%]:[%field.missing%],
[%|%]// Explicit null never falls through to a literal or host default.
        [%field.member%]:match self.[%field.member%] {
            stackpress_framework_schema::Input::Missing=>[%field.missing%],
            stackpress_framework_schema::Input::Null=>[%#?:field.required%]return Err(stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Invalid,"required create field is null"))[%|%]None[%/?:field.required%],
            stackpress_framework_schema::Input::Value(value)=>[%#!:field.required%]Some([%/!:field.required%][%field.convert%][%#!:field.required%])[%/!:field.required%],
        },
[%/?:field.generated%][%/@:values%]    })
}
}
