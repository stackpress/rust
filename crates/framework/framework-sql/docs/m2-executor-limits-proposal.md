# M2 executor budget accessor request

Status: accepted exactly as proposed by the coordinator; both getters and the bounded
relation loader are implemented. The sections below preserve the original request and
counterexample. Final verification belongs to the M2 receipt.

## Contract and concrete counterexample

The accepted `slice-6-design-stores.md` requires relation keys to be deduplicated and loaded
in chunks derived from **the selected executor's bind budget**, with lowerable limits and
no partial/truncated result. `Engine::with_limits` stores private `QueryLimits`, and an
owned `Transaction` freezes that same private copy at begin. Neither exposes a getter.
`DataSession` holds precisely one of those executor references and cannot inspect its limits.
Using `QueryLimits::default()` loses the caller's lower limits. Executing queries to discover
a budget would introduce hidden IO/failure probes; duplicating limits in app configuration
would permit drift from the exact executor and frozen transaction.

A real SQLite proof, `tests/executor_budgets.rs::reduced_executor_bind_budget`, passes under
an outer kill/reap watchdog. An Engine configured with `binds: 2` accepts three one-bind
inserts; the default renderer accepts an IN query with three keys. That Engine and its
Transaction both reject the three-key query with `ErrorKind::Limit`. Two chunks of two and
one keys through the exact borrowed DataSession return every identity. This is an executor
visibility gap, not permission to increase bounds or retry a rejected query. The production
relation loader has not been added yet.

## Exact proposed additive API

```rust
impl Engine {
    /// Return the configured per-statement budgets without acquiring a connection.
    pub fn limits(&self) -> QueryLimits {
        self.limits
    }
}

impl Transaction {
    /// Return the originating Engine's budgets frozen when this transaction began.
    pub fn limits(&self) -> QueryLimits {
        self.limits
    }
}
```

`QueryLimits` is already public, Copy and value-only. Both getters return the existing
stored value; they perform no IO, state transition or mutation, add no dependency, change
no adapter trait and do not expose a raw executor or widen a budget. No schema contract
revision-2 change is involved. Existing `with_limits`, render validation and adapter output
validation remain authoritative.

Framework-sql can then add its own `DataSession::limits(&self) -> QueryLimits` delegation
inside its existing write set. Planning uses the minimum of the selected executor's limits,
renderer ceilings and explicit relation limits, reserving binds for fixed predicates such as
`active = true`. An insufficient/zero remaining budget fails before relation IO. Aggregate
row/byte/key/statement ceilings remain separate; a getter does not relax them.

## Requested exclusive shared paths and verification

- `crates/sql/sql/src/engine.rs`: the two read-only methods and Rustdoc examples only.
- `crates/sql/sql/tests/engine.rs`: default/custom/frozen-copy and no-native-call tests.
- `crates/sql/sql/docs/api/engine.md`: document `Engine::limits`.
- `crates/sql/sql/docs/api/transaction.md`: document `Transaction::limits`.

Tests will compare all four fields, show that changing a returned copy cannot alter an
executor, confirm an already-created Transaction keeps its old limits when its Engine is
subsequently moved through `with_limits`, and assert no native run/acquisition from reads.
Then the real generated include tests will use a reduced budget, assert complete relation
results and measured chunk counts, and prove required predicate overhead is reserved.
Existing SQL-core tests must pass unchanged. Record this shared diff separately in the M2
receipt if the coordinator sequences and grants it.

Requested decision: approve and sequence these exact additive getters, or select a revised
budget contract before relation loading is implemented. No other shared edit is requested.
