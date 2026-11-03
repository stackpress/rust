//! Approved public UI descriptor, containing no imports, source paths or private defaults.
/// Decode the versioned application-owned projection.
pub fn schema()->stackpress_framework_schema::SchemaResult<stackpress_framework_schema::UiSchema> {
    serde_json::from_str([%descriptor%]).map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"UI descriptor decoding failed").with_cause(cause))
}
