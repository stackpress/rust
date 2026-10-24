//! Extracted ordered compiled config; metadata meaning belongs to later framework crates.

use indexmap::IndexMap;

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
/// A flag or whitespace-separated method argument list.
pub enum AttributeValue {
    /// Flag state.
    Boolean(bool),
    /// Method arguments in source order.
    Array(Vec<Value>),
}

/// Extracted EnumConfig syntax/config representation.
pub type EnumConfig = IndexMap<String, Value>;
/// Extracted PluginConfig syntax/config representation.
pub type PluginConfig = IndexMap<String, Value>;
/// Extracted PropConfig syntax/config representation.
pub type PropConfig = IndexMap<String, Value>;

#[derive(Debug, Clone, PartialEq, Serialize)]
/// One ordered column with syntax-level type modifiers.
pub struct ColumnConfig {
    /// Source declaration or member name.
    pub name: String,
    #[serde(rename = "type")]
    /// Base type name after removing validated modifiers.
    pub column_type: String,
    /// Open metadata retained without framework interpretation.
    pub attributes: IndexMap<String, AttributeValue>,
    /// Whether null/absence is disallowed by the type modifier.
    pub required: bool,
    /// Whether this is a single list of the base type.
    pub multiple: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
/// A reusable fieldset or model with ordered columns.
pub struct TypeConfig {
    /// Source declaration or member name.
    pub name: String,
    /// Whether imports extend this declaration; false permits child replacement under G19.
    pub mutable: bool,
    /// Open metadata retained without framework interpretation.
    pub attributes: IndexMap<String, AttributeValue>,
    /// Columns in declared order.
    pub columns: Vec<ColumnConfig>,
}

/// Extracted ModelConfig syntax/config representation.
pub type ModelConfig = TypeConfig;
