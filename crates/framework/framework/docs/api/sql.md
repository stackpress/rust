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
| install | (&Engine, &Scripts, Option<(&str, &History)>) -> async Result<()>; uninstall then install; G48 permits MySQL partial progress, while PostgreSQL/SQLite use one transaction. Revision insertion follows complete SQL success. |
| purge / uninstall | (&Engine, &Scripts) -> async Result<()>; corresponding source sequence, non-atomic on MySQL and atomic on PostgreSQL/SQLite. |
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

The [real SQLite lifecycle tests](../../tests/cases/sql.rs) cover command events, rollback, script skips, query output and revision laziness beyond this small transaction. Separate [session/API live tests](../../../framework-session/docs/live-search.md) execute direct PostgreSQL/MySQL source-store ASCII searches. [Owned aggregate live tests](../../tests/live_setup.rs) execute the direct source commands, including one adjacent generated migration on both network backends, MySQL partial failures and PostgreSQL rollback. These finite calls do not establish live command-provider event routing.

G48 uses stackpress_framework_sql::setup for source setup callers. The public transaction
function remains atomic and still rejects MySQL transaction DDL. A non-atomic error retains
the original SQL kind/display/cause plus plan_progress; cancellation and lost acknowledgements
can leave effects. Source revision insertion occurs only after all setup statements succeed.
G27's generated MigrationRunner remains SQLite-only: no new MySQL runner, target lock, drift
detector or generated-workspace ledger is supplied. MySQL upgrades use the existing aggregate
adjacent-migration command.

G48 resolves MySQL purge after native ERROR 1064 for the source CASCADE spelling:
`Scripts::from_tables` requests CASCADE only on PostgreSQL. Unrelated MySQL catalogs
retain TRUNCATE in create order. InnoDB rejects TRUNCATE of referenced tables with
ERROR 1701, so a MySQL catalog declaring foreign keys uses DELETE for its tables in
reverse create order (children first). Supply parents before children, as for creation.
The [live proof](../../tests/live/cascade.rs) records the native 1701 diagnostic,
executes the actual purge, and checks that constraints still reject an orphan insert.
It also executes unchanged PostgreSQL parent-first TRUNCATE CASCADE. No renderer
capability, FOREIGN_KEY_CHECKS setting, runtime retry or drop/recreate is introduced.
App-authored `Scripts` retain their explicit queries; the purge command executes them
through the same setup selector. Standalone session/API actions already forward the
cascade flag only on PostgreSQL; their live teardown now passes `true` to pin that rule.

## Accepted SQLite divergence

G46 resolves SQLite purge by keeping the accepted DELETE FROM "table" mapping. No TRUNCATE or additional WHERE 1 form is introduced. See the [resolved decisions](../source-behavior.md), [TableCommand rendering](../../../../sql/sql/src/table.rs) and [SQLite DDL tests](../../../../sql/sql-sqlite/src/tests/m2/ddl.rs).
