//! Checked read plans and bounded eager relation loading for generated native stores.
//!
//! Generated selectors supply typed values; this module checks their immutable catalog
//! before rendering. Multi-statement results require the exact caller transaction. There
//! is no lazy fetch, ambient pool, hidden retry, partial collection or inferred join table.

use crate::{
    DataError, DataErrorKind, DataResult, DataSession, StorageCodec, StorageColumn, codec_error,
    encode_value,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework_schema::Predicate;
use stackpress_sql::{Expr, Identifier, Row, Select};

mod query;
mod relations;
pub use query::select;
#[doc(hidden)]
pub use relations::preflight_atomic_search;
pub use relations::{RelationLimits, search};

/// A matched base page. Total counts identities before offset/limit, independently of includes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Page<T> {
    /// Deterministically ordered, fully loaded rows.
    pub rows: Vec<T>,
    /// Requested exact base count, retaining zero distinctly from an unrequested count.
    pub total: Option<u64>,
    /// Applied bounded offset.
    pub skip: u64,
    /// Applied bounded page size.
    pub take: u16,
}
/// Eager include state; unloaded is never represented by an empty collection or absent row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", content = "value", rename_all = "snake_case")]
pub enum Loaded<T> {
    /// The caller did not request this relation.
    Unloaded,
    /// The complete requested relation, including an explicitly empty collection/optional None.
    Loaded(T),
}
impl<T> Default for Loaded<T> {
    /// Omitted include members are unloaded independently of the target type's defaults.
    fn default() -> Self {
        Self::Unloaded
    }
}
impl<T> Loaded<T> {
    /// Project a loaded relation without accidentally loading an unrequested one.
    pub fn try_map<U>(self, project: impl FnOnce(T) -> DataResult<U>) -> DataResult<Loaded<U>> {
        match self {
            Self::Unloaded => Ok(Loaded::Unloaded),
            Self::Loaded(value) => project(value).map(Loaded::Loaded),
        }
    }
}
/// Explicit soft-removal selection for trusted maintenance calls only.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visibility {
    /// Ordinary reads include active rows only.
    #[default]
    Active,
    /// Explicit maintenance selection of inactive rows.
    Inactive,
    /// Explicit maintenance selection of both states.
    All,
}
/// Distinguish trusted generated selectors from the independently allowlisted public query.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exposure {
    /// Trusted domain calls; this grants no transport authorization.
    Trusted,
    /// Public query and include paths only; all rows still require projection.
    Public,
}
/// Generated scalar/list query metadata, independent of UI presentation roles.
#[derive(Clone, Copy, Debug)]
pub struct QueryField {
    /// Stored codec and exact wire/column identity.
    pub storage: StorageColumn,
    /// Lists accept membership, not scalar comparison.
    pub multiple: bool,
    /// Scalar codec for list membership (the column itself uses JSON).
    pub scalar: StorageCodec,
    /// Canonical enum scalar choices, or an empty array for non-enums.
    pub choices: &'static str,
    /// Native declared text search.
    pub searchable: bool,
    /// Native declared ordering, including identity tie breakers.
    pub sortable: bool,
    /// Application-approved public filtering.
    pub public_filter: bool,
    /// Application-approved public ordering.
    pub public_sort: bool,
    /// Application-approved public search.
    pub public_search: bool,
}
/// Exact owning or inverse mapping, normalized by the schema tool before emission.
#[derive(Clone, Copy, Debug)]
pub struct RelationSpec {
    /// Include path segment, never an SQL fragment.
    pub name: &'static str,
    /// Target catalog table identity.
    pub target: &'static str,
    /// Source wire scalar mapped to the target scalar.
    pub local: &'static str,
    /// Target wire scalar (column name is looked up separately).
    pub foreign: &'static str,
    /// To-many versus singular cardinality.
    pub many: bool,
    /// Missing singular targets are integrity failures when true.
    pub required: bool,
    /// Explicit app approval for public include paths.
    pub public: bool,
}
/// Immutable generated native catalog; callbacks validate/project through actual typed contracts.
#[derive(Debug)]
pub struct StoreSpec {
    /// Checked table name.
    pub table: &'static str,
    /// Stored field codecs in declaration order.
    pub columns: &'static [StorageColumn],
    /// Complete ordered primary-key wire members.
    pub key: &'static [&'static str],
    /// Optional Boolean active wire member.
    pub active: Option<&'static str>,
    /// Finite scalar query definitions.
    pub fields: &'static [QueryField],
    /// Finite virtual relation definitions.
    pub relations: &'static [RelationSpec],
    /// Strict stored decoder and model validation before any row is exposed.
    pub decode: fn(Row) -> DataResult<Value>,
    /// Explicit public projection at every included node.
    pub project: fn(Value) -> DataResult<Value>,
}
/// A generated typed query envelope; selectors are compiled enum types, never SQL strings.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Query<F, S> {
    /// Case-sensitive literal containment over declared searchable text.
    pub q: Option<String>,
    /// Conjunctive predicates.
    pub filters: Vec<F>,
    /// Explicit direction and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<S>>,
    /// Explicit bounded eager paths.
    pub includes: Vec<String>,
    /// Bounded offset.
    pub skip: u64,
    /// Page size, default 20 and maximum 100.
    pub take: u16,
    /// Request an exact base total in the same SQLite transaction.
    pub total: bool,
}
impl<F, S> Default for Query<F, S> {
    /// Empty search remains a finite page, never a table-wide unbounded read.
    fn default() -> Self {
        Self {
            q: None,
            filters: Vec::new(),
            sort: Vec::new(),
            includes: Vec::new(),
            skip: 0,
            take: 20,
            total: false,
        }
    }
}
/// Checked internal query form, obtained only through bounded serialization of a typed envelope.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadQuery {
    /// Literal search string.
    q: Option<String>,
    /// Wire names still resolve through the immutable catalog.
    filters: Vec<Filter>,
    /// Finite enum wire identities.
    sort: Vec<stackpress_framework_schema::Sort<String>>,
    /// Validated before any IO.
    includes: Vec<String>,
    /// Requested offset.
    skip: u64,
    /// Requested page size.
    take: u16,
    /// Exact base total requested.
    total: bool,
}
/// One scalar predicate whose field name can only resolve through the generated catalog.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Filter {
    /// Exact generated wire selector.
    field: String,
    /// Typed Serde predicate representation.
    predicate: Predicate<Value>,
}
impl ReadQuery {
    /// Serialize within the shared one-MiB/tree budget before retaining an internal query copy.
    pub fn from_typed(value: &impl Serialize) -> DataResult<Self> {
        let value = stackpress_framework_schema::to_bounded_value(value).map_err(validation)?;
        serde_json::from_value(value).map_err(codec_error)
    }
}
/// Add context to checked schema validation while retaining its typed source privately.
pub fn projection_error(cause: stackpress_framework_schema::SchemaError) -> DataError {
    DataError::new("stored public projection failed")
        .with_kind(DataErrorKind::Decode)
        .with_cause(cause)
}
/// Add context to checked input validation, retaining its typed source privately.
pub fn validation(cause: stackpress_framework_schema::SchemaError) -> DataError {
    DataError::new("store input validation failed")
        .with_kind(DataErrorKind::Validation)
        .with_cause(cause)
}
/// Distinguish unavailable explicit host capabilities from malformed values before write effects.
pub fn preparation_error(cause: stackpress_framework_schema::SchemaError) -> DataError {
    let kind = if cause.kind() == stackpress_framework_schema::SchemaErrorKind::Unsupported {
        DataErrorKind::Unsupported
    } else {
        DataErrorKind::Validation
    };
    DataError::new("store value preparation failed")
        .with_kind(kind)
        .with_cause(cause)
}
/// Reject an unrepresentable or over-budget query without including field values in diagnostics.
fn invalid() -> DataError {
    DataError::new("store query is invalid or exceeds its budget")
}
/// Escape the eager wrapper's base member; normalized source names cannot use this prefix.
/// Include paths keep their source spelling; only the internal finite-view protocol is escaped.
pub(crate) fn relation_member(name: &str) -> &str {
    if name == "row" { "__row" } else { name }
}
/// Restore logical source spelling when flattening an already projected finite eager edge.
pub(crate) fn source_relation_member(name: &str) -> &str {
    if name == "__row" { "row" } else { name }
}
/// Bound aggregation failures are distinct from successful, truncated relations.
fn limit() -> DataError {
    DataError::new("complete relation result exceeds its budget")
        .with_kind(DataErrorKind::Validation)
}
impl StoreSpec {
    /// Resolve one exact declared stored member; aliases and client SQL never become identifiers.
    pub fn column(&self, wire: &str) -> DataResult<&StorageColumn> {
        self.columns
            .iter()
            .find(|column| column.wire == wire)
            .ok_or_else(invalid)
    }
    /// Compile a complete, nonnullable scalar identity as a conjunctive predicate.
    pub fn key_predicate(&self, key: &impl Serialize) -> DataResult<Expr> {
        let key = stackpress_framework_schema::to_bounded_value(key).map_err(validation)?;
        let object = key.as_object().ok_or_else(invalid)?;
        // A partial/extra identity must never widen a keyed mutation or lookup.
        if self.key.is_empty() || object.len() != self.key.len() {
            return Err(invalid());
        }
        // Encode in declared identity order; every member remains a typed bind.
        self.key
            .iter()
            .map(|name| {
                let column = self.column(name)?;
                let value = object
                    .get(*name)
                    .filter(|value| !value.is_null())
                    .ok_or_else(invalid)?;
                Ok(Expr::eq(column.column, encode_value(value, column)?))
            })
            .collect::<DataResult<Vec<_>>>()
            .map(Expr::all)
    }
    /// Compare prepared create identity with the complete separately supplied key, before IO.
    pub fn agrees_with_key(&self, row: &impl Serialize, key: &impl Serialize) -> DataResult<bool> {
        // Validate the separate identity before comparing prepared values; null is never a key.
        let _ = self.key_predicate(key)?;
        let row = stackpress_framework_schema::to_bounded_value(row).map_err(validation)?;
        let key = stackpress_framework_schema::to_bounded_value(key).map_err(validation)?;
        Ok(self.key.iter().all(|name| row.get(*name) == key.get(*name)))
    }
    /// Build a key lookup with an explicit ordinary/maintenance active policy.
    pub fn select_key(&self, key: &impl Serialize, visibility: Visibility) -> DataResult<Select> {
        Ok(self
            .visible(self.base(), visibility)?
            .and_where(self.key_predicate(key)?)
            .limit(1))
    }
    /// Select only the declared columns; virtual relations never enter the base query.
    fn base(&self) -> Select {
        Select::new(
            self.columns
                .iter()
                .map(|column| Identifier::from(column.column)),
        )
        .from(self.table)
    }
    /// Apply the active policy at the SQL boundary, including related ordinary reads.
    fn visible(&self, query: Select, visibility: Visibility) -> DataResult<Select> {
        // Visibility is a trusted explicit enum, independent of public query decoding.
        match (self.active, visibility) {
            (Some(active), Visibility::Active | Visibility::Inactive) => Ok(query.where_eq(
                self.column(active)?.column,
                visibility == Visibility::Active,
            )),
            (None, Visibility::Inactive) => Err(invalid()),
            _ => Ok(query),
        }
    }
}
/// Count matched base identities, without relation joins, sorting or paging.
pub async fn count(
    db: &mut DataSession<'_>,
    spec: &StoreSpec,
    query: &ReadQuery,
    exposure: Exposure,
    visibility: Visibility,
) -> DataResult<u64> {
    // Count the filtered base table directly so eager child fanout cannot inflate totals.
    let rows = db
        .fetch(&query::filtered(
            spec,
            query,
            db.dialect(),
            exposure,
            visibility,
            true,
        )?)
        .await?;
    if rows.len() != 1 {
        return Err(DataError::new("base count returned an invalid row count")
            .with_kind(DataErrorKind::Decode));
    }
    u64::try_from(rows[0].try_get::<i64>("total")?).map_err(|cause| {
        DataError::new("base count is outside its supported range")
            .with_kind(DataErrorKind::Decode)
            .with_cause(cause)
    })
}
