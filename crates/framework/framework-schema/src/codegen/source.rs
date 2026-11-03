//! Source-template mechanics; normalized language atoms and Rust literal encoding stay here.

use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use serde_json::Value;
use stackpress_idea_transformer::{TransformError, TransformErrorKind};
use stackpress_lib::{TemplateEngine, TemplateError, TemplateErrorKind, TemplateOptions};

/// Keep context JSON overhead separate from unchanged one-MiB raw/formatted writer caps.
fn engine() -> Result<TemplateEngine, TemplateError> {
    let mut options = TemplateOptions {
        delimiters: ["[%".into(), "%]".into()],
        ..Default::default()
    };
    // View models repeat encoded descriptors and pre-rendered fragments; they are not artifacts.
    options.limits.max_data_bytes = 16 * 1024 * 1024;
    TemplateEngine::new(options)
}

/// Render structural generator source with a concrete local error adapter.
pub(super) fn generate(
    asset: &'static str,
    template: &str,
    values: &Value,
) -> stackpress_idea_transformer::Result<String> {
    engine()
        .and_then(|engine| engine.render(template, values))
        .map_err(|cause| transform_error(asset, cause))
}

/// Preserve the schema-facing result contract of the public Rust type resolver.
pub(super) fn type_fragment(template: &str, values: &[(&str, &str)]) -> SchemaResult<String> {
    engine()
        .and_then(|engine| engine.render_pairs(template, values))
        .map_err(|cause| {
            let kind = if cause.kind() == TemplateErrorKind::Limit {
                SchemaErrorKind::Limit
            } else {
                SchemaErrorKind::Codec
            };
            SchemaError::new(kind, "Rust type template failed").with_cause(cause)
        })
}

/// Encode only a Rust string literal; JSON escaping is deliberately not substituted here.
pub(super) fn string_literal(value: &str) -> String {
    format!("{value:?}")
}

/// Keep direct emitter size errors as Limit beneath the registry's existing Callback layer.
fn transform_error(asset: &'static str, cause: TemplateError) -> TransformError {
    let kind = if cause.kind() == TemplateErrorKind::Limit {
        TransformErrorKind::Limit
    } else {
        TransformErrorKind::Callback
    };
    TransformError::with_source(
        kind,
        format!("schema source template failed: {asset}"),
        cause,
    )
    // Only a short authored asset path reaches this context; preserve constructor errors if changed.
    .unwrap_or_else(|error| error)
}
