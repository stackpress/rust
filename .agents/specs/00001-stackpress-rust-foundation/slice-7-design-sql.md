# Slice 7 network SQL execution contract

Proposed contract under [decisions](slice-7-design-decisions.md). Current
`crates/sql/sql` and `sql-sqlite` source governs inherited behavior; this is not
permission to change those crates or widen existing public value shapes.

## Adapter ownership and startup

Create `crates/sql/sql-postgres` and `crates/sql/sql-mysql`, packages
`stackpress-sql-postgres` / `stackpress-sql-mysql`. Public database owners are
non-Clone and implement existing Database unchanged. Share Engine through an
explicit Arc. Driver types and pools remain private. No runtime listeners,
macros, migrations, retries or global current transaction.

Options are typed, serde unknown-field-rejecting, with redacted Debug. Use
explicit host, port, database, user and a caller-supplied secret, not a freely
logged arbitrary driver URL. No implicit PG environment fallback or database
creation. Test fixture URL compatibility is separate from application options.

| Option | Proposed policy |
| --- | --- |
| max_connections | Default 4, range 1..=128; min_connections fixed zero, eager first connection. |
| max_in_flight | Default 128, range 1..=4096; immediate try-acquire includes pool waiters/transactions/cleanup. |
| acquisition/open | Default 5000ms; finite range 1..=300000ms, whole connection handshake bounded. |
| statement/cleanup | Statement deadline 30000ms, cleanup 5000ms; same positive finite range. |
| TLS | VerifyFull (Postgres) / VerifyIdentity (MySQL); explicit CA source, server identity checked. Plaintext opt-in for local/tests only. No opportunistic downgrade default. |
| Session | Set only per-connection policy; MySQL utf8mb4 and agreed SQL escaping mode. No SET GLOBAL, role changes, extensions or arbitrary connection hooks. |

Use SQLx `=0.9.0`, defaults off, `runtime-tokio`, only the adapter's backend,
`tls-rustls-ring` and JSON codec when needed. Disable default driver statement
logging. Explicitly selected feature providers avoid relying on another crate's
feature unification. Native TLS is a future optional backend, not enabled by
default. Pooler compatibility (e.g. transaction-mode PgBouncer) is not assumed;
prepared caching is bounded/disabled initially until explicitly proved.

## State/lease contract, matching SQLite

1. Validate Query limits/dialect before effects, including direct trait calls.
2. Try admission immediately; overload returns Limit. Acquire under finite
   timeout; cancel before lease publication releases admission and no application
   statement has been sent. SQLx owns unpublished handshake cleanup.
3. Couple owned PoolConnection and OwnedSemaphorePermit before first command.
   Mark close-on-drop before uncertain IO. Prepare, validate arity, bind, execute
   and decode on that same connection.
4. Fully acknowledged ordinary success may recycle its lease. Any driver,
   decode, limit, timeout or cancellation error retires it. Do not return a
   half-consumed protocol stream or failed transaction to the pool.
5. BEGIN publishes a session only after acknowledgement. Session run sets
   rollback_only before its first possible suspension; clears only on success.
   Every failed/cancelled run blocks later run and commit, but explicit rollback
   remains allowed. This is stricter than PostgreSQL's own failed-TX behavior.
6. Commit/rollback consume session ownership into their returned future even
   before polling. Retire transaction leases on completion/drop like SQLite.
   Retain primary and cleanup failures separately; never overwrite primary.
7. Close shuts admission, wakes waiters and awaits held/retiring pool leases.
   Cancelling close can be re-awaited. Host shutdown must drain HTTP/domain tasks
   before close; a caller retaining a transaction can legitimately delay it.

Drop cannot await. While runtime lives, move connection plus admission permit
into a tracked cleanup task: async close with deadline, then forced retirement
if close does not acknowledge. Keep admission coupled until cleanup completion
or forced transport disposal; never return an uncertain socket for reuse. Pool
close waits on its connection count. Outside a live runtime, best-effort RAII
teardown only, with no claim of acknowledged rollback or permit coupling.
M0 must verify SQLx Drop/socket behavior for this fallback; no undocumented
detach-and-reuse shortcut is acceptable.

Cancelling a future means loss of observation. A write or COMMIT can already
have taken effect. Closing a connection normally makes the server roll back an
open transaction, but local close acknowledgement is not proof of immediate
remote session termination. No CANCEL/KILL query to arbitrary backend sessions,
no automatic retries or exactly-once claim. Slow remote cleanup is bounded
locally and visible as an uncertain outcome.

## Values, native preparation and outcomes

