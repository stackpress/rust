//! File SQLite connection acquisition; creation is always explicit.
use stackpress_lib::{Error, Result};
use stackpress_sql::Engine;
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};
use std::{path::Path, sync::Arc};

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
