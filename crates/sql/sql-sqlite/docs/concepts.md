# SQLite execution concepts

Several operations may share a database file, but a transaction must keep its own connection
until it finishes. The adapter manages that ownership with bounded admission and connection
leases. Start with the [temporary-file walkthrough](getting-started.md) for the concrete
open, query and close sequence; the [SQL core concepts](../../sql/docs/concepts.md#5-engine-and-transaction-ownership) explain
the shared Engine, Transaction, builders and values.

Contents: [File policy](#file-policy-comes-from-the-application) ·
[Pool](#eager-open-and-a-bounded-pool) · [Leases](#admission-travels-with-the-lease) ·
[Transactions](#transactions-keep-exclusive-ownership) ·
[Rollback-only](#failure-can-make-a-transaction-rollback-only) ·
[Cancellation](#cancellation-loses-observation) · [Shutdown](#closing-and-runtime-shutdown) ·
[Arity](#prepared-arity-is-checked-before-execution) · [Limits](#rows-and-sizes-are-bounded) ·
[Errors](#outcomes-and-error-disclosure).

## File policy comes from the application

`SqliteOptions::file(path)` names an existing file by default. Creation requires
`create_if_missing(true)`; opening an existing file read-only requires `read_only(true)`.
Read-only and creation cannot both be enabled. Parent directories are not created for you.

Validation rejects empty, non-UTF-8 and control-containing paths, the exact `:memory:` name
and paths starting with `file:`. URI and in-memory modes are not exposed. Relative paths
resolve from the process working directory; validation does not canonicalize or sandbox
the path. Keep file selection in application configuration, as the
[profile example](guides/sql-profile.md) does.

Options also reject unknown serialized fields and invalid finite budgets. A missing
serialized `read_only` field defaults to false for older configuration compatibility.
See [SqliteOptions](api/options.md) for all defaults and bounds. `validate()` checks data;
permissions, parent directories, file existence and native opening are checked by `open()`.

## Eager open and a bounded pool

Awaiting `SqliteDatabase::open` opens a real pool connection before returning. This exposes
missing files, permissions and native-open failures at startup. It creates no tables and
falls back to no alternative database. Cancelling an open retires its unpublished pool;
creation may already have left an empty file.

The default maximum is four connections, with a zero minimum. Eager open does not mean all
four are preallocated. Every connection enables foreign keys and FULL synchronous policy;
writable opens request WAL, while read-only opens preserve the existing journal mode.
Worker-command and row channels each have capacity 16.

Pool acquisition has a finite default budget of 5,000 ms. Native lock contention has a
separate 1,000 ms busy timeout. These are acquisition/lock budgets, not a deadline for every
SQL statement. SQLx runs SQLite work on its native worker; an expensive query need not block
the Tokio executor, but it can continue after its async observer is cancelled.

Before each statement, the adapter refreshes SQLite's schema view and discards cached
statement metadata. This favors correct results after explicit sequential DDL over statement
cache reuse. It does not coordinate external online schema changes or file replacement.

## Admission travels with the lease

Before pool acquisition, the adapter tries to take one admission slot. The default cap is
128 in-flight operations, including held transactions and callers waiting for a connection.
Exhausted admission returns `ErrorKind::Limit` immediately. An admitted caller can still
wait for a connection and receive `PoolTimedOut`.

Once acquired, a lease owns both the connection and its admission permit. An acknowledged,
successful ordinary statement returns its connection for reuse. Failed or cancelled work
retires the connection. Every completed or dropped transaction also retires its connection,
even after a successful commit or rollback.

While the cleanup runtime remains alive, retirement retains admission until the worker
acknowledges connection shutdown. Cancelling a waiter before it acquires a connection drops
its admission permit; no application SQL has been queued on that waiter's behalf. The
[shutdown rules](#closing-and-runtime-shutdown) explain the weaker no-runtime fallback.

## Transactions keep exclusive ownership

`Engine::begin()` returns a non-Clone Transaction with one leased connection across awaits.
Query methods borrow it through `&mut`; commit and rollback consume it. This is exclusive
Rust ownership of a connection, not SQLite's `BEGIN EXCLUSIVE` mode: the adapter issues
ordinary deferred `BEGIN` and exposes no isolation or savepoint options.

Pass `&mut Transaction` into helpers that must join the same transaction. Calling Engine
again acquires separate work; neither nested event dispatch nor handler success implicitly
inherits or commits a transaction. With a size-one pool, holding a transaction and awaiting
an Engine query can run into the acquisition timeout.

Uncommitted drop initiates rollback by closing the native connection. This also covers a
task abort or a panic that unwinds and drops the guard. Drop is nonblocking and does not
acknowledge rollback completion; process abort, power loss and runtime shutdown are separate
limits. Prefer explicit rollback when your application needs its result.

## Failure can make a transaction rollback-only

Before a session starts native statement work, it marks itself rollback-only. Only a
successful acknowledgement clears that state. A native statement failure or cancellation
of a polled session run therefore prevents more queries and commit with
`ErrorKind::RollbackOnly`; explicit rollback remains available. A core rendering or
validation failure before entering the session does not itself poison the native session.

For example, a missing table fails natively even though its SQL is valid:

```rust
use stackpress_sql::{DialectId, Engine, ErrorKind, Sql};
use stackpress_sql_sqlite::{SqliteDatabase, SqliteOptions};

async fn example() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let engine = Engine::new(
        SqliteDatabase::open(
            SqliteOptions::file(root.path().join("failure.sqlite")).create_if_missing(true),
        )
        .await?,
    );
    let mut transaction = engine.begin().await?;
    let primary = transaction
        .fetch(&Sql::literal(
            DialectId::Sqlite,
            "SELECT * FROM missing_profile",
        ))
        .await
        .unwrap_err();
    let blocked = transaction
        .fetch(&Sql::literal(DialectId::Sqlite, "SELECT 1"))
        .await
        .unwrap_err();
    assert_eq!(blocked.kind(), ErrorKind::RollbackOnly);
    let primary = transaction.rollback_after(primary).await;
    assert_eq!(primary.kind(), ErrorKind::Driver);
    engine.close().await?;
    Ok(())
}
```

`rollback_after` consumes the guard and returns the original error. A failed rollback is
available through `cleanup_error()`; it does not replace the original native source.
The shared transaction walkthrough belongs to the [SQL core](../../sql/docs/guides/transactions.md).

## Cancellation loses observation

Dropping an async future does not prove that its native write did not happen. An autocommit
write may already have persisted. Cancelling a consuming commit future can leave either a
committed or rolled-back outcome; the caller cannot infer which from cancellation alone.

The adapter never retries a statement, transaction, plan or commit automatically. A timeout
around commit also creates this unknown outcome. Reconcile through application-owned state
before deciding what to do next; replaying blindly can duplicate effects. Dropping even an
unpolled consuming commit/rollback future consumes its session and initiates retirement.

## Closing and runtime shutdown

First stop application admission and finish or drop outstanding guards. Then await
`Engine::close()` while the owning Tokio runtime is alive. On its first poll, close closes
adapter admission, rejects new work and wakes pool waiters. It waits for held leases;
keeping a transaction alive while awaiting close can prevent completion.

The close future may be cancelled and re-awaited. The application supplies a shutdown
deadline; a timed-out close is not an acknowledgement that all native work stopped.
There is no universal server async-finalization hook in this example; embeddings explicitly
close the Engine, while the ordinary CLI follows its documented RAII path.

With a live runtime, Drop spawns native connection close and retains the permit until
acknowledgement. Without a live runtime, Drop detaches/drops the worker handle and releases
the permit without blocking. That fallback initiates best-effort teardown, but guarantees
neither worker completion nor continued admission coupling. Await close before runtime
shutdown when acknowledged cleanup matters.

## Prepared arity is checked before execution

SQLite normally accepts missing arguments as NULL. The adapter instead prepares the query
and compares native parameter metadata with the supplied bind count on the same lease,
before binding and execution. Missing or excess arguments return
`ErrorKind::Arity { expected, actual }`.

The core raw-SQL gate accepts anonymous `?` SQLite binds and rejects batches, control
commands and named/sparse placeholders. Question marks inside literals and comments are
not parameters. Trusted SQL remains an author assertion; preparation does not sanitize
request-controlled SQL. Follow the [core trusted-SQL concepts](../../sql/docs/concepts.md#8-trusted-sql)
for that boundary.

## Rows and sizes are bounded

Default QueryLimits allow 1 MiB of SQL, 1,000 binds, 10,000 retained rows and 16 MiB of
retained column-name/value bytes. The byte limit also bounds bind bytes independently.
Choose smaller limits with `Engine::with_limits` before sharing it.

The collector reserves each row before decoding, then measures borrowed names, text and
blobs before copying. Integer and real payloads charge eight bytes; NULL charges no value
bytes, but its column name still counts. UTF-8 sizes are byte lengths. Exceeding a limit
returns an error, never a truncated or partially decoded success.

These limits do not bound container/allocator overhead, SQLite VM memory or time, or a
native cell's allocation before decoding. A write may already have effects before result
collection fails; use an explicit transaction when rollback is required. SQLite storage
types determine decoded values: booleans read as integers, JSON/date text stays text, and
nonfinite computed reals fail decoding. Strict conversions and duplicate-name lookup live
in the [core decoding guide](../../sql/docs/guides/decode-rows.md) and
[value concepts](../../sql/docs/concepts.md#3-values-and-identifiers).

## Outcomes and error disclosure

Affected counts and optional insert IDs belong to the current statement. `insert_id` is an
observed change in native last-rowid, not a guaranteed key for every insert. Reusing an
explicit rowid or inserting into a WITHOUT ROWID table can yield `None`; use
`INSERT ... RETURNING` when each insert needs an authoritative key.

Adapter-produced SqlError Display and Debug hide SQL text, file paths and native diagnostics.
`std::error::Error::source()` deliberately retains the original driver detail for trusted
diagnostics; it may reveal identifiers and paths. Formatting the source chain is an explicit
disclosure choice. Configuration and first-party validation errors may have no native
source. See the [adapter error categories](api/database.md#6-error-categories).

You can now distinguish a completed operation from an unobserved one, and a bounded pool
from bounded retained output. Apply these rules in the [service guide](guides/server-service.md)
and [profile application walkthrough](guides/sql-profile.md).
