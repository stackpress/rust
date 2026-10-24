//! Bounded Idea parsing and ordered schema compilation, without IO or ambient environment.
//!
//! The extracted grammar/compiler layers expose immutable syntax through checked staged APIs.
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
mod json;
mod parser;
mod references;
mod stages;
mod token;
mod types;

use indexmap::IndexMap;

pub use ast::{
    ArrayToken, DataToken, DeclarationToken, DeclaratorToken, IdentifierToken, ImportToken,
    LiteralToken, ObjectToken, PropertyToken, SchemaToken, Statement,
};
pub use error::{ErrorCode, IdeaError};
pub use finalize::{FinalSchema, SchemaConfig};
pub use references::finalize_with_references;
pub use stages::{
    Ast, CompiledSchema, DeclarationKind, Reference, SourceFile, SourceSpan, ValuePath,
    compile_ast, compile_ast_with_references, finalize_ast, parse_ast,
};
pub use token::{Token, TokenKind, TokenValue, tokenize};
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

    /// Maximum syntax/JSON nodes; lower byte limits also lower this derived budget.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::ParseLimits;
    /// assert_eq!(ParseLimits::default().max_nodes(), 100_000);
    /// ```
    pub fn max_nodes(self) -> usize {
        self.max_bytes.min(100_000)
    }

    /// Maximum entries in one collection; lower byte limits also lower this budget.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::ParseLimits;
    /// assert_eq!(ParseLimits::default().max_collection(), 10_000);
    /// ```
    pub fn max_collection(self) -> usize {
        self.max_bytes.min(10_000)
    }
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
/// For composition, use [`parse_ast`] and [`compile_ast_with_references`] to retain actual
/// reference sites separately from literal strings containing the same placeholder spelling.
///
/// # Errors
/// Returns source-positioned syntax, duplicate, environment or resource errors.
/// No file is loaded and no environment key is read from the process.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{parse, CompileInputs};
/// let schema = parse("a.idea", "model A { id String }", &CompileInputs::default())?;
/// assert!(schema.model.contains_key("A"));
/// # Ok::<(), stackpress_idea_parser::IdeaError>(())
/// ```
pub fn parse(file: &str, source: &str, inputs: &CompileInputs) -> Result<SchemaConfig, IdeaError> {
    compile(file, source, inputs, false)
}

/// Resolve prior-declaration references and remove prop/use from a single source.
///
/// This compatibility entry mirrors the pinned final compiler: it discards use directives
/// without loading them. The transformer owns source composition; callers needing imports
/// must not treat this single-source result as a composed schema. The typed
/// [`finalize_with_references`] entry is the correct path for composed schemas.
///
/// # Errors
/// Returns parse errors or unknown prior-reference/expansion-limit failures.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{finalize, CompileInputs};
/// let schema = finalize("a", "prop P { x 1 } model A @x(P) { id String }",
///     &CompileInputs::default())?;
/// assert!(schema.model.contains_key("A"));
/// # Ok::<(), stackpress_idea_parser::IdeaError>(())
/// ```
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
    check_inputs(file, source, inputs)?;
    let ast = parse_ast(&SourceFile::new(file, source), inputs.limits)?;
    stages::compile_checked(&ast, inputs, final_values)
}

/// Bound source and explicit environment before allocating AST or expansion copies.
pub(crate) fn check_inputs(
    file: &str,
    source: &str,
    inputs: &CompileInputs,
) -> Result<(), IdeaError> {
    // The aggregate environment map is bounded before any repeated AST expansion.
    let env_bytes = inputs
        .environment
        .iter()
        .try_fold(0usize, |total, (key, value)| {
            total.checked_add(key.len())?.checked_add(value.len())
        });
    if source.len() > inputs.limits.max_bytes
        || file.len() > inputs.limits.max_bytes
        || env_bytes.is_none_or(|bytes| bytes > inputs.limits.max_bytes)
    {
        return Err(
            IdeaError::new(ErrorCode::Limit, "Idea byte budget exceeded")
                .with_span(0, 0)
                .locate(file, source),
        );
    }
    Ok(())
}
