//! Owned [%name%] contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct [%name%] {
[%#@:fields field%]/// Declared stored value.
#[serde(rename = [%field.wire%])]
pub [%field.member%]: [%field.ty%],
[%/@:fields%]}
impl [%name%] {
/// Decode the bounded runtime-only record policy; private metadata is absent.
fn record_schema() -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
    serde_json::from_str([%record%]).map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
}
/// Validate stored values without rerunning time-relative creation rules.
pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
    self.validate_with(&stackpress_framework_schema::ValueContext::default(), true)
}
/// Validate against one explicit clock/provider context.
pub fn validate_with(&self, context: &stackpress_framework_schema::ValueContext<'_>, stored: bool) -> stackpress_framework_schema::SchemaResult<()> {
    let value=stackpress_framework_schema::to_bounded_value(self)?;
    Self::record_schema()?.validate(&value,context,false,stored)?.into_result()
}
}
[%create%]