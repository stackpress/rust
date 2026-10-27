//! Explicit generated table installation and immutable native store catalog.
/// Build every declared table definition; importing never installs one.
pub fn definition() -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {
    Ok(vec![
        super::profile::ProfileStore::table()?,
        super::project::ProjectStore::table()?,
        super::task::TaskStore::table()?,
        super::link::LinkStore::table()?,
    ])
}
/// Return the finite generated catalog; it owns no executor or mutable cache.
pub fn catalog() -> Vec<&'static stackpress_framework_sql::stores::StoreSpec> {
    vec![
        &super::profile::SPEC,
        &super::project::SPEC,
        &super::task::SPEC,
        &super::link::SPEC,
    ]
}
