//! Checked syntax stages with owned source identity and typed environment provenance.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ast::{DataToken, SchemaToken, Statement};
use crate::{CompileInputs, ErrorCode, FinalSchema, IdeaError, ParseLimits, SchemaConfig};

/// Owned logical source identity and UTF-8 text. Construction performs no IO.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    /// Logical identity used in diagnostics; callers keep names distinct per source.
    name: String,
    /// Exact source text, never normalized in place.
    text: String,
    /// Byte starts of logical lines (CRLF advances once).
    lines: Vec<usize>,
    /// Byte starts of Unicode scalars, used for bounded-cost coordinate lookups.
    scalars: Vec<usize>,
}

impl SourceFile {
    /// Own a caller-named source; parsing checks budgets before copying syntax.
    ///
    /// # Examples
    /// ```
    /// use stackpress_idea_parser::SourceFile;
    /// let source = SourceFile::new("schema.idea", "model Profile { id String }");
    /// assert_eq!(source.name(), "schema.idea");
    /// ```
    pub fn new(name: impl Into<String>, text: impl Into<String>) -> Self {
        let text = text.into();
        let mut lines = vec![0];
        let mut scalars = Vec::new();
        // Build indexes once so many token/reference coordinates avoid rescanning long lines.
        let mut chars = text.char_indices().peekable();
        while let Some((offset, ch)) = chars.next() {
            scalars.push(offset);
            // A CRLF pair establishes one next-line start while retaining both byte offsets.
            if ch == '\r' {
                if chars.peek().is_some_and(|(_, ch)| *ch == '\n') {
                    if let Some((offset, _)) = chars.next() {
                        scalars.push(offset);
                        lines.push(offset + 1);
                    }
                } else {
                    lines.push(offset + 1);
                }
            } else if ch == '\n' {
                lines.push(offset + 1);
            }
        }
        Self {
            name: name.into(),
            text,
            lines,
            scalars,
        }
    }

    /// Return the caller's logical identity; callers use distinct names for distinct sources.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Borrow the unchanged UTF-8 source.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Validated, immutable syntax. Only the bounded parser can construct this owner.
#[derive(Debug, Clone, PartialEq)]
pub struct Ast {
    /// Owned text and coordinate indexes for every syntax range.
    pub(crate) source: SourceFile,
    /// Parser-created tree; no public mutable access or unchecked compilation exists.
    pub(crate) syntax: SchemaToken,
}

impl Ast {
    /// Borrow the source used to interpret every byte offset in this syntax.
    pub fn source(&self) -> &SourceFile {
        &self.source
    }

    /// Inspect syntax without obtaining mutable access to the checked owner.
    ///
    /// Environment nodes contain names, never environment values; identifiers remain typed.
    pub fn syntax(&self) -> &SchemaToken {
        &self.syntax
    }
}

/// Source identity and half-open UTF-8 byte coordinates, without a source excerpt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    /// Logical source identity supplied by the caller.
    pub file: String,
    /// Inclusive byte offset.
    pub start: usize,
    /// Exclusive byte offset.
    pub end: usize,
    /// One-based line, with CRLF treated as one break.
    pub line: usize,
    /// One-based Unicode scalar column.
    pub column: usize,
}

impl SourceSpan {
    /// Locate a range in a checked source, clamping synthetic EOF locations safely.
    pub(crate) fn new(source: &SourceFile, start: usize, end: usize) -> Self {
        // Synthetic ranges and EOF must still land on real UTF-8 boundaries.
        let mut start = start.min(source.text.len());
        let mut end = end.min(source.text.len()).max(start);
        while !source.text.is_char_boundary(start) {
            start -= 1;
        }
        while !source.text.is_char_boundary(end) {
            end += 1;
        }
        // Binary searches resolve line and scalar column without copying source prefixes.
        let line_index = source.lines.partition_point(|offset| *offset <= start) - 1;
        let line_start = source.lines[line_index];
        let column = source.scalars.partition_point(|offset| *offset < start)
            - source
                .scalars
                .partition_point(|offset| *offset < line_start)
            + 1;
        Self {
            file: source.name.clone(),
            start,
            end,
            line: line_index + 1,
            column,
        }
    }

