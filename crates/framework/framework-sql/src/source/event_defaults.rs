//! Source generated query defaults used by detail/get/search and management consumers.
use super::SourceModel;
use stackpress_framework_schema::source::SourceSchema;
/// Source model @query columns, including plaintext credentials and joined Profile.
pub fn columns<M: SourceModel>() -> Vec<String> {
    let model = M::Schema::describe();
    if model.query.is_empty() {
        vec!["*".into()]
    } else {
        model.query
    }
}
pub(super) use super::query_input::selected;
