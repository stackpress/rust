# Slice 7 SQL adapter inventory

Companion to [decisions](slice-7-design-decisions.md). Read for upstream/native
coverage and exact inheritance from current `stackpress-sql` / SQLite source.

## Sources and disposition terms

[Database adapter contracts](../../references/00026-upstream-database-adapter-contracts.md)
and [Slice 4 adapter inventory](slice-4-design-inventory-framework.md) supply the
source map. Read-only source checked:
`inquire/packages/{inquire-pg,inquire-mysql2}/src/{index,types,helpers,Connection}.ts`
and both `tests/Connection.test.ts`. `Now` below is planned work, not execution.

## Public exports and behavior

| Export / exact behavior | Disposition and reason |
| --- | --- |
| PG `Results<R>`: command, rowCount, oid, rows, fields, RowCtor, rowAsArray | Now existing `QueryResult { rows, affected, insert_id }`, ordered `Row`. Native field/cursor/constructor metadata intentionally not public. No opaque native `raw` result. |
| MySQL `Results<R> = [R[], FieldPacket[]]` | Now same statement-local QueryResult, not driver tuple. Native field packets stay private. |
| `Resource = Client|PoolClient` / mysql2 `Connection` | Now adapter-owned bounded pools and exclusive leases. No native escape hatch; standalone driver use remains possible outside Stackpress. |
| `Connector = Resource | (() => Promise<Resource>)` | Replace lazy cached factory with eager validated open; accepted G24. Pool transaction affinity is explicit, not dependent on callback returning one client. |
| `PGConnection`, `Mysql2Connection`, named/default `connect` | Now `PostgresDatabase::open(PostgresOptions)` and `MysqlDatabase::open(MysqlOptions)` implementing the existing traits, wrapped in Engine. No JS default export alias. |
| Reexport Inquire builders/dialects/types from facade | Retain existing `stackpress::sql`; add optional `sql_postgres` / `sql_mysql`. No redundant adapter-wide reexport surface. |
| `.format`, PostgreSQL `?`→`$n`, protected `??`, arity | Retain current renderer-owned placeholder grammar and dialect tag; now independently validate native prepared metadata. Never perform a second string replacement in the driver. |
| `.query` normalized rows | Now bounded strict owned rows; no silent discard of unsupported cell types or native command errors. |
| `.raw` native result | Intentionally not ported; G24 already chose explicit typed execution. Trusted SQL still uses normal validation and result limits. |
| `.resource` lazy cached native access | Intentionally not ported; could bypass admission/transaction ownership/close. S4-X can propose a reviewed escape hatch. |
| `.transaction(callback)` begin/commit/rollback | Now existing exclusive, non-Clone transaction/session and consuming commit/rollback. Callback ergonomics/savepoints/isolation options remain S4-X. |
| `lastId` getter / `_lastId` connection cache | Replace with statement-local QueryResult.insert_id. PostgreSQL always None; use RETURNING. MySQL zero means None, checked i64 conversion, never inherit an earlier ID. |
| `before` mutating connection hook / Engine short-circuit hook | Not ported under G24. Observation cannot rewrite SQL or supply fabricated rows. |
| Date→ISO, arrays/objects→JSON, missing values→[] | Preserve only explicit canonical UTC/JSON constructors and zero binds. No dynamic guessing/coercion. Native date/decimal codecs are S4-X. |
| PostgreSQL pgsql facade / MySQL facade | Now accepted Rust package names `stackpress-sql-postgres` / `stackpress-sql-mysql` with valid underscore imports. No legacy JS string imports. |
| PGlite Resource/connect/query/raw/transaction | S4-H, deployment/runtime not selected; PostgreSQL dialect/client support is not a PGlite proof. |
| better-sqlite3 synchronous resource and named/default connect | Existing SQLx SQLite contract remains; no rewrite in this slice. Shared cleanup extraction would require separate coordinator review. |
| Generated stores/actions/helpers/migration plugins reexported upstream | Slice 6/task 00011, not transport/driver scope. Adapters run generated Query values through the existing Engine contract. |

## Individual PostgreSQL tests

All nine source tests have a planned disposition:

