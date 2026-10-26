# Slice 4 execution and SQL contracts

Companion to [API sketches](slice-4-design-api.md) and [decisions](slice-4-design-decisions.md). Load before implementing the driver, renderer or transaction guard. This is proposed public behavior requiring the listed owner decisions and M0 proof; the risk table supplies each proving test.

## Ownership, executor and lifecycle

Engine is non-Clone and owns one shared-driver object; callers deliberately share an Arc. Query/definition clones are ordinary owned data with no live handles. Database is Send+Sync; every exposed execution future is Send and lifetime bounded. Transaction/session is Send, non-Clone, exclusive through `&mut`; no session enters a shared service registry or Request data. No connection references escape returned owned rows/results.

Shared query calls acquire a lease, validate/render/prepare/bind, execute and collect a bounded outcome, then release the lease before returning or invoking application callbacks. Transactions acquire exactly once and retain that same native connection until consumed/dropped. There is no mutex around Engine, mutable global before hook, process-global current transaction, implicit task-local context or unsafe Send/Sync implementation.

For SQLite, SQLx owns blocking C work on a dedicated worker per connection. Do not wrap that worker API in another per-statement spawn_blocking call. Config/service registration uses existing ServerBuilder startup, and failures cannot publish a partial host. Database installation/migration/population is separate from service startup. No listener starts in this slice.

## Proposed native options and limits

| Setting | Initial proposal / semantics |
| --- | --- |
| Open target | Explicit file PathBuf, create_if_missing false, read-only optional. No network-failure fallback or implicit cwd/process mutation. |
| Pool | max_connections 4, min_connections 0; eager first connection/validation, finite positive maximum. No public Clone for adapter; internal SQLx handles remain intentional shared ownership. |
| Acquire/busy timeout | 5 seconds acquire, 1 second SQLite busy; lowerable finite values, validated overflow/ranges. Busy/locked is a typed error, no hidden retry loop. |
| In-flight admission | Finite configured count (initial 128), fail promptly when full rather than queue unlimited owned query data. Configure SQLx command buffering finitely. Cancelled native work can continue; do not count future cancellation as native completion. |
| Query budgets | Initially 1 MiB SQL text, 1,000 binds, 10,000 returned rows and 16 MiB retained outcome bytes, all lowerable with checked arithmetic. Reject budget overflow with typed Limit, never silent truncation. |
| Collection | Stream internally and check before retaining each next row. This bounds first-party retained output, not SQLite VM memory/time or allocation of a single native value before decoding. Trusted raw SQL can be expensive; no sandbox claim. |
| Pragmas | foreign_keys ON for every connection; explicit journal/synchronous policy. Recommend WAL for file databases and documented default durability, but read-only/network filesystem cases require validation. No PRAGMA mutation from request data. |
| In-memory mode | Explicit opt-in for demos only, one connection; no pooling persistence guarantee. Recommended integration tests use independent tempfile-backed DBs because connection loss can destroy in-memory state. |
| close | Idempotent `close().await`: reject new work, wake waiters and wait for leases; a caller deadline can cancel observation. Re-await is allowed. Drop initiates best-effort cleanup, no synchronous join or durability certificate. |

M0 freezes exact supported options/defaults and tests boundary values. WAL choice must be visible and does not make SQLite a multi-writer server. Bundled SQLite requires the existing native C toolchain, not a database server or installed system SQLite package. Only the installed native target is assessed.

## Transaction state, cancellation and panic

State is Acquiring → Active → Committing/ RollingBack → Finished. Begin must either return one owned active session or clean up/retire the acquired connection. Query failure does not silently commit or retry; explicit rollback preserves the original failure and retains a secondary rollback cause. Guard drop queues rollback through the driver. A connection with uncertain transaction state is closed, not returned as clean.

| Transition/event | Required behavior |
| --- | --- |
| Acquisition cancelled before a guard exists | No leaked permit/lease; SQLx may discard an acquired connection. File-backed data survives; no assertion of in-memory persistence. |
| Begin cancelled before acknowledgement | Driver must rollback or retire the connection; no orphan transaction reused by another query. M0 requires a transition-controlled proof, not only a query timeout. |
| Active statement future dropped | Native work may already execute. Transaction remains exclusive; mark session rollback-only if state cannot be established. Next unrelated borrower never enters that active transaction. |
| Transaction owner dropped / handler future aborted | Uncommitted transaction rollback is initiated; next lease is clean or replaced. Rollback is not synchronously complete at Drop. |
| Result error / explicit rollback | Preserve original source. Rollback failure is a secondary diagnostic, and the connection is retired. No rollback after a failed begin blindly changes a different transaction. |
| Commit cancelled / commit response lost | **Outcome can be unknown**: commit may have completed. No retry or promise of rollback. Caller uses stable operation IDs/idempotent domain design if it needs recovery. |
| Panic in application while guard owned | Ordinary unwind drops guard/queues rollback; no blanket catch_unwind around native internals, no poisoned mutex recovery by unwrap. Abort/power loss is outside unwind guarantee. |
| Driver worker crash | Typed closed/driver error; discard connection. Never manufacture successful empty rows. Driver dependencies may contain unsafe internally; first-party unsafe remains forbidden. |
| Nested transaction request | No implicit pool reacquisition or nested BEGIN. Return explicit unsupported nested transaction; real savepoints Later S4-X after a borrowed-lifetime proof. |

