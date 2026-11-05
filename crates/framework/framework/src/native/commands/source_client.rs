//! One application source graph emits native models/client and a separate portable UI partition.
//!
//! Call from an explicit build-host transform owning source/stores/ui/client. No generated
//! module is loaded or executed here; the existing managed writer installs/checks the plan.
use stackpress_framework_schema::codegen::{DefinitionRegistry, SourceGraph, source_module};
use stackpress_idea::transformer::{OutputPlan, Result, TransformError};
use stackpress_lib::template::{TemplateEngine, TemplateOptions};

/// An application-selected Rust function, optionally selected by its own Cargo feature.
#[derive(Clone, Debug)]
pub struct Binding {
    /// Compiled Rust path; never a module name to import at runtime.
    pub path: String,
    /// Application feature, such as api or ssr; None makes the callback unconditional.
    pub feature: Option<String>,
}

/// Explicit application bindings; absent values contribute no plugin/admin callback.
#[derive(Clone, Debug)]
pub struct Options {
    /// Inspectable application build identity, not an executable lookup key.
    pub build: String,
    /// Aggregate runtime path, for example crate::framework or stackpress_framework.
    pub framework: String,
    /// Generic SessionModels composite registrar path.
    pub session: Option<Binding>,
    /// Generic ApiModels process registrar path; data listeners are the model catalog.
    pub api: Option<Binding>,
    /// Application admin function: fn(&mut ServerBuilder, &'static str) -> Result<()>.
    pub admin: Option<Binding>,
    /// Model names whose nested processes need native Date results before the host wire boundary.
    pub native_results: Vec<String>,
}
impl Options {
    /// No plugin/admin callback is inferred from model names or config data.
    pub fn new(build: impl Into<String>, framework: impl Into<String>) -> Self {
        Self {
            build: build.into(),
            framework: framework.into(),
            session: None,
            api: None,
            admin: None,
            native_results: Vec::new(),
        }
    }
}

/// Emit the complete graph through the existing public schema/SQL/admin generators.
///
/// The client traverses models once in catalog order; SQL queues model listeners, while
/// admin callbacks stay last in the aggregate. Optional callbacks use app Cargo features.
/// Include source/stores/client only in a native module and ui in the renderer artifact.
/// # Errors
/// Refuses malformed Rust paths and missing core families before contributing any output;
/// preserves source-role/codec/template and managed output failures. Performs no database IO.
pub fn emit(
    graph: &SourceGraph,
    definitions: &DefinitionRegistry,
    options: &Options,
    output: &mut OutputPlan,
) -> Result<()> {
    checked_path(&options.framework)?;
    for binding in [&options.session, &options.api, &options.admin]
        .into_iter()
        .flatten()
    {
        checked_path(&binding.path)?;
    }
    let has = |name| graph.models().iter().any(|model| model.name == name);
    if options.session.is_some() && !(has("Profile") && has("Auth")) {
        return Err(TransformError::new(
            "session binding requires generated Profile/Auth",
        ));
    }
    if options.api.is_some()
        && !(has("Profile") && has("Auth") && has("Application") && has("Session"))
    {
        return Err(TransformError::new(
            "API binding requires one complete generated API family",
        ));
    }
    if options
        .native_results
        .iter()
        .any(|name| !has(name.as_str()))
    {
        return Err(TransformError::new("native response model is absent"));
    }
    let ui = graph
        .project_source_ui(definitions)
        .map_err(stackpress_framework_schema::codegen::transform_error)?;
    let mut family = "family".to_owned();
    while graph.models().iter().any(|model| model.module == family) {
        family.insert(0, '_');
    }
    let models = graph.models().iter().map(|model| serde_json::json!({
        "schema": source_module(model),
        "store": if model.module == "stores" { "_stores" } else { &model.module },
        "rust": model.rust_name, "representation": if options.native_results.contains(&model.name) {"Native"}else{"Json"}, "name": literal(&model.name), "module": literal(&model.module),
    })).collect::<Vec<_>>();
    let engine = TemplateEngine::new(TemplateOptions {
        delimiters: ["[%".into(), "%]".into()],
        ..Default::default()
    })
    .map_err(template_error)?;
    let data = serde_json::json!({
        "framework": options.framework, "build": literal(&options.build), "family": family,
        "models": models, "session": binding(&options.session), "api": binding(&options.api), "admin": binding(&options.admin),
    });
    let client = engine
        .compile(CLIENT)
        .and_then(|template| template.render(&data))
        .map_err(template_error)?;
    // Only normalized declarations and compiled, validated paths reach the structural emitters.
    stackpress_framework_schema::codegen::emit_source_contracts(graph, output)?;
    stackpress_framework_sql::codegen::emit_source_stores(graph, output)?;
    stackpress_framework_admin::codegen::emit_source_pages(graph, &ui, definitions, output)?;
    output.write("client/framework.rs", client)?;
    output.contribute_module("client", "framework")
}

