//! Read-only lexical tokens produced by the parser's own scanner rules.

use crate::{IdeaError, ParseLimits, SourceFile, SourceSpan};
use serde::Serialize;

/// Lexical category; declaration and metadata meanings belong to later stages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TokenKind {
    /// Identifier, declaration keyword or complete type modifier spelling.
    Word,
    /// Raw Idea string, exact number, boolean or null.
    Literal,
    /// Environment expression retaining a name without reading its value.
    Environment,
    /// Validated lowercase metadata namespace.
    Attribute,
    /// Bracket, brace, parenthesis or immutable marker.
    Delimiter,
}

/// Owned token payload; environment tokens use a literal string containing the key name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum TokenValue {
    /// Identifier or metadata namespace text.
    Text(String),
    /// Scalar value or environment key name.
    Literal(serde_json::Value),
    /// One grammar delimiter.
    Symbol(char),
}

/// Lexical token with source coordinates and the exact original spelling.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Token {
    /// Scanner category.
    pub kind: TokenKind,
    /// Typed lexical payload.
    pub value: TokenValue,
    /// Half-open range within the supplied named source.
    pub span: SourceSpan,
    /// Exact source spelling; no environment expansion occurs.
    pub raw: String,
}

/// Scan bounded tokens without evaluating env or enforcing declaration grammar.
///
/// Lexing uses the parser's scanners for raw strings, exact numbers, names, env and comments.
/// A lexical success does not promise balanced delimiters or a valid schema.
///
/// # Errors
/// Returns malformed lexical input, unterminated literals/comments or resource diagnostics.
///
/// # Examples
/// ```
/// use stackpress_idea_parser::{tokenize, SourceFile, ParseLimits, TokenKind};
/// let tokens = tokenize(&SourceFile::new("value.idea", "env(\"KEY\")"), ParseLimits::default())?;
/// assert_eq!(tokens[0].kind, TokenKind::Environment);
/// # Ok::<(), stackpress_idea_parser::IdeaError>(())
/// ```
pub fn tokenize(source: &SourceFile, limits: ParseLimits) -> Result<Vec<Token>, IdeaError> {
    crate::parser::tokenize_source(source, limits)
}
