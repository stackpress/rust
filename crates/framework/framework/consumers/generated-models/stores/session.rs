//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Session store, including application-added fields.
pub type SessionStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::session::SessionSchema>;
/// Full generated Session actions; plugins do not own or replace its field inventory.
pub type SessionActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::session::SessionSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct SessionModel;
impl ModelContract for SessionModel {
    type Role = stackpress_framework_sql::source::ApiSessionRole;
    type Schema = super::super::source::session::SessionSchema;
    type Store = SessionStore;
    type Actions = SessionActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<SessionModel>(app)
}
