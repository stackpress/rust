//! Generated public page adapters; imports register nothing.
use leptos::prelude::*;
use stackpress_framework_admin::{[%#?:has_controls%]RoleContext,[%/?:has_controls%]Draft,pages::{Action,Form,PageSpec}};
use stackpress_framework_schema::{[%#?:has_controls%]Role,[%/?:has_controls%]SchemaError,SchemaErrorKind,SchemaResult};
[%#@:models model%]
/// Typed adapters for one explicitly selected public model.
pub mod [%model.module%] {
use super::*;
/// Decode only the approved revision-3 model specification.
pub fn spec() -> SchemaResult<PageSpec> {
    serde_json::from_str([%model.spec_literal%]).map_err(codec)
}
/// Decode through the actual generated public create/patch DTOs.
pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
    if action == Action::Update {
        serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicPatch[%model.type_name%]>(value).map_err(codec)?).map_err(codec)
    } else {
        serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicCreate[%model.type_name%]>(value).map_err(codec)?).map_err(codec)
    }
}
/// Validate all supplied import members through the actual generated create contract.
pub fn validate_import(value: serde_json::Value, context: &stackpress_framework_schema::ValueContext<'_>) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
    let input: super::super::super::contracts::public::PublicCreate[%model.type_name%] = serde_json::from_value(value).map_err(codec)?;
    Ok(input.validate_create(context)?.issues)
}
/// Validate the actual generated query type before sending its envelope.
pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
    let query: super::super::super::contracts::public::[%model.type_name%]Query = serde_json::from_value(value).map_err(codec)?;
    query.validate()?;
    serde_json::to_value(query).map_err(codec)
}
/// Compose selected field adapters; updates cannot edit the stored identity.
pub fn form(form: Form) -> SchemaResult<AnyView> {
[%#?:model.has_controls%]let draft = &form.draft;
let instance = &form.instance;
let mut fields = super::super::[%model.module%]::descriptors()?;
// Related scope can add a disabled constraint, never enable a forbidden field.
if let Some(locked) = &form.locked {
    for field in &mut fields {
        if &field.wire_name == locked {
            for binding in &mut field.roles {
                if binding.role == Role::Field {
                    binding.props.insert("disabled".into(), true.into());
                }
            }
        }
    }
}
let mut controls = Vec::<AnyView>::new();
[%|%]let _ = form;
let controls = Vec::<AnyView>::new();
[%/?:model.has_controls%]
[%#@:model.controls control%][%#?:control.key_guard%]if form.action != Action::Update {
[%/?:control.key_guard%][%control.body%][%#?:control.key_guard%]}
[%/?:control.key_guard%][%/@:model.controls%]Ok(controls.collect_view().into_any())
}
[%#?:model.has_filter%]/// Compose the explicitly selected filter roles.
pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    super::super::[%model.module%]::filter(draft,instance)
}
[%|%]/// This page has no selected filter controls.
pub fn filters(_: &Draft, _: &str) -> SchemaResult<AnyView> {
    Ok(Vec::<AnyView>::new().collect_view().into_any())
}
[%/?:model.has_filter%][%#?:model.has_list%]
/// Render actual public rows through their typed DTO and selected list adapters.
pub fn table(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    let spec = spec()?;
    let mut rows = Vec::new();
    for value in value.get("rows").and_then(serde_json::Value::as_array).ok_or_else(unavailable)? {
        let row = value.get("row").unwrap_or(value);
        let key = stackpress_framework_admin::pages::key(&spec,row)?;
        let typed: super::super::super::contracts::public::[%model.public_type%] = serde_json::from_value(row.clone()).map_err(codec)?;
        rows.push((key.to_string(),typed));
    }
    super::super::[%model.module%]::table(&rows,instance)
}
[%|%]/// No list role was selected.
pub fn table(_: &serde_json::Value, _: &str) -> SchemaResult<AnyView> {
    Err(unavailable())
}
[%/?:model.has_list%][%#?:model.has_view%]/// Render actual public detail through its typed DTO and selected view adapters.
pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    let typed: super::super::super::contracts::public::[%model.public_type%] = serde_json::from_value(value.clone()).map_err(codec)?;
    super::super::[%model.module%]::view(&typed,instance)
}
[%|%]/// No view role was selected.
pub fn detail(_: &serde_json::Value, _: &str) -> SchemaResult<AnyView> {
    Err(unavailable())
}
[%/?:model.has_view%]/// Keep the concrete codec cause private under one adapter context.
fn codec(cause: serde_json::Error) -> SchemaError {
    SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
}
}
[%/@:models%][%dispatch%]
