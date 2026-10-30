//! Finite API generator: source API management contracts/metadata and selected SQL stores.
//! No database opens, migration, application host or listener is involved.
use stackpress_framework_schema::{
    Schema,
    codegen::{emit_contracts, transform_error},
};
use stackpress_idea_transformer::{
    OutputPlan, OutputWorkspace, Registry, Result, TransformContext,
};
use std::path::Path;

/// Every transform normalizes the same already-composed immutable source input.
fn normalized(context: &TransformContext<'_>) -> Result<Schema> {
    Schema::normalize(
        context.schema(),
        &stackpress_framework_api::codegen::definitions().map_err(transform_error)?,
        Default::default(),
    )
    .map_err(transform_error)
}
/// Emit the selected public contracts through the shared writer.
fn contracts(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    emit_contracts(&normalized(context)?, plan)
}
/// Emit source stores plus the shared operation bindings.
fn stores(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    stackpress_framework_sql::codegen::emit_stores_with_diagnostics(
        &normalized(context)?,
        plan,
        &stackpress_framework_schema::codegen::DiagnosticMap {
            entries: Vec::new(),
        },
        true,
    )
}
/// Compile the selected source roles and page descriptors.
fn ui(context: &TransformContext<'_>, plan: &mut OutputPlan) -> Result<()> {
    let definitions = stackpress_framework_api::codegen::definitions().map_err(transform_error)?;
    let schema = normalized(context)?;
    stackpress_framework_admin::codegen::emit_roles(
        &schema.project_ui().map_err(transform_error)?,
        &definitions,
        &stackpress_framework_schema::codegen::DiagnosticMap {
            entries: Vec::new(),
        },
        plan,
    )?;
    stackpress_framework_admin::codegen::emit_pages(
        &schema,
        &definitions,
        &["Application", "Session"],
        plan,
    )
}
/// Generate or check one deterministic owned output plan.
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // The input selects the API models; the external Profile is resolved by its owner.
    let selection: serde_json::Value = serde_json::from_str(include_str!("../selection.json"))?;
    assert_eq!(
        selection["models"],
        serde_json::json!(["Application", "Session"])
    );
    let config = stackpress_framework_api::codegen::config()?;
    let mut registry = Registry::new();
    stackpress_framework_schema::codegen::register(&mut registry, contracts)?;
    stackpress_framework_sql::codegen::register(&mut registry, stores)?;
    stackpress_framework_admin::codegen::register_pages(&mut registry, ui)?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("generated"))?;
    let check = std::env::args().any(|arg| arg == "--check");
    let report = if check {
        workspace.check(&plan)?
    } else {
        workspace.install(plan)?
    };
    if check && !report.is_current() {
        return Err("generated API selection is stale".into());
    }
    println!("API source selection is fresh");
    Ok(())
}
/// Report generator failure as a nonzero finite command exit.
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:?}");
        std::process::exit(1);
    }
}
