//! Source generated query defaults used by detail/get/search and management consumers.
use crate::models::{ModelKind, Record, StoreQuery, Value};
/// Source model @query columns, including plaintext credentials and joined Profile.
pub fn columns<M: ModelKind>() -> Vec<String> {
    let model = super::super::models::Schema::<M>::new().model();
    if model.query.is_empty() {
        vec!["*".into()]
    } else {
        model.query
    }
}
/// Extract the generated source query's known fields; unrelated request data is ignored.
/// Typed paging is a Rust mechanics boundary, not a new public data allowlist.
pub fn query(data: &Record) -> crate::models::Result<StoreQuery> {
    StoreQuery::from_record(data)
}

/// Source Array.isArray/every guard accepts an empty selector array.
pub fn selected(value: Option<&Value>) -> Option<Vec<String>> {
    value?
        .as_array()?
        .iter()
        .map(|value| value.as_str().filter(|s| !s.is_empty()).map(str::to_owned))
        .collect()
}
