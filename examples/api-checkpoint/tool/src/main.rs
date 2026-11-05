//! Finite source-native API generation from the complete, unmodified package composition.
//! Native schemas/stores and portable UI use one graph; generation opens no database.
use stackpress_framework_schema::{Schema, codegen::transform_error};
use stackpress_idea_transformer::{
    OutputPlan, OutputWorkspace, Registry, Result, TransformContext, TransformSpec,
};
use std::path::Path;

/// All source constructs flow through the public native and portable emitters.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(context.schema(), Default::default(), &[])
        .map_err(transform_error)?;
    let definitions = stackpress_framework_api::codegen::definitions().map_err(transform_error)?;
    let ui = graph
        .project_source_ui(&definitions)
        .map_err(transform_error)?;
    stackpress_framework_schema::codegen::emit_source_contracts(&graph, output)?;
    stackpress_framework_sql::codegen::emit_source_stores(&graph, output)?;
    stackpress_framework_admin::codegen::emit_source_pages(&graph, &ui, &definitions, output)
}
/// Explicit managed install/check; the selected application routes remain Application/Session.
fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let selection: serde_json::Value = serde_json::from_str(include_str!("../selection.json"))?;
    assert_eq!(
        selection["models"],
        serde_json::json!(["Application", "Session"])
    );
    let config = stackpress_framework_api::codegen::config()?;
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("source-api").owns(&["source", "stores", "ui"]),
        emit,
    )?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(Path::new(env!("CARGO_MANIFEST_DIR")).join("generated"))?;
    let check = std::env::args().any(|arg| arg == "--check");
    if std::env::args().any(|arg| arg == "--plan") {
        let report = workspace.check(&plan)?;
        let paths = plan
            .files()
            .map(|(path, _, _)| path)
            .collect::<std::collections::BTreeSet<_>>();
        let previous: serde_json::Value = serde_json::from_slice(&std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("generated/.stackpress-generated.json"),
        )?)?;
        let removed = previous["files"]
            .as_object()
            .ok_or("invalid managed manifest")?
            .keys()
            .filter(|path| !paths.contains(path.as_str()))
            .collect::<Vec<_>>();
        println!(
            "SOURCE_PLAN {}",
            serde_json::json!({"planned":paths,"removed":removed,"differences":report.differences})
        );
        return Ok(());
    }
    let report = if check {
        workspace.check(&plan)?
    } else {
        workspace.install(plan)?
    };
    if check && !report.is_current() {
        return Err("generated API source is stale".into());
    }
    println!("API source selection is fresh");
    Ok(())
}
/// Retain the original typed diagnostic instead of hiding an emitter or writer cause.
fn main() {
    if let Err(error) = run() {
        eprintln!("{error:?}");
        std::process::exit(1)
    }
}
