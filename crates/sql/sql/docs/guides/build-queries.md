# Join tables and compose filters

A profile list may need data from its team as well as its own table. Use explicit aliases and
qualified identifiers so every projected column and join condition has one meaning.

## Build the joined lookup

Construct the join as typed data. This program projects the profile ID under an unambiguous
result name and appends two filters:

```rust
use stackpress_sql::{
    DialectId, Expr, Identifier, Join, JoinKind, Render, Result, Select, Selector, TableRef,
};

fn main() -> Result<()> {
    let profile_id = Identifier::qualified(["p", "id"]);
    let lookup = Select::new([Selector::from(profile_id).alias("profile_id")])
        .from(TableRef::new("profile").alias("p"))
        .join(Join {
            kind: JoinKind::Inner,
            table: TableRef::new("team").alias("t"),
            on: Some(
                Expr::column(Identifier::qualified(["p", "team_id"])).compare(
                    stackpress_sql::Compare::Eq,
                    Expr::column(Identifier::qualified(["t", "id"])),
                ),
            ),
        })
        .where_eq(Identifier::qualified(["p", "active"]), true)
        .where_eq(Identifier::qualified(["t", "name"]), "Research")
        .limit(20);
    let query = lookup.render(DialectId::Sqlite)?;
    assert!(query.sql().contains("INNER JOIN"));
    assert_eq!(query.binds().len(), 2);
    assert_eq!(lookup.build().filters.len(), 2);
    Ok(())
}
```

The ON comparison uses two column expressions; the filters bind ordinary values. `where_eq`
and `and_where` append conditions joined with AND in call order on Select, Update and Delete.
Use `and_where` for predicates that need an explicit expression.

## Group alternatives and page the result

Compose `Expr::any` for OR and `Expr::all` for AND; parentheses preserve the requested tree.
Empty groups and IN lists are invalid, so handle an empty caller choice before rendering.
`Cross` joins require no ON condition; every other join kind requires one.

Add stable ordering before pagination. `order` appends ascending order, `order_by` chooses a
direction, and `sort` accepts a typed expression. Zero limit/offset remains explicit; page values
must fit signed i64. Offset-only SQL uses each dialect's native form.

You now have named columns, explicit join ownership, and filters whose bind order follows SQL.
See [Select](../api/select.md), [expressions](../api/expressions.md), and the larger
[build_inspect example](../../examples/build_inspect.rs) for lookup details.
