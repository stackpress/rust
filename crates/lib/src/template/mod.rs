//! Strict, bounded text templates for build tools, independent of source languages.
//!
//! Text and inserted values are never evaluated as code. Conditions use booleans;
//! loops retain array/object order and render only visited branches. Rust literal
//! encoding and identifier checks belong to the generator using this module.
//!
//! Values use dotted ASCII paths; array indices have canonical decimal spelling.
//! `{{#?:flag}}yes{{|}}no{{/?:flag}}` requires a boolean; `#!` inverts it.
//! `{{#@:items item key}}...{{/@:items}}` borrows array/object entries in order.
//! Loops expose `@this`, `@key`, `@index`, `@first`, `@last`, `@odd`, `@even`;
//! aliases shadow enclosing bindings, and `@root.path` selects the original object.
//! Item properties are never implicitly spread into the enclosing context.
//!
//! Missing/invalid visited values fail; unselected branches and empty loop bodies
//! are not evaluated. Every branch is still syntax-checked during compilation.
//! Whitespace remains exact; double an opening delimiter to emit it literally.
//! Built-in inline helpers are `A:path`, `a:path`, `.:path count [suffix]`, and
//! `|:path alternatives...`. Arguments are strict paths or JSON scalar literals;
//! the primary argument is always a path. Closing delimiters end tags even within
//! quoted arguments, so pass delimiter-bearing text as data. No expressions,
//! arbitrary code evaluation, includes, custom blocks, or recursive expansion.
//!
//! ```
//! use stackpress_lib::template::{TemplateEngine, TemplateOptions};
//! let engine = TemplateEngine::default();
//! assert_eq!(engine.render_pairs("Vec<{{ty}}>", &[("ty", "String")])?, "Vec<String>");
//! let source = "{{#@:fields field}}{{field.name}}{{#?:@last}}{{|}}, {{/?:@last}}{{/@:fields}}";
//! let template = engine.compile(source)?;
//! assert_eq!(template.render(&serde_json::json!({"fields":[{"name":"id"},{"name":"title"}]}))?, "id, title");
//! # Ok::<(), stackpress_lib::TemplateError>(())
//! ```
//!
//! Custom delimiters preserve Rust format syntax and inserted template markers:
//! ```
//! use stackpress_lib::template::{TemplateEngine, TemplateOptions};
//! let engine = TemplateEngine::new(TemplateOptions {
//!     delimiters: ["[%".into(), "%]".into()],
//!     ..Default::default()
//! })?;
//! assert_eq!(engine.render_pairs("format!(\"{}\", [%value%])", &[("value", "{{inert}}")])?,
//!            "format!(\"{}\", {{inert}})");
//! # Ok::<(), stackpress_lib::TemplateError>(())
//! ```

use std::collections::BTreeMap;

use serde_json::Value;

mod data;
mod error;
mod helpers;
mod parse;
mod render;

pub use error::{TemplateError, TemplateErrorKind, TemplateResult};

/// A trusted compiled inline helper; arguments are resolved once in source order.
///
/// Returned text is inserted verbatim and checked against the output budget.
/// The engine does not sandbox helper allocations, effects, or panics.
pub type TemplateHelper = fn(&[&Value]) -> TemplateResult<String>;

/// Explicit positive per-template/per-render budgets; there is no unlimited sentinel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateLimits {
    /// Maximum UTF-8 source bytes, checked before copying or parsing.
    pub max_template_bytes: usize,
    /// Maximum compact JSON bytes, counted on borrowed data before rendering.
    pub max_data_bytes: usize,
    /// Maximum UTF-8 bytes in the engine's accumulated output.
    pub max_output_bytes: usize,
    /// Maximum nested blocks, with the first block at depth one; at most 128.
    pub max_nesting: usize,
    /// Maximum data depth, with the root at depth one; at most 128.
    pub max_data_depth: usize,
    /// Total visited array/object entries across all loops in one render.
    pub max_iterations: usize,
    /// Total node visits, path-segment lookups, and helper calls in one render.
    pub max_steps: usize,
}

impl Default for TemplateLimits {
    /// Use one-MiB byte budgets, depth 64, 100,000 iterations and 1,000,000 steps.
    fn default() -> Self {
        Self {
            max_template_bytes: 1024 * 1024,
            max_data_bytes: 1024 * 1024,
            max_output_bytes: 1024 * 1024,
            max_nesting: 64,
            max_data_depth: 64,
            max_iterations: 100_000,
            max_steps: 1_000_000,
        }
    }
}

/// Literal delimiter pair and budgets captured by each compiled template.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TemplateOptions {
    /// Opening and closing delimiters; default `{{` and `}}`.
    pub delimiters: [String; 2],
    /// Engine-owned work/output limits; callers can raise byte/work budgets.
    pub limits: TemplateLimits,
}

impl Default for TemplateOptions {
    /// Use familiar brace delimiters and finite development defaults.
    fn default() -> Self {
        Self {
            delimiters: ["{{".into(), "}}".into()],
            limits: TemplateLimits::default(),
        }
    }
}

/// A configured compiler with an explicit registry of trusted inline helpers.
pub struct TemplateEngine {
    /// Validated options copied into each immutable compiled template.
    options: TemplateOptions,
    /// Function pointers only; compiling snapshots registrations, not mutable state.
    helpers: BTreeMap<String, TemplateHelper>,
}

