
/// Explicit [%model%] operation family; importing this module installs nothing.
pub mod [%store%] {
use stackpress_framework_sql::{DataError, DataResult, DataSession};
use stackpress_framework_sql::operations::{self, Flow, Invocation, ListenerId, Operation, Policy, Registration, Request, Response, Result, ServerInvocationExt, StartupContext};
use crate::generated::contracts::public::{[%model%]Key as Key, PublicCreate[%model%] as CreateInput, PublicPatch[%model%] as PatchInput, Patch[%model%] as Patch};
use crate::generated::stores::[%store%]::{[%model%]s};
/// Strict key envelope; unknown request fields are invalid.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRequest {
    /// Complete declared identity.
    pub key: Key
}
/// Strict approved creation envelope.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest {
    /// Approved creation members only.
    pub input: CreateInput
}
/// Strict keyed update envelope.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateRequest {
    /// Complete immutable identity.
    pub key: Key,
    /// Nonempty approved partial replacement.
    pub patch: PatchInput
}
/// Strict keyed atomic UPSERT envelope.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpsertRequest {
    /// Explicit primary conflict identity.
    pub key: Key,
    /// Candidate row with matching identity.
    pub input: CreateInput,
    /// Independently validated conflict patch.
    pub patch: PatchInput
}
/// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
/// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case", deny_unknown_fields)]
pub enum BatchItem {
    /// Insert one prepared candidate.
    Create {
        /// Approved creation members.
        input: Box<CreateInput>
    },
    /// Patch one complete identity.
    Update {
        /// Immutable target identity.
        key: Key,
        /// Approved partial replacement.
        patch: Box<PatchInput>
    },
    /// One atomic native primary-key UPSERT.
    Upsert {
        /// Explicit conflict identity.
        key: Key,
        /// Candidate with the same identity.
        input: Box<CreateInput>,
        /// Approved conflict replacement.
        patch: Box<PatchInput>
    }
}
/// Strict finite batch envelope; native whole-batch validation precedes writes.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchRequest {
    /// Ordered finite atomic work, fully preflighted before the first write.
    pub items: Vec<BatchItem>
}
/// Native-only record policy used for safe host assembly and field-error filtering.
fn policy_schema() -> DataResult<stackpress_framework_schema::RecordSchema> {
    serde_json::from_str([%policy_literal%]).map_err(stackpress_framework_sql::codec_error)
}
/// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
fn create_input(input: CreateInput, host: &serde_json::Value, policy: &stackpress_framework_schema::RecordSchema, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<crate::generated::contracts::[%module%]::Create[%model%]> {
    operations::assemble(&input, host, policy, false, context)
}
/// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
fn patch_input(input: PatchInput, host: &serde_json::Value, policy: &stackpress_framework_schema::RecordSchema, context: &stackpress_framework_schema::ValueContext<'_>) -> DataResult<Patch> {
    let [%#?:has_fieldsets%]mut [%/?:has_fieldsets%]value: serde_json::Value = operations::assemble(&input, host, policy, true, context)?;
[%#?:has_fieldsets%]    let mut budget = stackpress_framework_schema::PreparationBudget::default();
    let object = value.as_object_mut().ok_or_else(|| DataError::new("patch assembly requires an object"))?;
[%#@:fieldsets field%]    // Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.
    if let Some(raw) = object.get_mut([%field.name_literal%]).filter(|value| !value.is_null()) {
[%#?:field.multiple%]        let inputs: Vec<crate::generated::contracts::[%field.module%]::Create[%field.model%]> = serde_json::from_value(raw.take()).map_err(stackpress_framework_sql::codec_error)?;
        let values = inputs.into_iter().map(|input| input.prepare_bounded(context, &mut budget).map_err(stackpress_framework_sql::stores::preparation_error)).collect::<DataResult<Vec<_>>>()?;
        *raw = stackpress_framework_schema::to_bounded_value(&values).map_err(stackpress_framework_sql::stores::validation)?;
[%|%]        let input: crate::generated::contracts::[%field.module%]::Create[%field.model%] = serde_json::from_value(raw.take()).map_err(stackpress_framework_sql::codec_error)?;
        *raw = stackpress_framework_schema::to_bounded_value(&input.prepare_bounded(context, &mut budget).map_err(stackpress_framework_sql::stores::preparation_error)?).map_err(stackpress_framework_sql::stores::validation)?;
[%/?:field.multiple%]    }
[%/@:fieldsets%][%|%]    let _ = context;
[%/?:has_fieldsets%]    serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)
}
