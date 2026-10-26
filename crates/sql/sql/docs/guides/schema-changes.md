# Compare schemas and inspect a change plan

A schema diff needs two explicit storage definitions. Keep those snapshots under application
control, inspect the difference, and choose deliberately whether to execute it.

## Add a nullable field

Build the original snapshot, append the new field to a clone, then render the difference. This
program checks the plan without opening a database:

```rust
use stackpress_sql::{Create, DialectId, Field, Result, SqlType};

fn main() -> Result<()> {
    let source = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer))
        .definition()?;
    let mut target = source.clone();
    target.fields.push(Field::new("nickname", SqlType::Text));
    let diff = source.diff(&target)?;
    let plan = diff
        .render(DialectId::Sqlite)?
        .expect("added field changes the schema");
    assert_eq!(plan.queries().len(), 1);
    assert!(plan.queries()[0].sql().contains("ADD COLUMN"));
    assert!(source.diff(&source)?.render(DialectId::Sqlite)?.is_none());
    Ok(())
}
```

`definition()` normalizes the CREATE description; `diff` compares snapshots without querying
native state. The new field appends physically and is nullable, which SQLite can add natively.
The no-op case returns None rather than an empty statement.

## Reconcile renames and unsupported changes

A removed field and added field are not automatically a rename. Use `SchemaDiff::rename_field`
with one pending pair; the existing [schema_diff example](../../examples/schema_diff.rs) walks
through this choice. Property changes remain separate operations after reconciliation.

Reordering existing columns and inserting a new field between existing ones require physical
reconstruction. SQLite type/nullability/key/FK changes also return Unsupported before any plan
is exposed. The core does not reconstruct tables, infer external constraints, track revisions,
or decide whether a destructive change is acceptable.

## Execute only after review

Render for `engine.dialect()`, then pass the plan to `Engine::execute_plan` for validation and
one transaction. Outcomes follow statement order. Use `Transaction::execute_plan` when a caller
already owns completion; see [transactions](transactions.md). MySQL plans can be inspected but
atomic plan execution is rejected because its DDL can commit implicitly.

You can now separate snapshot comparison, plan inspection, and execution. See
[schema diff](../api/schema-diff.md), [Create](../api/create.md), [Alter](../api/alter.md), and
[QueryPlan](../api/query-plan.md) for normalization and native limits.
