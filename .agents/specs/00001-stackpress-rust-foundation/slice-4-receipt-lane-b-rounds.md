# Slice 4 lane B — refactor rounds 1–4 receipt

Task: [00007, Slice 4 SQL](tasks/00007-slice-4-sql.md). Date: 2026-10-26.

## Scope and intended behavior

- Base: `104d96d500401fca5955ce71e92d080adefc946c`, branch `trial/3-s4-m0`, worktree `trial3-s4-m0`.
- Owners: `crates/sql/sql-sqlite/**` and `examples/sql-profile/**` only, plus this requested receipt.
- Preserve the frozen public API, native execution/transaction contracts and exact capability-host envelopes.
- No edits to core SQL, shared lib/server/CLI/Idea, manifests, features or lockfile. No new dependencies, commits, agent messages or listeners.
- Target/toolchain: native `aarch64-apple-darwin`, Rust 1.96.0. Development coverage floor: 70% per crate.

Evidence directory: `/private/tmp/stackpress-s4-lane-b-rounds-evidence/`. Logs and JSON reports survive target cleanup. The command supervisor enforces finite outer deadlines; native tests additionally retain their 8-second cooperative / 10-second process watchdogs, 3-second worker barriers and 180-second compile-fail deadline.

## Baseline

`baseline-tests.log`: both owned crates, `cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline`, passed: adapter 10 parent/unit tests and app 4. Ignored child entries are deliberately launched by watchdog parents; they are not counted as independent parent successes.

The coordinator's before-rounds review carried no unresolved defect. Reviewed the round checklists, Rust adaptation of the coding skill, current source, native tests and installed SQLx 0.9.0 ownership/drop implementation.

## Round 1 — logic and complexity

Reviewed prepared validation/binding, row budget checks, counter ordering, transaction poisoning, completion/error precedence, admission/close, setup transaction scope and prefix/filter/order/limit semantics. The native state machine and application policy remain explicit; no production logic rewrite was warranted.

**Defect fix, test infrastructure only:** the adapter watchdog bounded retained diagnostics but did not reject a growing capture file. Added `watchdog_rejects_output_overrun` and its finite-output/stalled child first: it failed against the old helper (`round1-regression-before.log`). The watchdog now polls capture size, kills/reaps on overrun, checks growth after the final poll, and decodes arbitrary diagnostic bytes lossily. This is a poll-based output guard, not a filesystem quota.

Verification: `round1-tests.log`, adapter 11 / app 4 passed, including the new regression and existing synchronous-stall negative control. Production behavior is unchanged.

## Round 2 — responsibility and abstraction

Moved native transaction ownership into `src/session.rs`, beside lease/execution/collection/options modules. Session fields are private; its internal constructor centralizes the initial rollback-only state. Pool construction/admission stay in `lib.rs`. No new public abstraction or exported item. The deliberate invalid-native-state test uses the internal constructor.

Kept application database registration, profile capability and explicit setup separate in their existing modules; further splitting would scatter their cohesive flows.

Verification: `round2-clean-tests.log`, adapter 11 / app 4 passed without warnings. The first extraction run exposed an unused import, removed before this clean run.

## Round 3 — consistency, Rustdoc and comments

- Renamed the cumulative native field to `total_changes`, distinguishing it from statement-local `changes()`.
- Explained permit ownership before acquisition, rollback-only state before suspension, consuming completion and SQLx's asynchronous idle return.
- Documented eager-open cancellation/file creation and unchanged exact input validation; parameter copies remain owned binds.
- Safety follow-up corrected overly broad admission wording: no-runtime teardown is best effort and cannot acknowledge worker completion or continued admission coupling. Callers should await close before runtime shutdown. This documents the existing fallback; it does not change it.

Verification: `round3-tests.log`, scoped doctests, Clippy `-D warnings`, Rustdoc `-D warnings` passed. After the safety wording follow-up, `followup-*.log` repeats fmt, both suites (adapter 14 / app 4), all 3 scoped doctests, Clippy, Rustdoc and the 1.96 check successfully.

## Round 4 — dedicated safety and soundness

Added `src/tests/rounds.rs`: six finite 64-cut sweeps over real Send futures (begin, run, commit, rollback, Engine plan, caller-owned transaction plan), each requiring both cancelled and acknowledged paths; queued-acquisition cancellation for public begin/run/plan; native failure-rollback cancellation; native error disclosure; positive public thread bounds; and runtime teardown variants. No production test-hook API, unsafe code, lifetime widening or new dependencies.

`round4-final-tests.log` observed cancellation cuts of 3 / 6 / 1 / 1 / 16 / 12 respectively. Counts are scheduling observations, not a claim of deterministic per-await branch coverage. Exact native phases remain established by the existing ready/go VM/update/commit/rollback/collation hooks. One new idle-pool test initially tried to reuse admission before retirement acknowledgement; it now waits for the real permit handoff.

