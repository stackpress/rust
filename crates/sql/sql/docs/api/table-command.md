# Drop, rename, and truncate tables

`TableCommand` implements Render for one explicit table operation. Source:
[table.rs](../../src/table.rs).

```rust
use stackpress_sql::{DialectId, Render, Result, TableCommand};

fn main() -> Result<()> {
    let drop = TableCommand::Drop {
        table: "old_profile".into(),
        if_exists: true,
        cascade: false,
    };
    assert!(drop.render(DialectId::Sqlite)?.sql().contains("IF EXISTS"));
    assert!(
        TableCommand::drop_table("old_profile")
            .render(DialectId::Postgres)?
            .sql()
            .contains("DROP TABLE")
    );
    assert!(
        TableCommand::rename("old_profile", "profile")
            .render(DialectId::Sqlite)?
            .sql()
            .contains("RENAME TO")
    );
    assert_eq!(
        TableCommand::truncate("profile")
            .render(DialectId::Sqlite)?
            .sql(),
        "DELETE FROM \"profile\""
    );
    Ok(())
}
```

SQLite truncate is DELETE without resetting the sequence. These descriptions do not execute
until passed to an engine or transaction.

## 1. Variants and constructors

The enum is Clone/Debug/Eq/PartialEq. Public variants expose native options directly:

| Variant | Fields |
| --- | --- |
| `Drop` | `table: String`, `if_exists: bool`, `cascade: bool`. |
| `Rename` | `from: String`, `to: String`. |
| `Truncate` | `table: String`, `cascade: bool`. |

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `drop_table(table)` | `impl Into<String>` | Describe Drop with if_exists=false and cascade=false. |
| `rename(from, to)` | two `impl Into<String>` | Describe a rename. |
| `truncate(table)` | `impl Into<String>` | Describe Truncate with cascade=false. |
| `render(&self, dialect)` | `DialectId`, through Render | Validate and render one statement. |

**Returns** Constructors return Self; `render` returns `Result<Query>`. Names are validated and
quoted. Rename requires distinct names under its ASCII case-insensitive check. Cascade is
supported only by PostgreSQL; SQLite/MySQL return Unsupported instead of ignoring it.

Native table existence, relationships, and privileges are adapter execution concerns. Commands
are single statements rather than publicly constructed QueryPlans; MySQL native DDL can commit
implicitly even if manually executed through a transaction.

Related: [Engine](engine.md), [dialects](dialects.md), [Create](create.md), [Alter](alter.md).
