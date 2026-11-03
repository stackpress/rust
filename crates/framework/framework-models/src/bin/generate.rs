//! Finite default-family generation with the same public emitter available to applications.
use stackpress_framework_schema::{
    Schema,
    codegen::{NormalizeOptions, emit_source_contracts, transform_error},
};
use stackpress_idea_transformer::{
    Composer, CompositionLimits, MemoryLoader, OutputPlan, OutputWorkspace, PackageLoader,
    Registry, Result, TransformContext, TransformSpec,
};
use std::path::{Path, PathBuf};

/// Normalize the unmodified composed graph and emit only the released source schema stage.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(context.schema(), NormalizeOptions::default(), &[])
        .map_err(transform_error)?;
    emit_source_contracts(&graph, output)
}

/// Resolve the package catalog without importing the session/API plugins as dependencies.
fn loader(
    input: String,
) -> std::result::Result<PackageLoader<MemoryLoader>, stackpress_idea_transformer::LoadError> {
    let mut app = MemoryLoader::new("/framework-models")?;
    app.insert("schema.idea", input);
    PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            include_str!("../../../framework-session/schema.idea"),
        )?
        .with_source(
            "stackpress-api/schema.idea",
            include_str!("../../../framework-api/schema.idea"),
        )?
        .with_source(
            "stackpress/stackpress.idea",
            include_str!("../../../framework/stackpress.idea"),
        )
}

/// Parse finite tool arguments; arbitrary app extensions still use the public generator.
fn options() -> std::result::Result<(PathBuf, String, bool), Box<dyn std::error::Error>> {
    let mut destination = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/generated");
    let mut input = "use \"stackpress/stackpress.idea\"".to_owned();
    let mut check = false;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--check" => check = true,
            "--output" => destination = args.next().ok_or("--output needs a path")?.into(),
            "--input" => {
                input = std::fs::read_to_string(args.next().ok_or("--input needs a path")?)?
            }
            _ => return Err("unknown generator argument".into()),
        }
    }
    Ok((destination, input, check))
}

/// Install through the existing managed writer, or verify freshness without changing bytes.
fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let (destination, input, check) = options()?;
    let config = Composer::new(loader(input)?, CompositionLimits::default())
        .compose("schema.idea")?
        .config;
    let mut registry = Registry::new();
    registry.add(TransformSpec::new("source-models").owns(&["source"]), emit)?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(destination)?;
    let report = if check {
        workspace.check(&plan)?
    } else {
        workspace.install(plan)?
    };
    // Install returns the differences it applied; only check mode treats differences as stale.
    if check && !report.is_current() {
        return Err("generated default source schemas are stale".into());
    }
    println!(
        "default source schemas {}",
        if check {
            "freshness passed"
        } else {
            "installed"
        }
    );
    Ok(())
}
