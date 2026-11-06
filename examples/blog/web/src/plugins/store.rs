//! File-based SQLite ownership; registration never installs or populates tables.

use serde::{Deserialize, Serialize};
use stackpress_framework_sql::source::SourceEnvironment;
use stackpress_lib::{Error, Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::{Engine, TableDefinition};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::generated::stores;

/// Application file selection, separate from framework SQL command configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Existing SQLite file; relative paths resolve from the application's working directory.
    pub path: PathBuf,
}

impl Default for DatabaseConfig {
    /// Select the documented default without creating it.
    fn default() -> Self {
        Self {
            path: ".build/blog.sqlite".into(),
        }
    }
}

/// Open only the selected file; creation is an explicit setup argument.
///
/// # Errors
/// Preserves SQLite configuration, open, and IO failures.
pub async fn open(path: &Path, create: bool) -> Result<Arc<Engine>> {
    let database = SqliteDatabase::open(SqliteOptions::file(path).create_if_missing(create))
        .await
        .map_err(Error::from_source)?;
    Ok(Arc::new(Engine::new(database)))
}

/// Register file configuration and connection ownership as ordinary plugin callbacks.
///
/// # Errors
/// Preserves duplicate section and callback errors; performs no IO here.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("sqlite", DatabaseConfig::default(), |config| {
        SqliteOptions::file(&config.path)
            .validate()
            .map_err(Error::from_source)
    })?;
    app.on_config(configure)
}

/// Publish an existing-file Engine and source conversion seed, without changing schema/data.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let engine = open(&start.config::<DatabaseConfig>()?.path, false).await?;
    let options = &start
        .config::<stackpress_framework::config::DatabaseOptions>()?
        .0;
    let seed = options
        .get("seed")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("abc123");
    let environment = SourceEnvironment {
        seed: seed.into(),
        offset_minutes: 0,
    };
    start.register(engine)?;
    start.register(Arc::new(environment))?;
    Ok(Flow::Continue)
}

/// Derive all eight table definitions from actual generated stores, in dependency order.
///
/// # Errors
/// Retains generated DDL errors; no second handwritten field inventory is used.
pub fn tables() -> Result<Vec<TableDefinition>> {
    Ok(vec![
        stores::profile::ProfileStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::category::CategoryStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::article::ArticleStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::catalog::CatalogStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::comment::CommentStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::application::ApplicationStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::session::SessionStore::new(String::new(), 0)
            .create()
            .definition()
            .map_err(Error::from_source)?,
        stores::auth::AuthStore::new(String::new(), 0)
            .create()
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
    let engine = open(path, true).await?;
    let installed = stackpress_framework::commands::sql::install(&engine, &scripts, None).await;
    let closed = engine.close().await.map_err(Error::from_source);
    installed.and(closed)
}
