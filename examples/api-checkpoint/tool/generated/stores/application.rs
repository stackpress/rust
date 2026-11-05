//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Application store, including application-added fields.
pub type ApplicationStore = stackpress_framework_sql::source::NativeStore<
    super::super::source::application::ApplicationSchema,
>;
/// Full generated Application actions; plugins do not own or replace its field inventory.
pub type ApplicationActions = stackpress_framework_sql::source::NativeActions<
    super::super::source::application::ApplicationSchema,
>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct ApplicationModel;
impl ModelContract for ApplicationModel {
    type Role = stackpress_framework_sql::source::ApplicationRole;
    type Schema = super::super::source::application::ApplicationSchema;
    type Store = ApplicationStore;
    type Actions = ApplicationActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<ApplicationModel>(app)
}
