//! Generated typed page adapters and explicit host route/operation selection.
use super::emit::{control_source, public_type};
use stackpress_framework_schema::{
    Role, Schema, SchemaResult, UiModel,
    codegen::{DefinitionRegistry, transform_error},
};
use stackpress_idea_transformer::{OutputPlan, Result};

/// Generate typed adapters for the application's explicit, currently present model selection.
///
/// Call `emit_roles` first for its complete role/source-coordinate preflight. Pure adapters
/// compile in the frontend; the separate host file is explicitly included only by the native
/// application. Both files belong to the existing UI partition and the same output writer.
/// # Errors
/// Refuses duplicate selections/private identities and preserves schema/generator causes.
pub fn emit_pages(
    schema: &Schema,
    definitions: &DefinitionRegistry,
    selected: &[&str],
    output: &mut OutputPlan,
) -> Result<()> {
    let mut unique = std::collections::BTreeSet::new();
    if selected.iter().any(|name| !unique.insert(*name)) {
        return Err(stackpress_idea_transformer::TransformError::new(
            "duplicate admin model selection",
        ));
    }
    let ui = schema.project_ui().map_err(transform_error)?;
    // Select import tokens before embedding any untrusted labels or props into source.
    let controls = ui
        .models
        .iter()
        .filter(|model| selected.contains(&model.id.as_str()))
        .any(|model| {
            model.fields.iter().any(|field| {
                field.roles.iter().any(|binding| {
                    binding.role == Role::Field && crate::draft::admitted(field, Role::Field)
                })
            })
        });
    let present = schema
        .models()
        .iter()
        .any(|model| selected.contains(&model.name.as_str()));
    let context_import = if controls { "RoleContext," } else { "" };
    let role_import = if controls { "Role," } else { "" };
    let mut pure = format!(
        "//! Generated public page adapters; imports register nothing.\nuse leptos::prelude::*;\nuse stackpress_framework_admin::{{{context_import}Draft,pages::{{Action,Form,PageSpec}}}};\nuse stackpress_framework_schema::{{{role_import}SchemaError,SchemaErrorKind,SchemaResult}};\n"
    );
    let operation_import = if present { "Operation," } else { "" };
    let (start, marker, builder, mount) = if present {
        ("start", "marker", "builder", "mount")
    } else {
        ("_start", "_marker", "_builder", "_mount")
    };
    let mut host = format!(
        "//! Explicit native compiled admin registration.\nuse stackpress_framework_sql::operations::{{{operation_import}Registration,StartupContext,Result}};\n/// Select the actual generated operation handlers; no handwritten operation substitute.\npub fn operations({start}:&mut StartupContext,{marker}:&mut Registration)->Result<()> {{\n"
    );
    let mut routes = format!(
        "/// Register fixed routes beneath a previously validated application mount.\npub fn routes({builder}:&mut stackpress_server::ServerBuilder,{mount}:&str)->stackpress_lib::Result<()> {{\n"
    );
    let mut models = Vec::new();
    for name in selected {
        // A reduced schema removes the old model's adapters and registrations in one install.
        let Some(model) = schema.models().iter().find(|model| model.name == *name) else {
            continue;
        };
        let projected = ui
            .models
            .iter()
            .find(|item| item.id == *name)
            .ok_or_else(|| {
                stackpress_idea_transformer::TransformError::new(
                    "admin model has no public projection",
                )
            })?;
        if projected.key.is_empty() {
            return Err(stackpress_idea_transformer::TransformError::new(
                "admin requires an approved complete identity",
            ));
        }
        let module = &model.module;
        let soft = model.fields.iter().any(|field| field.active);
        models.push((module.clone(), model.name.clone()));
        pure.push_str(
            &model_source(module, projected, soft, definitions).map_err(transform_error)?,
        );
        // SQL owns its reserved aggregate-module escape; route names remain source-owned.
        let native_module = if matches!(module.as_str(), "schema" | "operations") {
            format!("_{module}")
        } else {
            module.clone()
        };
        let optional = if soft {
            ",Operation::Remove,Operation::Restore"
        } else {
            ""
        };
        host.push_str(&format!("crate::generated::stores::operations::{native_module}::register(start,marker,{module:?},&[Operation::Get,Operation::Detail,Operation::Search,Operation::Create,Operation::Update{optional}])?;\n"));
        for action in [
            "search", "detail", "create", "update", "copy", "remove", "restore",
        ] {
            if !soft && matches!(action, "remove" | "restore") {
                continue;
            }
            routes.push_str(&format!(
                "builder.get(&format!(\"{{mount}}{module}/{action}\"),crate::host::page)?;\n"
            ));
        }
        for action in [
            "search", "get", "detail", "create", "update", "remove", "restore",
        ] {
            if !soft && matches!(action, "remove" | "restore") {
                continue;
            }
            routes.push_str(&format!(
                "builder.post(&format!(\"{{mount}}api/{module}/{action}\"),crate::host::api)?;\n"
            ));
        }
    }
    pure.push_str(&dispatch(&models));
    host.push_str("Ok(())}\n");
    routes.push_str("Ok(())}\n");
    host.push_str(&routes);
    // Contribute only after every selected model passed; the writer owns final publication.
    output.write("ui/pages.rs", pure)?;
    output.contribute_module("ui", "pages")?;
    output.write("ui/pages_host.rs", host)?;
    Ok(())
}

