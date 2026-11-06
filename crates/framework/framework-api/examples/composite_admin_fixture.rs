//! Finite official source-UI fixture generation; no database, listener or application IO.
#[cfg(feature = "codegen")]
/// Regenerate or freshness-check only this test fixture's portable presentation catalog.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use stackpress_framework_schema::{Schema, codegen::transform_error};
    use stackpress_idea_transformer::{OutputWorkspace, Registry, TransformContext, TransformSpec};
    let source = include_str!("../tests/fixtures/admin-composite/schema.idea");
    let config = stackpress_idea_parser::parse("schema.idea", source, &Default::default())?;
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("admin-composite-ui").owns(&["ui"]),
        |context: &TransformContext<'_>, output: &mut stackpress_idea_transformer::OutputPlan| {
            let graph = Schema::normalize_source(context.schema(), Default::default(), &[])
                .map_err(transform_error)?;
            let definitions = stackpress_framework_admin::codegen::source_definitions()
                .map_err(transform_error)?;
            let ui = graph
                .project_source_ui(&definitions)
                .map_err(transform_error)?;
            stackpress_framework_admin::codegen::emit_source_pages(
                &graph,
                &ui,
                &definitions,
                output,
            )
        },
    )?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/admin-composite/presentation"),
    )?;
    if std::env::args().any(|arg| arg == "--check") {
        if !workspace.check(&plan)?.is_current() {
            return Err("composite admin UI fixture is stale".into());
        }
    } else {
        workspace.install(plan)?;
    }
    Ok(())
}

/// Default graphs do not select any build-time dependency.
#[cfg(not(feature = "codegen"))]
fn main() {}
