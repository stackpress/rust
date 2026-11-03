//! Complete generated native catalog metadata, separate from public UI projections.
use crate::Model;
use serde::{Deserialize, Serialize};

/// Private source graph retained by a generated native model package.
///
/// This is descriptive metadata, not an executable SQL store or a browser publication.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceCatalog {
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