    /// Attach stored coordinates to a safe diagnostic without loading source text.
    pub(crate) fn error(&self, code: ErrorCode, message: &str) -> IdeaError {
        let mut error = IdeaError::new(code, message).with_span(self.start, self.end);
        error.file = self.file.clone();
        error.line = self.line;
        error.column = self.column;
        error
    }
}

/// Declaration category used by structural reference sites during composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeclarationKind {
    /// Reusable fieldset.
    Type,
    /// Model fieldset.
    Model,
}

/// One step into an attribute argument, including nested arrays and objects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ValuePath {
    /// Argument or nested-array element.
    Index(usize),
    /// Nested-object member.
    Key(String),
}

/// An actual reference, structurally separate from placeholder-looking literal strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reference {
    /// Owning declaration category.
    pub kind: DeclarationKind,
    /// Owning declaration name, independent of its source position.
    pub declaration: String,
    /// Column name, or `None` for declaration metadata.
    pub column: Option<String>,
    /// Metadata namespace containing this argument.
    pub attribute: String,
    /// Argument index followed by nested array indices or object keys.
    pub path: Vec<ValuePath>,
    /// Name to resolve; the placeholder string itself is never used for discovery.
    pub target: String,
    /// Original source location, retained when composition moves the declaration.
    pub span: SourceSpan,
    /// Whether the same source explicitly declares this target later or at this declaration.
    /// Imported targets are not forward declarations in the originating source.
    pub forward: bool,
}

/// Compiled compatibility config plus lossless typed reference sites.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CompiledSchema {
    /// Ordered compiled data; unresolved sites use the legacy placeholder JSON view.
    pub config: SchemaConfig,
    /// The only sites eligible for reference substitution after composition.
    pub references: Vec<Reference>,
}

/// Parse pure syntax, retaining environment names without requiring their values.
///
/// # Errors
/// Returns syntax, duplicate or resource-limit diagnostics in the named source.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{parse_ast, SourceFile, ParseLimits};
/// let source = SourceFile::new("env.idea", "prop Config { url env(\"URL\") }");
/// let ast = parse_ast(&source, ParseLimits::default())?;
/// assert_eq!(ast.syntax().body.len(), 1);
/// # Ok::<(), stackpress_idea_parser::IdeaError>(())
/// ```
pub fn parse_ast(source: &SourceFile, limits: ParseLimits) -> Result<Ast, IdeaError> {
    // Reject text before constructing recursive syntax or cloning the source owner.
    crate::check_inputs(
        source.name(),
        source.text(),
        &CompileInputs {
            limits,
            ..Default::default()
        },
    )?;
    let inputs = CompileInputs {
        limits,
        ..Default::default()
    };
    let syntax = crate::parser::parse(source.text(), &inputs)
        .map_err(|error| error.locate(source.name(), source.text()))?;
    Ok(Ast {
        source: source.clone(),
        syntax,
    })
}

/// Compile checked syntax using only the supplied immutable environment snapshot.
///
/// Use [`compile_ast_with_references`] when a composed schema will be finalized later.
///
/// # Errors
/// Returns missing environment, duplicate declaration or resource-limit diagnostics.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{compile_ast, parse_ast, SourceFile, ParseLimits, CompileInputs};
/// let source = SourceFile::new("a.idea", "model A { id String }");
/// let ast = parse_ast(&source, ParseLimits::default())?;
/// assert!(compile_ast(&ast, &CompileInputs::default())?.model.contains_key("A"));
/// # Ok::<(), stackpress_idea_parser::IdeaError>(())
/// ```
pub fn compile_ast(ast: &Ast, inputs: &CompileInputs) -> Result<SchemaConfig, IdeaError> {
    compile_checked(ast, inputs, false)
}

