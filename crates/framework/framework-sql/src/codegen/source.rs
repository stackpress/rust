//! Source store emission uses SourceGraph directly; Typed output never passes through here.
use super::templates::Templates;
use serde_json::json;
use stackpress_framework_schema::codegen::SourceGraph;
use stackpress_idea_transformer::{OutputPlan, Result};

/// Emit native stores/actions and optional source event registrars from a complete source graph.
/// The registry transform must own `stores`; schema contracts are emitted separately in `source`.
/// # Errors
/// Preserves template, ownership and output-budget failures without publishing partial files.
pub fn emit_source_stores(graph: &SourceGraph, output: &mut OutputPlan) -> Result<()> {
    let templates = Templates::new()?;
    let mut files = Vec::new();
    for model in graph.models() {
        let role = match model.name.as_str() {
            "Profile" => "stackpress_framework_sql::source::ProfileRole",
            "Auth" => "stackpress_framework_sql::source::AuthRole",
            "Application" => "stackpress_framework_sql::source::ApplicationRole",
            "Session" => "stackpress_framework_sql::source::ApiSessionRole",
            _ => "()",
        };
        let text = templates.render(
            "source store template failed",
            include_str!("templates/source-store.rs.tpl"),
            &json!({"name":model.rust_name,"module":stackpress_framework_schema::codegen::source_module(model),"role":role}),
        )?;
        files.push((
            if model.module == "stores" {
                "_stores".into()
            } else {
                model.module.clone()
            },
            text,
        ));
    }
    let module = |name: &str| {
        graph.models().iter().find(|m| m.name == name).map(|m| {
            if m.module == "stores" {
                "_stores"
            } else {
                m.module.as_str()
            }
        })
    };
    let session = module("Profile").is_some() && module("Auth").is_some();
    let mut family = "family".to_owned();
    while graph.models().iter().any(|m| m.module == family) {
        family.insert(0, '_');
    }
    let text=templates.render("source family template failed",include_str!("templates/source-family.rs.tpl"),&json!({"session":session,"api":session&&module("Application").is_some()&&module("Session").is_some(),"profile":module("Profile"),"auth":module("Auth"),"application":module("Application"),"session_module":module("Session")}))?;
    files.push((family, text));
    for (module, text) in files {
        output.write(format!("stores/{module}.rs"), text)?;
        output.contribute_module("stores", &module)?;
    }
    Ok(())
}