/// Validate a function/type path, not arbitrary Rust source or expression text.
fn checked_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.split("::").any(|part| {
            part.is_empty()
                || !part.bytes().enumerate().all(|(index, byte)| {
                    byte == b'_'
                        || byte.is_ascii_alphabetic()
                        || (index > 0 && byte.is_ascii_digit())
                })
        })
    {
        return Err(TransformError::new(
            "source client binding requires a Rust path",
        ));
    }
    Ok(())
}
/// Encode literal contents independently of the template's structural source fragments.
fn literal(value: &str) -> String {
    format!("{value:?}")
}
/// Preserve the concrete renderer cause behind a bounded phase description.
fn template_error(cause: stackpress_lib::template::TemplateError) -> TransformError {
    TransformError::with_source(
        stackpress_idea::transformer::TransformErrorKind::Other,
        "source client structural template failed",
        cause,
    )
    .unwrap_or_else(|error| error)
}
/// Template conditionals select app-owned callbacks without dynamic module loading.
fn binding(binding: &Option<Binding>) -> serde_json::Value {
    match binding {
        Some(binding) => {
            serde_json::json!({"selected":true,"path":binding.path,"gated":binding.feature.is_some(),"feature":binding.feature.as_deref().map(literal)})
        }
        None => serde_json::json!({"selected":false}),
    }
}

/// Native client structure is a real template; the renderer never evaluates Rust or Idea.
const CLIENT: &str = r#"//! Generated application client; native only. Regenerate, never edit.
use stackpress_framework_sql::source::{SourceModel,SourceStore};
use stackpress_server::ServerBuilder;
use stackpress_lib::Result;
/// Complete native application metadata; never a browser/SSR seed.
pub fn describe()->Vec<stackpress_framework_schema::Model> { vec![
[%#@:models model%]    super::super::source::[%model.schema%]::describe(),
[%/@:models%]] }
/// Explicit DDL preparation; installing this module performs no database work.
pub fn definition()->stackpress_sql::Result<Vec<stackpress_sql::TableDefinition>> { Ok(vec![
[%#@:models model%]    table::<super::super::stores::[%model.store%]::[%model.rust%]Model>()?,
[%/@:models%]]) }
/// Prepare a definition from the actual associated generated store, without an Engine.
fn table<M:SourceModel>()->stackpress_sql::Result<stackpress_sql::TableDefinition> { M::Store::new(String::new(),0).create().definition() }
[%#@:models model%]
/// Queue only this model's generated source event inventory, preserving catalog order.
fn listen_[%model.store%](app:&mut ServerBuilder)->Result<()> { stackpress_framework_sql::source::register_with::<super::super::stores::[%model.store%]::[%model.rust%]Model>(app,stackpress_framework_sql::source::ResponseValues::[%model.representation%]) }
[%#?:admin.selected%]
[%#?:admin.gated%]#[cfg(feature=[%admin.feature%])]
[%/?:admin.gated%]/// Application-owned rendering/route adapter, registered at the aggregate's final stage.
fn admin_[%model.store%](app:&mut ServerBuilder)->Result<()> { [%admin.path%](app,[%model.module%]) }
[%/?:admin.selected%]
[%/@:models%]
[%#?:session.selected%]
[%#?:session.gated%]#[cfg(feature=[%session.feature%])]
[%/?:session.gated%]/// Bind handwritten auth orchestration to this exact generated family.
fn session(app:&mut ServerBuilder)->Result<()> { [%session.path%]::<super::super::stores::[%family%]::Models>(app) }
[%/?:session.selected%]
[%#?:api.selected%]
[%#?:api.gated%]#[cfg(feature=[%api.feature%])]
[%/?:api.gated%]/// API processes consume this family's ordinary named model events.
fn api(app:&mut ServerBuilder)->Result<()> { [%api.path%]::<super::super::stores::[%family%]::Models>(app) }
[%/?:api.selected%]
/// One static source catalog; callbacks run only when explicitly registered by the app.
pub static CLIENT:[%framework%]::CompiledClient=[%framework%]::CompiledClient {
    build:[%build%], describe,
    bindings:[%framework%]::FrameworkBindings {
        session: {
[%#?:session.selected%][%#?:session.gated%]            #[cfg(feature=[%session.feature%])] { Some(session as [%framework%]::Register) }
            #[cfg(not(feature=[%session.feature%]))] { None }
[%|%]            Some(session as [%framework%]::Register)
[%/?:session.gated%][%|%]            None
[%/?:session.selected%]        },
        api: {
[%#?:api.selected%][%#?:api.gated%]            #[cfg(feature=[%api.feature%])] { Some(api as [%framework%]::Register) }
            #[cfg(not(feature=[%api.feature%]))] { None }
[%|%]            Some(api as [%framework%]::Register)
[%/?:api.gated%][%|%]            None
[%/?:api.selected%]        },
    },
    models:&[
[%#@:models model%]        [%framework%]::CompiledModel { name:[%model.name%], listen:Some(listen_[%model.store%]), admin: {
[%#?:admin.selected%][%#?:admin.gated%]            #[cfg(feature=[%admin.feature%])] { Some(admin_[%model.store%] as [%framework%]::Register) }
            #[cfg(not(feature=[%admin.feature%]))] { None }
[%|%]            Some(admin_[%model.store%] as [%framework%]::Register)
[%/?:admin.gated%][%|%]            None
[%/?:admin.selected%]        } },
[%/@:models%]    ],
};
"#;
