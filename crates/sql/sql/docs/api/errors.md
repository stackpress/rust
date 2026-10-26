# SQL results and typed errors

`Result<T>` aliases `std::result::Result<T, SqlError>`. ErrorKind supports stable classification;
SqlError preserves native and cleanup causes. Source: [error.rs](../../src/error.rs).

```rust
use stackpress_sql::{ErrorKind, SqlError};
use std::error::Error;

fn main() {
    let primary = SqlError::with_source(
        ErrorKind::Driver,
        "statement failed",
        std::io::Error::other("private native detail"),
    )
    .with_cleanup(SqlError::new(ErrorKind::Closed, "rollback failed"));
    assert_eq!(primary.kind(), ErrorKind::Driver);
    assert_eq!(primary.to_string(), "statement failed");
    assert!(primary.source().is_some());
    assert_eq!(primary.cleanup_error().unwrap().kind(), ErrorKind::Closed);
}
```

Display keeps the safe message. Inspecting source/cleanup details is an application disclosure
choice, and those diagnostics can contain sensitive native data.

## 1. ErrorKind

ErrorKind is Copy/Clone/Debug/Eq/PartialEq. Match the enum rather than native message text:

| Variant | Meaning |
| --- | --- |
| `InvalidQuery` | Invalid shape, identifier, or value. |
| `Unsupported` | Backend/operation/representation outside the supported subset. |
| `Arity { expected: usize, actual: usize }` | Native prepared count versus supplied argument count. |
| `Limit` | Finite input/output/admission budget exceeded. |
| `PoolTimedOut` | Connection acquisition deadline exceeded. |
| `Closed` | Database closed or worker unavailable. |
| `Busy` | Native lock/busy condition. |
| `RollbackOnly` | Session must not commit after uncertain work. |
| `Decode` | Missing/ambiguous/incompatible/out-of-range returned value or metadata. |
| `Configuration` | Invalid adapter options. |
| `Driver` | Other native execution failure. |

These categories do not prove a failed/cancelled write had no effects. No implicit retry occurs.
Native classification belongs to the adapter; see [SQLite](../../../sql-sqlite/docs/README.md).

## 2. SqlError

SqlError implements Debug, Display, and std::error::Error; its fields are private.

| Operation | Parameters | Purpose |
| --- | --- | --- |
| `new(kind, message)` | `ErrorKind`, `&'static str` | Construct a static safe message. |
| `with_source(kind, message, source)` | `ErrorKind`, `&'static str`, `impl Error + Send + Sync + 'static` | Preserve a native cause separately. |
| `kind(&self)` | none | Inspect classification. |
| `with_cleanup(self, cleanup)` | `SqlError` | Attach a secondary cleanup failure. |
| `cleanup_error(&self)` | none | Inspect secondary rollback failure. |

**Returns** Constructors/with_cleanup return Self; kind returns ErrorKind;
cleanup_error returns `Option<&SqlError>`. Error::source returns the primary native cause,
while the cleanup error retains its own cause. Display omits SQL/binds/paths; callers constructing
errors are responsible for keeping their static message non-sensitive. SqlError Debug also
omits native sources. Error::source, cleanup diagnostics, and Query Debug/inspection can expose
richer data and should be disclosed deliberately.

Related: [error and limit guide](../guides/errors-and-limits.md),
[Transaction::rollback_after](transaction.md), [QueryLimits](query.md#2-querylimits).
