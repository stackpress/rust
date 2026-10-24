//! Extracted private syntax representation with bounded parser ownership.

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted SchemaToken syntax/config representation.
pub struct SchemaToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Extracted declaration or property kind.
    pub kind: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Declarations/imports in source order.
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
/// Extracted Statement syntax/config representation.
pub enum Statement {
    /// One plugin, prop, enum, type or model declaration.
    Declaration(DeclarationToken),
    /// One unresolved source import.
    Import(ImportToken),
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted ImportToken syntax/config representation.
pub struct ImportToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Reserved empty import-specifier list.
    pub specifiers: Vec<()>,
    /// Literal import specifier.
    pub source: LiteralToken,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted DeclarationToken syntax/config representation.
pub struct DeclarationToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Extracted declaration or property kind.
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Whether imports extend this declaration; false permits child replacement under G19.
    pub mutable: Option<bool>,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Validated declaration payload.
    pub declarations: Vec<DeclaratorToken>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted DeclaratorToken syntax/config representation.
pub struct DeclaratorToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Named declaration identity.
    pub id: IdentifierToken,
    /// Declaration object structure.
    pub init: ObjectToken,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted IdentifierToken syntax/config representation.
pub struct IdentifierToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Source declaration or member name.
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted ObjectToken syntax/config representation.
pub struct ObjectToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Object entries in source order.
    pub properties: Vec<PropertyToken>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted PropertyToken syntax/config representation.
pub struct PropertyToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Extracted declaration or property kind.
    pub kind: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Source member key.
    pub key: IdentifierToken,
    /// Typed source data or normalized value.
    pub value: DataToken,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Extracted property method flag.
    pub method: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Extracted shorthand flag.
    pub shorthand: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Extracted computed-key flag.
    pub computed: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted ArrayToken syntax/config representation.
pub struct ArrayToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Array elements in source order.
    pub elements: Vec<DataToken>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// Extracted LiteralToken syntax/config representation.
pub struct LiteralToken {
    #[serde(rename = "type")]
    /// Extracted syntax-node discriminator.
    pub node_type: &'static str,
    /// Inclusive source byte offset.
    pub start: usize,
    /// Exclusive source byte offset.
    pub end: usize,
    /// Typed source data or normalized value.
    pub value: serde_json::Value,
    /// Literal representation retained for syntax provenance.
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
/// Extracted DataToken syntax/config representation.
pub enum DataToken {
    /// An actual reference, distinct from literal strings.
    Identifier(IdentifierToken),
    /// An ordinary scalar literal.
    Literal(LiteralToken),
    /// An unevaluated environment key with its original source range.
    Environment(LiteralToken),
    /// Ordered object members.
    Object(ObjectToken),
    /// Ordered nested values.
    Array(ArrayToken),
}

impl IdentifierToken {
    /// Construct an internal node; checked compilation accepts only parser-owned Ast.
    pub(crate) fn new(start: usize, end: usize, name: impl Into<String>) -> Self {
        Self {
            node_type: "Identifier",
            start,
            end,
            name: name.into(),
        }
    }
}

impl LiteralToken {
    /// Construct an internal node; checked compilation accepts only parser-owned Ast.
    pub(crate) fn new(
        start: usize,
        end: usize,
        value: serde_json::Value,
        raw: impl Into<String>,
    ) -> Self {
        Self {
            node_type: "Literal",
            start,
            end,
            value,
            raw: raw.into(),
        }
    }
}

impl PropertyToken {
    /// Construct an internal node; checked compilation accepts only parser-owned Ast.
    pub(crate) fn new(start: usize, end: usize, key: IdentifierToken, value: DataToken) -> Self {
        Self {
            node_type: "Property",
            kind: "init",
            start,
            end,
            key,
            value,
            method: Some(false),
            shorthand: Some(false),
            computed: Some(false),
        }
    }
}

impl ObjectToken {
    /// Construct an internal node; checked compilation accepts only parser-owned Ast.
    pub(crate) fn new(start: usize, end: usize, properties: Vec<PropertyToken>) -> Self {
        Self {
            node_type: "ObjectExpression",
            start,
            end,
            properties,
        }
    }
}

impl ArrayToken {
    /// Construct an internal node; checked compilation accepts only parser-owned Ast.
    pub(crate) fn new(start: usize, end: usize, elements: Vec<DataToken>) -> Self {
        Self {
            node_type: "ArrayExpression",
            start,
            end,
            elements,
        }
    }
}
