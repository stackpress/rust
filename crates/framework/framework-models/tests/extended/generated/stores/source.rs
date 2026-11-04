//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Source store, including application-added fields.
pub type SourceStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::_source::SourceSchema>;
/// Full generated Source actions; plugins do not own or replace its field inventory.
pub type SourceActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::_source::SourceSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct SourceModel;
impl ModelContract for SourceModel {
    type Role = ();
    type Schema = super::super::source::_source::SourceSchema;
    type Store = SourceStore;
    type Actions = SourceActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<SourceModel>(app)
}
