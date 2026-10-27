//! Owned, serializable schema contracts, independent of native hosts and database drivers.
//!
//! The optional `codegen` feature interprets finalized Idea config and emits Rust before
//! application compilation. Default features expose data only. M0 normalizes a bounded
//! scalar/enum/fieldset/relation subset; unsupported selected rules fail explicitly.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "codegen")]
pub mod codegen;

/// Version of the M0 descriptor and generated-code contract; not a stable release promise.
pub const CONTRACT_REVISION: u32 = 1;

/// Fallible framework schema operation with safe context and a typed private source.
pub type SchemaResult<T> = Result<T, SchemaError>;

/// Stable classification independent of diagnostic wording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaErrorKind {
    /// A direct config violates shape, naming or type rules.
    Invalid,
    /// The selected M0 consumer has not implemented the requested semantics.
    Unsupported,
    /// A finite declaration/value/output budget was exceeded.
    Limit,
    /// An underlying codec or source operation failed.
    Codec,
}

/// Safe schema diagnostic; input values and private causes never enter Display.
#[derive(Debug)]
pub struct SchemaError {
    /// Semantic category chosen by the failing boundary.
    kind: SchemaErrorKind,
    /// Independently authored context, without rejected data.
    context: &'static str,
    /// Optional original typed codec/source cause.
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl SchemaError {
    /// Describe one schema boundary without disclosing input values.
    pub fn new(kind: SchemaErrorKind, context: &'static str) -> Self {
        Self {
            kind,
            context,
            cause: None,
        }
    }

    /// Retain the original concrete cause beneath distinct safe context.
    pub fn with_cause(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Return the category without parsing private driver or codec text.
    pub fn kind(&self) -> SchemaErrorKind {
        self.kind
    }
}

impl fmt::Display for SchemaError {
    /// Display exactly one explanation, leaving source traversal explicit.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.context)
    }
}

impl Error for SchemaError {
    /// Preserve the concrete cause for private downcast/traversal.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as _)
    }
}

/// Immutable normalized schema; relations use names rather than recursive object pointers.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Schema {
    /// Ordered storage model declarations, shared by both compiled transforms.
    pub(crate) models: Vec<Model>,
    /// Ordered embedded value declarations, validated as an acyclic graph.
    pub(crate) fieldsets: Vec<Model>,
    /// Ordered scalar enum declarations.
    pub(crate) enums: Vec<Enumeration>,
}

impl Schema {
    /// Borrow ordered model descriptors without changing another consumer's normalization.
    pub fn models(&self) -> &[Model] {
        &self.models
    }
    /// Borrow bounded embedded value descriptors.
    pub fn fieldsets(&self) -> &[Model] {
        &self.fieldsets
    }
    /// Borrow checked scalar enum definitions.
    pub fn enums(&self) -> &[Enumeration] {
        &self.enums
    }
}

/// One checked model or embedded value descriptor.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Model {
    /// Exact Idea spelling and wire identity.
    pub name: String,
    /// Portable Rust type name.
    pub rust_name: String,
    /// Portable module and SQL table name.
    pub module: String,
    /// Finite ordered fields, including virtual relations.
    pub fields: Vec<Field>,
}

/// One field, with missing/default/null decisions retained separately.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    /// Exact external spelling.
    pub name: String,
    /// Portable Rust member identifier, with a deterministic keyword suffix.
    pub rust_name: String,
    /// Portable SQL column identity.
    pub column: String,
    /// Checked value/relation kind.
    pub value_type: ValueType,
    /// Whether explicit null is forbidden.
    pub required: bool,
    /// A bounded JSON list when not a virtual model relation.
    pub multiple: bool,
    /// Explicit scalar identity membership.
    pub key: bool,
    /// Literal default, preserving false, zero, empty string/list and JSON null.
    pub default: Option<serde_json::Value>,
    /// The M0 nonempty text assertion.
    pub nonempty: bool,
    /// Boolean soft-removal metadata; operations are not implemented by M0.
    pub active: bool,
    /// Explicit owning relation mapping; reverse collections have no mapping.
    pub relation: Option<Relation>,
}

/// Scalar/value/relation families, never guessed from placeholder-looking strings.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "name")]
pub enum ValueType {
    /// Unchanged UTF-8 text.
    String,
    /// Exact signed 64-bit integer.
    Integer,
    /// Boolean, without truthiness coercion.
    Boolean,
    /// Finite floating-point value.
    Number,
    /// Bounded JSON object.
    Object,
    /// Named scalar enum.
    Enum(String),
    /// Named acyclic embedded value, stored as one JSON column.
    Fieldset(String),
    /// Virtual model relation, never emitted as a stored column.
    Model(String),
}

/// One explicit scalar foreign-key relationship.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    /// Local stored FK field in the owning model.
    pub local: String,
    /// Unique target field.
    pub foreign: String,
    /// Explicit reverse relation name, when supplied.
    pub inverse: Option<String>,
}

/// Scalar enum descriptor. M0 supports string-valued enums only and refuses other families.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enumeration {
    /// Exact Idea identity.
    pub name: String,
    /// Portable Rust type name.
    pub rust_name: String,
    /// Portable module name.
    pub module: String,
    /// Source-ordered variant/wire-value pairs.
    pub variants: Vec<(String, String)>,
}

/// Missing and null remain separate at the transport boundary.
#[derive(Clone, Debug, PartialEq)]
pub enum Input<T> {
    /// No value was supplied, so preparation may consult a declared default.
    Missing,
    /// An explicit JSON null was supplied.
    Null,
    /// An explicit typed value was supplied.
    Value(T),
}

/// A patch never turns an omitted field into a null write.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum Patch<T> {
    /// Preserve the stored value.
    #[default]
    Unchanged,
    /// Write an explicit typed value.
    Set(T),
    /// Write SQL NULL, legal only for nullable fields.
    Null,
}

impl<T> Input<T> {
    /// Let generated struct serializers omit missing fields without collapsing explicit null.
    pub fn is_missing(&self) -> bool {
        matches!(self, Self::Missing)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Input<T> {
    /// An absent struct field uses Default; a supplied null reaches this decoder distinctly.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Ok(match Option::<T>::deserialize(decoder)? {
            Some(value) => Self::Value(value),
            None => Self::Null,
        })
    }
}

impl<T: Serialize> Serialize for Input<T> {
    /// Serialize supplied values/null; generated fields skip Missing before reaching here.
    fn serialize<S: serde::Serializer>(&self, encoder: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Value(value) => value.serialize(encoder),
            Self::Null => encoder.serialize_none(),
            Self::Missing => Err(serde::ser::Error::custom("missing input must be omitted")),
        }
    }
}

impl<T> Default for Input<T> {
    /// Omission is independent of whether the eventual value type has a default.
    fn default() -> Self {
        Self::Missing
    }
}
