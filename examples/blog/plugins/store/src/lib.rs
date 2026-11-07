//! File-based SQLite ownership; registration never installs or populates tables.

use serde::{Deserialize, Serialize};
use stackpress_framework_sql::source::SourceEnvironment;
use stackpress_lib::{Error, Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql_sqlite::SqliteOptions;
use std::{path::PathBuf, sync::Arc};

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

/// Native connection ownership is independent of generated models.
pub mod connect;
pub use connect::open;
