//! Test-only regeneration through the public package loader and managed source emitters.
use stackpress_framework_schema::{
    Schema,
    codegen::{emit_source_contracts, transform_error},
};
use stackpress_idea_transformer::{
    Composer, CompositionLimits, OutputPlan, OutputWorkspace, PackageLoader, ProjectLoader,
    Registry, Result, TransformContext, TransformSpec,
};
use std::path::Path;

/// Emit complete authored contracts and native stores without opening a database.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    // Session-only composition retains its two declared external reverse links.
    let external = ["Application", "Session"]
        .into_iter()
        .filter(|name| !context.schema().model.contains_key(*name))
        .collect::<Vec<_>>();
    let graph = Schema::normalize_source(context.schema(), Default::default(), &external)
        .map_err(transform_error)?;
    emit_source_contracts(&graph, output)?;
    stackpress_framework_sql::codegen::emit_source_stores(&graph, output)
}

/// Check or install one crate-owned family. Check mode never repairs stale bytes.
pub fn generate(
    input: &Path,
    output: &Path,
    check: bool,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Resolve imports from the application source without changing process cwd.
    let input = std::fs::canonicalize(input)?;
    let loader = PackageLoader::new(ProjectLoader::new(
        input.parent().ok_or("input has no parent")?,
    )?)
    .with_source(
        "stackpress-session/schema.idea",
        stackpress_framework_session::SCHEMA_SOURCE,
    )?;
    let loader = loader.with_source(
        "stackpress-api/schema.idea",
        stackpress_framework_api::SCHEMA_SOURCE,
    )?;
    // Compose raw declarations before normalizing; all authored fields remain present.
    let config = Composer::new(loader, CompositionLimits::default())
        .compose(input.to_str().ok_or("input must be UTF-8")?)?
        .config;
    // The same public emitters own both managed native partitions.
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("source-models").owns(&["source", "stores"]),
        emit,
    )?;
    let plan = registry.plan(&config, Vec::new())?;
    // Check mode compares the plan without installing or repairing any files.
    let workspace = OutputWorkspace::new(output)?;
    let report = if check {
        workspace.check(&plan)?
    } else {
        workspace.install(plan)?
    };
    if check && !report.is_current() {
        return Err(format!(
            "stale generated family at {}: {:?}",
            output.display(),
            report.differences
        )
        .into());
    }
    Ok(())
}

/// Portable management UI is generated independently from the same unmodified API graph.
#[cfg(feature = "codegen")]
fn presentation(check: bool) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let config = stackpress_framework_api::codegen::config()?;
    // The same public emitters own both managed native partitions.
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("api-test-presentation").owns(&["ui"]),
        |context, output| {
            let graph = Schema::normalize_source(context.schema(), Default::default(), &[])
                .map_err(transform_error)?;
            let definitions =
                stackpress_framework_api::codegen::definitions().map_err(transform_error)?;
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
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/presentation"),
    )?;
    let report = if check {
        workspace.check(&plan)?
    } else {
        workspace.install(plan)?
    };
    if check && !report.is_current() {
        return Err("stale API test presentation".into());
    }
    Ok(())
}

/// Regenerate every maintained local family, or verify them all with `--check`.
pub fn fixtures(check: bool) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    for (input, output) in [("models.idea", "models"), ("extended.idea", "extended")] {
        generate(&base.join(input), &base.join(output), check)?;
    }
    #[cfg(feature = "codegen")]
    presentation(check)?;
    Ok(())
}
