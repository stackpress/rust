## Codec matrix

| Native type | `SqlValue` output | Bind contract |
| --- | --- | --- |
| Signed integer families | Integer, exactly widened to i64 | Integer uses the native signed i64 codec |
| Unsigned integer families | Integer when at most i64::MAX; overflow is Decode | No unsigned `SqlValue` is invented |
| BOOL / TINYINT(1) | Integer; Row bool conversion accepts only 0/1, rejecting 2 | Bool has a native typed bind |
| FLOAT / DOUBLE | Finite Real; f32 exactly widened to f64 | Finite Real uses the native f64 codec |
| UTF-8 CHAR/VARCHAR/TEXT families | Text | Text; canonical DateTime uses validated UTC text |
| BINARY/VARBINARY/BLOB families | Blob, including empty/non-UTF-8 bytes | Blob; charset/type flags distinguish binary from text |
| JSON | Validated, compact, recursively key-sorted Text | Text with an explicit JSON cast or JSON column; server validates input |
| Supported type with SQL NULL | Null | Native typed NULL bit; casts/columns supply the target |
| DECIMAL, DATE/TIME/DATETIME/TIMESTAMP, ENUM/SET, geometry, other types | Decode error, also for typed NULL | Explicitly cast unsupported output to a supported wire type |

JSON null is Text("null"), distinct from SQL NULL. JSON decoding preserves numeric
wire tokens without converting through f64 and rejects duplicate keys visible on
the wire. MySQL can normalize JSON before
returning it; this adapter cannot reconstruct its original spelling or duplicates.
Traversal is explicitly limited to 128 levels. Deterministic JSON here does not
promise RFC 8785 numeric normalization. Ordinary TEXT is never guessed to be JSON.

Column order and duplicate names survive; ambiguous name lookup fails. Names and
values are charged before retained copies. JSON wire input must fit the remaining
byte budget before parsing and serialization is separately bounded. These limits
bound retained payload and parsing input, not total heap, driver packets, server
memory or every transient parser allocation. Failures return no partial result.
Invalid UTF-8 and nonfinite reals fail explicitly; strict MySQL rejects native
nonfinite arithmetic before decoding.

## Conservative lease reuse

SQLx 0.9 exposes no public MySQL session reset. Query carries no builder/raw
provenance. Without a driver fork, the adapter uses a token-aware allowlist:

- Only plain SELECT/INSERT/UPDATE/DELETE qualify. Quoted strings and ordinary
  comments are scanned; malformed tokens, executable comments and batches retire.
- Variables (`@`), SET commands, temporary/locking/handler/prepared commands,
  OUTFILE/DUMPFILE, SQL_CALC_FOUND_ROWS, and unknown constructs retire. UPDATE's
  ordinary SET clause is allowed.
- Only explicitly listed pure builtin/grouping calls qualify. Unknown, qualified
  or quoted calls retire, as do GET_LOCK/RELEASE_LOCK and other session functions.
  CTEs, EXPLAIN and DDL conservatively retire even when their effect is harmless.

Allowed calls/grouping forms are VALUES, IN, EXISTS, AS, CAST, CONVERT, COUNT,
SUM, MIN, MAX, AVG, ABS, COALESCE, NULLIF, IFNULL, IF, CONCAT, LOWER, UPPER,
LENGTH, CHAR_LENGTH, JSON_EXTRACT, JSON_UNQUOTE, JSON_CONTAINS, JSON_ARRAY,
JSON_OBJECT, CONNECTION_ID, VERSION and DATABASE. Other functions retire.

This is a lexical scope, not a general SQL/session-effects analyzer. It assumes
reusable DML has no session-mutating server triggers; it does not inspect trigger
or routine definitions. Applications needing such hidden effects must arrange a
separate connection boundary. There is no new provenance API or general reset
claim. Raw SET/variables already fail the core Query gate; the native test also
injects private session state to prove the retirement boundary clears it.

An allowlisted lease returns only after complete row/command acknowledgement and
an acknowledged ROLLBACK safety command. That command does not clear arbitrary
session state. Any disallowed statement makes the entire lease retire, including
inside a transaction after acknowledged commit/rollback. A clean transaction may
reuse only after acknowledged completion and exclusively allowlisted successful
runs. Errors/cancellation poison the transaction and retire even after rollback.

Reset and retirement each have finite cleanup deadlines. Background retirement
retains admission and is included in re-awaitable close. Uncertain writes/COMMITs
are never retried or called certainly rolled back. Cleanup failure after a write
can return an error although that write succeeded. Primary errors retain typed
causes; secondary retirement errors remain in `cleanup_error`.

## Outcomes and transaction scope

Affected counts and insert IDs come from this statement's native completion;
zero ID becomes None, positive ID is checked into i64. An overflowing ID fails
decoding after the write, with no retry. No cached last-ID query is used. Native
prepared parameter counts independently reject missing/extra binds; literal
question marks do not count. Transactions require transactional tables such as
InnoDB. MySQL DDL is rejected inside transactions before its implicit commit;
external nontransactional schemas have no rollback guarantee.

## Native verification

Plain tests report ignored native groups when STACKPRESS_TEST_MYSQL_URL is unset.
Opt-in M0/M1 runners create one unique database each, watchdog their child cases,
close their own connections and drop only that exact owned database. MySQL 8.0.27
is the characterized server. Other versions, TLS, pool stress and generated-store
integration require separate evidence.