### Await and ownership audit

| Await boundary | Cancellation ownership / evidence |
| --- | --- |
| Eager open and pool initialization | Unpublished SQLx pool/initialization guards retire initialized workers. Existing dirty `after_connect` cancellation proof, repeated three times, then a fresh writer; unpolled open creates no file. |
| Run/begin pool acquisition | Local admission permit drops with the waiter; application SQL has not been queued. New queued begin/run/plan tests verify exact permit restoration while the only connection is held. |
| Native BEGIN | Lease already owns both connection and permit. Existing VM gate cancels inside BEGIN repeatedly; new begin suspension sweep and reopen/write checks. |
| Schema read, cache clear, before-counters, prepare | Outer ordinary lease survives all awaits; borrowed session is poisoned before any await. Native preparation failure/interruption, schema-cache proofs and suspension cuts exercise the same owner. Cache clear has no separate SQLite VM hook; its cancellation owner was audited directly. |
| Every native fetch poll and trailing counters | Ordinary cancellation retires the lease. Borrowed cancellation leaves rollback-only set, including after a write before trailing-counter acknowledgement. Existing active fetch/update barriers plus the run sweep prove rollback, bounded retained output and healthy replacement. |
| Consuming COMMIT / ROLLBACK | Future owns the session even before its first poll. Dropping it retires the lease. Native gates and suspension sweeps cover active cancellation; unpolled consuming futures are already tested. Lost commit acknowledgement remains unknown and is never replayed. |
| Engine plan begin, each statement, commit | Whole plan validates before acquisition; one owned transaction spans the batch. Existing exact statement-two barrier and new sweep prove no partial schema; commit cancellation permits either all effects or none. |
| Engine plan failure rollback | New `failed_plan_cleanup` pauses native ROLLBACK after statement two fails, aborts the public plan future, verifies admission retention, identical schema and file reopen. Existing failure proofs retain the primary and secondary sources. |
| Caller-owned Transaction plan statements | New borrowed-plan sweep leaves caller ownership and rollback-only on cancellation; explicit completion/rollback leaves no DDL effects. Existing caller-owned success/commit/rollback/isolation proofs remain green. |
| Pool close | Admission closes before awaiting SQLx close. Existing held-lease/queued-waiter proof covers cancellation and re-await; new shutdown test starts close then tears down the runtime. Close can wait for a caller-held guard; callers choose a deadline. |
| Detached lease close | `parts.take()` transfers connection and permit together exactly once; `close_on_drop` prevents recycling uncertain state. Existing worker barriers retain admission until native acknowledgement on a live runtime. New shutdown variants cancel unpolled retirement, start/cancel close, and drop reusable-pool ownership outside a runtime. |

### Additional risk inventory

- **Lease/drop:** normal reuse only follows acknowledged success. Error, panic, abort and consuming completion retire uncertain state. A detached close failure cannot be returned from Drop; explicit operation errors preserve their causes. Runtime teardown initiates best-effort cleanup without an acknowledgement guarantee or a promise of admission coupling after the runtime dies. Fresh-runtime/file-writer proofs check eventual rollback, not synchronous shutdown acknowledgement.
- **Borrowing and threads:** production holds no mutex, RefCell or entered tracing guard across await. Exclusive native connection and transaction borrows intentionally span await to preserve affinity. Positive bounds prove `SqliteDatabase`, `SqliteOptions`, Engine and QueryResult are Send + Sync, and Transaction/session are Send. Transaction is not required to be Sync. Seven existing compile-fail fixtures still reject aliasing, clone, consumed/escaped guards and non-Send use.
- **Errors/disclosure:** actual native missing-table errors retain the original source, while Display/Debug omit SQL identifiers, SQL text and file paths. Existing category/decode/open/cleanup tests cover the remaining mappings. Opt-in source inspection intentionally exposes diagnostic detail.
- **Input/trust:** missing/empty IDs fail identically to capability-host; non-strings fail typed decoding. Valid strings are not trimmed/normalized. The only example TrustedSql expression is a constant BLOB-prefix predicate with two owned bound inputs. Wildcards, quotes, backslashes, Unicode and embedded NUL remain literal; request data cannot select SQL, file or limit.
- **Setup/refusal:** startup remains read-only and never installs/seeds. Existing installation is refused without changing rows; mid-plan collision rolls back. Added two concurrent explicit installers: exactly one succeeds, the loser has an error source, and the final ordinal/id/name rows equal the canonical fixture.
- **Native/storage limits:** no raw connection leaks publicly, no user SQL concatenation, no unbounded acquisition queue, no automatic retries. Retained payload limits do not cap SQLite VM/single-cell allocation or allocator overhead. Cancellation does not promise immediate native interruption. Sequential DDL refresh does not serialize external online schema/file replacement; files remain application-owned.
- **Platform boundary:** native macOS only; PostgreSQL/MySQL production execution, other OS targets and WASM were not newly proven. No unsafe worker crash injection; the existing worker-loss classification seam is synthetic.

