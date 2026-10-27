//! Explicit generated table installation and immutable native store catalog.
/// Build every declared table definition; importing never installs one.
pub fn definition() -> stackpress_framework_sql::DataResult<Vec<stackpress_sql::TableDefinition>> {
    Ok(vec![
        super::matrix::MatrixStore::table()?,
        super::link::LinkStore::table()?,
        super::compound::CompoundStore::table()?,
        super::query::QueryStore::table()?,
    ])
}
/// Return the finite generated catalog; it owns no executor or mutable cache.
pub fn catalog() -> Vec<&'static stackpress_framework_sql::stores::StoreSpec> {
    vec![
        &super::matrix::SPEC,
        &super::link::SPEC,
        &super::compound::SPEC,
        &super::query::SPEC,
    ]
}