/// Keep typed codecs, field-role composition and public table/detail renderers centered per model.
fn model_source(
    module: &str,
    model: &UiModel,
    soft: bool,
    definitions: &DefinitionRegistry,
) -> SchemaResult<String> {
    let public = public_type(&model.id)?;
    let name = public.strip_prefix("Public").expect("public type prefix");
    let spec = serde_json::json!({"module":module,"model":model,"soft_remove":soft}).to_string();
    let has_controls = model.fields.iter().any(|field| {
        field.roles.iter().any(|binding| {
            binding.role == Role::Field && crate::draft::admitted(field, Role::Field)
        })
    });
    let form_locals = if has_controls {
        "let draft=&form.draft;let instance=&form.instance;let mut fields=super::super::MODEL::descriptors()?;\n// Related scope can add a disabled constraint, never enable a forbidden field.\nif let Some(locked)=&form.locked {for field in &mut fields {if &field.wire_name==locked {for binding in &mut field.roles {if binding.role==Role::Field {binding.props.insert(\"disabled\".into(),true.into());}}}}}\nlet mut controls=Vec::<AnyView>::new();\n".replace("MODEL",module)
    } else {
        "let _=form;let controls=Vec::<AnyView>::new();\n".into()
    };
    let mut source = format!(
        r#"
/// Typed adapters for one explicitly selected public model.
pub mod {module} {{
use super::*;
/// Decode only the approved revision-2 model specification.
pub fn spec()->SchemaResult<PageSpec> {{serde_json::from_str({spec:?}).map_err(codec)}}
/// Decode through the actual generated public create/patch DTOs.
pub fn input(action:Action,value:serde_json::Value)->SchemaResult<serde_json::Value> {{
if action==Action::Update {{serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicPatch{name}>(value).map_err(codec)?).map_err(codec)}}
else {{serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicCreate{name}>(value).map_err(codec)?).map_err(codec)}}
}}
/// Validate the actual generated query type before sending its envelope.
pub fn query(value:serde_json::Value)->SchemaResult<serde_json::Value> {{
let query:super::super::super::contracts::public::{name}Query=serde_json::from_value(value).map_err(codec)?;
query.validate()?;serde_json::to_value(query).map_err(codec)
}}
/// Compose selected field adapters; updates cannot edit the stored identity.
pub fn form(form:Form)->SchemaResult<AnyView> {{
{form_locals}
"#
    );
    for field in &model.fields {
        let Some(binding) = field
            .roles
            .iter()
            .find(|binding| binding.role == Role::Field)
        else {
            continue;
        };
        if !crate::draft::admitted(field, Role::Field) {
            continue;
        }
        if model.key.contains(&field.wire_name) {
            source.push_str("if form.action!=Action::Update {\n");
        }
        source.push_str(&control_source(
            field,
            binding,
            Role::Field,
            definitions,
            false,
        )?);
        if model.key.contains(&field.wire_name) {
            source.push_str("}\n");
        }
    }
    source.push_str("Ok(controls.collect_view().into_any())}\n");
    if model.fields.iter().any(|field| {
        field.roles.iter().any(|binding| {
            binding.role == Role::Filter && crate::draft::admitted(field, Role::Filter)
        })
    }) {
        source.push_str(&format!("/// Compose the explicitly selected filter roles.\npub fn filters(draft:&Draft,instance:&str)->SchemaResult<AnyView>{{super::super::{module}::filter(draft,instance)}}\n"));
    } else {
        source.push_str("/// This page has no selected filter controls.\npub fn filters(_:&Draft,_:&str)->SchemaResult<AnyView>{Ok(Vec::<AnyView>::new().collect_view().into_any())}\n");
    }
    if model.fields.iter().any(|field| {
        field
            .roles
            .iter()
            .any(|binding| binding.role == Role::List && crate::draft::admitted(field, Role::List))
    }) {
        source.push_str(&format!(r#"
/// Render actual public rows through their typed DTO and selected list adapters.
pub fn table(value:&serde_json::Value,instance:&str)->SchemaResult<AnyView> {{
let spec=spec()?;let mut rows=Vec::new();
for value in value.get("rows").and_then(serde_json::Value::as_array).ok_or_else(unavailable)? {{
let row=value.get("row").unwrap_or(value);let key=stackpress_framework_admin::pages::key(&spec,row)?;
let typed:super::super::super::contracts::public::{public}=serde_json::from_value(row.clone()).map_err(codec)?;
rows.push((key.to_string(),typed));
}}super::super::{module}::table(&rows,instance)
}}
"#));
    } else {
        source.push_str("/// No list role was selected.\npub fn table(_: &serde_json::Value,_:&str)->SchemaResult<AnyView>{Err(unavailable())}\n");
    }
    if model.fields.iter().any(|field| {
        field
            .roles
            .iter()
            .any(|binding| binding.role == Role::View && crate::draft::admitted(field, Role::View))
    }) {
        source.push_str(&format!("/// Render actual public detail through its typed DTO and selected view adapters.\npub fn detail(value:&serde_json::Value,instance:&str)->SchemaResult<AnyView>{{let typed:super::super::super::contracts::public::{public}=serde_json::from_value(value.clone()).map_err(codec)?;super::super::{module}::view(&typed,instance)}}\n"));
    } else {
        source.push_str("/// No view role was selected.\npub fn detail(_: &serde_json::Value,_:&str)->SchemaResult<AnyView>{Err(unavailable())}\n");
    }
    source.push_str("/// Keep the concrete codec cause private under one adapter context.\nfn codec(cause:serde_json::Error)->SchemaError {SchemaError::new(SchemaErrorKind::Codec,\"public admin codec failed\").with_cause(cause)}\n}\n");
    Ok(source)
}

/// Generate the same closed dispatch table for shared UI and native host callers.
fn dispatch(models: &[(String, String)]) -> String {
    let modules: Vec<_> = models.iter().map(|(module, _)| module).collect();
    let mut source = format!(
        "/// Explicit compiled page modules in navigation order.\npub const MODELS:&[&str]=&{modules:?};\n"
    );
    if models.is_empty() {
        source.push_str("/// No page model is selected.\npub fn identity(_: &str)->Option<(&'static str,&'static str)>{None}\n");
    } else {
        source.push_str("/// Resolve only compiler-owned static model identities.\npub fn identity(module:&str)->Option<(&'static str,&'static str)>{match module {\n");
        for (module, name) in models {
            source.push_str(&format!("{module:?}=>Some(({module:?},{name:?})),\n"));
        }
        source.push_str("_=>None}}\n");
    }
    for (name, args, result, call) in [
        ("spec", "module:&str", "PageSpec", "spec()"),
        (
            "input",
            "module:&str,action:Action,value:serde_json::Value",
            "serde_json::Value",
            "input(action,value)",
        ),
        (
            "query",
            "module:&str,value:serde_json::Value",
            "serde_json::Value",
            "query(value)",
        ),
        ("form", "module:&str,form:Form", "AnyView", "form(form)"),
        (
            "filters",
            "module:&str,draft:&Draft,instance:&str",
            "AnyView",
            "filters(draft,instance)",
        ),
        (
            "table",
            "module:&str,value:&serde_json::Value,instance:&str",
            "AnyView",
            "table(value,instance)",
        ),
        (
            "detail",
            "module:&str,value:&serde_json::Value,instance:&str",
            "AnyView",
            "detail(value,instance)",
        ),
    ] {
        if models.is_empty() {
            let args = args
                .split(',')
                .map(|arg| format!("_:{}", arg.split_once(':').expect("fixed signature").1))
                .collect::<Vec<_>>()
                .join(",");
            source.push_str(&format!("/// This adapter is unavailable without a selected model.\npub fn {name}({args})->SchemaResult<{result}>{{Err(unavailable())}}\n"));
        } else {
            source.push_str(&format!("/// Dispatch only to an explicitly generated typed adapter.\npub fn {name}({args})->SchemaResult<{result}>{{match module{{\n"));
            for (module, _) in models {
                source.push_str(&format!("{module:?}=>{module}::{call},\n"));
            }
            source.push_str("_=>Err(unavailable())}}\n");
        }
    }
    source.push_str("/// Missing model/role is unavailable; never substitute another operation.\nfn unavailable()->SchemaError {SchemaError::new(SchemaErrorKind::Invalid,\"admin model or role unavailable\")}\n");
    source
}
