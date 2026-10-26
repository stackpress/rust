# Slice 4 M0 implementation receipt

Task: [00007 Slice 4 SQL](tasks/00007-slice-4-sql.md). Authority: G24/D01-D08, [the accepted design](slice-4-design-decisions.md), and the coordinator's M0 release. This receipt stops at the [riskiest-proof gate](slice-4-design-verification.md); it does not mark task 00007 verified or release M1.

## Baseline and write set

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0`.
- Branch: `trial/3-s4-m0`; clean starting HEAD `c4d22ed5bd571437847fd0bf2b4f09fac59102da`.
- Compiler: `rustc 1.96.0 (ac68faa20 2026-05-25)`, LLVM 22.1.2; Cargo 1.96.0; target `aarch64-apple-darwin`. The explicitly selected `+1.96.0` toolchain also passed.
- New real crates: `crates/sql/sql` (`stackpress-sql`) and `crates/sql/sql-sqlite` (`stackpress-sql-sqlite`). The native proof app is an example in the adapter, not a third placeholder crate.
- Integration edits: root `Cargo.toml`, `Cargo.lock`, `src/lib.rs`; this receipt is the only `.agents` write. No lib/server/CLI/Idea crate, existing app, task status, or verification field was edited.
- All previous locked package identities remain. The lock adds 58 approved transitive dependency identities plus the two new workspace packages. Additional versions of syn/hashbrown coexist with the old versions; no existing locked version was removed.
- No commit, push, listener, publishing, deployment, KB promotion, or messages to other agents were performed.

## Implemented M0 contracts

[Core source](../../../crates/sql/sql/src/lib.rs) owns `Database`, `TransactionSession`, `SqlFuture`, `Engine`, `Transaction`, strict native values, ordered rows/errors, finite query limits, and detached rendering. It has no dependencies, including no host, driver, or runtime dependency.

- `Database: Send + Sync + 'static`; `SqlFuture` has `Send` and the actual borrow lifetime. `TransactionSession: Send + 'static` uses `&mut self` across await and consuming commit/rollback.
- Engine, Transaction and SqliteDatabase have no Clone implementation. Applications explicitly share `Arc<Engine>`. Returned rows own data, preserve duplicate names, and reject ambiguous lookup.
- M0 renders one `Query` directly through `Render`, a small owned `Select`, static `Sql::literal`, or an explicit `TrustedSql::audited` marker. Only the SQLite renderer is implemented. `QueryPlan`, `execute_plan`, the other five builder families, network renderers, schema/diff/JSON breadth, and immutable observation policy remain later milestones.
- The lexical gate admits one ordinary statement, respects SQLite quote/comment states, rejects raw transaction/policy control and named/sparse parameters, and never replaces question marks globally. Raw SQL remains trusted application code, not sanitization or request templates.
- Adapter preparation and parameter-count validation happen on the same leased connection before binding/execution. Missing arguments cannot silently become NULL; excess and zero-parameter mismatches also error.
- One adapter function contains the dynamic SQLx `AssertSqlSafe` marker. Native internal BEGIN/COMMIT/ROLLBACK and accounting commands are static literals.
- Rows use native storage metadata and native result streaming, including RETURNING and WITH. Native cumulative change snapshots prevent SQLite's sticky `changes()` result from leaking into SELECT/DDL/EXPLAIN outcomes. Insert IDs remain `None` at M0; `INSERT ... RETURNING` supplies statement-local IDs.
- Nonfinite input and SQLite-calculated reals fail. Display and Debug omit native causes, SQL and configured paths; explicit `Error::source` retains diagnostics. `Transaction::rollback_after` preserves the original failure and exposes a secondary rollback error through `cleanup_error()`.

[SQLite source](../../../crates/sql/sql-sqlite/src/lib.rs) owns eager opening, bounded pooling/admission, same-connection execution and exclusive sessions.

- Options require an explicit UTF-8 file path; creation defaults false. Empty/control-containing paths, `:memory:` and `file:` URIs are rejected. There is no fallback, implicit installation, read-only API or pooled in-memory claim.
- Defaults: maximum 4 connections (valid 1-128), minimum 0; acquire 5000 ms and busy 1000 ms (each valid 1-300000 ms); admission 128 (valid 1-4096); worker and row channels 16. Every connection enables foreign keys, WAL and FULL synchronous policy.
- Query defaults: 1 MiB SQL, 1000 binds, 10000 retained rows, 16 MiB retained name/value bytes and bind bytes. Limits produce errors rather than truncation. These account for retained values, not allocator/container overhead, native VM resources or allocation of a single native cell before decode.
- A transaction owns one pool connection for its entire lifetime. Internal control commands run only on that connection; ordinary public raw SQL cannot enter/exit the transaction. A failed or cancelled statement leaves the session rollback-only.
- The M0 retirement policy is deliberately conservative: every transaction connection, plus failed/cancelled ordinary statement connections, closes instead of being recycled. Successfully acknowledged ordinary statements return their lease for reuse. This is a correctness proof, not a connection-churn performance claim.
- A `Lease` couples native connection ownership to its admission permit. Drop transfers both into an async close task, retaining both admission and SQLx pool capacity until native shutdown acknowledgement. Closing a native SQLite connection rolls back its open transaction; see the [SQLite close contract](https://www.sqlite.org/c3ref/close.html).
- With no live runtime, Drop detaches/drops the raw worker handle without blocking or requiring a Tokio context. Minimum connections is fixed at zero, so this fallback does not spawn replacement work. It initiates teardown without claiming acknowledged async cleanup.
- `close().await` closes admission, wakes waiters and waits for held leases. Cancelling and re-awaiting close is supported. The caller supplies a shutdown deadline.
- A cancelled commit has an unknown caller-visible outcome and is never replayed. A worker-hook proof actually persisted the insert despite cancellation before commit acknowledgement. Cancellation is not immediate interruption or proof that a write did not occur.

## Production-type proofs

The adapter test suite has five passing parent tests, four ignored child entrypoints invoked by those parents, ten native async scenarios, seven expected compiler failures and a runtime-teardown scenario. Core has five pure contract tests. The children are not silently skipped: parents launch their exact names with `--ignored` and inspect their exit status/output.

- Native scenarios: bound statements/arity/value decoding/reopen; transactions; pool; retained limits; configuration; host integration; transition cancellation; runtime responsiveness; native failures; admission retained during cancelled native work.
- Commit, explicit rollback, uncommitted Drop, statement-error/rollback-only handling, application panic/unwind, and task abort all use actual Engine/Transaction/SqliteDatabase types and real tempfile files. An outside lease cannot see uncommitted inserts; reopened files show only committed data.
- Three controlled cancelled BEGIN attempts recover with a new clean session. A safe native update hook pauses after an actual mutation, then dropping its query future makes the retained session rollback-only. A commit hook pauses before acknowledgement; cancellation permits exactly one persisted outcome with no retry.
- A cancelled autocommit write keeps the only admission slot while its native update hook is paused. Subsequent admission returns Limit until actual worker cleanup releases that slot; the unobserved write is not replayed.
- Native failure seams include busy writer contention, interrupted BEGIN, deferred-FK COMMIT failure, explicit ROLLBACK failure after a test-only native control command, and an interrupted rollback after a real constraint failure. Both primary and secondary native sources survive the last case; later leases are clean.
- Waiting acquisition can be cancelled without leaking its admission slot. A size-one pool acquisition times out finitely. Close wakes/rejects new work, remains pending with a held lease, can be cancelled, and succeeds when re-awaited after release.
- Named Send startup/operation handlers register the real Engine through existing ServerBuilder/config/service APIs. A transaction-holding handler's nested acquisition returns PoolTimedOut. Barrier/notification-controlled concurrent roots do the same; explicit `&mut Transaction` composition succeeds and a later root succeeds after release.
- A costly real recursive SQLite query leaves a current-thread runtime timer responsive, both at the controlled native gate and after release while native execution remains pending. SQLx's SQLite worker performs the native work; no first-party spawn_blocking wrapper is used.
- Compiler failures: E0499 transaction aliasing; E0599 Clone on Engine/Transaction/adapter; consumed completion; escaping borrow; non-Send SqlFuture; non-Send service; non-Send named host handler.
- Every async concurrency/cancellation/host/native-failure scenario runs in a separate child with an 8-second cooperative bound and an independent 10-second parent process watchdog. A deliberately parked synchronous child is killed/reaped by a 150 ms negative control. Compiler proofs have a separate 180-second process deadline. Test-only worker handshakes have finite ready/go waits.
- All native hooks and test-only transaction disruption stay in test modules; none is a production extension API. Tests use private tempfile roots and close/drop owned resources before their directory cleanup.

The [native proof app](../../../crates/sql/sql-sqlite/examples/m0-host.rs) explicitly installs/seeds its own disposable file, commits a bound insert, closes setup, registers a new Engine during startup, invokes a named `profile-name` capability and explicitly closes the service. It prints `profile-name: Ada`. Its compiled binary also completed under a separate 10-second parent deadline, excluding compilation time.

## Test-verified M0 review rounds

These bounded reviews apply the Rust programming workflow inside M0. They do not replace the later slice-wide coordinator checkpoint, milestone rounds or documentation release.

1. **Logic/complexity:** baseline core/adapter tests and doctests passed. Added failing regressions for SQLite-calculated infinity and URI paths bypassing file semantics; fixed strict decoding/options validation. Busy classification now recognizes native extended codes by primary code. Targeted tests/doctests passed again.
2. **Responsibility/ownership:** separated prepared arity and retained-row accounting from execution. Found that future cancellation released admission before native completion. Introduced the focused Lease owner, conservative connection retirement and the no-runtime fallback; added actual native admission/teardown proofs. Re-ran the full targeted suite and compile failures successfully.
3. **Consistency/Rustdoc/comments:** documented ownership, supported M0 options, cancellation/unknown commit, native allocation limits, feature boundaries and private state. A failing disclosure regression led to safe Debug formatting while preserving explicit sources. Reran targeted tests, doctests, fmt, Clippy and Rustdoc.
4. **Targeted correctness/safety follow-up:** reproduced sticky native affected counts on SELECT, DDL and EXPLAIN, then corrected them with same-lease native counters. Added explicit primary/secondary rollback-error preservation and a real interrupted-rollback proof. Reviewed internal static transaction commands, mutable borrows, cleanup ordering, runtime teardown and feature edges. No first-party unsafe, lifetime widening, production shared mutex or callback under a first-party lock exists. Final checks below passed after these changes.

One intermediate cancellation test timed out because it stopped polling at prepare before execution reached the native gate. The test was corrected to poll the real statement concurrently with its ready signal; no driver guarantee was inferred from the failed test. Another review corrected the responsiveness hook to trigger in the costly query rather than the small native accounting query.

## Final verification

Run from this worktree, on the installed native target, with locked/offline resolution after the initial approved dependency fetch:

```sh
cargo fmt --all -- --check
cargo test --workspace --all-targets --all-features --locked --offline
cargo test --workspace --doc --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features --locked --offline
cargo +1.96.0 check --workspace --all-targets --all-features --locked --offline
```

- **Workspace tests:** 275 passed, 0 failed, 27 ignored child/feature entrypoints, across 54 test targets. SQL's child entrypoints are exercised through the watchdog parents described above.
- **Workspace doctests:** 431 passed. The two new SQL crate examples compile; the native-resource doctest is no_run, while the actual app and real-IO tests execute separately.
- **Fmt, warning-clean Clippy/Rustdoc, explicit Rust 1.96 check:** passed.
- Default facade, `sql` only and `sql-sqlite` checks passed separately with `cargo check -p stackpress --no-default-features [--features sql|sql-sqlite] --locked --offline`.
- `cargo build -p stackpress-sql-sqlite --example m0-host --locked --offline` passed; the executable completed and produced the expected output.
- `git diff --check` passed. No shared-crate source changed.

Normal-edge `cargo tree` audits passed for the default facade, core-only SQL facade, lib and server: none contains SQLx, libsqlite3-sys or Tokio. The `sql-sqlite` facade contains SQLx exactly 0.9.0 and native SQLite/Tokio, with no active PostgreSQL/MySQL/WASM driver or SQLx macros. The lock may contain optional macro package records; those are not active build edges. Commands use `--edges normal --locked --offline`, with `-p stackpress --no-default-features`, the appropriate feature, or `-p stackpress-lib|stackpress-server`.

## Fresh coverage

Tool: cargo-llvm-cov 0.8.6 with the compiler/target above. Both new libraries have no crate feature flags. Coverage ran after `cargo llvm-cov clean --workspace` in this worktree's isolated absolute `CARGO_TARGET_DIR=/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s4-m0/target/s4-m0-coverage`; no shared checkout/other lane target was cleaned.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov -p stackpress-sql -p stackpress-sql-sqlite --all-targets --locked --offline --no-report
cargo llvm-cov run -p stackpress-sql-sqlite --example m0-host --locked --offline --no-report
cargo llvm-cov report -p stackpress-sql -p stackpress-sql-sqlite --json --summary-only \
  --ignore-filename-regex '(/tests/|/src/tests\.rs)' \
  --output-path target/s4-m0-evidence/coverage.json
```

