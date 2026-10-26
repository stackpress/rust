# Table definitions and schema differences

`TableDefinition` is an explicit storage snapshot; `SchemaDiff` compares two snapshots without
native introspection. Source: [diff.rs](../../src/diff.rs).

```rust
use stackpress_sql::{Create, DialectId, Field, Result, SchemaDiff, SqlType};

fn main() -> Result<()> {
    let source = Create::new("profile")
        .add_field(Field::new("name", SqlType::Text))
        .definition()?;
    let normalized = source.normalized()?;
    assert_eq!(normalized.create().definition()?, source);
    assert_eq!(
        SchemaDiff::between(&source, &source)?,
        SchemaDiff::NoChanges
    );
    let mut target = source.clone();
    target.fields[0].name = "display_name".into();
    let diff = source.diff(&target)?.rename_field("name", "display_name")?;
    assert!(diff.alteration().is_some());
    assert_eq!(diff.render(DialectId::Sqlite)?.unwrap().queries().len(), 1);
    Ok(())
}
```

A rename is a deliberate reconciliation of one removed/added pair. The canonical larger
[schema_diff example](../../examples/schema_diff.rs) prints operations and SQL.

Contents: [Definition](#1-tabledefinition) · [Difference](#2-schemadiff) · [Limits](#3-comparison-boundaries)

## 1. TableDefinition

The type is Clone/Debug/PartialEq. Public fields are `name: String`, `fields: Vec<Field>`,
`primary: Vec<String>`, `indexes: Vec<Index>`, and `foreign_keys: Vec<ForeignKey>`.
It does not retain CREATE idempotence or any applied-state ledger.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `create(&self)` | none | Rebuild a detached CREATE description. |
| `normalized(&self)` | none | Validate/normalize a separate snapshot. |
| `diff(&self, target)` | `&TableDefinition` | Compare explicit source and target. |
| `Create::definition(&self)` | none | Produce a normalized snapshot from a builder. |

**Returns** `create` returns Create without IO; `normalized` and `definition` return
`Result<TableDefinition>`; `diff` returns `Result<SchemaDiff>`. Normalization validates names,
defaults, references, and same-name index precedence/conflicts.

## 2. SchemaDiff

SchemaDiff is Clone/Debug/PartialEq. Variants are `NoChanges` and
`Changes { source: Box<TableDefinition>, target: Box<TableDefinition>, alteration: Box<Alter> }`.
Its public data can be inspected; rendering still validates the alteration and field order.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `between(source, target)` | two `&TableDefinition` | Compare all declared storage properties. |
| `alteration(&self)` | none | Borrow the normalized change builder if present. |
| `rename_field(self, from, to)` | two `&str` | Reconcile exactly one pending removal/addition. |
| `render(&self, dialect)` | `DialectId` | Render the complete difference, if any. |

**Returns** `between` and `rename_field` return `Result<SchemaDiff>`; `alteration` returns
`Option<&Alter>`; `render` returns `Result<Option<QueryPlan>>`. NoChanges returns None.
Rename errors include absent halves, no-op differences, missing source metadata, or conflicts.
Changed properties remain a separate ChangeField after rename reconciliation.

## 3. Comparison boundaries

Definitions must describe the same logical table. Field properties compare structurally;
compound key/index order matters. Dependent constraints are removed before field changes and
added afterward. Existing-column reordering is Unsupported. New columns append physically;
a requested middle insertion fails rendering unless an explicit rename preserves that ordinal.

This is not a native catalog diff or a migration runner. It infers no renames or applied state,
and reconstructs no SQLite tables. Unsupported type/key/FK changes fail before returning a plan.
Caller-provided source knowledge cannot establish arbitrary external constraint names or history.

Related: [Alter](alter.md), [schema values](schema.md), [schema guide](../guides/schema-changes.md),
[Engine::diff](engine.md).
