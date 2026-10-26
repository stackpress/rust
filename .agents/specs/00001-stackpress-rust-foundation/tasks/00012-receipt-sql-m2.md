# Task 00012: SQL M2 released-scope receipt

2026-10-27, branch `trial/3-s7-sql-m0`. The requested `git merge trial/3` fast-forwarded from accepted M1 `d952f88` to base/HEAD `2bceb5386cdbe508b698c328309812fffddfe8b5`.
Evidence for [task 00012](00012-slice-7-production-servers.md); independent coordinator review remains pending. The released contract-harness and pool-stress work is complete.
Generated-store integration was not released and was not started. No whole-slice
acceptance or Context promotion is claimed.

## Changes and authority

The MySQL allowlist and PostgreSQL domain wire-type rule are **coordinator decisions**, not owner decisions. M2 preserves those accepted policies and all production behavior.
Only the two owned adapter crates, their Cargo.lock dev-edge and this pre-approved
receipt changed. SQL core, SQLite, root Cargo.toml/facade and other lanes are untouched.
SQLx remains unmodified `=0.9.0`; no fork, patch, tool install or new package/version.

- One shared suite operates through `Arc<dyn Database>` and exclusive TransactionSession.
  Its physical source is PostgreSQL's `src/tests/support/contract.rs` and `contract/`;
  MySQL includes those same files by path. SQLite runs the identical suite as an
  offline reference through the PostgreSQL M2 executable. A dev-only dependency on
  the existing SQLite crate adds one Cargo.lock dependency edge; no shared crate.
- The suite covers every SqlValue variant, SQL/JSON null, column order/duplicates,
  statement-local outcomes/IDs/RETURNING, commit/rollback/Drop/unpolled completion,
  affinity, native/pre-effect-error poison, DDL capability differences, prepared
  arity, dialect/limit validation, source/redaction, cancelled queued/native work,
  unknown-commit increments without duplication, and interrupted/re-awaited close.
  Dialect differences vary syntax or documented native behavior, not assertions.
- Shared `stress.rs` runs eight owned workers at max connections 1, 2 and 4 for up
  to two seconds each, capped at 2,000 requests per worker. It mixes reads/writes
  and acknowledged commit/rollback, joins every task, verifies committed rows,
  samples pool/admission state and observes actual native identities. Private
  cfg(test) counters record coupled acquisition, acknowledged/reset normal return,
  or retirement; no driver or instrumentation API becomes public.
- Quiescence requires idle native count == pool size, zero cleanup tasks and every
  admission permit restored. Close requires native/idle/cleanup counts all zero.
  Error/temp/cancel phases prove exact policy dispositions. Controls distinguish
  prompt Limit from a 200 ms PoolTimedOut and Closed, repeatedly cancel queued
  acquire/begin futures, and re-await interrupted close without extra published leases.
- The existing M0/M1 matrices remain, with their risky native proofs replayed. Public
  codec docs now describe M2 and MySQL's explicit binary-result cast: a bare Blob
  parameter can advertise text metadata, so strict decoding follows wire metadata.
  [Shared test guide](../../../../crates/sql/sql-postgres/src/tests/support/README.md)
  explains syntax, limits, fixture ownership, counters and reproducible commands.

## Fixture and cancellation boundaries

Both exact existing environment gates remain opt-in. Unset native gates report ignored groups; PostgreSQL's target still actually executes/reports SQLite.
Each native target CREATEs one unique bounded ASCII database without IF NOT EXISTS,
arms cleanup after success, closes its own connections and DROPs only the exact
owned name. Parent watchdogs kill/reap scenario children (30 s each, 120 s total).
Administration has a separate 9 s IO / 10 s wall deadline; PostgreSQL retries only
its owned DROP on structured 55006. No catalog scan, role/global-setting mutation,
unrelated session termination, database adoption or prefix sweep occurs.

SQLite's parent reserves create_new on one exact unique file and removes only that file and its -wal/-shm sidecars after reaping the child. File cleanup has its
own 10 s wall deadline and armed unwind guard. Failure, unwind or timeout stays
failing; parent SIGKILL/power loss cannot guarantee remote/file cleanup. Every
created native database and SQLite file in this work has a cleanup confirmation.

Cancellation remains loss of observation, including COMMIT. Shared assertions allow a polled commit's increment to be zero or one, never duplicated. M1's exact
prepare/execute/row/control/reset/cleanup gates still pass. No retry or certain
remote-rollback claim is added. Counters describe adapter return eligibility;
SQLx may dispose of a normally returned connection after pool close. Physical
native identities are independently checked during sustained open-pool load.

## Four test-verified reviews

1. Complexity: reviewed admission vs native acquisition, counter terminal actions,
   deadline placement, cancellation, failure precedence and cleanup authority.
   Added an independent SQLite file-cleanup deadline/armed guard, a lower-bound
   timeout assertion, and exact no-extra-lease accounting for queued cancellations.
   `round1-offline.log` passed; native replay verifies the strengthened controls.
