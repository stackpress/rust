# Limits, deadlines and disclosure

Connection options bound admission and IO duration. QueryLimits independently bound input
and retained results. Configure both before sharing the Engine.

## Pool and operation budgets

The defaults are finite:

| Budget | Default | Exhaustion / timeout |
| --- | --- | --- |
| Native connections | 4 | Admitted calls wait for a connection |
| In-flight work | 128 | Immediate `Limit` when admission is full |
| Eager open / pool acquisition | 5000 ms | `PoolTimedOut`; close wakes waiters with `Closed` |
| Whole prepare/bind/execute/decode or BEGIN/COMMIT/ROLLBACK | 30000 ms | `Driver` with typed elapsed cause; outcome may be unknown |
| Each reset and each retirement phase | 5000 ms | Cleanup fails; uncertain transport is disposed |

Bounds and valid ranges are in [options](options.md). Cleanup has a separate budget from
the statement: reset failure can be followed by retirement, each using its own cleanup
deadline. Total caller time is not just the statement deadline. Close itself has no overall
timeout and can await a held transaction; supply an application shutdown deadline.

## Retained query budgets

Core QueryLimits defaults are SQL bytes 1 MiB, binds 1000, rows 10,000 and retained
column-name/value plus bind bytes 16 MiB. Zero rows permits rowless outcomes only.
Each query/transaction run receives these limits; a failure returns no partial result.

```rust
use stackpress_sql::{Engine, QueryLimits};
use stackpress_sql_postgres::PostgresDatabase;

fn bounded_engine(database: PostgresDatabase) -> Engine {
    Engine::new(database).with_limits(QueryLimits {
        rows: 1_000,
        bytes: 2 * 1024 * 1024,
        ..QueryLimits::default()
    })
}
```

The helper lowers owned result retention before the Engine is shared. Names and values are
charged before retained copies; JSON input and serialization are bounded separately.
These are not hard heap limits: native packets, server memory and transient parser/driver
allocations can exceed them. There is no public streaming cursor API.

## Error classification

Call `SqlError::kind()` rather than parsing messages:

| Kind | Meaning here |
| --- | --- |
| `Configuration` | Invalid fields/budgets or rejected configuration |
| `InvalidQuery` | Invalid SQL/value shape, checked input narrowing or nonfinite input |
| `Unsupported` | Wrong dialect, unsupported operation or prepared target |
| `Arity { expected, actual }` | Native prepared parameter count differs from supplied binds |
| `Limit` | Admission or query/result retention budget exceeded |
| `PoolTimedOut` / `Closed` | Acquisition deadline / closed owner |
| `RollbackOnly` | A failed/cancelled polled session cannot continue or commit |
| `Decode` | Unsupported result metadata, invalid bytes or value overflow |
| `Driver` | Native execution/authentication/transport/control failure or statement deadline |

Constraint and deadlock failures retain a typed native cause; the adapter does not parse
native messages into a portable Busy/constraint taxonomy. Error kind alone never establishes
whether an unobserved write committed.

## Preserve causes without disclosing them

Public SqlError Display/Debug uses static context, hiding native messages and sensitive SQL.
`std::error::Error::source` deliberately retains the typed original cause for trusted
diagnostics. Formatting that cause or a full external source chain can disclose native
SQL, identifiers or credentials; do not send it to clients or ordinary logs.

Primary failure wins. `cleanup_error()` separately retains retirement/rollback failure;
`rollback_after` and `with_cleanup` preserve this precedence. Failure during reset after a
successful write is an error even though the write may have applied. Background cleanup is
observed by re-awaitable close.

Options Debug is redacted, but options Serialize includes the password. Query/bind data and
caller-provided values are also not safe diagnostic output by default.
