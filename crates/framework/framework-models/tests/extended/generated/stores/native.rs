//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Native store, including application-added fields.
pub type NativeStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::native::NativeSchema>;
/// Full generated Native actions; plugins do not own or replace its field inventory.
pub type NativeActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::native::NativeSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct NativeModel;
impl ModelContract for NativeModel {
    type Role = ();
    type Schema = super::super::source::native::NativeSchema;
    type Store = NativeStore;
    type Actions = NativeActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<NativeModel>(app)
}
