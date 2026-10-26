# Shared backend contract and pool stress

`contract.rs` runs one public Database/TransactionSession suite against PostgreSQL,
MySQL and the existing SQLite adapter. Its grouped assertions live in `contract/`.
MySQL includes these same physical files by path. Only native syntax, strict bool
representation, generated-ID rules and explicit capabilities vary. Assertions
are shared; there is no copied reference implementation or test-support crate.

`stress.rs` is likewise included by both network M2 executables. Backend fixture
construction stays in each adapter's `src/tests/m2.rs`; native SQLx handles remain
private. No SQL core or SQLite source change is required. PostgreSQL's SQLite
reference dependency is dev-only; Cargo.lock adds only that existing package edge.

## Run the checks

```sh
cargo +1.96.0 test --offline --locked \
  -p stackpress-sql-postgres -p stackpress-sql-mysql --test m2
```

With native gate variables unset, PostgreSQL's target still executes and reports
the SQLite reference; each backend explicitly ignores its two network groups.
Set STACKPRESS_TEST_POSTGRES_URL and STACKPRESS_TEST_MYSQL_URL to the authorized
local development servers for the real backend cases. Credentials are never
printed. Each native target CREATEs one unique database, arms cleanup after
success and DROPs only that exact name after killing/reaping its children.
SQLite reserves an exact unique file using create_new and removes only that
file and its exact -wal/-shm sidecars after its child exits.

Every scenario has a 30-second parent watchdog with kill/reap; the native fixture
has a 120-second total budget. Native administration has separate 9-second IO and
10-second wall deadlines. SQLite file cleanup has its own 10-second wall deadline.
Failure/unwind never converts a failed scenario or cleanup into a passing run.
Parent SIGKILL/power loss can leave an exact named fixture behind; never sweep
by prefix. Generated-store integration remains deferred by the coordinator.

## What is proved

The shared contract covers every SqlValue variant, SQL/JSON null, column order
and duplicates, statement-local counts/IDs/RETURNING, transaction affinity,
commit/rollback/Drop/unpolled completion, poison after native or validation errors,
transactional-DDL capability differences, prepared arity, limits, dialect checks,
retained causes/redaction, cancelled acquisition/native work, unknown commit
outcome without duplicated increments, and interrupted/re-awaited close.
MySQL blob results use an explicit binary cast: bare parameter metadata can
advertise text, and strict decoding follows that wire type. This does not loosen
the codec. M1 retains the larger backend-specific native codec/cancellation matrix.

Stress runs eight owned workers at native pool sizes 1, 2 and 4. Each run lasts
up to two seconds, capped at 2,000 requests per worker. Reads, writes, acknowledged
commit/rollback, native identities, final row counts and live pool/admission
snapshots provide independent observations. Every worker is joined before the
final checks. A cancelled/error worker is owned by JoinSet and aborted on failure.
The parent retains fixture cleanup authority after process exit.

Counters are test-only adapter dispositions: acquisition, acknowledged/reset
normal return, or retirement. Normal return is eligibility for SQLx reuse; SQLx
can still dispose of a returned connection after pool close. Actual native IDs
are checked separately during sustained load and stay within the configured pool
size. No extra IDs, errors or retirement are allowed in clean load. Quiescence
requires all native connections idle, zero background cleanup and all admission
permits restored. Close requires native/idle/cleanup counts all zero.

The deliberate error/temp-table/cancel phase requires exact disposition counts:
PostgreSQL 78 acquired / 48 returned / 30 retired; MySQL 78 / 24 / 54. PostgreSQL
resets temp state, while MySQL retires it. Controls prove prompt admission overflow,
200 ms PoolTimedOut distinct from Closed, 64 cancelled acquire/begin waiters,
no extra published leases, and successful re-await of an interrupted close.
Logs record requests, committed rows, durations, peaks, identity sets and counts.
These are logical pool/resource observations, not a hard heap or OS socket bound.

## Policy and evidence limits

The conservative MySQL allowlist and PostgreSQL domain wire-type boundary were
coordinator decisions. Production codecs, reset/reuse policy, deadlines and
error precedence are unchanged in M2. Only private cfg(test) scalar snapshots
and counters were added alongside the new tests. No driver fork, patch, catalog
lookup or public instrumentation API is introduced.

Local evidence characterizes PostgreSQL 12.2 and MySQL 8.0.27 over explicit
plaintext development connections. TLS/auth variants, other versions/targets,
external poolers, hidden session-mutating MySQL triggers and malicious transport
stalls keep their earlier limits. Per-crate line coverage does not prove every
branch, target or publication readiness.
