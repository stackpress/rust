# Alter and explicit operations

`Alter` retains an ordered structural change request with optional source field knowledge.
Source: [ddl.rs](../../src/ddl.rs); network rendering has additional native constraints.

```rust
use stackpress_sql::{Alter, AlterOperation, DialectId, Field, Index, Result, SqlType};

fn main() -> Result<()> {
    let alter = Alter::new("profile")
        .source_field(Field::new("name", SqlType::Text))
        .rename_field("name", "display_name")
        .add_field(Field::new("nickname", SqlType::Text))
        .operation(AlterOperation::AddIndex(Index::new(
            "profile_nick",
            ["nickname"],
        )));
    assert_eq!(alter.build()?.operations.len(), 3);
    assert_eq!(alter.render(DialectId::Sqlite)?.queries().len(), 3);
    Ok(())
}
```

Source fields are caller-supplied metadata, not native introspection. The returned plan exposes
nothing until the complete operation set has rendered successfully.

Contents: [Builder](#1-alter) · [Data and operations](#2-alterdata-and-alteroperation) ·
[Native limits](#3-native-limits)

## 1. Alter

Alter, AlterData, and AlterOperation are Clone/Debug/PartialEq. Operations are explicit;
`new` starts empty and must gain an operation before build/render.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(table)` | `impl Into<String>` | Name a table. |
| `source_field(self, field)` | `Field` | Append original field metadata. |
| `operation(self, operation)` | `AlterOperation` | Append any supported structural operation. |
| `add_field(self, field)`, `change_field(self, field)` | `Field` | Append addition or attribute change. |
| `remove_field(self, name)` | `impl Into<String>` | Append removal. |
| `rename_field(self, from, to)` | two `impl Into<String>` | Rename directly or reconcile pending remove/add. |
| `add_index(self, index)` | `Index` | Append an index. |
| `remove_index(self, name)` | `impl Into<String>` | Drop a named regular/unique index. |
| `primary_key(self, columns)` | `impl IntoIterator<Item = impl Into<String>>` | Append AddPrimary. |
| `remove_primary_key(self)` | none | Append RemovePrimary. |
| `add_foreign_key(self, foreign)` | `ForeignKey` | Append a named relationship. |
| `remove_foreign_key(self, name)` | `impl Into<String>` | Append named constraint removal. |
| `build(&self)` | none | Validate/normalize operations and rename state. |
| `render(&self, dialect)` | `DialectId` | Render the complete ordered plan. |

**Returns** Constructors/modifiers return Self; `build` returns `Result<AlterData>`;
`render` returns `Result<QueryPlan>`. Rendering is inherent, not Render. Empty or over-1,000
operation lists, invalid names, conflicting/incomplete renames, inconsistent metadata/defaults,
and unsupported native semantics return errors before IO.

## 2. AlterData and AlterOperation

AlterData has public `table: String`, `source_fields: Vec<Field>`, and
`operations: Vec<AlterOperation>`. Build returns an independent normalized snapshot.

| AlterOperation variant | Payload |
| --- | --- |
| `AddField`, `ChangeField` | `Field` |
| `RemoveField` | `String` |
| `RenameField` | `{ from: String, to: String }` |
| `AddIndex` | `Index` |
| `RemoveIndex` | `String` |
| `AddPrimary` | `Vec<String>` |
| `RemovePrimary` | none |
| `AddForeignKey` | `ForeignKey` |
| `RemoveForeignKey` | `String` |

Rename reconciliation requires both pending remove/add halves or neither, with known source
metadata. Changed field properties survive as ChangeField. Declared validation cannot infer
all keys, foreign constraints, or nullability from an existing database.

## 3. Native limits

SQLite supports native add/drop/rename column and index operations. ChangeField, primary changes,
and foreign-key changes require reconstruction and return Unsupported. Native ADD rejects
auto-increment, timestamp/trusted defaults, and NOT NULL without a non-NULL default. Actual
native constraints can still reject an otherwise valid drop.

Network ChangeField requires source metadata to distinguish defaults, nullability, identity,
and comments. PostgreSQL source-field lookup preserves quoted case. Changing identity policy is Unsupported; adding identity fields also requires a
coordinated key definition. Primary removal assumes the conventional `<table>_pkey` name, so
arbitrary external naming requires explicit migration design. MySQL uses native field/constraint
operations, with [schema limits](schema.md), and its DDL is not atomic through execute_plan.

SET NULL must remain compatible with known local field nullability and primary effects.
Dropping a primary alone does not remove native NOT NULL; restoring nullable state needs an
explicit change. Physical reordering and SQLite reconstruction remain outside this API.

Related: [schema diff](schema-diff.md), [schema guide](../guides/schema-changes.md),
[QueryPlan](query-plan.md), [Engine](engine.md).
