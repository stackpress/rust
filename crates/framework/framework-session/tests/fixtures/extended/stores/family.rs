//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Family store, including application-added fields.
pub type FamilyStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::family::FamilySchema>;
/// Full generated Family actions; plugins do not own or replace its field inventory.
pub type FamilyActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::family::FamilySchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct FamilyModel;
impl ModelContract for FamilyModel {
    type Role = ();
    type Schema = super::super::source::family::FamilySchema;
    type Store = FamilyStore;
    type Actions = FamilyActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<FamilyModel>(app)
}
