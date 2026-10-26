# Select, joins, and ordering

`Select` owns a read description and implements `Render`. Source: [dml.rs](../../src/dml.rs).

```rust
use stackpress_sql::{DialectId, Expr, Order, Render, Result, Select, Sort};

fn main() -> Result<()> {
    let select = Select::default()
        .select("name")
        .from("profile")
        .distinct()
        .where_eq("id", 7)
        .where_eq("active", true)
        .and_where(Expr::eq("team_id", 3))
        .order_by("name", Order::Desc)
        .order("id")
        .sort(Sort {
            expression: Expr::column("team_id"),
            order: Order::Asc,
        })
        .limit(0)
        .offset(0);
    let data = select.build();
    assert_eq!(data.filters.len(), 3);
    assert_eq!(data.limit, Some(0));
    assert_eq!(
        select.render(DialectId::Sqlite)?.binds(),
        &[7.into(), true.into(), 3.into()]
    );
    Ok(())
}
```

Repeated `where_eq` appends predicates joined with AND in call order. Default
selects `*`, so `.select("name")` appends to that wildcard; use `new(["name"])` for an exact list.

Contents: [Select](#1-select) · [Inspectable data](#2-selectdata) · [Join and sort](#3-join-sort-and-enums)

## 1. Select

Select is Clone/Debug/PartialEq. Default selects a wildcard with no table, filters, sorting,
limit, or offset. `new` takes an explicit projection list; an empty list is a render error.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(columns)` | `impl IntoIterator<Item = impl Into<Selector>>` | Set initial projections. |
| `build(&self)` | none | Borrow the detached data. |
| `select(self, selector)` | `impl Into<Selector>` | Append one projection. |
| `from(self, table)` | `impl Into<TableRef>` | Set/replace the source. |
| `distinct(self)` | none | Enable duplicate elimination. |
| `join(self, join)` | `Join` | Append a join. |
| `where_eq(self, column, value)` | `impl Into<Identifier>`, `impl Into<SqlValue>` | Append equality. |
| `and_where(self, expression)` | `Expr` | Append a predicate. |
| `where_json(self, column, path, value)` | `impl Into<Identifier>`, `JsonPath`, `JsonScalar` | Append exact JSON scalar equality. |
| `where_json_contains(self, column, path, value)` | same types | Append direct scalar array membership. |
| `order_by(self, column, order)` | `impl Into<Identifier>`, `Order` | Append column ordering. |
| `order(self, column)` | `impl Into<Identifier>` | Append ascending column ordering. |
| `sort(self, sort)` | `Sort` | Append expression ordering. |
| `limit(self, limit)`, `offset(self, offset)` | `u64` | Set explicit pagination values, including zero. |
| `render(&self, dialect)` | `DialectId`, through Render | Validate and produce SQL. |

**Returns** `new` and consuming modifiers return Self; `build` returns `&SelectData`;
`render` returns `Result<Query>`. Page values must fit signed i64 for every renderer. Offset-only
queries use SQLite LIMIT -1, MySQL's unbounded-limit form, or bare PostgreSQL OFFSET.

Rendering requires a table and projections. Empty groups/IN lists, malformed names/operands,
unsupported FULL JOIN, too many joins, and expression/input overflow fail before IO. RIGHT/FULL
joins also require a compatible native SQLite version; see [dialects](dialects.md).

## 2. SelectData

The Clone/Debug/PartialEq snapshot has public fields below, but `build()` only gives a shared
borrow. Cloning a snapshot does not provide a public constructor that installs it into a builder.

| Field | Type | Meaning |
| --- | --- | --- |
| `selectors` | `Vec<Selector>` | Ordered projections. |
| `table` | `Option<TableRef>` | Source relation. |
| `joins` | `Vec<Join>` | Ordered joins. |
| `filters` | `Vec<Expr>` | Predicates joined with AND. |
| `sort` | `Vec<Sort>` | Ordered sorting terms. |
| `limit`, `offset` | `Option<u64>` | Absent versus explicit zero. |
| `distinct` | `bool` | Duplicate elimination flag. |

## 3. Join, Sort, and enums

These public data types are Clone/Debug/PartialEq; Order and JoinKind are also Copy/Eq.
Construct them directly, as shown in the [join guide](../guides/build-queries.md).

| Type | Public fields or variants |
| --- | --- |
| `Join` | `kind: JoinKind`, `table: TableRef`, `on: Option<Expr>`. |
| `JoinKind` | `Inner`, `Left`, `Right`, `Full`, `Cross`. |
| `Sort` | `expression: Expr`, `order: Order`. |
| `Order` | `Asc` (Default), `Desc`. |

Cross joins require `on: None`; all other kinds require an ON expression. At most 64 joins are
accepted. SQL/binds follow projection, join, filter, and sort order. See [expressions](expressions.md),
[identifiers](identifiers.md), [JSON](json.md), and [build_inspect](../../examples/build_inspect.rs).
