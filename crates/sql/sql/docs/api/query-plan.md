# Ordered query plans

`QueryPlan` owns validated, ordered statements from Create, Alter, or SchemaDiff. Source:
[ddl.rs](../../src/ddl.rs).

```rust
use stackpress_sql::{Create, DialectId, Field, Index, QueryLimits, Result, SqlType};

fn main() -> Result<()> {
    let plan = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer))
        .add_index(Index::new("profile_id", ["id"]))
        .render(DialectId::Sqlite)?;
    plan.validate(DialectId::Sqlite, QueryLimits::default())?;
    assert_eq!(plan.queries().len(), 2);
    assert!(plan.queries()[1].sql().starts_with("CREATE INDEX"));
    Ok(())
}
```

The plan retains one bind list per query. It neither acquires a connection nor installs schema.

## 1. Inspection and validation

QueryPlan is Clone/Debug with private contents. There is no public arbitrary-vector constructor
or mutating query accessor; produce it through a schema builder/diff.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `queries(&self)` | none | Inspect exact statement order. |
| `validate(&self, dialect, limits)` | `DialectId`, `QueryLimits` | Validate every statement before effects. |

**Returns** `queries` returns `&[Query]`; `validate` returns `Result<()>`. Validation rejects
empty plans, mixed/mismatched dialects, invalid value representation, or exceeded input limits.
It does not check prepared arity or execute native grammar. Builder inputs bound fields and
operations; QueryPlan adds no shared statement-count or total-byte quota.

## 2. Execution ownership

`Engine::execute_plan(&plan)` validates the whole plan before acquisition, requires transactional
DDL, owns one transaction, and returns `Result<Vec<QueryResult>>` in query order. The first
statement error triggers rollback with primary/cleanup error preservation; success commits.
Cancellation drops its session, while a cancelled commit can still have an unknown outcome.

`Transaction::execute_plan(&plan)` uses the existing session and leaves completion to the caller,
including after failure or cancellation. Both reject MySQL before executing plan statements.
Budgets apply per member; the returned vector retains every outcome until dropped.

Related: [Engine](engine.md), [Transaction](transaction.md),
[recording example](../../examples/execute_plan.rs), [schema guide](../guides/schema-changes.md).