| Measured first-party source | Covered/executable lines | Coverage | Development floor |
| --- | --- | --- | --- |
| stackpress-sql library | 385/388 | 99.23% | 70%, passed |
| stackpress-sql-sqlite library | 315/330 | 95.45% | 70%, passed |
| Executed native proof app | 64/65 | 98.46% | Measured separately |

Only test-only source/fixtures are excluded by the regex. Dependency code is excluded by selected packages. No first-party library behavior is excluded. Facade re-exports add no executable statements; they are checked through the feature matrix. Doctests run separately from stable coverage.

cargo-llvm-cov's normal report omits the example even after `run`. Its real executable was therefore measured explicitly using the installed toolchain's `lib/rustlib/aarch64-apple-darwin/bin/llvm-cov export`, with `-summary-only`, the binary at `$CARGO_TARGET_DIR/llvm-cov-target/debug/examples/m0-host`, merged `trial3-s4-m0.profdata` in that directory, and the absolute source filename `crates/sql/sql-sqlite/examples/m0-host.rs`. The app is not silently excluded from evidence.

Ignored local receipts are in `target/s4-m0-evidence/`: workspace test/check/doc/lint logs, normal-edge trees and audit JSON, raw per-file coverage JSON, explicit example coverage JSON, and `coverage-by-crate.json`. These are disposable local evidence; the tracked receipt includes the counts and reproduction commands.

