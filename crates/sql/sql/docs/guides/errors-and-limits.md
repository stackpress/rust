# Classify failures and lower budgets

Applications need stable failure categories without exposing native SQL details. Match the
error kind, retain the cause for an intentional diagnostic path, and set limits before sharing
an engine.

## Check a tighter input budget

Use `Query::validate` to inspect a rendered statement under application limits. This program
allows no binds and verifies that a bound lookup fails with Limit:

```rust
use stackpress_sql::{DialectId, ErrorKind, QueryLimits, Render, Result, Select};

fn main() -> Result<()> {
    let query = Select::new(["id"])
        .from("profile")
        .where_eq("id", 7)
        .render(DialectId::Sqlite)?;
    let limits = QueryLimits {
        binds: 0,
        ..QueryLimits::default()
    };
    assert_eq!(query.validate(limits).unwrap_err().kind(), ErrorKind::Limit);
    Ok(())
}
```

Apply the same policy to execution using `Engine::new(adapter).with_limits(limits)` before
sharing the engine. The adapter enforces row and retained-byte output limits while collecting;
overflow returns an error, not a truncated successful result.

## Preserve causes and choose disclosure

`SqlError::kind()` classifies shape/capability, arity, limit, admission, closed/busy,
rollback-only, decode, configuration, and driver failures. Display uses a static message;
`std::error::Error::source()` and `cleanup_error()` retain richer diagnostics that may contain
sensitive native details. Query Debug and explicit `binds()` inspection can also reveal data.

Use `rollback_after(primary)` when transaction cleanup must retain both failures. A Busy,
timeout, driver error, or cancellation does not justify an automatic write retry. See the
[SQLite error disclosure](../../../sql-sqlite/docs/concepts.md#outcomes-and-error-disclosure)
for native classification and lifecycle.

## Budget the whole operation

Limits apply per statement, including each plan member. A plan/outcome vector can retain several
statements and their results at once. Renderer construction also has fixed SQL, bind, expression,
JSON, and schema limits; raising Engine limits cannot disable them. The fixed structural
budgets include these boundaries:

| Input | Renderer / encoder boundary |
| --- | --- |
| Expressions | Depth through 64 and 10,000 total visited nodes. |
| Encoded JSON | Depth through 64, 10,000 nodes, 1 MiB text. |
| JSON paths | 64 segments, 16 KiB encoded text, indices through i32::MAX. |
| Qualified identifiers | 64 segments; common 1,024-byte segment limit plus native dialect limits. |
| Joins | 64 per Select. |
| INSERT rows | 1–1,000 in explicit row mode, also bounded by binds/SQL. |
| CREATE fields / foreign keys | 1–1,000 fields; at most 1,000 foreign keys. |
| ALTER operations | 1–1,000. |

Some shape/cardinality failures are InvalidQuery rather than Limit; classify the actual error.
These checks do not bound arbitrary caller construction or derived recursive cloning/drop.
Retained-byte accounting excludes allocator overhead and native VM resources.

You can now reject oversized input before execution and report failures through a stable kind.
See [QueryLimits](../api/query.md#2-querylimits) and [errors](../api/errors.md) for exact fields
and categories.
