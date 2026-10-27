//! Owned, serializable schema contracts, independent of native hosts and database drivers.
//!
//! The optional `codegen` feature interprets finalized Idea config and emits Rust before
//! application compilation. Default features expose portable values, validation and public
//! descriptors only. Selected unsupported storage/security/validation semantics fail explicitly.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[cfg(feature = "codegen")]
pub mod codegen;

/// Generated schema contract, including explicit optional string storage length (G38).
pub const CONTRACT_REVISION: u32 = 3;

mod query;
pub use query::{NullOrder, Predicate, Sort, SortDirection, check_include, query_budget};
mod input;
pub use input::InputSchema;
mod record;
pub use record::{RecordSchema, value_matches};
mod values;
pub use values::{
    DateValue, DatetimeValue, FiniteNumber, JsonObject, PreparationBudget, TimeValue,
    browser_integer, check_browser_json, check_json, to_bounded_value,
};
mod validation;
pub use validation::{
    FieldIssue, ValidationErrors, ValidationRule, ValueContext, ValueProviders, ValueRule,
    check_rule,
};
mod public;
pub use public::{
    Attribute, BoundedProps, ComponentId, Documentation, FieldAccess, FieldId, NameSet,
    PublicDefault, PublicRule, PublicType, Role, RoleBinding, UiField, UiModel, UiRelation,
    UiSchema,
};

/// Fallible framework schema operation with safe context and a typed private source.
pub type SchemaResult<T> = Result<T, SchemaError>;

/// Stable classification independent of diagnostic wording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SchemaErrorKind {
    /// A direct config violates shape, naming or type rules.
    Invalid,
    /// The selected consumer has not implemented the requested semantics.
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
    /// Logical location independent of private rejected values.
    path: Vec<String>,
    /// Optional tool-only original source coordinate.
    #[cfg(feature = "codegen")]
    location: Option<Box<codegen::DiagnosticLocation>>,
}

impl SchemaError {
    /// Describe one schema boundary without disclosing input values.
    pub fn new(kind: SchemaErrorKind, context: &'static str) -> Self {
        Self {
            kind,
            context,
            cause: None,
            path: Vec::new(),
            #[cfg(feature = "codegen")]
            location: None,
        }
    }

    /// Retain the original concrete cause beneath distinct safe context.
    pub fn with_cause(mut self, cause: impl Error + Send + Sync + 'static) -> Self {
        self.cause = Some(Box::new(cause));
        self
    }

    /// Attach a logical schema path at the failing semantic boundary.
    pub fn at(mut self, path: Vec<String>) -> Self {
        if self.path.is_empty() {
            self.path = path;
        }
        self
    }
    /// Borrow the logical field/attribute path.
    pub fn path(&self) -> &[String] {
        &self.path
    }
    /// Borrow an original source location, available only to the build host.
    #[cfg(feature = "codegen")]
    pub fn location(&self) -> Option<&codegen::DiagnosticLocation> {
        self.location.as_deref()
    }
    /// Resolve the nearest winning source coordinate after semantic normalization fails.
    #[cfg(feature = "codegen")]
    pub fn with_diagnostics(mut self, map: &codegen::DiagnosticMap) -> Self {
        self.location = map.resolve(&self.path).cloned().map(Box::new);
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
    /// Owned descriptive metadata, published only through explicit app policy.
    pub documentation: Documentation,
    /// Explicit query paths checked against fields/relations.
    pub query: Vec<String>,
    /// Privately retained exact source metadata.
    pub attributes: Vec<Attribute>,
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
    /// Explicit maximum Unicode scalar count for a stored String/Text model field.
    ///
    /// Normalized from `@store.length(n)` on stored, non-list String/Text model fields.
    /// Values are positive and at most i32::MAX, checked again during deserialization.
    /// Absence remains TEXT; a validation upper bound alone never selects VARCHAR.
    /// Counts are Unicode scalars, including combining marks, NUL and trailing spaces.
    /// SQLite records the declaration but does not enforce it; generated validation and
    /// native store guards do. Private bounds are omitted from browser record snapshots.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "storage_length"
    )]
    pub storage_length: Option<u32>,
    /// Whether explicit null is forbidden.
    pub required: bool,
    /// A bounded JSON list when not a virtual model relation.
    pub multiple: bool,
    /// Explicit scalar identity membership.
    pub key: bool,
    /// Literal default, preserving false, zero, empty string/list and JSON null.
    pub default: Option<serde_json::Value>,
    /// Canonical nonempty assertion, also retained as focused compatibility metadata.
    pub nonempty: bool,
    /// Boolean soft-removal metadata; mutation behavior belongs to explicit store operations.
    pub active: bool,
    /// Explicit owning relation mapping; reverse collections have no mapping.
    pub relation: Option<Relation>,
    /// Storage uniqueness is authoritative, never a preflight-only promise.
    pub unique: bool,
    /// Explicit query search participation.
    pub searchable: bool,
    /// Explicit query ordering participation.
    pub sortable: bool,
    /// Refresh from the explicit clock on writes (execution belongs to M2).
    pub timestamp: bool,
    /// Reject caller writes and obtain a value from the registered default.
    pub generated: bool,
    /// Whether an omitted create input lacks any declared host/literal source.
    pub input_required: bool,
    /// Prepared host default, distinct from literal null/false/zero/empty.
    pub value_rule: Option<ValueRule>,
    /// Checked canonical assertions.
    pub rules: Vec<ValidationRule>,
    /// Owned descriptions and examples.
    pub documentation: Documentation,
    /// Ordered canonical component roles.
    pub roles: Vec<RoleBinding>,
    /// Explicit app-owned access/projection policy; defaults deny publication.
    pub access: FieldAccess,
    /// Private recognized and foreign attributes.
    pub attributes: Vec<Attribute>,
}

