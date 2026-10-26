# Update and mutation data

`Update` requires assignments and an explicit filter or all-row choice. Source:
[dml.rs](../../src/dml.rs).

```rust
use stackpress_sql::{DialectId, Expr, Render, Result, Update};

fn main() -> Result<()> {
    let update = Update::new("profile")
        .set("name", "Ada")
        .where_eq("id", 7)
        .where_eq("active", true)
        .and_where(Expr::eq("team_id", 3))
        .returning_all();
    assert_eq!(update.build().filters.len(), 3);
    assert_eq!(update.assignments().len(), 1);
    assert_eq!(update.render(DialectId::Sqlite)?.binds().len(), 4);
    let all = Update::new("profile")
        .set("active", false)
        .all_rows()
        .returning(["id"]);
    assert!(all.build().all_rows);
    Ok(())
}
```

Unlike Select, repeated mutation `where_eq` appends predicates. Setting all_rows permits an
unfiltered mutation but does not discard filters already present.

## 1. Update operations

Update is Clone/Debug/PartialEq. Consuming modifiers append unless described as replacing.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(table)` | `impl Into<Identifier>` | Set destination; no all-row permission. |
| `build(&self)` | none | Borrow common mutation data. |
| `assignments(&self)` | none | Inspect ordered assignments. |
| `set(self, column, value)` | `impl Into<String>`, `impl Into<WriteValue>` | Append an assignment. |
| `where_eq(self, column, value)` | `impl Into<Identifier>`, `impl Into<SqlValue>` | Append equality. |
| `and_where(self, expression)` | `Expr` | Append predicate. |
| `where_json(self, column, path, value)` | `impl Into<Identifier>`, `JsonPath`, `JsonScalar` | Append exact JSON equality. |
| `where_json_contains(self, column, path, value)` | same types | Append scalar array membership. |
| `all_rows(self)` | none | Authorize missing filters. |
| `returning(self, selectors)` | `impl IntoIterator<Item = impl Into<Selector>>` | Replace RETURNING list. |
| `returning_all(self)` | none | Return wildcard. |
| `render(&self, dialect)` | `DialectId`, through Render | Validate and render. |

**Returns** Constructors/modifiers return Self; `build` returns `&MutationData`;
`assignments` returns `&[(String, WriteValue)]`; `render` returns `Result<Query>`.
No assignments, repeated assignment names, unfiltered scope without all_rows, invalid expressions,
and resource overflow fail at render. SQLite rejects DEFAULT assignments; MySQL rejects RETURNING.

## 2. MutationData

This Clone/Debug/PartialEq type is shared by Update and Delete. It has no public inherent
operations. The public fields are:

| Field | Type | Meaning |
| --- | --- | --- |
| `table` | `Identifier` | Target relation. |
| `filters` | `Vec<Expr>` | Ordered AND predicates. |
| `all_rows` | `bool` | Explicit permission for an unfiltered mutation. |
| `returning` | `Vec<Selector>` | Ordered returned projections. |

Assignments belong to Update's separate `assignments()` view. `build()` is an immutable borrow,
so mutating a cloned snapshot does not change its builder.

Related: [Delete](delete.md), [WriteValue](insert.md#2-writevalue),
[write guide](../guides/write-rows.md), [JSON](json.md).
