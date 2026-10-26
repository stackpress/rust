# Slice 4 lane B M1 receipt

Task: [00007 SQL](tasks/00007-slice-4-sql.md), SQLite execution lane, milestone M1. Authority: the coordinator's M1 release, G24, and [M1 lane B in the milestone plan](slice-4-design-verification.md). This receipt stops at M1 for coordinator re-verification; it does not release M2 or mark the entire slice complete.

## Baseline and ownership

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0`; branch `trial/3-s4-m0`.
- Starting/unchanged HEAD: `8985fe03300237023adcedc147b0e90d4ea1af26`, clean after the coordinator's M0 merge/re-verification.
- Production/test writes are restricted to `crates/sql/sql-sqlite/**`. This receipt is the sole `.agents` addition.
- No edits to core SQL, lib, server, CLI, Idea, root manifests/facade or Cargo.lock. No dependency additions or core trait/type change was needed. No commit, push, listener, publishing, deployment or messages to other agents.
- The adapter builds against the frozen M0 core surface. Lane A's future SqlValue/column-metadata/builder additions are not present in this checkout and require integration re-verification after the coordinator merges them.
- Compiler: rustc 1.96.0 (`ac68faa20`, LLVM 22.1.2), Cargo 1.96.0, native `aarch64-apple-darwin`. SQLx remains exactly 0.9.0, defaults off with runtime-tokio/sqlite. No crate feature flags were introduced.

## Implemented M1 behavior

### Prepared binding and native errors

- Native preparation/arity checking remains on the execution lease, before binding/effects. Missing, excess and zero-argument mismatches return typed Arity errors. Literal/comment/quoted-identifier question marks remain unchanged; named/sparse markers and batches remain rejected by the M0 core gate.
- All M0 scalar representations bind without mutating detached input: NULL, bool, exact signed integers, finite reals, UTF-8 text and arbitrary blobs. JSON/date content remains explicitly selected text; no implicit object coercion, row JSON parsing, date conversion or epoch fallback was added.
- The native suite covers i64 extrema, f64 MAX/MIN_POSITIVE, empty text/blob, every byte value, embedded NUL/newline, SQL-looking text, quote/comment forms, Unicode, explicit JSON/date strings, repeated execution, exact 1000 binds and one-over input limits.
- SQLx decode/column errors now map to ErrorKind::Decode; encode/invalid-argument errors map to InvalidQuery; configuration errors map to Configuration. Closed, pool timeout, busy/locked and other native failures retain their stable existing categories. Every mapped failure preserves its original native source and safe default formatting.
- An actual invalid-UTF-8 SQLite TEXT result returns Decode with a source; it cannot become an empty result. Real constraint, missing-table, read-only, busy, rollback and commit failures also remain failures.

### Retained row collection and limits

- New private `collect::OutputBudget` owns row/name/value-byte accounting. The execution module owns prepare/bind/native streaming/outcomes; existing Lease owns native/admission lifetime; options remain separate configuration data.
- Previously a whole row's owned text/blob values were copied before its byte check. The collector now borrows each native name/text/blob to measure it, reserves bytes with checked arithmetic, and copies only after that reservation succeeds.
- Row admission occurs before decoding. Failed reservations leave the byte counter unchanged, never exceed the cap, and return Limit instead of partial/truncated success. Row order, column order and duplicate names are retained; core named lookup still rejects ambiguity.
- Boundary/one-over tests cover text, blobs, integers, reals, NULL/name bytes, cumulative bytes across rows, row limits, zero budgets with zero rows, UTF-8 byte lengths, SQL/bind/input-byte limits, arithmetic overflow, and a 1 MiB native blob under an 8-byte retained cap.
- A transactional multi-row INSERT RETURNING that crosses its row cap returns Limit, becomes rollback-only, and leaves no uncommitted rows after guard cleanup. Inherited M0 tests continue proving admission retention through cancelled native cleanup.
- These are retained payload limits, not a total RAM limit: allocator/container overhead, SQLx's bounded native row channel, native VM work and allocation of a native cell before decoding remain outside the byte counter. No allocator instrumentation or immediate native interruption is claimed.

### Statement-local outcomes

- Streaming native metadata still determines rows, including leading comments, WITH and INSERT/UPDATE/DELETE RETURNING; no SQL-prefix row detection was introduced.
- Before/after counters are read on the same exclusive lease. `changes()` supplies SQLite's 64-bit direct statement count; cumulative `total_changes()` prevents a stale count from leaking into SELECT, DDL or EXPLAIN. Trigger side effects do not inflate the root statement's affected count. The [SQLite scalar-function contract](https://www.sqlite.org/lang_corefunc.html#changes) identifies changes() as the changes64 wrapper.
- `insert_id` is now an optional **observed native rowid transition**: it is Some only when the statement changed data and the native last-rowid value changed on that lease. It includes a changed rowid of zero and never returns an old ID on SELECT/UPDATE/DELETE.
- Reusing the previous explicit rowid, inserting the initial rowid zero, or writing a WITHOUT ROWID table can yield None. This field does not promise a domain primary key or an ID for every insert. RETURNING remains authoritative. This conservative contract avoids SQL-text classification or a core API change; see the [native rowid contract](https://www.sqlite.org/c3ref/last_insert_rowid.html).
- Tests cover multi-row affected counts, zero-row writes, INSERT OR IGNORE, CTAS/EXPLAIN, duplicate/quoted aliases, trigger isolation, reused/zero IDs, WITHOUT ROWID, and four barrier-released concurrent inserts whose returned IDs/counts belong to their own statements.

### Pool, options and close

- Added explicit `read_only(bool)` file options. Missing read_only in serialized M0 configuration defaults false; creation remains opt-in and conflicting read-only+creation options fail validation.
- Writable opens retain WAL/FULL policy. Read-only connections preserve the existing journal mode, including a real DELETE-mode file, and reject writes without opening a fallback file. Missing read-only files fail without creation.
- Expanded option-bound/unknown-field/UTF-8-path/URI tests. Defaults and finite maxima remain unchanged; no in-memory, native-resource, network or arbitrary-policy API was added.
- Two simultaneously acquired native connections each prove foreign_keys ON, WAL, synchronous FULL and the configured busy timeout. A third acquisition hits the finite pool deadline; close leaves zero live connections.
- A controlled already-queued waiter receives Closed when close begins while a transaction holds the only lease. Close remains pending, can be cancelled/re-awaited, and completes after the existing transaction rolls back. New queries and begins remain Closed.
- Conservative M0 retirement, rollback-only state, rollback/drop/abort/unwind behavior, unknown commit outcomes and no automatic query retry are retained. The unchanged native host proof still uses ServerBuilder registration and prints `profile-name: Ada`.

## Upstream integration inventory mapping

The [integration inventory](slice-4-design-tests-integration.md) and [adapter inventory](slice-4-design-inventory-framework.md) govern this mapping. Upstream TypeScript tests were read as reference, not run or copied as production code.

| Inventory Now row | M1 lane B evidence / boundary |
| --- | --- |
| inquire-sqlite3 #1, Should create profile table | `tests::m1::outcomes::crud`: real native schema, defaults, nullable fields and owned metadata. Uses trusted static SQL until lane A merges; no claim that the full Create builder is present here. |
| #2, Should safely replay creation of a table with indexes | Same scenario executes idempotent table/unique-index/regular-index statements twice and inspects sqlite_master for exactly two indexes. Multi-statement plan validation/atomic execution remains the later core integration. |
| #4, Should flatten data | `values::binding` and `outcomes::crud`: deterministic typed bind order, explicit JSON/date text, bool-to-integer, unchanged detached input. Generic nested-record flattening is S4-F, not an adapter-owned helper. |
| #5, Should insert profile values | Bound multi-row INSERT and RETURNING; malicious-looking text and false/empty/NULL values survive exactly. |
| #6, Should update profile values | Bound WITH/UPDATE RETURNING, strict values, exact affected count and no inherited insert ID. |
| #7, Should fetch all profiles | Explicit ORDER BY, owned ordered cells, nullable/native scalar values and text JSON/date values without automatic decoding. |
| #12, Should delete profile | WITH/DELETE RETURNING, exact own-statement outcome, empty match and persisted remaining data after reopen. |
| stackpress-sql helpers #1/#2 Now encoding aspects | Native scalar and explicit-text IO are exercised; permissive coercion/invalid-date-to-epoch behavior is deliberately not reproduced. Core conversions belong to lane A; schema/model conversions remain S4-F. |
| New inventory expansion relevant to M1 | Exact arity, hostile bound input, WITH/RETURNING, NULL/false/zero, duplicate aliases, finite limits, pool/close/reopen and typed native failures; inherited M0 ownership/cancellation proofs rerun. |

SQLite #3 native supported ALTER and #8-#11 JSON predicate execution are M2 lane B. Network/PGlite/framework rows retain their named later scope. G26's opt-in network renderer tests are outside this SQLite-only write set; no network production edge was added.

## Bounded M1 review passes

These implement the standing Rust programming checks inside this milestone, without claiming the later slice-wide refactor/docs gate is released.

1. **Logic/error/limit review:** reproduced M0's absent insert-ID observation with a failing native regression. Enforced byte admission before owned copies, typed real decode failures and conservative native ID snapshots. A follow-up uses changes64 through SQL instead of SQLx's 32-bit sqlite3_changes wrapper. Exact/multi-row/trigger/zero/one-over/overflow tests and the inherited suite passed.
2. **Responsibility/ownership review:** collection policy is a focused private module; execution, options and Lease retain their natural responsibilities. No speculative generic layer or additional shared-crate abstraction was needed. Borrowed native text/blob values are copied before returning and never cross an await. Reviewed cleanup after output errors and close/waiter ordering; no extra refactor was justified. Targeted tests passed again.
3. **Consistency/Rustdoc/safety review:** documented optional rowid semantics, read-only journal preservation, serialized compatibility and retained-memory limits. Added a real pure-options doctest. Reviewed explicit source chains, default diagnostic disclosure, finite test deadlines, dependency scope and feature boundaries. No first-party unsafe or production lock/callback API was introduced. Final checks below passed.

## Final verification and coverage

All checks ran after the final source changes, on the native target above:

```sh
cargo fmt --all -- --check
cargo test -p stackpress-sql-sqlite --all-targets --locked --offline
cargo test -p stackpress-sql-sqlite --doc --locked --offline
cargo clippy -p stackpress-sql-sqlite --all-targets --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql-sqlite --no-deps --locked --offline
cargo +1.96.0 check -p stackpress-sql-sqlite --all-targets --locked --offline
cargo check -p stackpress --no-default-features --features sql-sqlite --locked --offline
```

- **Passed:** nine package test/parent entries, two doctests, fmt, warning-clean Clippy/Rustdoc, the explicit Rust 1.96 check and the facade consumer check. The unchanged native proof app was built and run successfully beneath a separate 10-second process deadline.
- Five ignored child entrypoints are actually invoked by their parents: ten inherited M0 async scenarios, nine M1 native scenarios, seven compiler-fail fixtures, the runtime-teardown proof and the deliberately stuck watchdog negative control.
- Native scenarios retain an independent 10-second parent watchdog and an 8-second cooperative deadline. Compiler proofs retain their separate finite 180-second parent deadline. Ready/go/barrier synchronization establishes concurrency boundaries; no listener or external database service is needed.

Coverage used cargo-llvm-cov 0.8.6, **only `-p stackpress-sql-sqlite`**, after a clean. Isolated absolute CARGO_TARGET_DIR: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0/target/s4-lane-b-m1-coverage`.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-sql-sqlite --all-targets --locked --offline --no-report
cargo llvm-cov report -p stackpress-sql-sqlite --json --summary-only --fail-under-lines 70 \
  --ignore-filename-regex '(/tests/|/src/tests\.rs)' \
  --output-path target/s4-lane-b-m1-evidence/coverage.json
```

| First-party adapter source | Covered/executable lines | Line coverage |
| --- | --- | --- |
| collect.rs | 62/66 | 93.94% |
| execute.rs | 88/101 | 87.13% |
| lease.rs | 37/38 | 97.37% |
| lib.rs | 111/111 | 100% |
| options.rs | 55/55 | 100% |
| **Library, package alone** | **353/371** | **95.15%, passes 70%** |

The report was saved **before** running the example, so the package figure uses only the adapter's own tests. Dependency/core source is absent from its report; only test-only source is excluded by the regex. No core coverage is claimed. The coordinator's M0 core-alone **81.19%** remains the figure of record; the former combined-run figure is not reused as per-crate evidence.

The native proof app was then executed with `cargo llvm-cov run -p stackpress-sql-sqlite --example m0-host --locked --offline --no-report`. A subsequent report merged that run before explicit LLVM export of the example binary/source, because the normal report omits examples. Its separate coverage is **64/65 lines, 98.46%**; it is not silently excluded or used to raise the library gate.

Local ignored evidence: `target/s4-lane-b-m1-evidence/` contains check/test/doc/lint logs, package-only raw JSON, the separate example export and coverage-summary.json. `git diff --check` passed and the write-set audit found no core/shared/manifest/lock changes.

## Remaining scope and next gate

- Defensive unreachable/current-driver metadata/type failures and a negative native change-count conversion remain unforced; no multi-billion-row fixture was created to exercise the 32-bit boundary. Native VM/native-row allocation is not bounded by retained payload accounting. Drop cleanup remains best effort outside a live runtime; cancelled commit outcomes remain unknown.
- Read-only support preserves existing journal mode; it is not a filesystem-level security or snapshot-isolation certificate. Native rowid observation is intentionally optional; RETURNING supplies definitive keys, including repeated IDs.
- Only aarch64 macOS is verified. No network SQL, WASM, mobile, Linux/Windows, power-loss or filesystem-durability certification is implied. Full public docs, M2 schema/JSON execution and integration with lane A's new core types/builders remain unreleased.
- KB validator baseline before this receipt: **146 existing broken-link errors, 20 warnings**, principally relocated sibling-repository references. No existing KB file was repaired. Post-write comparison is recorded in the handoff; code checks do not imply a green KB validator.

Stop here for coordinator M1 re-verification and sequenced core integration. No outstanding core type/trait change is requested by this M1 implementation.