/// Recheck descriptor input before a storage renderer can narrow the integer type.
fn storage_length<'de, D: serde::Deserializer<'de>>(decoder: D) -> Result<Option<u32>, D::Error> {
    let value = Option::<u32>::deserialize(decoder)?;
    if value.is_some_and(|length| length == 0 || i32::try_from(length).is_err()) {
        return Err(serde::de::Error::custom("invalid declared storage length"));
    }
    Ok(value)
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
    /// Checked YYYY-MM-DD calendar date.
    Date,
    /// Checked local wall time, without timezone inference.
    Time,
    /// Checked UTC instant, normalized to RFC3339 Z.
    Datetime,
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

/// Homogeneous String, i64 or Boolean enumeration with checked unique scalar values.
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
    pub variants: Vec<(String, serde_json::Value)>,
}

/// Missing and null remain separate at the transport boundary.
///
/// # Examples
///
/// ```
/// use stackpress_framework_schema::Input;
/// #[derive(serde::Deserialize)]
/// struct CreateNote {
///     #[serde(default)]
///     note: Input<String>,
/// }
/// let omitted: CreateNote = serde_json::from_str("{}")?;
/// let cleared: CreateNote = serde_json::from_str(r#"{"note":null}"#)?;
/// assert_eq!(omitted.note, Input::Missing);
/// assert_eq!(cleared.note, Input::Null);
/// # Ok::<(), serde_json::Error>(())
/// ```
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
///
/// # Examples
///
/// ```
/// use stackpress_framework_schema::Patch;
/// #[derive(serde::Serialize, serde::Deserialize)]
/// struct UpdateNote {
///     #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
///     note: Patch<String>,
/// }
/// let omitted: UpdateNote = serde_json::from_str("{}")?;
/// assert_eq!(omitted.note, Patch::Unchanged);
/// assert_eq!(serde_json::to_value(&omitted)?, serde_json::json!({}));
/// let cleared = UpdateNote { note: Patch::Null };
/// assert_eq!(serde_json::to_value(&cleared)?, serde_json::json!({"note": null}));
/// # Ok::<(), serde_json::Error>(())
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
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

impl<T> Patch<T> {
    /// Omitted patch fields preserve the existing stored value.
    pub fn is_unchanged(&self) -> bool {
        matches!(self, Self::Unchanged)
    }
}
impl<'de, T: Deserialize<'de>> Deserialize<'de> for Patch<T> {
    /// Supplied null is explicit; absent fields use Unchanged through Default.
    fn deserialize<D: serde::Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        Ok(match Option::<T>::deserialize(decoder)? {
            Some(value) => Self::Set(value),
            None => Self::Null,
        })
    }
}
impl<T: Serialize> Serialize for Patch<T> {
    /// Struct fields must omit Unchanged instead of serializing it as null.
    fn serialize<S: serde::Serializer>(&self, encoder: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Set(value) => value.serialize(encoder),
            Self::Null => encoder.serialize_none(),
            Self::Unchanged => Err(serde::ser::Error::custom("unchanged patch must be omitted")),
        }
    }
}

impl Model {
    /// Borrow the normalized source/Rust/SQL identity as one owned focused view.
    pub fn names(&self) -> NameSet {
        NameSet {
            source: self.name.clone(),
            rust: self.rust_name.clone(),
            sql: self.module.clone(),
        }
    }
}
impl Field {
    /// Borrow exact wire spelling alongside the checked Rust member and SQL column identities.
    pub fn names(&self) -> NameSet {
        NameSet {
            source: self.name.clone(),
            rust: self.rust_name.clone(),
            sql: self.column.clone(),
        }
    }
}
