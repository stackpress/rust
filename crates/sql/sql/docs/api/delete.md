# Delete

`Delete` describes a filtered deletion or an explicit all-row operation. Source:
[dml.rs](../../src/dml.rs).

```rust
use stackpress_sql::{Delete, DialectId, Expr, Render, Result};

fn main() -> Result<()> {
    let delete = Delete::new("profile")
        .where_eq("active", false)
        .where_eq("team_id", 3)
        .and_where(Expr::eq("archived", true))
        .returning(["id"]);
    assert_eq!(delete.build().filters.len(), 3);
    assert_eq!(delete.render(DialectId::Sqlite)?.binds().len(), 3);
    let all = Delete::new("profile").all_rows().returning_all();
    assert!(all.render(DialectId::Sqlite)?.sql().contains("RETURNING *"));
    Ok(())
}
```

Each equality appends. A deliberate all_rows choice is required when no predicates exist;
empty groups or invalid predicates cannot silently widen scope.

## 1. Operations

Delete is Clone/Debug/PartialEq. Its inspectable state is the shared
[MutationData](update.md#2-mutationdata).

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(table)` | `impl Into<Identifier>` | Set target without all-row permission. |
| `build(&self)` | none | Borrow mutation state. |
| `where_eq(self, column, value)` | `impl Into<Identifier>`, `impl Into<SqlValue>` | Append equality. |
| `and_where(self, expression)` | `Expr` | Append predicate. |
| `where_json(self, column, path, value)` | `impl Into<Identifier>`, `JsonPath`, `JsonScalar` | Append exact JSON equality. |
| `where_json_contains(self, column, path, value)` | same types | Append direct scalar array membership. |
| `all_rows(self)` | none | Permit a deletion without predicates. |
| `returning(self, selectors)` | `impl IntoIterator<Item = impl Into<Selector>>` | Replace returned projections. |
| `returning_all(self)` | none | Return wildcard. |
| `render(&self, dialect)` | `DialectId`, through Render | Validate and render. |

**Returns** Constructors/modifiers return Self; `build` returns `&MutationData`;
`render` returns `Result<Query>`. Missing filter permission, malformed predicates/names, limits,
or unsupported MySQL RETURNING return typed errors before IO. `all_rows` preserves any existing
filters; it only grants permission for their absence.

Related: [write guide](../guides/write-rows.md), [expressions](expressions.md),
[JSON](json.md), [Engine](engine.md), [table truncate](table-command.md).
