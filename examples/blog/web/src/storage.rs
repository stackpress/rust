//! Explicit installation from this host's generated model inventory.

use crate::generated::stores;
use stackpress_lib::{Error, Result};
use stackpress_sql::TableDefinition;
use std::path::Path;

/// Derive all eight table definitions from actual generated stores, in dependency order.
///
/// # Errors
/// Retains generated metadata/preparation and DDL errors; no second field inventory is used.
pub fn tables() -> Result<Vec<TableDefinition>> {
    Ok(vec![
        stores::profile::ProfileStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::category::CategoryStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::article::ArticleStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::catalog::CatalogStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::comment::CommentStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::application::ApplicationStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::session::SessionStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
        stores::auth::AuthStore::new(String::new(), 0)
            .create()
            .map_err(Error::from_source)?
            .definition()
            .map_err(Error::from_source)?,
    ])
}

/// Explicit source installation, including its uninstall-first behavior; never a startup action.
///
/// # Errors
/// Preserves setup/close errors. Closing is attempted even after installation fails.
pub async fn setup(path: &Path) -> Result<()> {
    // Prepare immutable SQL before acquiring the native resource, including failure paths.
    let scripts = stackpress_framework::commands::sql::Scripts::from_tables(
        &tables()?,
        stackpress_sql::DialectId::Sqlite,
        serde_json::json!({}),
    )?;
    let engine = blog_store::open(path, true).await?;
    let installed = stackpress_framework::commands::sql::install(&engine, &scripts, None).await;
    let closed = engine.close().await.map_err(Error::from_source);
    installed.and(closed)
}
