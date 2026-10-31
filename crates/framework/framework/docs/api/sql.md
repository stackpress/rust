# SQL scripts and revision providers

The aggregate reuses stackpress-sql Engine/dialects. The application supplies an Engine service, compiled Scripts and any required History callbacks; registration neither connects to a database nor reconstructs a driver.

## Data and callbacks

These native objects are not browser snapshot values.

| Type | Fields / purpose |
| --- | --- |
| Scripts | install, purge, uninstall: Vec<Query>; schema: serde_json::Value, kept for revision insertion. |
| Scripts::from_tables(tables, dialect, schema) | Builds real SQL in catalog create order; uninstall reverses it and uses IF EXISTS; purge follows accepted dialect mapping. Returns Result<Scripts> with rendering errors. |
| Migration | epoch: i64, ordered queries: Vec<Query>, warning: Option<String>. |
| History.all | fn(&str, &Engine) -> Result<Vec<Migration>>: baseline then adjacent plans. |
| History.latest | fn(&str, &Engine) -> Result<Option<Migration>>: latest adjacent plan only. |
| History.has_pair | fn(&str) -> Result<bool>: inspect last two source revisions. |
| History.insert | fn(&str, &serde_json::Value) -> Result<()>: source comparison/write boundary. |

History owns source-schema decoding/planning/IO, not an extra refusal policy. Only upgrade enforces its warning; migrate still writes warned plans. Missing provider failures occur only once the selected operation actually needs that provider.

## Direct functions

All fallible functions return the original error without retrying. Here `root`/`destination` mean optional borrowed source paths; `inv` is the current borrowed Invocation.

| Function | Arguments / result / side effect |
| --- | --- |
| transaction | (&Engine, &[Query]) -> async Result<()>; begin, execute sequentially, commit; execution failure rolls back retaining rollback context. |
| install | (&Engine, &Scripts, Option<(&str, &History)>) -> async Result<()>; uninstall+install one transaction; revision insertion is after commit, so its failure does not undo committed SQL. |
| purge / uninstall | (&Engine, &Scripts) -> async Result<()>; corresponding statement sequence in one transaction. |
| push | (&Engine, root, &History, force, scripts: impl FnOnce() -> Result<Arc<Scripts>> + Send) -> async Result<()>; lazy install/upgrade decision. |
| populate | (&[serde_json::Value], &Invocation<'_>) -> async Result<()>; independent raw child responses, no Engine. |
| migrate | (&Engine, root, destination, &History) -> Result<()>; create directory and write epoch.sql for nonempty plans, statements joined with semicolon/newline. |
| upgrade | (&Engine, root, &History, force) -> async Result<()>; latest adjacent plan and warning check. |
| event | Standard async handler -> Result<Flow>; prior-status skip, explicit Engine lookup and wrapper status/errors from [commands](commands.md). |

This full-consumer example executes the direct transaction API against a real disposable SQLite file and explicitly closes the Engine.

<!-- example: full -->
```rust
use stackpress_framework::commands::sql::transaction;
use stackpress_sql::{Engine, Render, Sql, TrustedSql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

pub async fn sqlite_transaction() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let sqlite = SqliteDatabase::open(
        SqliteOptions::file(directory.path().join("docs.sqlite")).create_if_missing(true),
    )
    .await?;
    let engine = Engine::new(sqlite);
    let create = Sql::trusted(
        engine.dialect(),
        TrustedSql::audited("CREATE TABLE docs (id INTEGER)".to_owned()),
    )
    .render(engine.dialect())?;
    transaction(&engine, &[create]).await?;
    engine.close().await?;
    Ok(())
}
```

The [real SQLite lifecycle tests](../../tests/cases/sql.rs) cover command events, rollback, script skips, query output and revision laziness beyond this small transaction. Check the adapter's current constructors when integrating other backends; Postgres/MySQL dialect output is not proof of a live connection.

## Accepted SQLite divergence

G46 resolves SQLite purge by keeping the accepted DELETE FROM "table" mapping. No TRUNCATE or additional WHERE 1 form is introduced. See the [resolved decisions](../source-behavior.md), [TableCommand rendering](../../../../sql/sql/src/table.rs) and [SQLite DDL tests](../../../../sql/sql-sqlite/src/tests/m2/ddl.rs).