| Source test behavior | Disposition / required proof |
| --- | --- |
| Query without question mark but one value throws | Retain renderer arity rejection; native metadata mismatch also rejects before execution. |
| Date object converted to ISO | Replace with explicit `SqlValue::utc_text`; DateTime schema currently stores VARCHAR(30), not native timestamp. |
| Array converted to JSON | Replace with explicit WriteValue::Json; bounded JSON/JSONB round trip. |
| Nested object converted to JSON | Same explicit JSON; preserve nested types and JSON null versus SQL NULL. |
| Multiple placeholders but one value throws | Retain renderer/native prepared arity validation including gaps/casts/quoted literals. |
| Query returns no rows | Now empty owned rows and correct affected count; no inherited insert ID. |
| Callback commits successful transaction | Now exclusive transaction with persisted DML after consuming commit. |
| BEGIN throws; rollback attempted and original error retained | Now retire uncertain lease. Keep original cause and cleanup failure separately; do not reuse a connection just because rollback was attempted. |
| No values; `_query` returns native result | Now zero-bind statement succeeds, public QueryResult only. Native resource/command shape intentionally not exported. |

## Individual MySQL tests

All ten source tests have a planned disposition:

| Source test behavior | Disposition / required proof |
| --- | --- |
| Positive `_lastId` getter | Replace with statement-local ID; consecutive insert/update/select proves no sticky value. |
| Request has no `values` property | Now a zero-bind Query; explicit arity rejects a missing required bind. |
| Date converted to ISO | Replace with explicit canonical UTC text; no host-timezone inference. |
| Empty array becomes `[]` JSON text | Explicit JSON value; empty array distinct from SQL NULL. |
| Nested object becomes JSON text | Explicit validated JSON, native JSON round trip. |
| Non-array result header sets `_lastId` | Statement-local native completion packet only, i64 overflow failure; no arbitrary JS shape inspection. |
| Empty native array yields empty rows | Now empty result with statement-local affected count and no stale rows. |
| Successful callback commits | Now exclusive same-connection transaction, acknowledged commit. |
| BEGIN throws and rollback runs | Retire lease and retain primary/cleanup errors; no promise that an unstarted transaction accepts ROLLBACK. |
| Lowercase `isarray` branch returns Results | Intentionally do not reproduce source typo/array-shape bug. Decode ordered native rows using the driver's typed protocol. |

## Current Rust contract inherited without redesign

| Exact current surface | Slice 7 obligation |
| --- | --- |
| `Database: Send + Sync + 'static`, `SqlFuture<'a,T>: Send` | Both adapters satisfy these under real handlers and multithread runtime; no lifetime widening, unsafe impl or mutex held across await. |
| `dialect`, `run(Query,QueryLimits)`, `begin(TransactionOptions)`, `close` | Preserve signatures; validate direct trait calls as well as Engine calls. Eager open, bounded acquisition/admission, close re-awaitable after cancellation. |
| `TransactionSession: Send + 'static`, exclusive `run(&mut self)` | One lease through every await; failed/cancelled run stays rollback-only. No cloning/aliasing or implicit current transaction. |
| Consuming `commit` / `rollback` -> static future | Own session/lease inside future from construction; dropping before first poll still initiates retirement. |
| SQLite Lease permit coupled to uncertain connection close | Match conservatively for network pools, including normal error, BEGIN and COMMIT cancellation; no uncertain lease recycle. |
| Query validation/dialect/arity, trusted raw SQL audit | Preserve validation before effects; SQLx AssertSqlSafe is an audit marker, never a sanitizer. Prepared metadata read on the executing lease. |
| QueryLimits: rows, bytes and construction limits | Charge names/values before retained copies; fail whole result on overrun. Driver packet allocations remain an explicit limit, not hidden by our budget. |
| SqlValue Null/Bool/i64/finite f64/Text/Blob, Row ordered duplicate names | Preserve strict mapping and column order; checked unsigned conversion and no lossy native coercions. |
| Existing ErrorKind and SqlError source/cleanup | Reuse categories; retain native typed source with operation-specific wrapper text. No driver message interpolation (#11). |

## Existing renderer network harness

`crates/sql/sql/tests/network.rs` and `network/cases.rs` already gate on
`STACKPRESS_TEST_POSTGRES_URL` / `STACKPRESS_TEST_MYSQL_URL`. Preserve its explicit
ignored reporting and one uniquely owned database per configured backend.
Its eight cases are create_types_defaults, crud_returning, joins,
expressions_pagination, json_paths_membership, alter_fields_constraints,
schema_diff_rename and table_commands. All remain renderer tests; run equivalent
real Engine/adapter paths additionally. Native SELECT-only decoding in that
harness is insufficient for general production codecs/lease guarantees.

Database creation/drop is test-fixture-only. Open never creates a database,
changes a role/global setting, migrates schema, seeds data or installs extensions.
