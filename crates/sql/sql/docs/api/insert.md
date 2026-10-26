# Insert and write values

`Insert` describes one default row or an explicit bounded row list. Source:
[dml.rs](../../src/dml.rs).

```rust
use stackpress_sql::{DialectId, Insert, Render, Result, SqlValue, WriteValue};

fn main() -> Result<()> {
    let insert = Insert::new("profile")
        .columns(["name", "active"])
        .values([WriteValue::from("Ada"), WriteValue::from(false)])
        .values([WriteValue::from("Grace"), WriteValue::Value(SqlValue::Null)])
        .returning(["name"]);
    assert_eq!(insert.build().rows.len(), 2);
    assert_eq!(insert.render(DialectId::Postgres)?.binds().len(), 4);
    let defaults = Insert::new("profile").default_values().returning_all();
    assert!(
        defaults
            .render(DialectId::Sqlite)?
            .sql()
            .contains("DEFAULT VALUES")
    );
    Ok(())
}
```

Each `values` call appends one row in column order. False and SQL NULL are actual bound values;
neither requests a default.

## 1. Insert

Insert and InsertData are Clone/Debug/PartialEq. The builder owns its data; no method executes.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(table)` | `impl Into<Identifier>` | Set destination. |
| `build(&self)` | none | Inspect detached state. |
| `columns(self, columns)` | `impl IntoIterator<Item = impl Into<String>>` | Set/replace stable column order. |
| `values(self, values)` | `impl IntoIterator<Item = impl Into<WriteValue>>` | Append a row. |
| `default_values(self)` | none | Request one wholly defaulted row. |
| `returning(self, selectors)` | `impl IntoIterator<Item = impl Into<Selector>>` | Replace returned projections. |
| `returning_all(self)` | none | Replace RETURNING with wildcard. |
| `render(&self, dialect)` | `DialectId`, through Render | Validate and render. |

**Returns** Constructors/modifiers return Self; `build` returns `&InsertData`;
`render` returns `Result<Query>`. Default mode conflicts with explicit columns/rows. Explicit
mode requires nonempty unique columns, 1–1,000 rows, and matching arity; normal bind/SQL budgets
can reject smaller workloads. MySQL rejects RETURNING.

InsertData exposes `table: Identifier`, `columns: Vec<String>`, `rows: Vec<Vec<WriteValue>>`,
`returning: Vec<Selector>`, and `default_values: bool`. The builder gives only a shared borrow.

## 2. WriteValue

The Clone/Debug/PartialEq enum distinguishes these values for INSERT and UPDATE:

| Variant | Payload | Meaning |
| --- | --- | --- |
| `Value` | `SqlValue` | One bound scalar, including explicit SQL NULL. |
| `Json` | `JsonValue` | Explicit document with dialect-aware validation. |
| `Default` | none | Native database-default request. |
| `Expression` | `Expr` | Typed write expression. |

A blanket `From<T>` exists for any `T: Into<SqlValue>`, including strings, bools, and lossless
integer inputs. Use explicit WriteValue elements for a heterogeneous Rust array.

SQLite omits a column defaulted in every row; mixed DEFAULT/data in one included column is
Unsupported. A single all-default row becomes DEFAULT VALUES; multiple all-default rows are
Unsupported. SQLite UPDATE DEFAULT is also Unsupported. PostgreSQL/MySQL render native DEFAULT.

Related: [write guide](../guides/write-rows.md), [values](values.md), [Update](update.md),
[QueryResult](rows.md#3-queryresult).