/// Compile config and retain structurally addressed references for the composition owner.
///
/// # Errors
/// Returns the same diagnostics as [`compile_ast`].
///
/// # Examples
/// ```
/// use stackpress_idea_parser::*;
/// let ast = parse_ast(&SourceFile::new("a", "model A @x(Other) { id String }"),
///     ParseLimits::default())?;
/// let compiled = compile_ast_with_references(&ast, &CompileInputs::default())?;
/// assert_eq!(compiled.references[0].target, "Other");
/// # Ok::<(), IdeaError>(())
/// ```
pub fn compile_ast_with_references(
    ast: &Ast,
    inputs: &CompileInputs,
) -> Result<CompiledSchema, IdeaError> {
    let config = compile_ast(ast, inputs)?;
    let mut references = Vec::new();
    let declarations: IndexMap<_, _> = ast
        .syntax
        .body
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            // Imports do not establish reference targets; declaration order remains source order.
            if let Statement::Declaration(declaration) = item {
                Some((declaration.declarations[0].id.name.as_str(), index))
            } else {
                None
            }
        })
        .collect();
    let mut metadata_remaining = inputs.limits.max_bytes;
    for (index, item) in ast.syntax.body.iter().enumerate() {
        let Statement::Declaration(declaration) = item else {
            continue;
        };
        let kind = match declaration.kind {
            "type" => DeclarationKind::Type,
            "model" => DeclarationKind::Model,
            _ => continue,
        };
        let value = &declaration.declarations[0];
        // The parser creates a normalized metadata/columns object for both fieldset kinds.
        for property in &value.init.properties {
            let DataToken::Object(object) = &property.value else {
                continue;
            };
            if property.key.name == "attributes" {
                collect_attributes(
                    object,
                    None,
                    kind,
                    &value.id.name,
                    ast,
                    index,
                    &declarations,
                    &mut references,
                    &mut metadata_remaining,
                )?;
            } else if property.key.name == "columns" {
                for column in &object.properties {
                    let DataToken::Object(fields) = &column.value else {
                        continue;
                    };
                    for field in &fields.properties {
                        if field.key.name == "attributes" {
                            let DataToken::Object(attributes) = &field.value else {
                                continue;
                            };
                            collect_attributes(
                                attributes,
                                Some(&column.key.name),
                                kind,
                                &value.id.name,
                                ast,
                                index,
                                &declarations,
                                &mut references,
                                &mut metadata_remaining,
                            )?;
                        }
                    }
                }
            }
        }
    }
    Ok(CompiledSchema { config, references })
}

/// Gather references only inside a declaration or column's attribute arguments.
#[allow(clippy::too_many_arguments)]
fn collect_attributes(
    attributes: &crate::ObjectToken,
    column: Option<&str>,
    kind: DeclarationKind,
    declaration: &str,
    ast: &Ast,
    index: usize,
    declarations: &IndexMap<&str, usize>,
    references: &mut Vec<Reference>,
    remaining: &mut usize,
) -> Result<(), IdeaError> {
    for attribute in &attributes.properties {
        let mut pending = vec![(&attribute.value, Vec::new())];
        // Iterative traversal keeps nested paths explicit and avoids another recursive walker.
        while let Some((data, path)) = pending.pop() {
            match data {
                DataToken::Identifier(target) => {
                    // Charge repeated owner/source/path strings before creating the owned record.
                    let path_bytes: usize = path
                        .iter()
                        .map(|step| match step {
                            ValuePath::Key(key) => key.len() + 16,
                            ValuePath::Index(_) => 24,
                        })
                        .sum();
                    let bytes = declaration
                        .len()
                        .saturating_add(column.map_or(0, str::len))
                        .saturating_add(attribute.key.name.len())
                        .saturating_add(target.name.len())
                        .saturating_add(ast.source.name.len())
                        .saturating_add(path_bytes)
                        .saturating_add(256);
                    *remaining = remaining.checked_sub(bytes).ok_or_else(|| {
                        IdeaError::new(ErrorCode::Limit, "Reference metadata budget exceeded")
                            .with_span(target.start, target.end)
                            .locate(ast.source.name(), ast.source.text())
                    })?;
                    references.push(Reference {
                        kind,
                        declaration: declaration.into(),
                        column: column.map(Into::into),
                        attribute: attribute.key.name.clone(),
                        path,
                        target: target.name.clone(),
                        span: SourceSpan::new(&ast.source, target.start, target.end),
                        forward: declarations
                            .get(target.name.as_str())
                            .is_some_and(|order| *order >= index),
                    });
                }
                DataToken::Array(array) => {
                    // Reverse stack insertion restores source order on removal.
                    for (index, data) in array.elements.iter().enumerate().rev() {
                        let mut child = path.clone();
                        child.push(ValuePath::Index(index));
                        pending.push((data, child));
                    }
                }
                DataToken::Object(object) => {
                    for property in object.properties.iter().rev() {
                        let mut child = path.clone();
                        child.push(ValuePath::Key(property.key.name.clone()));
                        pending.push((&property.value, child));
                    }
                }
                DataToken::Literal(_) | DataToken::Environment(_) => {}
            }
        }
    }
    Ok(())
}

