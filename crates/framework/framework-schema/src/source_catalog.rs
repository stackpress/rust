//! Complete generated native catalog metadata, separate from public UI projections.
use crate::{Enumeration, Model};
use serde::{Deserialize, Serialize};

/// Private source graph retained by a generated native model package.
///
/// This is descriptive metadata, not an executable SQL store or a browser publication.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SourceCatalog {
    /// Original SQL table spellings, independent of Rust keyword/module escaping.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tables: Vec<SourceTable>,
    /// Exact source type spellings, retaining String/Text and Number/Float SQL distinctions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub types: Vec<SourceType>,
    /// Source fieldsets, stored inside JSON columns rather than hidden tables.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fieldsets: Vec<Model>,
    /// Exact enum names/choices used by source column shapes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enums: Vec<Enumeration>,
    /// Models in composed source order, with every attribute and presentation binding.
    pub models: Vec<Model>,
    /// Declared external references; none is synthesized into a model/table.
    pub external_models: Vec<String>,
    /// Owning and reverse relation mappings derived from the same model graph.
    pub relations: Vec<SourceRelation>,
    /// Default selectors and their scalar path expansion, without executing a query.
    pub queries: Vec<SourceQuery>,
}

/// A source relation edge; absent mappings identify a declared external target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceRelation {
    /// Declaring model.
    pub model: String,
    /// Exact source relation field name.
    pub field: String,
    /// Referenced model name.
    pub target: String,
    /// Whether the source field is a collection.
    pub multiple: bool,
    /// Local storage member, including a reverse edge's referenced key.
    pub local: Option<String>,
    /// Foreign storage member, including a reverse edge's owning key.
    pub foreign: Option<String>,
    /// Sole inferred inverse relation field, when the target is in this graph.
    pub inverse: Option<String>,
    /// True for an explicitly declared external model, never a hidden table.
    pub external: bool,
}

/// One model's source query selection; execution and pagination belong to SQL emitters.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceQuery {
    /// Declaring model.
    pub model: String,
    /// Exact explicit query paths, or the source's implicit `*` default.
    pub selectors: Vec<String>,
    /// Scalar paths in selector/declaration order, with wildcards expanded once per path.
    pub paths: Vec<String>,
}

/// Exact declared source type independently of its shared native value family.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceType {
    /// Model or fieldset declaration name.
    pub model: String,
    /// Exact field name.
    pub field: String,
    /// Original scalar/enum/fieldset/model spelling, without the list/nullable suffix.
    pub name: String,
}

/// SQL identity produced by the original schema snakerize helper, separate from Rust modules.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceTable {
    /// Exact source model name.
    pub model: String,
    /// Original SQL table spelling.
    pub name: String,
}
