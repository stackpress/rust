//! Generated public role wrappers; handwritten adapters remain outside this partition.
[%#?:rendering%]use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
[%|%]use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult, UiField};
[%/?:rendering%]/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str([%fields_literal%]).map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
}
[%#@:roles role%]/// Render the explicitly selected [%role.name%] controls for one caller-owned instance.
[%#?:role.typed_input%]pub fn [%role.name%](value: &super::super::contracts::public::[%role.public_type%], instance: &str) -> SchemaResult<AnyView> {
    let seed = stackpress_framework_schema::to_bounded_value(value)?;
    let fields = descriptors()?;
    let draft = Draft::new(&fields, &seed, Role::[%role.variant%])?;
[%|%]pub fn [%role.name%](draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
[%/?:role.typed_input%]    let mut controls = Vec::new();
[%#@:role.controls control%][%control%][%/@:role.controls%]    Ok(controls.collect_view().into_any())
}
[%/@:roles%][%#?:has_table%][%table%][%/?:has_table%]
