//! Full, unmodified API/session package composition for source-native generation.
use stackpress_framework_schema::{
    Schema, SchemaResult,
    codegen::{DefinitionRegistry, NormalizeOptions, SourceGraph},
};
use stackpress_idea_transformer::{Composer, MemoryLoader, PackageLoader};
/// Exact shipped source asset, retaining its original package import.
pub const SOURCE: &str = crate::SCHEMA_SOURCE;
/// Source role vocabulary and shared compiled adapters, independently of native stores.
pub fn definitions() -> SchemaResult<DefinitionRegistry> {
    stackpress_framework_admin::codegen::source_definitions()
}
/// Compose the complete shipped graph with the same opt-in package loader used by applications.
/// No import stripping, synthetic Profile, relation patch or three-model projection occurs.
pub fn config() -> stackpress_idea_transformer::Result<stackpress_idea_parser::SchemaConfig> {
    let mut app = MemoryLoader::new("/framework-api").map_err(load_error)?;
    app.insert("schema.idea", SOURCE);
    let loader = PackageLoader::new(app)
        .with_source(
            "stackpress-session/schema.idea",
            stackpress_framework_session::SCHEMA_SOURCE,
        )
        .map_err(load_error)?;
    Ok(Composer::new(loader, Default::default())
        .compose("schema.idea")?
        .config)
}
/// Retain loader diagnostics as the private cause of this public composition boundary.
fn load_error(
    cause: stackpress_idea_transformer::LoadError,
) -> stackpress_idea_transformer::TransformError {
    stackpress_idea_transformer::TransformError::with_source(
        stackpress_idea_transformer::TransformErrorKind::Parse,
        "API package source loading failed",
        cause,
    )
    .unwrap_or_else(|error| error)
}
/// Normalize all four models and source attributes; renderer selection remains explicit.
pub fn normalize(definitions: &DefinitionRegistry) -> SchemaResult<SourceGraph> {
    let config = config().map_err(|cause| {
        stackpress_framework_schema::SchemaError::new(
            stackpress_framework_schema::SchemaErrorKind::Invalid,
            "API Idea composition failed",
        )
        .with_cause(cause)
    })?;
    let graph = Schema::normalize_source(&config, NormalizeOptions::default(), &[])?;
    graph.project_source_ui(definitions)?;
    Ok(graph)
}
