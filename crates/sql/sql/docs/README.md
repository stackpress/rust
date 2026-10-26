# Stackpress SQL

`stackpress-sql` builds owned SQL descriptions, renders SQLite/PostgreSQL/MySQL statements,
and defines explicit execution and transaction contracts. Build and inspect queries without a
database or async runtime; choose a database adapter when you need IO.

These pages describe the public API in this checkout. The workspace uses Rust 1.96 and edition
2024, with `publish = false`. Use local path dependencies. The core has no normal dependencies
and no crate feature flags; direct imports use `stackpress_sql`. The optional workspace facade
exposes `stackpress::sql` with feature `sql`.

## Start here

Follow [Getting started](getting-started.md) to print SQL and inspect a separate bind list.
Then read [Concepts](concepts.md) for ownership, dialect support, strict values, and execution.
Neither step requires a running database.

## Choose a task

The guides start with the result you want. Detailed fields, parameters, and failure behavior
live in the [API reference](api/README.md).

| I want to… | Read |
| --- | --- |
| Join tables and compose filters | [Build a query](guides/build-queries.md) |
| Insert defaults or safely update/delete rows | [Write rows](guides/write-rows.md) |
| Store JSON and match a path or array member | [Query JSON](guides/query-json.md) |
| Turn owned results into application data | [Decode rows](guides/decode-rows.md) |
| Compose work on one transaction | [Execute and complete transactions](guides/transactions.md) |
| Compare explicit schemas and install a plan | [Plan schema changes](guides/schema-changes.md) |
| Classify failures and lower resource budgets | [Handle errors and limits](guides/errors-and-limits.md) |

## Run the existing examples

Run `cargo run -p stackpress-sql --example <name>` from the repository root. These programs
are the canonical larger examples; the docs link to their implementation rather than copying it.

| Example | What it demonstrates |
| --- | --- |
| [build_inspect](../examples/build_inspect.rs) | All six builder families, predicates, JSON, SQL and binds; no database. |
| [capabilities](../examples/capabilities.rs) | Every renderer flag and rejected MySQL RETURNING. |
| [schema_diff](../examples/schema_diff.rs) | Explicit snapshots and deliberate rename reconciliation. |
| [execute_plan](../examples/execute_plan.rs) | Ordered plan outcomes and transaction ownership with a recording adapter, not native storage. |

For actual SQLite storage, continue in the [SQLite adapter docs](../../sql-sqlite/docs/README.md).
The [sql-profile application](../../../../examples/sql-profile/README.md) owns its server service
and domain operations. PostgreSQL/MySQL have renderers and opt-in native validation tests;
production network adapters remain deferred.

Source authority: [public exports and Rustdoc](../src/lib.rs), [contract tests](../tests/), and
[runnable examples](../examples/). Markdown examples are checked separately from Rustdoc doctests.
