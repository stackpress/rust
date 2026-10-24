//! Bounded Idea parsing and ordered schema compilation, without IO or ambient environment.
//!
//! This M0 extraction retains the upstream grammar/compiler layers behind a private AST.
//! Callers receive serializable config; they cannot construct a deep AST bypassing limits.
//! See PROVENANCE.md for the pinned source and Apache-2.0 license retention.
//!
//! ```
//! use stackpress_idea_parser::{parse, CompileInputs};
//! let schema = parse("schema.idea", "model Profile { id String name String }",
//!     &CompileInputs::default())?;
//! assert_eq!(schema.model["Profile"].columns[0].name, "id");
//! # Ok::<(), stackpress_idea_parser::IdeaError>(())
//! ```

mod ast;
mod error;
mod finalize;
mod parser;
mod types;

use indexmap::IndexMap;

pub use error::{ErrorCode, IdeaError};
pub use finalize::{FinalSchema, SchemaConfig};
pub use types::TypeConfig;
pub use types::{AttributeValue, ColumnConfig, EnumConfig, ModelConfig, PluginConfig, PropConfig};

/// Source and recursion budgets; the hard recursive ceiling cannot be raised.
#[derive(Debug, Clone, Copy)]
pub struct ParseLimits {
    /// Maximum source bytes and aggregate environment expansion bytes.
    pub max_bytes: usize,
    /// Maximum nested object/array depth, capped by [`Self::HARD_DEPTH`].
    pub max_depth: usize,
}

impl ParseLimits {
    /// Maximum supported recursive data depth in this native implementation.
    pub const HARD_DEPTH: usize = 64;
}

impl Default for ParseLimits {
    /// Bound source and recursion even when callers supply no configuration.
    fn default() -> Self {
        Self {
            max_bytes: 1024 * 1024,
            max_depth: Self::HARD_DEPTH,
        }
    }
}

/// Explicit per-run compile inputs; cloning copies ordinary data independently.
#[derive(Debug, Clone, Default)]
pub struct CompileInputs {
    /// Only explicitly supplied keys are visible to env("NAME").
    pub environment: IndexMap<String, String>,
    /// Explicit compatibility opt-in for missing environment values becoming empty strings.
    pub missing_env_is_empty: bool,
    /// Caller-lowered bounded-resource policy.
    pub limits: ParseLimits,
}

/// Compile one named source while retaining imports/props and unresolved placeholders.
///
/// # Errors
/// Returns source-positioned syntax, duplicate, environment or resource errors.
/// No file is loaded and no environment key is read from the process.
pub fn parse(file: &str, source: &str, inputs: &CompileInputs) -> Result<SchemaConfig, IdeaError> {
    compile(file, source, inputs, false)
}

/// Resolve prior-declaration references and remove prop/use from a single source.
///
/// This M0 compatibility entry mirrors the pinned final compiler: it discards use directives
/// without loading them. Use composition is a later milestone; callers needing imports must
/// not treat this single-source result as a composed schema.
///
/// # Errors
/// Returns parse errors or unknown prior-reference/expansion-limit failures.
pub fn finalize(
    file: &str,
    source: &str,
    inputs: &CompileInputs,
) -> Result<FinalSchema, IdeaError> {
    let schema = compile(file, source, inputs, true)?;
    Ok(FinalSchema {
        plugin: schema.plugin,
        r#enum: schema.r#enum,
        r#type: schema.r#type,
        model: schema.model,
    })
}

/// Check budgets before constructing an AST, then normalize every error at one source boundary.
fn compile(
    file: &str,
    source: &str,
    inputs: &CompileInputs,
    final_values: bool,
) -> Result<SchemaConfig, IdeaError> {
    // The aggregate environment map is bounded before any repeated AST expansion.
    let env_bytes = inputs
        .environment
        .iter()
        .try_fold(0usize, |total, (key, value)| {
            total.checked_add(key.len())?.checked_add(value.len())
        });
    if source.len() > inputs.limits.max_bytes
        || env_bytes.is_none_or(|bytes| bytes > inputs.limits.max_bytes)
    {
        return Err(
            IdeaError::new(ErrorCode::Limit, "Idea byte budget exceeded")
                .with_span(0, 0)
                .locate(file, source),
        );
    }
    // The private AST is always created under the same hard recursion policy.
    parser::parse(source, inputs)
        .and_then(|ast| finalize::compile_schema(&ast, final_values, inputs.limits))
        .map_err(|error| error.locate(file, source))
}
