//! Only the executor borrow is non-static; owned returned records do not make its future static.
use stackpress_framework_sql::source::ActionError;
#[path = "../main/source_models/support.rs"]
mod support;
use stackpress_framework_sql::{
    DataSession,
    source::{ProfileRole, SourceActions, SourceModel},
};
use std::future::Future;
type Actions = <support::Model<support::CompileSpec<String>, ProfileRole> as SourceModel>::Actions;
fn escape<'a>(
    actions: &'static Actions,
    db: &'a mut DataSession<'a>,
    query: &'static support::Query,
) -> impl Future<Output = Result<Option<support::Record>, ActionError>> + Send + 'static {
    actions.find_in(db, query)
}
fn main() {}
