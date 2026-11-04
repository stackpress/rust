//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Lists store, including application-added fields.
pub type ListsStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::lists::ListsSchema>;
/// Full generated Lists actions; plugins do not own or replace its field inventory.
pub type ListsActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::lists::ListsSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct ListsModel;
impl ModelContract for ListsModel {
    type Role = ();
    type Schema = super::super::source::lists::ListsSchema;
    type Store = ListsStore;
    type Actions = ListsActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<ListsModel>(app)
}
