//! Source generated query defaults used by detail/get/search and management consumers.
use super::{SourceModel, SourceResult as Result, StoreQuery};
use stackpress_framework_schema::source::{Record, SourceSchema};
/// Source model @query columns, including plaintext credentials and joined Profile.
pub fn columns<M: SourceModel>() -> Vec<String> {
    let model = M::Schema::describe();
    if model.query.is_empty() {
        vec!["*".into()]
    } else {
        model.query
    }
}
/// Decode through the same source query representation used by direct stores.
pub fn query(data: &Record) -> Result<StoreQuery> {
    StoreQuery::from_record(data)
}
pub(super) use super::query_input::selected;
