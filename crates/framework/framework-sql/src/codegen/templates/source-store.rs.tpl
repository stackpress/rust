//! Generated source store/actions; regenerate from the application Idea graph.
use stackpress_framework_sql::source::{SourceModel as ModelContract};
/// Full generated [%name%] store, including application-added fields.
pub type [%name%]Store=stackpress_framework_sql::source::NativeStore<super::super::source::[%module%]::[%name%]Schema>;
/// Full generated [%name%] actions; plugins do not own or replace its field inventory.
pub type [%name%]Actions=stackpress_framework_sql::source::NativeActions<super::super::source::[%module%]::[%name%]Schema>;
/// Compile-time model identity. Construction and registration perform no database work.
pub struct [%name%]Model;
impl ModelContract for [%name%]Model {
    type Role=[%role%];
    type Schema=super::super::source::[%module%]::[%name%]Schema;
    type Store=[%name%]Store;
    type Actions=[%name%]Actions;
}
/// Standalone source CRUD event registrar; the app chooses whether and when to call it.
#[cfg(feature="server")]
pub fn register(app:&mut stackpress_framework_sql::source::__private::ServerBuilder)->stackpress_framework_sql::source::__private::ServerResult<()> {
    stackpress_framework_sql::source::register::<[%name%]Model>(app)
}
