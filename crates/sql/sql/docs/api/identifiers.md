# Identifiers and table references

`Identifier` retains explicit name segments; `TableRef` adds a separately quoted alias.
Source: [identifier.rs](../../src/identifier.rs).

```rust
use stackpress_sql::{DialectId, Identifier, Render, Result, Select, TableRef};

fn main() -> Result<()> {
    let column = Identifier::qualified(["p", "id"]);
    assert_eq!(column.segments(), ["p", "id"]);
    assert_eq!(Identifier::from("p.id").segments(), ["p.id"]);
    let table = TableRef::new("profile").alias("p");
    let query = Select::new([column])
        .from(table)
        .render(DialectId::Sqlite)?;
    assert!(query.sql().contains(r#""p"."id""#));
    Ok(())
}
```

A dotted string remains one literal name. Qualification is structural, and the table alias
is its own name rather than an SQL suffix embedded in a string.

## 1. Identifier

The type is Clone/Debug/Eq/PartialEq with private segments. `From<&str>` and `From<String>`
create a single segment, preserving dots and quotes as name data.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `qualified(parts)` | `impl IntoIterator<Item = impl Into<String>>` | Retain explicitly separated name segments. |
| `segments(&self)` | none | Inspect the original segments. |

**Returns** `qualified` returns Self without validation; `segments` returns `&[String]`.
Rendering checks a nonempty list of at most 64 segments, nonempty names, control characters,
and dialect limits. Embedded quote characters are escaped by the selected dialect, not parsed.
Common validation also bounds each segment to 1,024 bytes.

## 2. TableRef

The type is Clone/Debug/Eq/PartialEq. Public fields are `name: Identifier` and
`alias: Option<String>`. A blanket `From<T: Into<Identifier>>` creates unaliased references,
including conversions from Identifier, &str, and String.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(name)` | `impl Into<Identifier>` | Describe a relation. |
| `alias(self, alias)` | `impl Into<String>` | Set one literal alias segment. |

**Returns** Each operation returns Self; validation occurs on rendering the enclosing query.
Neither constructor opens a connection or checks that a relation exists.

Related: [Select](select.md), [Selector](expressions.md#3-selector),
[join guide](../guides/build-queries.md).