2. Responsibility/abstraction: split the shared assertions into values, outcomes,
   transactions, errors and lifecycle modules with one public-trait router. Native
   fixture construction stays backend-owned; stress observations stay private.
   A new support crate/core extraction is unnecessary. `round2-offline.log` and
   `rounds-native.log` pass after the split.
3. Consistency/Rustdoc/comments: aligned native skip/SQLite pass reporting, readable
   select branches, comments, exact counter semantics, binary-cast documentation
   and the shared test guide. fmt, Clippy -D warnings, doctests and Rustdoc pass.
4. Safety/soundness: reviewed SQLx pool/lease ownership, Drop/close guard order,
   Atomic counter timing, owned JoinSet failure aborts, exclusive transaction borrows,
   generated identifiers, create_new authority, cleanup deadlines and error redaction.
   Test hooks are cfg(test) only; production behavior and public APIs are unchanged.
   No further material edit was justified. Final native M0/M1/M2 coverage replay passes.

Initial failures were test findings: the aggregation omitted workers' collected identity sets, and MySQL's bare binary parameter result advertised text. Fixed
aggregation and requested an explicit binary wire result without loosening codecs.
All failed-run databases were dropped. Early logs are retained in native-first.log
and native-diagnostic.log; native-baseline.log establishes the passing M2 baseline.

## Verification and resources

Rust/Cargo 1.96.0 (`ac68faa20`), aarch64-apple-darwin; servers PostgreSQL 12.2 and
MySQL 8.0.27, explicit local plaintext. All Cargo verification after the one dev-edge lock update used --offline --locked. CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0;
one worktree target directory. Logs and machine-readable evidence are retained at
`/private/tmp/s7-sql-m2/`; checks.py contains exact final matrix commands.

| Check | Result / log |
| --- | --- |
| `cargo +1.96.0 fmt --all -- --check` | Pass; fmt-final.log |
| scoped `clippy --all-targets -- -D warnings` | Pass; clippy-final.log |
| scoped `test`, native gates unset | 17 units + 8 doctests + actual SQLite contract pass; 14 network cases ignored; offline-final.log |
| scoped native M0/M1/M2, both gates set | All pass; rounds-native.log; final source replay in coverage.log |
| `RUSTDOCFLAGS='-D warnings' cargo doc ... --no-deps` | Pass; rustdoc-final.log |
| `cargo +1.96.0 check -p stackpress --no-default-features --features ...` | Each backend and combined pass; check-facade-*.log |
| `cargo test --workspace --all-targets -- --test-threads=1` | 499 pass / 74 expected ignored; workspace-final.log |
| `cargo test --workspace --doc` | 880 pass; workspace-doc.log |
| normal-edge dependency trees | Default/browser no SQLx/driver; production PostgreSQL only sqlx-postgres, MySQL only sqlx-mysql; tree-*.log |

Final clean coverage-run load observations follow. A request is one ordinary query or a completed transaction group (two TX runs), not a native command count.
Every clean phase acquired/returned exactly its request count, with zero retirements;
final row counts equal acknowledged committed writes. Peak admission was 8 throughout.

| Backend | Max / observed native peak | Requests | Committed rows | Duration |
| --- | --- | --- | --- | --- |
| MySQL | 1 / 1 | 6394 | 2400 | 2.0063 s |
| MySQL | 2 / 2 | 9544 | 3582 | 2.0061 s |
| MySQL | 4 / 4 | 13415 | 5033 | 2.0049 s |
| PostgreSQL | 1 / 1 | 3441 | 1289 | 2.0081 s |
| PostgreSQL | 2 / 2 | 6026 | 2263 | 2.0068 s |
| PostgreSQL | 4 / 4 | 9938 | 3730 | 2.0067 s |

Native identity sets contained only the configured 1/2/4 connections. Deliberate retirement counts (acquire/return/retire): PostgreSQL 78/48/30, MySQL 78/24/54.
Each control cancelled 64 queued waiters and observed exactly 34/34/0 published
lease dispositions. Overflow: MySQL 3.416 us / PostgreSQL 7.041 us; acquisition
timeout: 201.509 / 201.335 ms. Stress totals: 6.342 / 6.678 s. Every phase restored
all permits and drained cleanup; final native/idle/cleanup counts were zero.
Whole coverage/build command `/usr/bin/time -l` observed 424099840 bytes maximum
RSS (rounds-native: 151994368). These are whole-command/resource observations,
not a hard heap, packet, OS socket or performance guarantee.

Coverage tool: cargo-llvm-cov 0.8.6, compiler +1.96.0, explicitly selected installed
stable 1.96 LLVM 22.1.2 llvm-cov/llvm-profdata. Fresh cargo clean and llvm-cov clean
--workspace preceded the native-gated scoped `llvm-cov --json --summary-only` run;
CARGO_LLVM_COV_TARGET_DIR=target. Test-only exclusion regex:
`/tests/|/src/(tests|probe|m1|m2)\.rs$`. No production file excluded; child profiles
included. Doctests passed separately. Reports: coverage.json / coverage-summary.json.

