//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated NativeRecord store, including application-added fields.
pub type NativeRecordStore = stackpress_framework_sql::source::NativeStore<
    super::super::source::native_record::NativeRecordSchema,
>;
/// Full generated NativeRecord actions; plugins do not own or replace its field inventory.
pub type NativeRecordActions = stackpress_framework_sql::source::NativeActions<
    super::super::source::native_record::NativeRecordSchema,
>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct NativeRecordModel;
impl ModelContract for NativeRecordModel {
    type Role = ();
    type Schema = super::super::source::native_record::NativeRecordSchema;
    type Store = NativeRecordStore;
    type Actions = NativeRecordActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<NativeRecordModel>(app)
}
