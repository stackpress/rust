//! Local source-template rendering, with Rust literal policy kept outside the generic engine.

use serde_json::Value;
use stackpress_framework_schema::{SchemaError, SchemaErrorKind, SchemaResult};
use stackpress_lib::template::{
    Template, TemplateEngine, TemplateErrorKind, TemplateLimits, TemplateOptions,
};
use std::{
    collections::{BTreeMap, btree_map::Entry},
    error::Error,
};

/// Preserve the writer's one-MiB output bound while allowing encoded context overhead.
const OUTPUT_BYTES: usize = 1024 * 1024;
/// JSON quoting of Rust literals/fragments is additional to their emitted source bytes.
const DATA_BYTES: usize = 8 * OUTPUT_BYTES;

/// Finite authored assets; identities and source cannot be mismatched by a caller.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Asset {
    /// Per-model role functions and public descriptors.
    Roles,
    /// One shared typed field context and selected component invocation.
    Control,
    /// A public list-role table.
    Table,
    /// Selected public page adapters.
    Pages,
    /// Closed typed adapter dispatch.
    Dispatch,
    /// Native operation and import-existence registration.
    Host,
    /// Shared direct and Route-phase mappings.
    Routes,
    /// Fixed function-based native plugin registration.
    Registration,
}

impl Asset {
    /// Resolve a static error identity together with its checked-in structural template.
    fn definition(self) -> (&'static str, &'static str) {
        match self {
            Self::Roles => ("admin/roles.rs.tpl", include_str!("templates/roles.rs.tpl")),
            Self::Control => (
                "admin/control.rs.tpl",
                include_str!("templates/control.rs.tpl"),
            ),
            Self::Table => ("admin/table.rs.tpl", include_str!("templates/table.rs.tpl")),
            Self::Pages => ("admin/pages.rs.tpl", include_str!("templates/pages.rs.tpl")),
            Self::Dispatch => (
                "admin/dispatch.rs.tpl",
                include_str!("templates/dispatch.rs.tpl"),
            ),
            Self::Host => (
                "admin/pages_host.rs.tpl",
                include_str!("templates/pages_host.rs.tpl"),
            ),
            Self::Routes => (
                "admin/routes.rs.tpl",
                include_str!("templates/routes.rs.tpl"),
            ),
            Self::Registration => (
                "admin/pages_host_registration.rs.tpl",
                include_str!("templates/pages_host_registration.rs.tpl"),
            ),
        }
    }
}

/// One emitter invocation's immutable-template cache; render data/counters stay call-local.
///
/// At most the eight authored assets are compiled. This owns no global state and is dropped
/// with its emitter call, so distinct generation jobs never share mutable rendering state.
pub(super) struct Renderer {
    /// Validated local delimiter/byte policy, shared by this invocation's templates.
    engine: TemplateEngine,
    /// Compile lazily once per used asset; unselected assets do no compilation work.
    compiled: BTreeMap<Asset, Template>,
}

impl Renderer {
    /// Validate local input headroom without changing writer or formatted-artifact caps.
    pub(super) fn new() -> SchemaResult<Self> {
        let engine = TemplateEngine::new(TemplateOptions {
            delimiters: ["[%".into(), "%]".into()],
            limits: TemplateLimits {
                max_data_bytes: DATA_BYTES,
                max_output_bytes: OUTPUT_BYTES,
                ..TemplateLimits::default()
            },
        })
        .map_err(|cause| template_error("admin/template-options", cause))?;
        Ok(Self {
            engine,
            compiled: BTreeMap::new(),
        })
    }

    /// Render fresh checked data through an immutable compiled asset, retaining identity/span.
    pub(super) fn render(&mut self, asset: Asset, data: &Value) -> SchemaResult<String> {
        let (identity, source) = asset.definition();
        let template = match self.compiled.entry(asset) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                // Compilation retains source/helper bindings only, never the render context.
                let template = self
                    .engine
                    .compile(source)
                    .map_err(|cause| template_error(identity, cause))?;
                entry.insert(template)
            }
        };
        // Each call separately counts compact encoded data and starts fresh work/output budgets.
        template
            .render(data)
            .map_err(|cause| template_error(identity, cause))
    }
}

/// Encode an ordinary Rust string literal, never JSON or HTML source escaping.
pub(super) fn literal(value: &str) -> String {
    format!("{value:?}")
}

/// Preserve the writer's public Limit category when a source template reaches it first.
pub(super) fn source_error(cause: SchemaError) -> stackpress_idea_transformer::TransformError {
    // Match only our concrete engine cause; unrelated schema Limit errors retain their policy.
    let template_limit = cause
        .source()
        .and_then(|cause| cause.downcast_ref::<stackpress_lib::template::TemplateError>())
        .is_some_and(|cause| cause.kind() == TemplateErrorKind::Limit);
    if template_limit {
        stackpress_idea_transformer::TransformError::with_source(
            stackpress_idea_transformer::TransformErrorKind::Limit,
            "generated output budget exceeded",
            cause,
        )
        .unwrap_or_else(|error| error)
    } else {
        stackpress_framework_schema::codegen::transform_error(cause)
    }
}

/// Preserve the size-error classification without disclosing a rejected literal or template.
fn template_error(
    asset: &'static str,
    cause: stackpress_lib::template::TemplateError,
) -> SchemaError {
    let kind = if cause.kind() == TemplateErrorKind::Limit {
        SchemaErrorKind::Limit
    } else {
        SchemaErrorKind::Invalid
    };
    // Only the authored asset identity enters Display; the concrete cause retains its span.
    SchemaError::new(kind, asset).with_cause(cause)
}

#[cfg(test)]
mod tests;
