//! Independent native generation for the blog's composed application schema.
//!
//! The default tool has no dependency on its generated output or a renderer. Package-owned
//! schema constants are imported with no optional plugin features. It uses the public
//! source emitters also used by framework-models. Generation
//! reads Idea files and installs/checks one managed output tree; it never opens a database.

use std::path::Path;

use stackpress_framework_schema::{
    Schema,
    codegen::{NormalizeOptions, emit_source_contracts, transform_error},
};
use stackpress_idea_parser::SchemaConfig;
use stackpress_idea_transformer::{
    CheckReport, Composer, CompositionLimits, LoadError, OutputPlan, OutputWorkspace,
    PackageLoader, ProjectLoader, Registry, Result, TransformContext, TransformError,
    TransformSpec,
};

/// Native compile proof only; the default generator builds when this output is absent.
#[cfg(feature = "native-proof")]
#[allow(missing_docs)]
#[path = "../../.build/generated/mod.rs"]
pub mod generated;

/// Retain loader failures as causes beneath the existing transform error boundary.
fn load_error(cause: LoadError) -> TransformError {
    TransformError::from(std::io::Error::other(cause))
}

/// Compose the entry with the exact shipped package catalog and normal G19 model merges.
///
/// This preserves all raw enum, relation and presentation attributes. Framework source
/// normalization is a separate emission step; successful composition does not prove a
/// concrete renderer or source presentation prop is supported.
///
/// # Errors
/// Retains file/path/loading and composition failures, without publishing generated output.
pub fn compose(input: &Path) -> Result<SchemaConfig> {
    // Resolve once without changing process cwd, so relative imports use the selected app root.
    let input = std::fs::canonicalize(input)?;
    let parent = input
        .parent()
        .ok_or_else(|| TransformError::new("Idea input has no parent directory"))?;
    let entry = input
        .to_str()
        .ok_or_else(|| TransformError::new("Idea input path must be UTF-8"))?;
    let project = ProjectLoader::new(parent).map_err(load_error)?;

    // Use package exports so the boilerplate can move outside this repository. No source file
    // path or sibling checkout is discovered by the application or build tool.
    let loader = PackageLoader::new(project)
        .with_source(
            "stackpress-session/schema.idea",
            stackpress_framework_session::SCHEMA_SOURCE,
        )
        .map_err(load_error)?
        .with_source(
            "stackpress-api/schema.idea",
            stackpress_framework_api::SCHEMA_SOURCE,
        )
        .map_err(load_error)?
        .with_source(
            "stackpress/stackpress.idea",
            stackpress_framework::SCHEMA_SOURCE,
        )
        .map_err(load_error)?;
    Ok(Composer::new(loader, CompositionLimits::default())
        .compose(entry)?
        .config)
}

/// Emit the same native schema/store partitions as the shipped framework-models generator.
fn emit(context: &TransformContext<'_>, output: &mut OutputPlan) -> Result<()> {
    let graph = Schema::normalize_source(context.schema(), NormalizeOptions::default(), &[])
        .map_err(transform_error)?;
    emit_source_contracts(&graph, output)?;
    stackpress_framework_sql::codegen::emit_source_stores(&graph, output)
}

/// Generate or check the complete native application output through the managed writer.
///
/// Check mode returns differences without repairing files; callers decide whether differences
/// are an error. Source roles are retained as metadata, not advertised as generated admin UI.
///
/// # Errors
/// Propagates source/normalization/emission/formatting and managed-output ownership errors.
pub fn generate(input: &Path, output: &Path, check: bool) -> Result<CheckReport> {
    let config = compose(input)?;
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("blog-source-models").owns(&["source", "stores"]),
        emit,
    )?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(output)?;
    if check {
        workspace.check(&plan)
    } else {
        workspace.install(plan)
    }
}

/// Emit the complete application client using the aggregate's supported public source path.
///
/// Unlike native-only generation, this requires a compiled adapter for every selected UI role.
/// Unsupported controls remain errors before publication; no field or metadata is removed.
///
/// # Errors
/// Retains unavailable UI adapters, compilation binding, and managed-output failures.
pub fn generate_application(input: &Path, output: &Path, check: bool) -> Result<CheckReport> {
    let config = compose(input)?;
    let mut registry = Registry::new();
    registry.add(
        TransformSpec::new("blog-application").owns(&["source", "stores", "ui", "client"]),
        |context, output| {
            let graph =
                Schema::normalize_source(context.schema(), NormalizeOptions::default(), &[])
                    .map_err(transform_error)?;
            let definitions = stackpress_framework_admin::codegen::source_definitions()
                .map_err(transform_error)?;
            let mut options = stackpress_framework::commands::source_client::Options::new(
                "blog",
                "stackpress_framework",
            );
            options.native_results = vec!["Profile".into(), "Auth".into()];
            options.session = Some(stackpress_framework::commands::source_client::Binding {
                path: "stackpress_framework_session::composite::register_with".into(),
                feature: None,
            });
            options.api = Some(stackpress_framework::commands::source_client::Binding {
                path: "stackpress_framework_api::register_with".into(),
                feature: None,
            });
            stackpress_framework::commands::source_client::emit(
                &graph,
                &definitions,
                &options,
                output,
            )
        },
    )?;
    let plan = registry.plan(&config, Vec::new())?;
    let workspace = OutputWorkspace::new(output)?;
    if check {
        workspace.check(&plan)
    } else {
        workspace.install(plan)
    }
}