| Crate | Executable lines covered | Coverage |
| --- | --- | --- |
| stackpress-sql-postgres | 762 / 827 | 92.14% |
| stackpress-sql-mysql | 913 / 979 | 93.26% |

Both exceed the development floor. Uncovered lines remain defensive metadata, unsupported pair/error classifications, encoding/allocation and configuration
branches; line coverage is not branch proof or publication acceptance. TLS/auth
variants, other versions/targets, external poolers, hidden session-mutating MySQL
triggers and malicious transport stalls retain their prior limits. Generated-store
integration awaits separate coordinator release.

## Exact native database ledger

All names below were uniquely created and have acknowledged exact-name DROP in the listed log, including failures. No credentials/URLs are retained here.

| Backend | Database | Log / drop confirmed |
| --- | --- | --- |
| mysql | `stackpress_s7_m0_135fa_18e225dccdf8c440` | coverage.log / yes |
| mysql | `stackpress_s7_m1_135ff_18e225dd11fc8a50` | coverage.log / yes |
| mysql | `stackpress_s7_m2_13604_18e225dd5a986298` | coverage.log / yes |
| postgres | `stackpress_s7_m0_13d76_18e225df2a0856b8` | coverage.log / yes |
| postgres | `stackpress_s7_m1_14260_18e225df9d83bd30` | coverage.log / yes |
| postgres | `stackpress_s7_m2_142c4_18e225e07e8c32d0` | coverage.log / yes |
| mysql | `stackpress_s7_m2_148d9_18e22519e0e3b038` | native-baseline.log / yes |
| postgres | `stackpress_s7_m2_148f8_18e2251bf07ad830` | native-baseline.log / yes |
| mysql | `stackpress_s7_m2_13f6f_18e22506465736d8` | native-diagnostic.log / yes |
| postgres | `stackpress_s7_m2_145ba_18e225083f8051f8` | native-diagnostic.log / yes |
| mysql | `stackpress_s7_m2_11799_18e224f1162d6a00` | native-first.log / yes |
| postgres | `stackpress_s7_m2_117a1_18e224f20db78418` | native-first.log / yes |
| mysql | `stackpress_s7_m0_1ee_18e2254a4f861690` | rounds-native.log / yes |
| mysql | `stackpress_s7_m1_24d_18e2254a7184ffe0` | rounds-native.log / yes |
| mysql | `stackpress_s7_m2_260_18e2254a9caa6110` | rounds-native.log / yes |
| postgres | `stackpress_s7_m0_3d3_18e2254c3de41a20` | rounds-native.log / yes |
| postgres | `stackpress_s7_m1_40e_18e2254c6edbb750` | rounds-native.log / yes |
| postgres | `stackpress_s7_m2_46a_18e2254d181b94e8` | rounds-native.log / yes |

## Exact SQLite file ledger

All ten parent-owned files and their exact sidecars were removed (also asserted absent). Paths use the unique basename below under the platform temporary directory;
full paths and confirmations are preserved in sqlite-ledger.json and each log.

| File basename | Log / removed |
| --- | --- |
| `stackpress_s7_m2_sqlite_142c4_18e225e02909c598.sqlite` | coverage.log / yes |
| `stackpress_s7_m2_sqlite_148f8_18e2251ba0fbb1a8.sqlite` | native-baseline.log / yes |
| `stackpress_s7_m2_sqlite_145ba_18e22507f17fc0d8.sqlite` | native-diagnostic.log / yes |
| `stackpress_s7_m2_sqlite_117a1_18e224f1bfd0eb68.sqlite` | native-first.log / yes |
| `stackpress_s7_m2_sqlite_4a5a_18e225791d006b60.sqlite` | offline-final.log / yes |
| `stackpress_s7_m2_sqlite_f145_18e224db0bef9880.sqlite` | offline-first.log / yes |
| `stackpress_s7_m2_sqlite_15fee_18e22537d87e46b0.sqlite` | round1-offline.log / yes |
| `stackpress_s7_m2_sqlite_18324_18e22546db51a490.sqlite` | round2-offline.log / yes |
| `stackpress_s7_m2_sqlite_46a_18e2254cc7247c30.sqlite` | rounds-native.log / yes |
| `stackpress_s7_m2_sqlite_e3d9_18e225bdbb398390.sqlite` | workspace-final.log / yes |

## Closeout

The single target directory was deleted (clean-final.log). All 54 adapter source, doc/manifest and lock hashes match source-sha256.json. The KB validator baseline
is 146 existing missing sibling-source link errors and 22 warnings; post-write
validation must preserve that issue set. No unrelated repair was authorized.
No commit, push, publish, listener, tool installation or generated-store integration.
Coordinator independent review is next; this receipt does not release more M2 work.
