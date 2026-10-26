# Insert defaults and scope row mutations

An omitted value, SQL NULL, and a database default express different intentions. State the
choice explicitly, and require a filter or an all-row choice before updating or deleting.

## Insert a value and a default

Use `WriteValue` when cells in one row have different roles. This example leaves `active` to
the database default while binding an empty name as actual data:

```rust
use stackpress_sql::{Delete, DialectId, ErrorKind, Insert, Render, Result, Update, WriteValue};

fn main() -> Result<()> {
    let insert = Insert::new("profile")
        .columns(["name", "active"])
        .values([WriteValue::from(""), WriteValue::Default]);
    let query = insert.render(DialectId::Sqlite)?;
    assert_eq!(query.binds().len(), 1);
    assert!(!query.sql().contains("active"));

    let update = Update::new("profile")
        .set("active", false)
        .where_eq("id", 7);
    assert_eq!(update.render(DialectId::Sqlite)?.binds().len(), 2);
    assert_eq!(
        Delete::new("profile")
            .render(DialectId::Sqlite)
            .unwrap_err()
            .kind(),
        ErrorKind::InvalidQuery
    );
    Ok(())
}
```

SQLite omits a column defaulted in every row because it has no per-cell DEFAULT token. Mixed
bound/default cells in the same included column are unsupported there. For a whole default row,
use `Insert::new("profile").default_values()`. Bulk inserts append `.values(...)` calls and
require matching column counts; SQLite cannot bulk-insert entirely defaulted rows.

## Make scope and returned data explicit

`Update` and `Delete` append each `where_eq`; typed predicates use `and_where`. Without a filter,
call `all_rows()` deliberately. Empty groups do not grant all-row permission. Duplicate assignment
names and empty updates are render errors.

Use `returning(...)` or `returning_all()` to request changed rows on SQLite/PostgreSQL. MySQL
rejects RETURNING. Read affected rows and any insert ID from this statement's `QueryResult`;
use RETURNING for native SQLite generated IDs. No engine-wide last-ID state is maintained.

You can now distinguish missing/default/NULL values and verify a mutation's scope before IO.
Look up [Insert and WriteValue](../api/insert.md), [Update](../api/update.md), and
[Delete](../api/delete.md), or continue to [transactions](transactions.md).
