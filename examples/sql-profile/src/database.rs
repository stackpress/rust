//! The application chooses a file at startup; this plugin publishes only a validated Engine.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use stackpress_lib::{Error, Flow, Result};
use stackpress_server::{ServerBuilder, StartupContext};
use stackpress_sql::{Engine, Identifier, Select};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

/// App-owned file selection; native handles and database policy are never wire DTO fields.
#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatabaseConfig {
    /// Existing file opened read-only; relative paths resolve from the caller's working directory.
    pub path: PathBuf,
}

impl Default for DatabaseConfig {
    /// Choose a visible application file, without opening or creating it.
    fn default() -> Self {
        Self {
            path: PathBuf::from("profiles.sqlite"),
        }
    }
}

impl DatabaseConfig {
    /// Fix native access policy in application code; config cannot opt into creation or writes.
    fn options(&self) -> SqliteOptions {
        SqliteOptions::file(&self.path).read_only(true)
    }
}

/// Register the typed sqlite section and one Config callback through the existing host API.
///
/// # Errors
/// Returns duplicate configuration/startup registration failures.
pub fn register(app: &mut ServerBuilder) -> Result<()> {
    app.config_section("sqlite", DatabaseConfig::default(), validate)?;
    app.on_config(configure)?;
    Ok(())
}

/// Validate paths/budgets before any startup callback can acquire a native connection.
fn validate(config: &DatabaseConfig) -> Result<()> {
    config.options().validate().map_err(Error::from_source)
}

/// Eagerly open and verify known columns without writing, then publish the shared service once.
async fn configure(start: &mut StartupContext) -> Result<Flow> {
    let options = start.config::<DatabaseConfig>()?.options();
    let engine = Engine::new(
        SqliteDatabase::open(options)
            .await
            .map_err(Error::from_source)?,
    );
    // Qualification prevents SQLite's legacy quoted-string fallback for a missing column.
    // LIMIT 0 verifies preparation without retaining rows or installing missing schema.
    let check =
        Select::new(["ordinal", "id", "name"].map(|name| Identifier::qualified(["profile", name])))
            .from("profile")
            .limit(0);
    if let Err(primary) = engine.fetch(&check).await {
        // A failed startup publishes no service. Await cleanup while preserving its primary source.
        let primary = match engine.close().await {
            Ok(()) => primary,
            Err(cleanup) => primary.with_cleanup(cleanup),
        };
        return Err(Error::from_source(primary));
    }
    start.register(Arc::new(engine))?;
    Ok(Flow::Continue)
}
