//! Explicit operation DTOs/handlers and app-owned registration, with no inferred route table.
use super::*;

/// Emit native host-only handler modules when the app tool explicitly selects that output.
pub(super) fn emit(
    schema: &Schema,
    output: &mut OutputPlan,
) -> stackpress_idea_transformer::Result<()> {
    let mut text = String::from(
        "//! Named generated handlers; registration and transport mapping are always explicit.\n",
    );
    for model in schema.models() {
        model_operations(schema, model, &mut text)?;
    }
    output.write("stores/operations.rs", text)?;
    output.contribute_module("stores", "operations")
}
/// Convert public fieldset create-shaped patch values into full replacement values once.
fn patch_preparation(
    schema: &Schema,
    model: &Model,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let fields: Vec<_> = model
        .fields
        .iter()
        .filter(|field| {
            !field.key && !field.generated && matches!(field.value_type, ValueType::Fieldset(_))
        })
        .collect();
    if fields.is_empty() {
        text.push_str("let _=context;\n");
        return Ok(());
    }
    text.push_str("let mut budget=stackpress_framework_schema::PreparationBudget::default();\nlet object=value.as_object_mut().ok_or_else(||DataError::new(\"patch assembly requires an object\"))?;\n");
    for field in fields {
        let ValueType::Fieldset(target) = &field.value_type else {
            continue;
        };
        let target = schema
            .fieldsets()
            .iter()
            .find(|model| model.name == *target)
            .ok_or_else(|| TransformError::new("patch fieldset missing"))?;
        let input = format!(
            "crate::generated::contracts::{}::Create{}",
            target.module, target.rust_name
        );
        text.push_str(&format!("// Fieldset patches replace a full value; prepare nested defaults/providers without a second validation.\nif let Some(raw)=object.get_mut({:?}).filter(|value|!value.is_null()) {{\n",field.name));
        if field.multiple {
            text.push_str(&format!("let inputs:Vec<{input}>=serde_json::from_value(raw.take()).map_err(stackpress_framework_sql::codec_error)?;\nlet values=inputs.into_iter().map(|input|input.prepare_bounded(context,&mut budget).map_err(stackpress_framework_sql::stores::preparation_error)).collect::<DataResult<Vec<_>>>()?;\n*raw=stackpress_framework_schema::to_bounded_value(&values).map_err(stackpress_framework_sql::stores::validation)?;\n"));
        } else {
            text.push_str(&format!("let input:{input}=serde_json::from_value(raw.take()).map_err(stackpress_framework_sql::codec_error)?;\n*raw=stackpress_framework_schema::to_bounded_value(&input.prepare_bounded(context,&mut budget).map_err(stackpress_framework_sql::stores::preparation_error)?).map_err(stackpress_framework_sql::stores::validation)?;\n"));
        }
        text.push_str("}\n");
    }
    Ok(())
}
/// Emit one model's operation-specific public envelopes, safe boundary and registration wrapper.
fn model_operations(
    schema: &Schema,
    model: &Model,
    text: &mut String,
) -> stackpress_idea_transformer::Result<()> {
    let name = &model.rust_name;
    let module = &model.module;
    let store = store_module(model);
    let mut policy = stackpress_framework_schema::RecordSchema {
        model: model.clone(),
        fieldsets: schema.fieldsets().to_vec(),
        enums: schema.enums().to_vec(),
    };
    let public = schema
        .project_ui()
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    // Reuse the authoritative public literal projection. Private literals remain named
    // host defaults even in this native assembly policy, rather than being copied into it.
    for record in std::iter::once(&mut policy.model).chain(&mut policy.fieldsets) {
        let projected = public
            .models
            .iter()
            .chain(&public.fieldsets)
            .find(|item| item.id == record.name);
        record.attributes.clear();
        for field in &mut record.fields {
            field.attributes.clear();
            if let Some(default) = &field.default {
                let safe=projected.and_then(|model|model.fields.iter().find(|item|item.wire_name==field.name)).is_some_and(|field|matches!(&field.default,stackpress_framework_schema::PublicDefault::Literal(value) if value==default));
                if !safe {
                    field.default = None;
                    field.value_rule = Some(stackpress_framework_schema::ValueRule {
                        provider: format!("default:{}.{}", record.name, field.name),
                        args: Vec::new(),
                        revision: "1".into(),
                    });
                }
            }
        }
    }
    let policy = serde_json::to_string(&policy)
        .map_err(|cause| TransformError::from(std::io::Error::other(cause)))?;
    text.push_str(&format!(r#"
/// Explicit {name} operation family; importing this module installs nothing.
pub mod {store} {{
use stackpress_framework_sql::{{DataError,DataResult,DataSession}};
use stackpress_framework_sql::operations::{{self,Flow,Invocation,ListenerId,Operation,Policy,Registration,Request,Response,Result,ServerInvocationExt,StartupContext}};
use crate::generated::contracts::public::{{{name}Key as Key,PublicCreate{name} as CreateInput,PublicPatch{name} as PatchInput,Patch{name} as Patch}};
use crate::generated::stores::{store}::{{{name}s}};
/// Strict key envelope; unknown request fields are invalid.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KeyRequest{{
/// Complete declared identity.
pub key:Key}}
/// Strict approved creation envelope.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateRequest{{
/// Approved creation members only.
pub input:CreateInput}}
/// Strict keyed update envelope.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateRequest{{
/// Complete immutable identity.
pub key:Key,
/// Nonempty approved partial replacement.
pub patch:PatchInput}}
/// Strict keyed atomic UPSERT envelope.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpsertRequest{{
/// Explicit primary conflict identity.
pub key:Key,
/// Candidate row with matching identity.
pub input:CreateInput,
/// Independently validated conflict patch.
pub patch:PatchInput}}
/// Public per-item batch modes, without nested relation writes or arbitrary mutation filters.
/// Box payloads so a rich upsert does not inflate every item to the combined create/patch size.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(tag="mode",rename_all="snake_case",deny_unknown_fields)]
pub enum BatchItem{{
/// Insert one prepared candidate.
Create{{
/// Approved creation members.
input:Box<CreateInput>}},
/// Patch one complete identity.
Update{{
/// Immutable target identity.
key:Key,
/// Approved partial replacement.
patch:Box<PatchInput>}},
/// One atomic native primary-key UPSERT.
Upsert{{
/// Explicit conflict identity.
key:Key,
/// Candidate with the same identity.
input:Box<CreateInput>,
/// Approved conflict replacement.
patch:Box<PatchInput>}}}}
/// Strict finite batch envelope; native whole-batch validation precedes writes.
#[derive(serde::Serialize,serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BatchRequest{{
/// Ordered finite atomic work, fully preflighted before the first write.
pub items:Vec<BatchItem>}}
/// Native-only record policy used for safe host assembly and field-error filtering.
fn policy_schema()->DataResult<stackpress_framework_schema::RecordSchema>{{serde_json::from_str({policy:?}).map_err(stackpress_framework_sql::codec_error)}}
/// Host fields cannot override public inputs; trusted Create prepares all defaults once later.
fn create_input(input:CreateInput,host:&serde_json::Value,policy:&stackpress_framework_schema::RecordSchema,context:&stackpress_framework_schema::ValueContext<'_>)->DataResult<crate::generated::contracts::{module}::Create{name}>{{operations::assemble(&input,host,policy,false,context)}}
/// Preserve missing/null/replacement semantics while preparing nested fieldset values once.
fn patch_input(input:PatchInput,host:&serde_json::Value,policy:&stackpress_framework_schema::RecordSchema,context:&stackpress_framework_schema::ValueContext<'_>)->DataResult<Patch>{{
let {mutable}value:serde_json::Value=operations::assemble(&input,host,policy,true,context)?;
"#, mutable=if model.fields.iter().any(|field|!field.key&&!field.generated&&matches!(field.value_type,ValueType::Fieldset(_))) {"mut "}else{""}));
    patch_preparation(schema, model, text)?;
    text.push_str(
        "serde_json::from_value(value).map_err(stackpress_framework_sql::codec_error)\n}\n",
    );
    let active = model.fields.iter().any(|field| field.active);
    for operation in [
        "get", "detail", "search", "create", "update", "upsert", "batch", "remove", "restore",
        "delete",
    ] {
        if matches!(operation, "remove" | "restore") && !active {
            continue;
        }
        let variant = operation[..1].to_ascii_uppercase() + &operation[1..];
        let host = matches!(
            operation,
            "create" | "update" | "upsert" | "batch" | "remove" | "restore"
        );
        text.push_str(&format!("/// Named authorized {operation} handler; prior completion skips every effect.\npub async fn {operation}(req:&mut Request,res:&mut Response,invocation:Invocation<'_>)->Result<Flow>{{\n// Respect an earlier listener before policy, decoding or database work.\nif operations::completed(res){{return Ok(Flow::Continue);}}\n// App authorization is required independently of schema/UI exposure.\nlet policy=invocation.service::<Policy>()?;\noperations::authorize(policy,{name:?},Operation::{variant},req)?;\nlet schema=policy_schema().map_err(operations::host_error)?;\n"));
        if host {
            text.push_str(&format!("// Host inputs use one explicit owned clock/provider set for this invocation.\nlet host=policy.0.values({name:?},Operation::{variant},req).map_err(operations::host_error)?;\nlet context=host.context();\n"));
        }
        let input = match operation {
            "get" | "detail" | "remove" | "restore" | "delete" => "KeyRequest".into(),
            "create" => "CreateRequest".into(),
            "update" => "UpdateRequest".into(),
            "upsert" => "UpsertRequest".into(),
            "batch" => "BatchRequest".into(),
            _ => format!("crate::generated::contracts::public::{name}Query"),
        };
        text.push_str(&format!("// Decode only this operation's strict public envelope.\nlet input:{input}=operations::decode(req)?;\n// The executing domain functions receive the exact registered executor.\nlet engine=invocation.service::<stackpress_sql::Engine>()?;\n"));
        match operation {
            "get"=>text.push_str(&format!("let row={name}s::get_key(&mut DataSession::pooled(engine),&input.key).await.map_err(|cause|operations::data_error(cause,&schema))?.ok_or_else(operations::not_found)?;\nlet result=row.project().map_err(|cause|operations::data_error(stackpress_framework_sql::stores::projection_error(cause),&schema))?;\n")),
            "detail"=>text.push_str(&format!("// Refuse the exact public detail plan before entering the selected executor.\n{name}s::preflight_detail(engine,&input.key).map_err(|cause|operations::data_error(cause,&schema))?;\nlet mut transaction=engine.begin().await.map_err(|cause|operations::data_error(cause.into(),&schema))?;\nlet outcome={name}s::detail(&mut DataSession::transaction(&mut transaction),&input.key).await;\nlet result=stackpress_framework_sql::finish(transaction,outcome).await.map_err(|cause|operations::data_error(cause,&schema))?.ok_or_else(operations::not_found)?;\n")),
            "search"=>text.push_str(&format!("// Preserve public DTO/catalog checks and validate selected limits before BEGIN.\n{name}s::preflight_search_public(engine,&input).map_err(|cause|operations::data_error(cause,&schema))?;\nlet mut transaction=engine.begin().await.map_err(|cause|operations::data_error(cause.into(),&schema))?;\nlet outcome={name}s::search_public(&mut DataSession::transaction(&mut transaction),input).await;\nlet result=stackpress_framework_sql::finish(transaction,outcome).await.map_err(|cause|operations::data_error(cause,&schema))?;\n")),
            "create"=>text.push_str(&format!("let input=create_input(input.input,&host.create,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?;\nlet row={name}s::create_with(&mut DataSession::pooled(engine),input,&context).await.map_err(|cause|operations::data_error(cause,&schema))?;\nlet result=row.project().map_err(|cause|operations::data_error(stackpress_framework_sql::stores::projection_error(cause),&schema))?;\n")),
            "update"|"upsert"=>{
                text.push_str("let patch=patch_input(input.patch,&host.patch,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?;\n");
                if operation=="upsert" {text.push_str(&format!("let create=create_input(input.input,&host.create,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?;\nlet row={name}s::upsert_key(&mut DataSession::pooled(engine),&input.key,create,patch,&context).await.map_err(|cause|operations::data_error(cause,&schema))?;\n"));}
                else{text.push_str(&format!("let row={name}s::update_key(&mut DataSession::pooled(engine),&input.key,patch,&context).await.map_err(|cause|operations::data_error(cause,&schema))?.ok_or_else(operations::not_found)?;\n"));}
                text.push_str("let result=row.project().map_err(|cause|operations::data_error(stackpress_framework_sql::stores::projection_error(cause),&schema))?;\n");
            }
            "batch"=>{
                text.push_str(&format!("if input.items.is_empty()||input.items.len()>100{{return Err(operations::data_error(DataError::new(\"batch item limit exceeded\"),&schema));}}\nlet mut items=Vec::new();\nfor item in input.items{{items.push(match item{{BatchItem::Create{{input}}=>crate::generated::stores::{store}::BatchInput::Create{{input:create_input(*input,&host.create,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?}},BatchItem::Update{{key,patch}}=>crate::generated::stores::{store}::BatchInput::Update{{key,patch:patch_input(*patch,&host.patch,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?}},BatchItem::Upsert{{key,input,patch}}=>crate::generated::stores::{store}::BatchInput::Upsert{{key,input:create_input(*input,&host.create,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?,patch:patch_input(*patch,&host.patch,&schema,&context).map_err(|cause|operations::data_error(cause,&schema))?}}}});}}\nlet rows={name}s::batch_atomic(engine,items,&context).await.map_err(|cause|operations::data_error(cause,&schema))?;\nlet result=rows.into_iter().map(|row|match row{{crate::generated::stores::{store}::BatchOutcome::Created(row)=>row.project().map(Some),crate::generated::stores::{store}::BatchOutcome::Updated(row)=>row.map(|row|row.project()).transpose(),crate::generated::stores::{store}::BatchOutcome::Upserted(row)=>row.project().map(Some)}}).collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>().map_err(|cause|operations::data_error(stackpress_framework_sql::stores::projection_error(cause),&schema))?;\n"));
            }
            _=>{
                let context=if operation=="delete"{""}else{",&context"};
                text.push_str(&format!("let row={name}s::{operation}(&mut DataSession::pooled(engine),&input.key{context}).await.map_err(|cause|operations::data_error(cause,&schema))?.ok_or_else(operations::not_found)?;\nlet result=row.project().map_err(|cause|operations::data_error(stackpress_framework_sql::stores::projection_error(cause),&schema))?;\n"));
            }
        }
        text.push_str("// Publish only the already projected owned result after data acknowledgement.\nres.results(&result)?;\nOk(Flow::Continue)\n}\n");
    }
    text.push_str("/// Select named capabilities explicitly; services must exist before this Listen callback.\npub fn register(start:&mut StartupContext,registration:&mut Registration,prefix:&str,operations:&[Operation])->Result<Vec<ListenerId>>{\nstart.service::<stackpress_sql::Engine>()?;\nstart.service::<Policy>()?;\n");
    if !model.fields.iter().any(|field| {
        !field.key && !field.generated && !matches!(field.value_type, ValueType::Model(_))
    }) {
        text.push_str("if operations.iter().any(|operation|matches!(operation,Operation::Update|Operation::Upsert)){return Err(operations::Error::invalid(\"model declares no mutable patch fields\"));}\n");
    }
    if !active {
        text.push_str("if operations.iter().any(|operation|matches!(operation,Operation::Remove|Operation::Restore)){return Err(operations::Error::invalid(\"soft removal requires a Boolean active model\"));}\n");
    }
    text.push_str("registration.claim(prefix,operations)?;\nlet mut handles=Vec::new();\nfor operation in operations{let event=format!(\"{prefix}.{}\",operation.as_str());\nlet handle=match operation{\n");
    for operation in [
        "get", "detail", "search", "create", "update", "upsert", "batch", "remove", "restore",
        "delete",
    ] {
        if matches!(operation, "remove" | "restore") && !active {
            continue;
        }
        let variant = operation[..1].to_ascii_uppercase() + &operation[1..];
        text.push_str(&format!(
            "Operation::{variant}=>start.on(&event,{operation})?,\n"
        ));
    }
    if !active {
        text.push_str("Operation::Remove|Operation::Restore=>return Err(operations::Error::invalid(\"soft removal is unavailable\")),\n");
    }
    text.push_str("};handles.push(handle);}\nOk(handles)\n}\n}\n");
    Ok(())
}
