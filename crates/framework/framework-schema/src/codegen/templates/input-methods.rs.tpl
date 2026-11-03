impl [%name%] {
/// Decode the bounded input policy; no private literal defaults or imports are embedded.
fn input_schema()->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
    serde_json::from_str([%descriptor%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
}
/// Validate supplied inputs without generating values or executing database work.
pub fn [%method%](&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors> {
    let value=stackpress_framework_schema::to_bounded_value(self)?;
[%#?:public%]    stackpress_framework_schema::check_browser_json(&value)?;
[%/?:public%]    Self::input_schema()?.validate(&value,context,false)
}
[%#?:trusted_create%]/// Resolve host defaults once and return the fully validated trusted value.
pub fn prepare_create(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<super::[%module%]::[%stored%]> {
    self.prepare_with(context)
}
[%/?:trusted_create%][%#?:public%]/// Evaluate client-checkable rules only; host rules and authorization remain required.
pub fn validate_client(&self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors> {
    let value=stackpress_framework_schema::to_bounded_value(self)?;
    stackpress_framework_schema::check_browser_json(&value)?;
    Self::input_schema()?.validate(&value,context,true)
}
[%#?:public_create%]/// Prepare approved literal defaults in public input only; host-generated values remain omitted.
pub fn prepare_create(self,context:&stackpress_framework_schema::ValueContext<'_>)->stackpress_framework_schema::SchemaResult<serde_json::Value> {
    let value=stackpress_framework_schema::to_bounded_value(&self)?;
    stackpress_framework_schema::check_browser_json(&value)?;
    Self::input_schema()?.prepare(value,context)
}
[%/?:public_create%][%/?:public%]}
