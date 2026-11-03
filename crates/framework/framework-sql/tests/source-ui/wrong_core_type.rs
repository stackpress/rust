//! An app's integer Profile.id cannot satisfy the source session role.
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_schema::source::{Id, SourceField};
fn string_id<S: SourceField<Id, FieldValue = String>>() {}
fn main() {
    string_id::<support::Schema<support::CompileSpec<i64>>>();
}
