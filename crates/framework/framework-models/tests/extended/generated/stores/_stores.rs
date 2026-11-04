//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Stores store, including application-added fields.
pub type StoresStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::stores::StoresSchema>;
/// Full generated Stores actions; plugins do not own or replace its field inventory.
pub type StoresActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::stores::StoresSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct StoresModel;
impl ModelContract for StoresModel {
    type Role = ();
    type Schema = super::super::source::stores::StoresSchema;
    type Store = StoresStore;
    type Actions = StoresActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<StoresModel>(app)
}