The disposable abort/panic tests prove ordinary post-insert guard drop, not every begin/commit/rollback interleaving. M0 must close those gaps with controlled driver seams and negative controls. No implementation is authorized to claim immediate SQLite interruption merely because tokio::time::timeout returned.

## Nested and concurrent capability dispatch

Nested emit/resolve does not inherit a transaction. Return owned rows and release a normal lease before dispatching other capabilities. Inside a transaction, compose reusable domain/store functions with an explicit `&mut Transaction`; do not ask the nested event to reacquire from the same Engine and assume affinity.

A caller can deliberately hold a transaction then emit a handler that acquires another lease. With a size-one pool this times out; more connections can also wait on SQLite's write lock. Finite acquire/busy budgets and an outer process watchdog prevent an indefinite hang, but do not make the operation succeed or atomic. Document this limit, prove it with actual concurrent/nested host calls, and never silently enlarge the pool, share a mutable session or run a callback under a pool lock.

## SQL/parameter and value boundary

Every builder uses a typed AST. Values always become separate bind nodes; identifier segments are validated and dialect-quoted, and expressions/operators/directions are enums. JSON paths are typed keys/indices and bound where the dialect permits. Empty name, embedded NUL/control, invalid length/type/default, incompatible key definitions and invalid mutation scope are render errors before effects. Literal dots/quotes in a name require explicit segment APIs, not ambiguous splitting.

Raw SQL is explicitly trusted caller-authored code with a dialect tag; it is not an HTTP filter or user template API. A static literal constructor and a visibly audited dynamic constructor make that responsibility apparent. No global `?`→`$n`, `??` or backtick replacement. A dialect-aware lexical gate admits one statement and rejects transaction-control commands for ordinary run APIs; it respects strings, quoted identifiers and comments. Complex trigger bodies/batches are outside this raw convenience surface until designed separately.

Prepare on the same acquired connection, compare actual parameter metadata to supplied values, then bind. SQLite otherwise permits a missing bind as NULL, confirmed by probe. Initial SQLite raw API admits positional anonymous `?` only; reject named/repeated/sparse `?NNN` forms rather than confusing maximum index with argument count. Scanner tests cover literal/comment question marks and all lexical quote states. Rendered bound AST nodes bypass guessing by tracking their parameter count, but adapter validation remains defense in depth.

SQLx 0.9 accepts dynamic SQL only through its explicit `AssertSqlSafe` marker. The adapter uses this safe marker only after first-party rendering/trusted-author validation; it is neither an unsafe block nor sanitization. Keep one reviewable conversion point. DDL defaults/types/constraint actions are still executable SQL and require the same typed/trusted distinction.

SqlValue supports NULL, bool, signed i64, finite f64, text, blob and explicit bounded JSON. Checked conversion rejects u64 above i64::MAX and nonfinite floats. SQLite returns native integer/real/text/blob/NULL values; JSON/date text stays text until explicit decoding. Strict bool accepts only 0/1. Preserve missing versus NULL versus DEFAULT in writes. JSON null and SQL NULL differ. Dates use explicit canonical UTC text or caller-selected types; no ambient timezone or automatic epoch fallback.

Statement results own their affected count/insert ID/rows. No stale shared lastId. RETURNING handles generated keys and UPDATE/DELETE rows, including WITH/leading comments; prepared metadata decides row presence rather than string prefixes. Public responses serialize an explicitly selected DTO, not an automatic native row/pool/config dump. Errors retain typed causes but logs/public messages omit bound values, file credentials and sensitive SQL by default.

## Dialects, DDL and schema diff

Pure renderers exist for SQLite/PostgreSQL/MySQL, each with a capability table and snapshots. Native execution is claimed only for the first adapter. AST build state is independent of dialect; rendering checks unsupported features. MySQL RETURNING and transactional DDL cannot be represented as supported just because a common method exists. PostgreSQL numbering comes from bind nodes, not a regex over arbitrary SQL.

Create has explicit fields/ordered primary/unique/index/FK definitions, checked defaults and quoting, and idempotent table/index create only when requested. Idempotent create does not verify an existing table's shape. Alter supports a full inspectable plan; SQLite natively supports checked add/drop/rename column and index changes, subject to actual constraints. Type/nullability/primary/FK changes return Unsupported before any statement. Table-copy reconstruction requires S4-M's preserving schema/data/trigger/index/FK proof.

Diff compares same logical table, field semantics, ordered primary/compound keys, named uniques/indexes/FKs. NoChanges is a typed empty plan. Explicit rename reconciles matching pending removal/addition; changed field semantics remain visible. No semantic guessing, live inspection or automatic execution. SQLite DELETE-style truncate leaves sequence behavior explicit and rejects cascade; destructive operator policy remains S4-M. Every batch validates all statements first, executes on one transaction and rolls back on failure where transactional DDL is supported.