| Native case | Required result/binding contract |
| --- | --- |
| PostgreSQL bool, int2/int4/int8, float4/float8, UTF-8 text/varchar, bytea | Map to Bool, checked i64, finite f64, Text, Blob; promote narrower numbers exactly, reject unsupported types. |
| MySQL signed integer families, unsigned integers, float/double, text/binary | Preserve integer values, check unsigned <= i64::MAX; reject overflow, invalid UTF-8 and nonfinite reals. Use type/charset flags to distinguish bytes from text. |
| MySQL BOOL alias | Native TINYINT is Integer. Existing Row bool decoding accepts only 0/1, rejecting 2; do not reinterpret every tinyint column as Bool. |
| JSON / JSONB | Decode bounded JSON to canonical Text using explicit JSON codec; SQL NULL remains Null, JSON null is Text("null"). Current WriteValue::Json supplies native casts; plain text is not guessed to be JSON. |
| DateTime | Existing renderer stores PostgreSQL/MySQL VARCHAR(30); preserve canonical UTC text and explicit validation. Native DATE/TIME/TIMESTAMP, timezone transforms remain S4-X, with Decode error now. |
| DECIMAL/NUMERIC, UUID, arrays, enum/custom/domain/geospatial | Unsupported native output is Decode, no float/string fallback. Renderer capability is not evidence that every native type has a codec. A caller may explicitly cast a trusted query to supported text. |
| SQL NULL parameters | PostgreSQL metadata selects typed NULL for supported parameter types; never bind every Null as i64/Text regardless of target. Unknown/unsupported type gets clear error or explicit SQL cast. MySQL native NULL binding likewise proved. |
| PostgreSQL parameter widths | Prepare metadata must be used for checked int2/int4/int8 and float width; do not always send i64 when a native narrower type is required. Out-of-range bind rejects before execution. |
| Result names/values | Preserve order and duplicate column names. Charge names and values before first-party copies; stream driver rows into the owned result with no unbounded fetch_all. Fail whole statement on limits. |
| Affected count / insert ID | Native statement-local completion count only. Postgres insert_id None; RETURNING rows authoritative. MySQL native zero ID -> None, positive checked i64, no cached last ID. |

Prepared arity must be validated against public statement metadata on the same
lease, independently of renderer validation. SQLx AssertSqlSafe marks the audited
dynamic-SQL boundary only. Query already separates binds and rejects batches and
connection/transaction control; no string interpolation of values/identifiers.
Native rows/packets may already allocate beyond retained QueryLimits. Document
this honestly and include large-cell workload tests, not a hard heap guarantee.

## Backend transaction differences

- PostgreSQL normal transaction and transactional DDL can follow current Engine
  execute_plan behavior. Nontransactional commands outside the allowed query
  subset are Unsupported; never issue them inside a generic plan. The
  [PostgreSQL protocol](https://www.postgresql.org/docs/12/protocol-flow.html)
  also distinguishes cancellation from acknowledged query completion.
- MySQL DML transactions require transactional tables (generated tables use
  InnoDB). No rollback promise for external MyISAM tables. Direct adapter docs
  require the application to choose a transactional schema; do not scan catalogs
  or mutate unrelated engines automatically.
- Existing Engine/Transaction execute_plan already checks
  `DialectId::Mysql.capabilities().transactional_ddl == false`. Preserve that
  pre-effect rejection. Single standalone rendered DDL can use Database::run.
- MysqlSession::run additionally rejects CREATE/DROP/ALTER before effects,
  including direct trait calls; those cause
  [implicit commits](https://dev.mysql.com/doc/refman/8.0/en/implicit-commit.html). Use a bounded
  lexical first-command check honoring existing allowed comments/CTEs, not an
  unsafe substring test. WITH/EXPLAIN forms must not smuggle DDL. Mark the
  session rollback-only on rejected/failed run consistently with SQLite.
- This slice does not invent nontransactional multi-query migration execution.
  Slice 6 must explicitly design reviewed, partially-applied MySQL migrations;
  an engine config switch does not solve that policy.

## Error and lifecycle integration

Reuse ErrorKind: Configuration, Unsupported, Arity, Limit, PoolTimedOut, Closed,
Busy where classification is reliable, Decode, Driver, RollbackOnly. Do not
parse localized message strings; classify PostgreSQL SQLSTATE / MySQL numeric
codes only where justified. Deadlocks/serialization failures are not retries.
SQL errors retain `SqlError::with_source(kind, "PostgreSQL operation failed",
native)` or MySQL equivalent. Host wraps with distinct operation context and
`with_cause`; never interpolate native Display into another wrapper (#11).

Service plugin opens Engine once during config and registers its Arc. Startup
failure publishes no partially usable Server. HTTP cancellation drops the awaited
SQL future; lease/session cleanup still follows this contract. CLI/in-process
use has identical SQL semantics without requiring HTTP or its runtime host.