impl Default for TemplateEngine {
    /// Construct the known-valid defaults without a fallible or panicking path.
    fn default() -> Self {
        Self {
            options: TemplateOptions::default(),
            helpers: helpers::builtins(),
        }
    }
}

impl TemplateEngine {
    /// Validate explicit delimiters and positive budgets before compilation.
    ///
    /// # Errors
    /// Returns [`TemplateErrorKind::InvalidOptions`] for empty/ambiguous delimiters,
    /// zero budgets, or depths outside `1..=128`.
    pub fn new(options: TemplateOptions) -> TemplateResult<Self> {
        validate_options(&options)?;
        Ok(Self {
            options,
            helpers: helpers::builtins(),
        })
    }

    /// Register one named inline helper without replacing an existing helper.
    ///
    /// # Errors
    /// Rejects invalid names or duplicate/built-in registrations. Existing compiled
    /// templates retain their original helper snapshot.
    pub fn register_helper(
        &mut self,
        name: &str,
        helper: TemplateHelper,
    ) -> TemplateResult<&mut Self> {
        // Built-in punctuation names are reserved even though custom names are identifiers.
        if self.helpers.contains_key(name) || matches!(name, "?" | "!" | "@") {
            return Err(TemplateError::new(
                TemplateErrorKind::DuplicateHelper,
                "helper is reserved or registered",
            ));
        }
        if !parse::identifier(name) {
            return Err(TemplateError::new(
                TemplateErrorKind::Syntax,
                "invalid helper name",
            ));
        }
        self.helpers.insert(name.into(), helper);
        Ok(self)
    }

    /// Compile an owned, reusable template, checking syntax in every branch.
    ///
    /// # Errors
    /// Reports malformed blocks/paths/arguments, unknown helpers, and source/depth limits.
    pub fn compile(&self, source: &str) -> TemplateResult<Template> {
        if source.len() > self.options.limits.max_template_bytes {
            return Err(TemplateError::new(
                TemplateErrorKind::Limit,
                "template byte limit",
            ));
        }
        let nodes = parse::compile(source, &self.options, &self.helpers)?;
        Ok(Template {
            source: source.into(),
            nodes,
            options: self.options.clone(),
            helpers: self.helpers.clone(),
        })
    }

    /// Compile and render once against a borrowed JSON object.
    ///
    /// # Errors
    /// Preserves compilation errors before data, lookup, type, helper, or render-limit errors.
    pub fn render(&self, source: &str, data: &Value) -> TemplateResult<String> {
        self.compile(source)?.render(data)
    }

    /// Compile and render string pairs as a fresh object, rejecting duplicate keys.
    ///
    /// # Errors
    /// Checks pair byte/depth budgets before cloning; otherwise follows [`Self::render`].
    pub fn render_pairs(&self, source: &str, pairs: &[(&str, &str)]) -> TemplateResult<String> {
        self.compile(source)?.render_pairs(pairs)
    }
}

/// An immutable parsed template with independent state for each render.
///
/// Literal source whitespace is preserved. Inserted strings/fragments/helper output
/// are not parsed again. Reuse this value across threads without shared render state.
pub struct Template {
    /// Owned UTF-8 text; text nodes borrow it by byte range while rendering.
    source: String,
    /// A depth-bounded tree; same-name blocks have distinct child lists.
    nodes: Vec<parse::Node>,
    /// Validated compile-time snapshot used by every render.
    options: TemplateOptions,
    /// Compiled callback snapshot, independent of later engine registration.
    helpers: BTreeMap<String, TemplateHelper>,
}

impl Template {
    /// Render an object with strict paths and types into a fresh checked buffer.
    ///
    /// # Errors
    /// Reports invalid/budget-exceeding data, visited missing/type errors, helper
    /// failures, and aggregate output/work limits. Returns no partial output.
    pub fn render(&self, data: &Value) -> TemplateResult<String> {
        data::validate(data, &self.options.limits)?;
        render::render(self, data)
    }

    /// Render string pairs after bounded validation and duplicate-key checks.
    ///
    /// # Errors
    /// Returns [`TemplateErrorKind::InvalidData`] for duplicate keys and otherwise
    /// preserves [`Self::render`] failures. Pair strings remain inert text.
    pub fn render_pairs(&self, pairs: &[(&str, &str)]) -> TemplateResult<String> {
        let data = data::pairs(pairs, &self.options.limits)?;
        self.render(&data)
    }
}

/// Compile and render once with default options and built-in helpers.
///
/// # Errors
/// Preserves [`TemplateEngine::render`] errors; there is no silent missing-value fallback.
pub fn render(source: &str, data: &Value) -> TemplateResult<String> {
    TemplateEngine::default().render(source, data)
}

/// Reject ambiguous delimiters and unusable budgets before accepting an engine.
fn validate_options(options: &TemplateOptions) -> TemplateResult<()> {
    let [open, close] = &options.delimiters;
    let limits = &options.limits;
    if [open, close]
        .iter()
        .any(|value| value.is_empty() || value.len() > 16 || value.chars().any(char::is_whitespace))
        || open.starts_with(close)
        || close.starts_with(open)
        || [
            limits.max_template_bytes,
            limits.max_data_bytes,
            limits.max_output_bytes,
            limits.max_iterations,
            limits.max_steps,
        ]
        .contains(&0)
        || !(1..=128).contains(&limits.max_nesting)
        || !(1..=128).contains(&limits.max_data_depth)
    {
        return Err(TemplateError::new(
            TemplateErrorKind::InvalidOptions,
            "invalid template options",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