/// Finalize a single checked source, requiring imports to be composed explicitly first.
///
/// # Errors
/// Returns an error for outstanding imports, forward/unknown references or exhausted limits.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::*;
/// let ast = parse_ast(&SourceFile::new("a", "prop P { x 1 } model A @x(P) { id String }"),
///     ParseLimits::default())?;
/// assert!(finalize_ast(&ast, &CompileInputs::default())?.model.contains_key("A"));
/// # Ok::<(), IdeaError>(())
/// ```
pub fn finalize_ast(ast: &Ast, inputs: &CompileInputs) -> Result<FinalSchema, IdeaError> {
    if let Some(import) = ast.syntax.body.iter().find_map(|item| {
        if let Statement::Import(import) = item {
            Some(import)
        } else {
            None
        }
    }) {
        return Err(IdeaError::new(
            ErrorCode::ValidationError,
            "Compose imports before finalization",
        )
        .with_span(import.start, import.end)
        .locate(ast.source.name(), ast.source.text()));
    }
    let schema = compile_checked(ast, inputs, true)?;
    Ok(into_final(schema))
}

/// Move final sections without copying resolved data again.
pub(crate) fn into_final(schema: SchemaConfig) -> FinalSchema {
    FinalSchema {
        plugin: schema.plugin,
        r#enum: schema.r#enum,
        r#type: schema.r#type,
        model: schema.model,
    }
}

/// Resolve typed environment nodes on a bounded clone before using the extracted compiler.
pub(crate) fn compile_checked(
    ast: &Ast,
    inputs: &CompileInputs,
    final_values: bool,
) -> Result<SchemaConfig, IdeaError> {
    crate::check_inputs(ast.source.name(), ast.source.text(), inputs)?;
    let result = (|| {
        // Reparse when callers lower depth: a previously valid AST cannot bypass new limits.
        let mut syntax = crate::parser::parse(ast.source.text(), inputs)?;
        let mut remaining = inputs.limits.max_bytes;
        for item in &mut syntax.body {
            if let Statement::Declaration(declaration) = item {
                for value in &mut declaration.declarations {
                    resolve_object(&mut value.init, inputs, &mut remaining)?;
                }
            }
        }
        crate::finalize::compile_schema(&syntax, final_values, inputs.limits)
    })();
    result.map_err(|error| error.locate(ast.source.name(), ast.source.text()))
}

/// Resolve environment inputs recursively within an already depth-bounded syntax object.
fn resolve_object(
    object: &mut crate::ObjectToken,
    inputs: &CompileInputs,
    remaining: &mut usize,
) -> Result<(), IdeaError> {
    for property in &mut object.properties {
        resolve_data(&mut property.value, inputs, remaining)?;
    }
    Ok(())
}

/// Replace an environment name only after charging its materialized bytes.
fn resolve_data(
    data: &mut DataToken,
    inputs: &CompileInputs,
    remaining: &mut usize,
) -> Result<(), IdeaError> {
    match data {
        DataToken::Environment(env) => {
            let key = env.value.as_str().unwrap_or_default();
            let value = match inputs.environment.get(key) {
                Some(value) => value.as_str(),
                None if inputs.missing_env_is_empty => "",
                None => {
                    return Err(IdeaError::new(
                        ErrorCode::MissingEnvironment,
                        "Missing explicit environment input",
                    )
                    .with_span(env.start, env.end));
                }
            };
            // Repetition must spend the budget before allocation, even for one huge value.
            *remaining = remaining.checked_sub(value.len()).ok_or_else(|| {
                IdeaError::new(ErrorCode::Limit, "Environment expansion budget exceeded")
                    .with_span(env.start, env.end)
            })?;
            let mut literal = env.clone();
            literal.node_type = "Literal";
            literal.value = Value::String(value.into());
            *data = DataToken::Literal(literal);
        }
        DataToken::Object(object) => resolve_object(object, inputs, remaining)?,
        DataToken::Array(array) => {
            for value in &mut array.elements {
                resolve_data(value, inputs, remaining)?;
            }
        }
        DataToken::Literal(_) | DataToken::Identifier(_) => {}
    }
    Ok(())
}
