//! Real Idea composition and app-owned storage normalization for explicit revision recording.
use super::external;
use stackpress_framework_sql::migrations::RevisionSnapshot;
use stackpress_lib::Error;
use std::path::Path;
/// Compose/finalize the real Idea input and reuse the same app-owned normalization/DDL catalog.
pub(super) fn snapshot(
    input: &Path,
    app: &str,
    name: &str,
    mysql_target: Option<(&str, u32)>,
) -> stackpress_lib::Result<RevisionSnapshot> {
    let input = std::path::absolute(input)
        .map_err(|cause| external("schema input resolution failed", cause))?;
    let root = input
        .parent()
        .ok_or_else(|| Error::invalid("schema input has no parent"))?;
    let entry = input
        .to_str()
        .ok_or_else(|| Error::invalid("schema path must be UTF-8"))?;
    let loader = stackpress_idea::transformer::ProjectLoader::new(root)
        .map_err(|cause| external("schema loader failed", cause))?;
    let request = stackpress_idea::GenerateRequest::new(
        entry,
        |_| Ok(Vec::new()),
        stackpress_idea::GenerationMode::Check,
    );
    let composed = stackpress_idea::transformer::Composer::new(loader, request.limits)
        .compose(entry)
        .map_err(|cause| external("schema composition failed", cause))?;
    let diagnostics = stackpress_framework_schema::codegen::DiagnosticMap::from_composed(&composed)
        .map_err(|cause| external("schema diagnostics failed", cause))?;
    let finalized = stackpress_idea::parser::finalize_with_references(
        &composed.config,
        &composed.references,
        &request.inputs,
    )
    .map_err(|cause| external("schema finalization failed", cause))?;
    let config = stackpress_idea::parser::SchemaConfig {
        plugin: finalized.plugin,
        r#enum: finalized.r#enum,
        r#type: finalized.r#type,
        model: finalized.model,
        ..Default::default()
    };
    let schema = super::normalize(&config, &diagnostics)
        .map_err(|cause| external("schema normalization failed", cause))?;
    // Target facts are explicit operator intent, frozen into signatures before publication.
    let tables = match mysql_target {
        None => stackpress_framework_sql::codegen::storage_definitions(&schema),
        Some((collation, page_bytes)) => {
            stackpress_framework_sql::codegen::mysql_storage_definitions(
                &schema, collation, page_bytes,
            )
        }
    }
    .map_err(|cause| external("storage normalization failed", cause))?;
    RevisionSnapshot::new(app, name, tables)
        .and_then(|s| {
            s.with_provenance(
                &schema.fingerprint().map_err(|cause| {
                    stackpress_framework_sql::DataError::new("schema fingerprint failed")
                        .with_kind(stackpress_framework_sql::DataErrorKind::Decode)
                        .with_cause(cause)
                })?,
                "app-fixture-1",
            )
        })
        .map_err(data_error)
}

/// Preserve the data-layer error beneath an operator context.
fn data_error(cause: stackpress_framework_sql::DataError) -> Error {
    external("operator schema preparation failed", cause)
}
