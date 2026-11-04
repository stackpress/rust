//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Keyless store, including application-added fields.
pub type KeylessStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::keyless::KeylessSchema>;
/// Full generated Keyless actions; plugins do not own or replace its field inventory.
pub type KeylessActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::keyless::KeylessSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct KeylessModel;
impl ModelContract for KeylessModel {
    type Role = ();
    type Schema = super::super::source::keyless::KeylessSchema;
    type Store = KeylessStore;
    type Actions = KeylessActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<KeylessModel>(app)
}
