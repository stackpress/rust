//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Auth store, including application-added fields.
pub type AuthStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::auth::AuthSchema>;
/// Full generated Auth actions; plugins do not own or replace its field inventory.
pub type AuthActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::auth::AuthSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct AuthModel;
impl ModelContract for AuthModel {
    type Role = stackpress_framework_sql::source::AuthRole;
    type Schema = super::super::source::auth::AuthSchema;
    type Store = AuthStore;
    type Actions = AuthActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<AuthModel>(app)
}
