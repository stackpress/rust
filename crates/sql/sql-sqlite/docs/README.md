# Stackpress SQLite

`stackpress-sql-sqlite` runs Stackpress SQL queries against an application-owned native SQLite
file. It supplies eager opening, a bounded pool and transaction cleanup; the
[SQL core](../../sql/docs/README.md) owns builders, values, rows and Engine.

## Start here

Use these pages when adding SQLite to a native Rust application. The
[getting started walkthrough](getting-started.md) creates a temporary database, installs a
table, reads a bound value and closes the pool. It prints `profile-name: Ada`.

## Reading paths

Choose the path that matches your next task:

| Goal | Read |
| --- | --- |
| Try a real database | [Getting started](getting-started.md) |
| Understand pool, transaction and shutdown behavior | [Concepts](concepts.md) |
| Share the database with operations | [Register an Engine service](guides/server-service.md) |
| Follow a complete application | [SQLite profile walkthrough](guides/sql-profile.md) |
| Look up options or adapter methods | [API reference](api/README.md) |
| Build queries or decode values | [SQL core docs](../../sql/docs/README.md) |

## Runtime and scope

The adapter uses SQLx 0.9.0 with its Tokio/SQLite features and requires a live Tokio runtime
for async work. Importing the crate opens no database. File creation and schema installation
are explicit application actions; queries are never automatically retried.

The crate exposes `SqliteDatabase` and `SqliteOptions`. The facade's `sql-sqlite` feature also
exposes them through `stackpress::sql_sqlite`, alongside `stackpress::sql`. Native
`aarch64-apple-darwin` is the verified target. These docs do not establish browser, mobile,
Linux, Windows or network-filesystem support. PostgreSQL/MySQL rendering and opt-in renderer
tests belong to the core; production network adapters remain future work.

Source: [crate Rustdoc and implementation](../src/lib.rs), [options](../src/options.rs),
[native tests](../src/tests.rs) and [canonical application](../../../../examples/sql-profile/README.md).
