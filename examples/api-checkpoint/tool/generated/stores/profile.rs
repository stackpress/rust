//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Profile store, including application-added fields.
pub type ProfileStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::profile::ProfileSchema>;
/// Full generated Profile actions; plugins do not own or replace its field inventory.
pub type ProfileActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::profile::ProfileSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct ProfileModel;
impl ModelContract for ProfileModel {
    type Role = stackpress_framework_sql::source::ProfileRole;
    type Schema = super::super::source::profile::ProfileSchema;
    type Store = ProfileStore;
    type Actions = ProfileActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<ProfileModel>(app)
}
