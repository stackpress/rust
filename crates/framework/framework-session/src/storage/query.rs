//! Lossless legacy String-q carrier over the shared source query representation.
use crate::columns::{Record, Value};
use serde::{Deserialize, Serialize};
/// Source select/filter envelope. Empty and missing maps have the same source defaults.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct StoreQuery {
    /// Source contains-search; only Profile has declared searchable columns.
    pub q: Option<String>,
    /// Requested source paths; defaults to all base columns.
    pub columns: Vec<String>,
    /// One equal value per field (G53); active=-1 removes the implicit active predicate.
    pub eq: Record,
    /// Not-equal values; arrays retain the source's indexed-path flattening.
    pub ne: Record,
    /// Greater-or-equal values.
    pub ge: Record,
    /// Less-or-equal values.
    pub le: Record,
    /// Contains-search values; % and _ retain SQL wildcard semantics.
    pub like: Record,
    /// JSON-array membership; every requested value must be present (G53).
    pub has: Record,
    /// Source currently compiles this as membership too, preserved deliberately.
    pub hasnt: Record,
    /// Source ASC/DESC map, case insensitive; invalid directions are skipped.
    pub sort: Record,
    /// Source offset.
    pub skip: u64,
    /// Source default 50; zero means unlimited.
    pub take: u64,
}
impl Default for StoreQuery {
    /// Preserve original generated select defaults rather than the framework public-query cap.
    fn default() -> Self {
        Self {
            q: None,
            columns: vec!["*".into()],
            eq: Record::new(),
            ne: Record::new(),
            ge: Record::new(),
            le: Record::new(),
            like: Record::new(),
            has: Record::new(),
            hasnt: Record::new(),
            sort: Record::new(),
            skip: 0,
            take: 50,
        }
    }
}
impl StoreQuery {
    /// Construct a focused equal-field query, retaining all ordinary defaults.
    pub fn eq(name: impl Into<String>, value: impl Into<Value>) -> Self {
        let mut query = Self::default();
        query.eq.insert(name.into(), value.into());
        query
    }
    /// Explicitly include inactive rows, as the original eq.active=-1 sentinel does.
    pub fn all(mut self) -> Self {
        self.eq.insert("active".into(), (-1).into());
        self
    }
}

impl StoreQuery {
    /// Convert an already decoded legacy query; arbitrary q values remain a native-source API.
    pub(crate) fn native(&self) -> stackpress_framework_sql::source::StoreQuery {
        stackpress_framework_sql::source::StoreQuery {
            q: self.q.clone().map(Value::from),
            columns: self.columns.clone(),
            eq: self.eq.clone(),
            ne: self.ne.clone(),
            ge: self.ge.clone(),
            le: self.le.clone(),
            like: self.like.clone(),
            has: self.has.clone(),
            hasnt: self.hasnt.clone(),
            sort: self.sort.clone(),
            skip: self.skip,
            take: self.take,
        }
    }
}
/// Existing source selector carrier, retained while resolution becomes generated.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StoreSelector {
    /// Original expanded expression.
    pub expression: String,
    /// Native output alias.
    pub alias: String,
    /// Source field name.
    pub field: String,
    /// SQL column name.
    pub column: String,
    /// Source table alias.
    pub table: String,
    /// Nested source record path.
    pub path: Vec<String>,
    /// JSON predicate path.
    pub json: Vec<String>,
    /// Retained compatibility flag for Auth's Profile relation.
    pub profile: bool,
}
impl StoreSelector {
    pub(super) fn from_native(
        value: stackpress_framework_sql::source::StoreSelector,
        base: &str,
    ) -> Self {
        Self {
            profile: base == "Auth" && value.model == "Profile",
            expression: value.expression,
            alias: value.alias,
            field: value.field,
            column: value.column,
            table: value.table,
            path: value.path,
            json: value.json,
        }
    }
}
