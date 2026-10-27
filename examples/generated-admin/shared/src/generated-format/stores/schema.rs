//! Explicit generated table installation and immutable native store catalog.
/// Build every declared table definition; importing never installs one.
pub fn definition() -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {
    Ok(vec![
        super::task::TaskStore::table()?,
        super::profile::ProfileStore::table()?,
        super::project::ProjectStore::table()?,
        super::role_proof::RoleProofStore::table()?,
    ])
}
/// Return the finite generated catalog; it owns no executor or mutable cache.
pub fn catalog() -> Vec<&'static stackpress_framework_sql::stores::StoreSpec> {
    vec![
        &super::task::SPEC,
        &super::profile::SPEC,
        &super::project::SPEC,
        &super::role_proof::SPEC,
    ]
}
/// Supply explicit MySQL target facts and check the entire managed catalog before rendering.
pub fn mysql_definition(
    collation: &str,
    page_bytes: u32,
) -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {
    stackpress_framework_sql::ddl_policy::mysql_definitions(definition()?, collation, page_bytes)
}
