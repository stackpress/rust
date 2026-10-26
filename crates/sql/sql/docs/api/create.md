# Create and normalized creation data

`Create` describes a table, its constraints, and indexes. Its inherent render method returns a
plan rather than one Query. Source: [ddl.rs](../../src/ddl.rs) and [diff.rs](../../src/diff.rs).

```rust
use stackpress_sql::{Create, DialectId, Field, ForeignKey, Index, Result, SqlType};

fn main() -> Result<()> {
    let create = Create::new("profile")
        .add_field(Field::new("id", SqlType::Integer).not_null())
        .add_field(Field::new("team_id", SqlType::Integer))
        .primary_key(["id"])
        .add_index(Index::new("profile_team", ["team_id"]))
        .add_foreign_key(ForeignKey::new(
            "profile_team_fk",
            ["team_id"],
            "team",
            ["id"],
        ))
        .if_not_exists();
    assert!(create.build()?.if_not_exists);
    assert_eq!(create.definition()?.fields.len(), 2);
    assert_eq!(create.render(DialectId::Sqlite)?.queries().len(), 2);
    Ok(())
}
```

This only renders: the foreign table is not checked against a catalog. SQLite creates the table
then its index as separate ordered statements.

## 1. Create operations

Create and CreateData are Clone/Debug/PartialEq. Constructors/modifiers consume and return owned
descriptions, with no database effects.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(table)` | `impl Into<String>` | Name a table. |
| `add_field(self, field)` | `Field` | Append a field in storage order. |
| `primary_key(self, columns)` | `impl IntoIterator<Item = impl Into<String>>` | Replace ordered primary columns. |
| `add_index(self, index)` | `Index` | Append an index. |
| `add_foreign_key(self, foreign_key)` | `ForeignKey` | Append a named relationship. |
| `if_not_exists(self)` | none | Request idempotent table/index creation. |
| `build(&self)` | none | Validate/normalize independent creation data. |
| `definition(&self)` | none | Produce a storage snapshot for comparison. |
| `render(&self, dialect)` | `DialectId` | Render a complete ordered native plan. |

**Returns** Constructors/modifiers return Self. `build` returns `Result<CreateData>`;
`definition` returns `Result<TableDefinition>` without retaining the idempotence flag;
`render` returns `Result<QueryPlan>`. No Render trait import is required for this inherent method.

Validation checks bounded nonempty fields, duplicate names, default/type compatibility, key
membership, foreign-key shapes, and index conflicts. Rendering additionally checks native types,
annotations, auto-increment rules, and dialect lengths. InvalidQuery / Unsupported / Limit errors
occur before exposing a plan; native existence, privileges, and external constraints remain IO concerns.

## 2. CreateData

The normalized snapshot exposes these fields. Modifying it does not change its originating
builder; there is no public install-snapshot constructor.

| Field | Type | Meaning |
| --- | --- | --- |
| `table` | `String` | Literal table name. |
| `fields` | `Vec<Field>` | Ordered storage fields. |
| `primary` | `Vec<String>` | Ordered compound key. |
| `indexes` | `Vec<Index>` | Normalized regular/unique indexes. |
| `foreign_keys` | `Vec<ForeignKey>` | Named relationships. |
| `if_not_exists` | `bool` | Explicit idempotence request. |

## 3. Native differences

SQLite rejects unsigned/comments and exact Decimal; auto-increment requires a sole Integer
primary without a default. PostgreSQL emits native fields/constraints plus index and comment
statements. MySQL places indexes inline when if_not_exists is selected because CREATE INDEX
IF NOT EXISTS is unavailable. Idempotent creation does not verify an existing table's shape.

See [schema values](schema.md) for field-level limits and [dialects](dialects.md) for flags.
Run [build_inspect](../../examples/build_inspect.rs), or use [Engine::execute_plan](engine.md)
when execution should own the transaction.
