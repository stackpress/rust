## Codec matrix

Results are decoded by their wire type; domains over supported base types decode
as that base type. PostgreSQL 12 RowDescription erases that domain identity.
Advertised custom/domain/unknown OIDs are rejected, including for NULL; no catalog
lookups or SQL analysis attempt to recover an erased domain. Server constraints
continue to govern domain writes.

| Wire type | `SqlValue` output | Bind contract |
| --- | --- | --- |
| BOOL | Bool | Bool only |
| INT2 / INT4 / INT8 | Integer, exactly widened to i64 | Integer checked against the prepared width |
| FLOAT4 / FLOAT8 | Finite Real, f32 exactly widened to f64 | Finite Real; f32 narrowing rejects overflow to infinity |
| TEXT / VARCHAR | UTF-8 Text | Text; canonical DateTime uses validated UTC text |
| BYTEA | Blob, including empty/non-UTF-8 bytes | Blob |
| JSON / JSONB | Validated, compact, recursively key-sorted Text | Validated Text only for an explicitly prepared JSON target |
| Supported type with SQL NULL | Null | Prepared metadata selects the matching typed NULL |
| NUMERIC, DATE/TIME/TIMESTAMP, UUID, arrays, enum, other OIDs | Decode error, also for typed NULL | Unsupported error; an explicit cast to a supported target is required |

JSON null is Text("null"), distinct from SQL NULL. JSON output preserves numeric
digits as raw tokens, rejects duplicate object keys visible on the wire, and
enforces an explicit 128-level traversal bound. This deterministic format does not
promise RFC 8785 numeric normalization. JSONB may already have normalized its
input on the server. Plain TEXT is never guessed to be JSON.

Output preserves column order and duplicate names; ambiguous name lookup fails.
Names and values are charged before retained copies. JSON wire input must fit
the remaining byte budget before parsing, and serialization is separately bounded.
Limits bound retained payload and parsing input, not total heap, driver packets,
server memory, or all transient parser allocations. A limit failure returns no
partial result. Invalid UTF-8 and nonfinite native reals fail explicitly.

## Completion and lease reuse

Every acknowledged ordinary query or completed transaction runs DISCARD ALL as
its own command outside a transaction, clears SQLx statement bookkeeping, then
restores the adapter's application name and standard string policy. This removes
temporary tables, session settings, advisory locks and prepared state. Only a
fully acknowledged reset returns the connection to the pool. Prepared caching
remains disabled; transaction-mode external poolers are not covered.

Failure, cancellation, incomplete rows, failed reset or uncertain transaction
control retires the connection. A failed/cancelled transaction run becomes
rollback-only; even an acknowledged rollback then retires its uncertain lease.
Reset and retirement each have a finite cleanup deadline. Background retirement
retains admission and is observed by re-awaitable database close.

Affected counts come from this statement's completion. `insert_id` is always
None; RETURNING rows carry generated values. Native parameter metadata on the
same lease independently checks missing/extra binds, including cast parameters
and literal question marks.

Cancellation is loss of observation. A write or COMMIT may have succeeded even
when the caller receives no result or reset/cleanup fails. The adapter never
retries, reports a cancelled commit as certainly rolled back, or promises
immediate remote session termination. Primary errors retain their typed cause;
secondary retirement errors remain separately available through `cleanup_error`.

## Native verification

Plain tests report ignored native groups when STACKPRESS_TEST_POSTGRES_URL is
unset. Opt-in M0/M1/M2 runners create one unique database each, watchdog their child
scenarios, close their own connections and drop only their exact owned database.
PostgreSQL 12.2 is the characterized local server; other versions, TLS, external
poolers and generated-store integration need separate evidence. M2 adds a shared
Database/TransactionSession suite against both network backends and the existing
SQLite reference, plus bounded concurrent load and resource accounting. SQLite
runs offline through the PostgreSQL M2 target; only the network cases are gated.
