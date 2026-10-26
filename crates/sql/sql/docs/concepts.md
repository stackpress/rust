# How SQL descriptions become owned results

An application needs to control both the statement it sends and the connection that executes
it. The core keeps those decisions separate: describe data, render for a dialect, then execute
through an adapter. The [first query](getting-started.md) anchors that flow without IO.

Contents: [Builders](#1-owned-builders) · [Dialects](#2-dialects-and-capabilities) ·
[Values](#3-values-and-identifiers) · [Rows](#4-owned-rows-and-decoding) ·
[Execution](#5-engine-and-transaction-ownership) · [Plans](#6-executing-a-plan) ·
[Schema](#7-explicit-schema-snapshots) · [Trust](#8-trusted-sql) · [Limits](#9-finite-work)

## 1. Owned builders

`Select`, `Insert`, `Update`, `Delete`, `Create`, and `Alter` retain their own data. Builder
methods consume and return the builder; cloning creates independent descriptions. They capture
no engine or live connection, and cannot be awaited. Engine convenience constructors return
the same detached types.

For example, `Select::new(["id"]).from("profile").limit(5)` is only a description. Its `build()`
borrows `SelectData`; the DDL builders instead return fallible normalized snapshots. Most
validation happens during rendering, so successful construction alone does not prove validity.

`Render::render(dialect)` returns one `Query` for reads, writes, raw SQL, and table commands.
`Create::render` and `Alter::render` are inherent methods returning an ordered `QueryPlan`,
because a table and its indexes can require several statements.

## 2. Dialects and capabilities

A dialect selects quoting, placeholders, supported operations, and native representation rules.
It does not install a driver or probe a server version. Inspect `DialectId::capabilities()`
before offering dialect-dependent features; rendering still validates the complete request.

| Capability | SQLite | PostgreSQL | MySQL |
| --- | --- | --- | --- |
| DML RETURNING | yes | yes | no |
| Transactional DDL / `execute_plan` | yes | yes | no |
| Native field/type/key/FK alteration | no | yes | yes |
| FULL OUTER JOIN | yes | yes | no |
| DROP/TRUNCATE CASCADE | no | yes | no |
| CREATE INDEX IF NOT EXISTS | yes | yes | no |
| Distinct JSON integer/real equality | yes | no | no |
| Text containing NUL | yes | no | yes |

SQLite RETURNING needs SQLite 3.35 or later; RIGHT/FULL JOIN needs 3.39 or later. MySQL puts
indexes inline for idempotent table creation. Operation-specific restrictions still apply,
including SQLite ADD COLUMN restrictions and network DDL attributes. See [dialects](api/dialects.md).

Only the [SQLite adapter](../../sql-sqlite/docs/README.md) supplies production native execution
here. PostgreSQL/MySQL native tests validate renderers under opt-in fixtures; they establish
neither production adapters nor general platform support.

## 3. Values and identifiers

A bind is data in a separate list. An identifier names a table, column, or alias and is validated
and quoted by the renderer. Ordinary strings such as `"profile.id"` and `"count(*)"` are literal
names; use `Identifier::qualified(["profile", "id"])` for qualification and typed `Expr` /
`Selector` constructors for expressions. Only the exact selector string `"*"` is a wildcard.

`SqlValue` preserves NULL, boolean, signed i64 integer, finite f64 real, text, and binary bytes.
Unsigned u64 conversion is fallible outside i64 range. Strings are never guessed to be dates,
numbers, or JSON. Explicit `utc_text` validates a canonical UTC timestamp; `json` encodes typed
JSON as text. False, zero, empty text, SQL NULL, an omitted column, and `WriteValue::Default`
remain different choices. Nonfinite reals fail validation rather than becoming zero or NULL.

Typed JSON paths keep keys and array indices separate. `JsonScalar::Null` means JSON null;
a missing path and SQL NULL do not match it. Membership searches for a scalar directly in an
array at that path. SQLite can distinguish JSON number 1 from 1.0; PostgreSQL/MySQL compare
them equally. See [Query JSON](guides/query-json.md).

## 4. Owned rows and decoding

`QueryResult` holds statement-local `rows`, `affected`, and optional `insert_id`; no mutable
engine-wide last ID exists. Rows own names, native values, and optional declared-type metadata
after their normal connection lease releases. Metadata is diagnostic and never forces coercion.

A joined row may contain two columns named `id`. `try_get("id")` rejects ambiguity;
`try_get_at(0)` selects an ordinal, and aliases avoid the conflict. Built-in `FromSqlValue`
implementations decode strict storage types and checked integer ranges. Boolean decoding accepts
Bool or integer 0/1, and `Option<T>` maps SQL NULL to None. Define application DTOs by combining
these lookups; the crate does not generate them.

## 5. Engine and transaction ownership

`Engine` owns a `Database` and finite query limits. It is non-Clone; share it explicitly using
`Arc<Engine>` when needed. `execute` renders and validates before adapter IO, and `fetch` returns
only the owned rows from that outcome. Adapters own preparation/arity, native binding, leases,
output collection, and shutdown.

`begin()` acquires one exclusive `Transaction`. Keep it local and pass `&mut Transaction` to
composed async functions, retaining the same session across await. `commit(self)` and
`rollback(self)` consume it. Calling another engine method does not join that transaction;
nested server dispatch has no implicit transaction context.

Dropping an uncommitted owner initiates adapter rollback or retirement; it does not synchronously
acknowledge cleanup. A cancelled write may already have executed. Commit cancellation can leave
an unknown outcome, so automatic retries are forbidden. The [SQLite ownership and cancellation
docs](../../sql-sqlite/docs/concepts.md#cancellation-loses-observation) own native lifecycle details.

## 6. Executing a plan

`Engine::execute_plan` validates every statement before acquiring a session, checks transactional
DDL support, then executes in one transaction. Its result vector follows query order. The first
statement failure stops execution and rolls back, retaining the primary error and any secondary
rollback error. Successful execution completes commit before returning outcomes.

`Transaction::execute_plan` uses the caller's existing session and leaves completion to that
caller, including after an error or cancellation. This lets one operation combine schema plans
and ordinary statements. Both forms reject MySQL before executing plan statements. Use the
[recording example](../examples/execute_plan.rs) to inspect this orchestration without storage.

## 7. Explicit schema snapshots

`TableDefinition` describes supplied fields, ordered primary keys, indexes, and foreign keys.
`SchemaDiff` compares two such snapshots for the same logical table. It does not read a native
catalog, infer renames, track applied migrations, or move physical column positions.

A field renamed from `name` to `display_name` first appears as removal plus addition. Call
`rename_field("name", "display_name")` on the diff to reconcile the pair deliberately. A property
change survives as a separate change. A no-op is `SchemaDiff::NoChanges`, whose render returns
None. Unsupported SQLite type/key/FK changes fail before exposing a plan; reconstruction and
migration history remain outside this crate. See [Plan schema changes](guides/schema-changes.md).

## 8. Trusted SQL

`Sql::literal` accepts static caller-authored SQL; `Sql::trusted` takes a `TrustedSql::audited`
marker for dynamic trusted code. This marker asserts authorship and auditing. It neither
sanitizes request text nor makes arbitrary input safe. Values still belong in binds.

The raw statement gate accepts one ordinary statement, rejects transaction/connection control,
batches, and NUL, and has a deliberately limited lexical subset. It is not a complete native
SQL parser or sandbox. Trusted expressions, type syntax, and defaults retain dialect tags and
have separate consuming checks. See [raw SQL](api/query.md#3-raw-statements-and-trust).

## 9. Finite work

Default per-statement budgets are 1 MiB SQL, 1,000 binds, 10,000 retained rows, and 16 MiB of
retained name/value/declaration bytes. `Engine::with_limits` can lower them; raising a limit
cannot bypass the renderer's construction budgets. Adapters must reject output overflow while
collecting, without publishing partial results.

Expressions, JSON documents, paths, joins, and schema operations have additional structural
limits. A complete plan can retain several statements and outcomes; these are not process-wide
heap/stack, native VM memory, or query-time quotas. Caller-constructed recursive values and their
derived cloning/destruction also sit outside a global resource guarantee.

You can now follow a description through dialect validation, explicit execution, and owned
results. Choose a [task guide](README.md#choose-a-task) next, or look up the exact
[public surface](api/README.md) you need.
