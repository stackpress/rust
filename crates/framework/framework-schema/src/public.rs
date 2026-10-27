//! Versioned browser descriptors and explicit projection policy, containing no executable imports.

use crate::{ValidationRule, ValueType};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Stable field identity, scoped by the source model and exact field name.
pub type FieldId = String;
/// Stable component key selected by the compiled application catalog.
pub type ComponentId = String;
/// Validated public props; only explicitly public definition arguments may appear.
pub type BoundedProps = BTreeMap<String, Value>;
/// Client rule metadata shares the checked canonical rule representation.
pub type PublicRule = ValidationRule;

/// The five independent placement roles; missing roles remain omitted.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    /// Create/update form control.
    Field,
    /// Query input control.
    Filter,
    /// Table/list value formatting.
    List,
    /// Compact input-derived presentation.
    Span,
    /// Detail value formatting.
    View,
}
/// Canonical role selection; fixed definition props have already overridden Idea props.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoleBinding {
    /// Placement in generated presentation.
    pub role: Role,
    /// Stable key, never a module path or executable expression.
    pub component: ComponentId,
    /// Approved, typed, bounded public props.
    pub props: BoundedProps,
    /// Catalog version used to interpret this binding.
    pub definition_revision: String,
    /// Whether the compiled catalog implements this component for actual UI generation.
    pub available: bool,
}
/// Explicit application field access; UI roles do not establish authorization.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldAccess {
    /// Include in explicitly projected output.
    pub readable: bool,
    /// Include in public input descriptors.
    pub writable: bool,
    /// Mask inputs and prohibit default echo/output projection.
    pub sensitive: bool,
    /// Opt in to publishing a safe literal default.
    pub publish_default: bool,
}
/// Public default never exposes provider arguments or an unapproved literal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PublicDefault {
    /// No approved public default.
    None,
    /// The host supplies the value; its provider identity is private.
    HostGenerated,
    /// Explicitly approved safe literal.
    Literal(Value),
}
/// Public type shape, including enum scalar choices and embedded/relation identities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PublicType {
    /// Checked portable type family.
    pub value: ValueType,
    /// Whether the nonnullable value is a list.
    pub multiple: bool,
    /// Finite scalar choices, when the value is an enumeration.
    pub choices: Vec<Value>,
}
/// Safe relation description. Lookup capabilities are explicit app IDs, never inferred URLs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiRelation {
    /// Exact target model identity.
    pub target: String,
    /// True for a to-many reference.
    pub many: bool,
    /// Exact owning key mapping, omitted for reverse references.
    pub mapping: Option<crate::Relation>,
    /// Optional explicitly registered app lookup operation.
    pub lookup: Option<String>,
}
/// One approved public field, detached from database and tool configuration.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiField {
    /// Stable model/field identity.
    pub id: FieldId,
    /// Exact wire member spelling.
    pub wire_name: String,
    /// Approved label, or the source field spelling.
    pub label: String,
    /// Portable checked type and collection shape.
    pub value_type: PublicType,
    /// True when the stored value cannot be null.
    pub required: bool,
    /// True when creation needs an input rather than a default/provider.
    pub input_required: bool,
    /// May be projected into public output.
    pub readable: bool,
    /// May be supplied by the public caller.
    pub writable: bool,
    /// Mask input and prohibit output/default echo.
    pub sensitive: bool,
    /// True when explicit null is legal.
    pub nullable: bool,
    /// Safe default indication.
    pub default: PublicDefault,
    /// Canonical public validation rules with stable messages and host/client distinction.
    pub validation: Vec<PublicRule>,
    /// Explicit approved scalar filter participation.
    pub filterable: bool,
    /// Literal text search participation.
    pub searchable: bool,
    /// Explicit approved scalar sort participation.
    pub sortable: bool,
    /// Ordered canonical role selections.
    pub roles: Vec<RoleBinding>,
    /// Explicit relation metadata without database access.
    pub relation: Option<UiRelation>,
}
/// An explicitly projected model descriptor, with finite query and operation capabilities.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiModel {
    /// Stable source model identity.
    pub id: String,
    /// Safe model label.
    pub label: String,
    /// Ordered approved fields only.
    pub fields: Vec<UiField>,
    /// Ordered public key member names, only when all key members are approved readable.
    pub key: Vec<String>,
    /// Explicit approved query field names.
    pub query_fields: Vec<String>,
    /// Default selected paths, narrowed through the app projection at every segment.
    pub default_selection: Vec<String>,
    /// Explicit app capability IDs; no route is automatically registered.
    pub operations: Vec<String>,
}
/// Browser-safe schema projection. It never contains private props or diagnostic source paths.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiSchema {
    /// Required descriptor revision.
    pub contract_revision: u32,
    /// Semantic compatibility signal, not an authorization token.
    pub schema_fingerprint: String,
    /// Explicitly approved model projections.
    pub models: Vec<UiModel>,
    /// Approved embedded shapes, projected independently of their containing models.
    pub fieldsets: Vec<UiModel>,
}
/// Owned descriptive metadata remains private until an app projection approves it.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Documentation {
    /// Optional field/model label.
    pub label: Option<String>,
    /// Optional plain description.
    pub description: Option<String>,
    /// Bounded literal examples.
    pub examples: Vec<Value>,
    /// Data-only display template.
    pub display: Option<String>,
    /// Data-only icon key.
    pub icon: Option<String>,
    /// Explicit plural labels; no linguistic inference.
    pub labels: Vec<String>,
}
/// Source metadata is retained privately, including inert foreign namespaces.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Attribute {
    /// Exact registered or foreign attribute spelling.
    pub name: String,
    /// True/false flag, or ordered method arguments.
    pub value: Value,
}
/// Deterministic names with no hidden pluralization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NameSet {
    /// Exact source/wire identity.
    pub source: String,
    /// Checked Rust type/member identity.
    pub rust: String,
    /// Portable storage/module identity.
    pub sql: String,
}
