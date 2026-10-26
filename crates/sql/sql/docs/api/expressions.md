# Expressions and projections

`Expr` represents typed operands and predicates; `Selector` makes one a projection.
Source: [expr.rs](../../src/expr.rs).

```rust
use stackpress_sql::{Compare, DialectId, Expr, Render, Result, Select, Selector};

fn main() -> Result<()> {
    let age = Expr::column("age").compare(Compare::Ge, Expr::value(18));
    let enabled = Expr::any([Expr::eq("active", true), Expr::eq("reviewer", true)]);
    let query = Select::new([Selector::expression(Expr::value("member")).alias("role")])
        .from("profile")
        .and_where(Expr::all([age, enabled]))
        .render(DialectId::Postgres)?;
    assert_eq!(query.binds().len(), 4);
    Ok(())
}
```

The selected string is bound data, and the comparison/group tree keeps its parentheses.
Bind order follows SQL traversal, including projections before joins and filters.

Contents: [Compare](#1-compare) · [Expr](#2-expr) · [Selector](#3-selector)

## 1. Compare

Variants are `Eq`, `Ne`, `Lt`, `Le`, `Gt`, `Ge`, and `Like`, mapping to SQL comparison operators.
The type is Copy/Clone/Debug/Eq/PartialEq. Use `Expr::Like` when an explicit escape character
is needed; `Compare::Like` is an ordinary two-operand comparison.

## 2. Expr

Expr is Clone/Debug/PartialEq. It exposes these data variants for construction and inspection:

| Variant | Payload and meaning |
| --- | --- |
| `Column` | `Identifier`; quoted operand. |
| `Value` | `SqlValue`; bound operand. |
| `Compare` | `Box<Expr>, Compare, Box<Expr>`; binary comparison. |
| `All`, `Any` | `Vec<Expr>`; AND/OR groups, nonempty at render. |
| `Not` | `Box<Expr>`; negated expression. |
| `IsNull` | `Box<Expr>, bool`; true selects IS NOT NULL. |
| `In` | `Box<Expr>, Vec<SqlValue>, bool`; nonempty list, true selects NOT IN. |
| `Like` | `Box<Expr>, String, char`; bound pattern and escape, no control escape. |
| `JsonExtract` | `Identifier, JsonPath`; extraction operand. |
| `JsonEq`, `JsonContains` | `Identifier, JsonPath, JsonScalar`; typed equality/direct array membership. |
| `Trusted` | `DialectId, TrustedSql, Vec<SqlValue>`; audited expression with separate binds. |

Convenience constructors retain data without validating the enclosing query:

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `column(name)` | `impl Into<Identifier>` | Construct Column. |
| `value(value)` | `impl Into<SqlValue>` | Construct Value. |
| `compare(self, operator, right)` | `Compare`, `Expr` | Construct binary Compare. |
| `eq(column, value)` | `impl Into<Identifier>`, `impl Into<SqlValue>` | Column equality. |
| `all(expressions)`, `any(expressions)` | `impl IntoIterator<Item = Expr>` | Collect an AND/OR group. |
| `json(column, path)` | `impl Into<Identifier>`, `JsonPath` | Construct extraction. |
| `json_eq(column, path, value)` | same column/path, `JsonScalar` | Compare a typed JSON scalar. |
| `json_contains(column, path, value)` | same column/path, `JsonScalar` | Direct scalar array membership. |

**Returns** Every constructor returns Expr. Validation happens at render. Equality/inequality
with a right-hand `SqlValue::Null` renders IS NULL / IS NOT NULL. General NULL comparisons keep
SQL semantics. Empty groups/IN lists fail; operands, depth/nodes, SQL bytes, and binds are bounded.
Trusted expressions require matching dialects and audited syntax, with no sanitization guarantee.

## 3. Selector

Selector is Clone/Debug/PartialEq with private fields. It supports `From<Identifier>`,
`From<&str>`, and `From<String>`. Only the exact string `"*"` is special; other strings are
literal identifiers, including function-looking text.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `expression(expression)` | `Expr` | Project a typed expression. |
| `wildcard(table)` | `Option<Identifier>` | Project all columns, optionally qualified. |
| `alias(self, alias)` | `impl Into<String>` | Set a quoted result name. |

**Returns** Constructors and alias return Self. Wildcard aliases are invalid; all names and
expression operands are validated by the enclosing query. Use aliases for unique row lookup.

Related: [Select](select.md), [JSON](json.md), [rows](rows.md),
[raw trust boundary](query.md#3-raw-statements-and-trust).