## Limits and next gate

- Actual native begin/active/commit/rollback seams passed, but not every disk-full/IO-loss/worker-crash interleaving was injected. The WorkerCrashed classifier and impossible current-native metadata/type fallback branches are not fully exercised. Close/Drop cleanup cannot return a secondary async close diagnostic; explicit rollback errors retain their source, and rollback_after retains both failures.
- A lost commit acknowledgement remains unknown despite the controlled example committing successfully. There are no immediate-interruption, automatic-retry, exactly-once, power-loss, filesystem durability or all-platform guarantees.
- Coverage is line coverage, not branch or soundness certification. The unexecuted core lines include an owned-string conversion; adapter gaps include defensive metadata/dialect/type failures. No shallow tests or production exclusions were added to inflate the gate.
- Native aarch64 macOS only. Linux, Windows, browser/WASM, mobile, network filesystems and network SQL execution are unverified. No extra platform/toolchain was installed.
- The M0 surface is intentionally small. All remaining M1-M3 work, six-builder/full-dialect breadth, S4-F/M/N/H/X follow-ups and full crate documentation remain outside this receipt.
- Baseline KB validation before receipt creation failed with **146 pre-existing broken-link errors and 20 warnings**, primarily relocated sibling-repository links. No existing KB document changed; this code verification does not claim a green KB validator. The final handoff records the post-write comparison.

Stop for coordinator re-verification of M0. The coordinator owns task status, shared integration/merge and the decision to release M1.
