# Engine

`Engine` owns one Database adapter and a copied per-statement policy. Source:
[engine.rs](../../src/engine.rs). It is deliberately non-Clone; explicit `Arc<Engine>` sharing
keeps ownership visible.

```rust
use stackpress_sql::{Engine, QueryResult, Result, Select};

async fn read_profile(engine: &Engine, id: i64) -> Result<QueryResult> {
    let lookup = Select::new(["id", "name"])
        .from("profile")
        .where_eq("id", id);
    engine.execute(&lookup).await
}
```

The helper executes explicitly; the builder retains no engine. Use a concrete native adapter
from the [SQLite docs](../../../sql-sqlite/docs/README.md) or the existing recording
[execute_plan example](../../examples/execute_plan.rs) for a runnable orchestration fixture.

Contents: [Ownership](#1-construction-and-policy) · [Builders](#2-detached-builder-conveniences) ·
[Execution](#3-execution-and-completion) · [Diff](#4-schema-comparison)

## 1. Construction and policy

Engine is Send + Sync and keeps its adapter and limits private. It exposes no raw mutable connection, implicit
transaction context, middleware, or automatic retry behavior.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(database)` | `impl Database` | Take adapter ownership with default query limits. |
| `with_limits(self, limits)` | `QueryLimits` | Replace policy before sharing. |
| `dialect(&self)` | none | Inspect the adapter identity without acquisition. |

**Returns** `new` / `with_limits` return Self; `dialect` returns DialectId. Construction itself
does not open an adapter; the adapter's own construction decides eager native open. There is
no Engine getter for its current limits. Limits are copied into each begun transaction.

## 2. Detached builder conveniences

All methods below take `&self` and perform no IO. Their builders are usable with any compatible
engine or explicit transaction later.

| Method | Parameters | Result |
| --- | --- | --- |
| `select(columns)` | `impl IntoIterator<Item = impl Into<Selector>>` | Select |
| `insert(table)` | `impl Into<Identifier>` | Insert |
| `update(table)` | `impl Into<Identifier>` | Update |
| `delete(table)` | `impl Into<Identifier>` | Delete |
| `create(table)` | `impl Into<String>` | Create |
| `alter(table)` | `impl Into<String>` | Alter |
| `drop_table(table)` | `impl Into<String>` | TableCommand::Drop |
| `rename_table(from, to)` | two `impl Into<String>` | TableCommand::Rename |
| `truncate(table)` | `impl Into<String>` | TableCommand::Truncate |

**Returns** The corresponding detached builder/command with the same defaults as its constructor.
For example, `engine.update("profile").set("active", false).where_eq("id", 7)` still needs an
explicit `engine.execute(&update).await`; construction alone never captures a session.

## 3. Execution and completion

Execution methods borrow an Engine. Per-statement input validation happens before Database IO;
output collection, native arity, admission, and shutdown belong to the adapter.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `execute(&self, query)` | `&(impl Render + Sync)` | Render/validate/run one statement. |
| `fetch(&self, query)` | `&(impl Render + Sync)` | Run one statement and keep only its rows. |
| `execute_plan(&self, plan)` | `&QueryPlan` | Validate all members, own one transaction, then commit. |
| `begin(&self)` | none | Acquire one exclusive ordinary transaction. |
| `close(&self)` | none | Reject new work and wait for leases through the adapter. |

**Returns** Awaited execute returns `Result<QueryResult>`; fetch returns `Result<Vec<Row>>`;
execute_plan returns `Result<Vec<QueryResult>>` in statement order; begin returns
`Result<Transaction>`; close returns `Result<()>`. Normal outcomes own their data after lease
release. Begin uses default TransactionOptions; no isolation/savepoint option is exposed here.

Plan validation precedes acquisition. Transactional-DDL capability is required, so MySQL is
rejected. The first statement failure rolls back and preserves primary plus cleanup failure;
success commits before return. A commit error/cancellation can leave an unknown outcome.
Budgets are per statement, and the outcome vector retains all member results.

No write is retried. Cancellation can lose observation of already-executed work. close may be
re-awaited after cancellation under the Database contract; an adapter's Drop is not an awaited
shutdown certificate. See [transactions](transaction.md) and the
[execution guide](../guides/transactions.md).

## 4. Schema comparison

`diff(&self, source: &TableDefinition, target: &TableDefinition)` delegates to
`SchemaDiff::between` without IO, catalog reads, or rename inference.

**Returns** `Result<SchemaDiff>`, with the same definition/order errors as the
[SchemaDiff API](schema-diff.md). This method does not check the snapshots against the adapter's
current database or execute their difference.
