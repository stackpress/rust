# Test the adapter without touching application data

Plain tests are self-contained. Native groups run only when the matching opt-in variable is
set; an unset gate reports ignored cases instead of passing an early-return test.

## Run offline checks

From the repository root, with both database variables unset, run:

```sh
cargo +1.96.0 test -p stackpress-sql-postgres --offline --locked
cargo +1.96.0 test -p stackpress-sql-postgres --doc --offline --locked
```

The ordinary Rustdoc example is compile-only (`no_run`); the ownership examples intentionally
fail compilation. Markdown Rust examples are separate compiled-consumer checks, not Cargo
doctests. Setting a native gate does not make a `no_run` doctest contact a server.

The M2 target also runs the shared contract suite against an independently owned temporary
SQLite file without a network gate.

## Opt into the authorized local server

Supply `STACKPRESS_TEST_POSTGRES_URL` privately for the existing development server:
PostgreSQL 12.2 on localhost:5432 as postgres.
The local fixtures use passwordless authentication and explicit plaintext; the gate is test
configuration, not a production URL API. Fixtures restrict host, port and identity to this
local policy. Do not point the gate at shared or production infrastructure.

With the gate set, the same test command runs M0 lifecycle, M1 codec/cancellation and M2
shared-contract/pool-stress groups. The shared suite covers values, outcomes, transactions,
errors, cancellation and close against PostgreSQL, MySQL and the SQLite reference.
Native child scenarios have parent watchdogs that kill and reap on timeout; a timeout fails
the test. Administrative cleanup has an independent deadline.

## Preserve fixture ownership

Each M0/M1/M2 runner creates one unique bounded ASCII database name with CREATE DATABASE,
without IF NOT EXISTS. Cleanup is armed only after creation succeeds. Children touch only
that database. The parent closes its own connections and drops that exact database on
success, failure and unwind; PostgreSQL 12 uses no DROP WITH (FORCE).

Never enumerate other databases, change roles/global settings, terminate unrelated sessions
or sweep names by prefix. A failed drop fails the test and reports the exact name. Retain
every printed `created owned database` / `drop confirmed` pair in verification evidence.
Do not print the URL, credentials, native SQL or native diagnostic messages.

## Interpret the evidence

M0 proves typed binds, exclusive transactions, cancelled-lease disposal and close. M1 adds
the strict codec matrix, native arity/outcomes, cancellation seams and acknowledged reuse.
M2 adds the common behavioral suite and bounded concurrent resource accounting with small
pools. Generated-store evidence is bounded as described below. These proofs do not establish
production load capacity,
certificate interoperability, arbitrary external poolers or other platforms.

## Generated stores

The same M2 suite executes the actual generated-workspace Profile, Project and Task stores
through Engine and Database on SQLite, PostgreSQL and MySQL. The observation wrapper counts
BEGIN, run, commit and rollback entries without retaining SQL or binds. Unsupported atomic
operations have zero entries, including when the underlying BEGIN would fail.

| Operation | SQLite reference | PostgreSQL | MySQL |
| --- | --- | --- | --- |
| Generated table plans | Executed | Executed | TEXT keys return SQL Unsupported before IO |
| Get, typed filters, paging, counts, base-only atomic reads and row codecs | Executed | Executed | Executed on harness-owned tables |
| Generated RETURNING CRUD, conflicts/upserts, batches and soft delete/restore | Executed | Executed | Data Unsupported before IO, including before an atomic BEGIN |
| Literal q containment | Executed | Data Unsupported before IO | Data Unsupported before IO |
| Includes and exact page totals | Executed in one SQLite snapshot | Data Unsupported before IO | Data Unsupported before IO |
| Core targeted InsertConflict | Executed | Executed | SQL Unsupported before IO |

MySQL fixture tables use explicit harness-owned VARCHAR(191) key/index widths and InnoDB;
this is not generated DDL support or a string-key width policy. Seeding uses generated
input preparation and codecs with a harness-owned insert that omits RETURNING. No generated
executing write is rewritten to bypass its capability gate. MySQL therefore proves generated
reads and write refusals; PostgreSQL and SQLite prove the successful mutation sequences.

The three-model fixture contains no datetime ordering field. That existing Unsupported
boundary on network backends is retained; generated temporal cases remain SQLite-only.
Includes, exact page totals and literal q are not enabled by adapter integration. See the
[shared harness guide](../../src/tests/support/README.md) for its bounded fixture and test layout.