Verification: `round4-final-tests.log` and final workspace checks passed. Follow-up review of the new harness/ownership comments found no additional in-scope fix. No public API decision is pending.

## Final checks and parity

All Cargo checks use `--locked --offline` where applicable.

| Check | Result / evidence |
| --- | --- |
| `cargo fmt --all -- --check` | Pass; `final-fmt.log`, final comment follow-up also passes. |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pass; `final-clippy.log`. |
| `cargo test --workspace --all-targets` | **399 passed, 0 failed**; `final-tests.log`. Sixteen opt-in network cases skipped because URL variables are unset. Deliberate child entries are driven by their watchdog parents. |
| `cargo test --workspace --doc` | **853 passed, 0 failed, 0 ignored**; `final-doctests.log`. Resource examples marked no_run compile. |
| `cargo +1.96.0 check --workspace --all-targets` | Pass; `final-msrv.log`. |
| Scoped Rustdoc with `RUSTDOCFLAGS=-D warnings` | Pass; `followup-rustdoc.log`. |
| Actual executable parity | **12/12 byte-identical stdout envelopes and matching exit codes**, including empty input: `native-parity.json`, separate raw stdout/stderr captures. Every process has a 10-second/output watchdog. |
| Direct-call / CLI runner / buffered HTTP | Existing actual-reference parity suite rerun in each round and final checks; limits 1/3/100, hostile inputs, validation/errors, URI spelling and framing preserved. |
| `cargo tree -p <package> --edges normal` | Default `stackpress`, `stackpress-lib`, `stackpress-server` remain free of SQLx/libsqlite3/core SQL; three `*-tree.txt` files. |
| Native service example | `m0-host` prints `profile-name: Ada` under a compile/runtime watchdog; `host-proof.log`. |

## Per-crate coverage

Each crate was measured alone after `cargo llvm-cov clean --workspace`; no combined core/adapter percentage. Tests retain process watchdogs during instrumentation. The adapter was remeasured after the final documentation follow-up; app source is unchanged since its separate measurement.

| Crate / production scope | Covered lines | Coverage |
| --- | --- | --- |
| `stackpress-sql-sqlite`, six `src` modules | 374 / 392 | **95.41%** |
| `stackpress-example-sql-profile`, all six `src` files, including both binaries | 185 / 188 | **98.40%** |
| Unchanged standalone adapter `m0-host` example, measured separately | 64 / 65 | **98.46%** |

Adapter command: `cargo llvm-cov -p stackpress-sql-sqlite --all-targets --no-report`, then `cargo llvm-cov report -p stackpress-sql-sqlite --json --summary-only --fail-under-lines 70 --ignore-filename-regex '(/tests/|/src/tests\.rs)'`. See `adapter-coverage-final.json` and `adapter-final-*.log`.

App command: `cargo llvm-cov -p stackpress-example-sql-profile --all-targets --no-default-ignore-filename-regex --ignore-filename-regex '/rustc/|/\.cargo/|/\.rustup/|/(crates|tools|tests|target)/|/examples/(capability-host|generated-profile|profile-ui)/' --json --summary-only --fail-under-lines 70`. See `app-coverage.json`, `app-coverage.log` and `app-coverage-report.log`. The default examples exclusion is disabled, and report filenames were checked to contain only the six owned production files. All 31 app functions execute.

Exclusions: tests, dependencies, generated outputs, other workspace crates/examples. No owned production module is excluded. The unchanged adapter demo is excluded by cargo-llvm-cov's ordinary example filter and exported explicitly with installed LLVM (`m0-host-coverage.json`). Adapter's 18 uncovered lines are defensive mapping/metadata/storage/error edges; app's three are startup-registration and secondary cleanup branches. Native error/cleanup paths also have direct regression evidence, rather than relying on the aggregate percentage.

## Closeout

- The only behavior fix is the test watchdog's capture-overrun rejection; native/application behavior is preserved.
- No material in-scope fix or public API request remains. Shutdown/native-allocation/external-writer/platform limits above remain explicit; this receipt does not grant publication or acceptance.
- Agent Workspace validation: unchanged baseline **146 errors / 21 warnings** (existing relocated TypeScript links and existing document warnings); no new finding from this receipt. Exact before/after logs and comparison are retained.
- One root target directory was reused and removed after final checks. Logs/reports remain outside it. No commit made; ready for coordinator re-verification.
