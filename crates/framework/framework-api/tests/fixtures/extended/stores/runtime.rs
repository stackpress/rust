//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Runtime store, including application-added fields.
pub type RuntimeStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::runtime::RuntimeSchema>;
/// Full generated Runtime actions; plugins do not own or replace its field inventory.
pub type RuntimeActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::runtime::RuntimeSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct RuntimeModel;
impl ModelContract for RuntimeModel {
    type Role = ();
    type Schema = super::super::source::runtime::RuntimeSchema;
    type Store = RuntimeStore;
    type Actions = RuntimeActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<RuntimeModel>(app)
}
