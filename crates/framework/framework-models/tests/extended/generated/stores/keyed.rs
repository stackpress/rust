//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Keyed store, including application-added fields.
pub type KeyedStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::keyed::KeyedSchema>;
/// Full generated Keyed actions; plugins do not own or replace its field inventory.
pub type KeyedActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::keyed::KeyedSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct KeyedModel;
impl ModelContract for KeyedModel {
    type Role = ();
    type Schema = super::super::source::keyed::KeyedSchema;
    type Store = KeyedStore;
    type Actions = KeyedActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<KeyedModel>(app)
}
