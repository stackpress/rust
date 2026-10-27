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
| `on_conflict(self, conflict)` | `InsertConflict` | Replace the optional explicit-target conflict action. |
| `conflict(&self)` | none | Inspect `Option<&InsertConflict>` independently of InsertData. |
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

## 3. Explicit conflict actions

`InsertConflict` is Clone/Debug/PartialEq and carries either
`DoNothing { target: Vec<String> }` or
`DoUpdate { target: Vec<String>, assignments: Vec<(String, WriteValue)> }`.
Both require a nonempty primary/unique column target; DoUpdate also requires nonempty,
distinct assignments. The database verifies that the target has a matching unique constraint.

`on_conflict(self, InsertConflict) -> Self` replaces the optional action;
`conflict(&self) -> Option<&InsertConflict>` inspects it. InsertData remains unchanged.
The clause follows VALUES and precedes RETURNING. Insert values bind first, then update
assignments in their declared order. No key lookup or automatic retry occurs.

```rust
use stackpress_sql::{DialectId, Insert, InsertConflict, Render, Result, WriteValue};

fn main() -> Result<()> {
    let query = Insert::new("project")
        .columns(["id", "title"])
        .values(["p1", "Initial"])
        .on_conflict(InsertConflict::DoUpdate {
            target: vec!["id".into()],
            assignments: vec![("title".into(), WriteValue::from("Updated"))],
        })
        .returning(["id", "title"])
        .render(DialectId::Postgres)?;
    assert_eq!(query.binds().len(), 3);
    assert!(query.sql().contains("ON CONFLICT (\"id\") DO UPDATE SET \"title\" = $3 RETURNING"));
    Ok(())
}
```

SQLite and PostgreSQL support one explicit VALUES row with either action. Default-only or
multi-row conflicted inserts are Unsupported, including SQLite's effective DEFAULT VALUES
after omitted defaulted columns. Existing WriteValue limits still apply. MySQL returns
Unsupported because its conflict action cannot be restricted to the supplied target;
it is not translated to an any-unique-key action.

**DoNothing with RETURNING:** an actual insert returns its row; a conflicting no-op returns
no row. An empty result is a valid already-exists/no-change outcome, not an execution error
or permission to fabricate a record. Callers choose an explicit result type for that outcome.
An acknowledged DoUpdate returns the inserted or updated row; no branch flag is inferred
from affected rows or insert IDs. Transaction completion stays caller-owned.
