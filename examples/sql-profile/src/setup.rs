//! Explicit one-shot installation/seeding. Startup and request handling never call this module.

use std::path::PathBuf;

use stackpress_sql::{Create, DialectId, Engine, Field, Index, Insert, Result, SqlType, SqlValue};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

/// Create the profile table/index and the exact reference rows in one explicit transaction.
///
/// The caller controls the file. Existing profile installations are refused, never cleared,
/// reseeded or migrated. A failed transaction leaves no partially installed schema/data;
/// a newly created empty file can remain after failure. No directory is implicitly created.
///
/// # Errors
/// Returns path/open/DDL/seed/commit errors with their native source and any cleanup failure.
/// Cancellation loses observation; an uncommitted guard initiates rollback, but a lost commit
/// acknowledgement can have an unknown outcome. No installation is automatically retried.
pub async fn install_and_seed(path: impl Into<PathBuf>) -> Result<()> {
    let engine = Engine::new(
        SqliteDatabase::open(SqliteOptions::file(path.into()).create_if_missing(true)).await?,
    );
    let result = install(&engine).await;
    // Await cleanup on both success and failure without replacing the original native error.
    match (result, engine.close().await) {
        (Ok(()), cleanup) => cleanup,
        (Err(primary), Ok(())) => Err(primary),
        (Err(primary), Err(cleanup)) => Err(primary.with_cleanup(cleanup)),
    }
}

/// Keep table, index and seed writes on the same caller-owned native transaction.
async fn install(engine: &Engine) -> Result<()> {
    let plan = Create::new("profile")
        .add_field(Field::new("ordinal", SqlType::Integer).not_null())
        .add_field(Field::new("id", SqlType::Text).not_null())
        .add_field(Field::new("name", SqlType::Text).not_null())
        .primary_key(["ordinal"])
        .add_index(Index::new("profile_id", ["id"]).unique())
        .render(DialectId::Sqlite)?;
    let mut transaction = engine.begin().await?;
    let result: Result<()> = async {
        transaction.execute_plan(&plan).await?;
        // The public fixture and this stored order exactly match capability-host's seed service.
        for (ordinal, (id, name)) in [
            ("abc123", "Ada Lovelace"),
            ("abc456", "Grace Hopper"),
            ("abc789", "Margaret Hamilton"),
        ]
        .into_iter()
        .enumerate()
        {
            transaction
                .execute(
                    &Insert::new("profile")
                        .columns(["ordinal", "id", "name"])
                        .values([SqlValue::Integer(ordinal as i64), id.into(), name.into()]),
                )
                .await?;
        }
        Ok(())
    }
    .await;
    match result {
        Ok(()) => transaction.commit().await,
        Err(primary) => Err(transaction.rollback_after(primary).await),
    }
}
