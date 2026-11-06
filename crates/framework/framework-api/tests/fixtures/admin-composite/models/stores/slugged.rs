//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::SourceModel as ModelContract;
/// Full generated Slugged store, including application-added fields.
pub type SluggedStore =
    stackpress_framework_sql::source::NativeStore<super::super::source::slugged::SluggedSchema>;
/// Full generated Slugged actions; plugins do not own or replace its field inventory.
pub type SluggedActions =
    stackpress_framework_sql::source::NativeActions<super::super::source::slugged::SluggedSchema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct SluggedModel;
impl ModelContract for SluggedModel {
    type Role = ();
    type Schema = super::super::source::slugged::SluggedSchema;
    type Store = SluggedStore;
    type Actions = SluggedActions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature = "server")]
pub fn register(
    app: &mut stackpress_framework_sql::source::__private::ServerBuilder,
) -> stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<SluggedModel>(app)
}
