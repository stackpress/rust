//! Checked generated storage definitions; importing never installs them.
/// Return all managed table definitions in source order.
pub fn definition() -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {
    Ok(vec![
        super::profile::ProfileStore::table()?,
        super::project::ProjectStore::table()?,
        super::task::TaskStore::table()?,
    ])
}
