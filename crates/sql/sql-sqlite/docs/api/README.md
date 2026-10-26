# SQLite API reference

The crate exports two public types. It implements the core Database contract; Engine,
Transaction, builders, query limits, rows, values and errors remain SQL-core types.

| Public surface | Reference |
| --- | --- |
| SqliteOptions and its seven fields | [Options](options.md) |
| SqliteDatabase::open and Database implementation | [Database](database.md) |
| Engine and Transaction ownership | [SQL core concepts](../../../sql/docs/concepts.md) |
| Core methods and types | [SQL core API index](../../../sql/docs/api/README.md) |

SqliteOptions implements Clone, Debug, Serialize and Deserialize; it has no Default
implementation. SqliteDatabase is Send + Sync + 'static and is not Clone. Share an
`Arc<Engine>` explicitly when callers need the same pool.

## Find an operation

The adapter's inherent methods and implemented trait operations are listed here for lookup:

| Operation | Purpose |
| --- | --- |
| [SqliteOptions::file](options.md#1-sqliteoptionsfile) | Choose a file and finite defaults. |
| [create_if_missing](options.md#2-create_if_missing) | Opt into creation. |
| [read_only](options.md#3-read_only) | Preserve an existing file without permitting writes. |
| [validate](options.md#4-validate) | Check options before native IO. |
| [SqliteDatabase::open](database.md#1-sqlitedatabaseopen) | Validate and eagerly open. |
| [Database::dialect](database.md#2-databasedialect) | Read the stable SQLite identity. |
| [Database::run](database.md#3-databaserun) | Execute one prepared, bound statement. |
| [Database::begin](database.md#4-databasebegin) | Own one deferred transaction session. |
| [Database::close](database.md#5-databaseclose) | Reject new work and await leases. |

For first use, start with [getting started](../getting-started.md). For lifecycle details,
read [concepts](../concepts.md). Source: [lib.rs](../../src/lib.rs) and
[options.rs](../../src/options.rs); private leases and sessions are not extension APIs.
