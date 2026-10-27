# 00014 SQLite and fixture scheduling receipt

Completed implementation on 2026-10-27 in `trial/3-sqlite-contention`; coordinator acceptance remains separate. This follow-up supersedes the initial focused result preserved below. Acceptance criteria: none; these are native test repairs without visual output.

## Final authorized scope

Changes stay inside SQLite tests plus its dev-only Tokio `test-util` feature, three PostgreSQL shared test-harness files, the single Idea formatter test file, and this receipt. MySQL includes the PostgreSQL contract/stress files directly; no duplicate MySQL edit is needed. Production adapters, Idea implementation, SQL core, protected maintenance files, root manifests and lockfile are unchanged. No installs, commits, pushes, listeners, network databases or inter-agent messages.

## Expanded root cause and repair

The coordinator's 15 full SQLite runs during six simultaneous agents' builds had **5 failing runs (33.3%)** after the initial contention repair. Copied logs `coordinator-baseline-{01,09,10,11,14}.log` and `coordinator-baseline.json` preserve the evidence: four responsive scenario deadline failures, an ordinary JSON acquisition timeout in run 11, and run 14's failed healthy host recovery (`None` versus `Some(1)`). The provided run 14 log is a recovery failure, rather than a successful acquisition where timeout was expected. These are test setup/scheduling assumptions; review found no adapter ownership or permit-release defect.

- SQLite tests accidentally applied an 80 ms acquisition SLA to ordinary correctness checks and recovery. They now use the public 5 s default; recovery paths that require retirement await all actual admission permits. Intentional timeout tests explicitly retain every connection, poll the real operation pending before and 1 ms before its configured deadline, then advance Tokio test time across that same deadline. Native open/IO/cleanup runs with real time. No timeout guarantee was increased.
- Native gate callbacks now wait for explicit release rather than panic after 3 s. Independent 10 s parent watchdogs remain. Responsiveness holds an acknowledged native gate while the current-thread timer advances, then checks the real result. Its recursive probe is bounded at 2,000 rows instead of spending millions of VM iterations racing another timer. Redundant 8 s cooperative wrappers are removed; the independent parent still kills/reaps stalled scenarios.
- Host nested/root timeout probes have their own blocked handler and explicit lease ownership. Healthy recovery waits for retirement before reading. Eager-open cancellation now separately acknowledges callback Drop and native close: an unused uniquely named collation owns a Drop notification. Pinned SQLite destroys that collation after rollback/b-tree closure, so recovery cannot race the abandoned writer. No unsafe code or production hook was added.
- SQLite watchdog negative controls explicitly signal ready (or fully flush excessive output) before their 150/500 ms observations. Missing readiness is independently bounded at 10 s; children are killed/reaped on unwind. The output test now requires the exact 64 KiB retained cap.
- PostgreSQL's offline SQLite reference used a separate 1 s acquisition override for scalar/value correctness; it now uses SQLite's public bounded defaults. This addresses the coordinator's scalar case 9 timeout without changing production policy. Shared PostgreSQL/MySQL lifecycle tests observe actual `Pending` futures and hold native capacity explicitly, replacing 5/10/15 ms timing guesses. A harmless SELECT completing immediately is accepted and retried (bounded at 64); only an observed Pending is dropped. Dirty-transaction rollback-only, zero leaked writes, unknown commit at most once, waiter wakeup and re-awaitable close assertions remain. Exact native cancellation phases remain covered by adapter transition-gated tests.
- Shared native stress controls check overflow as `Ready` on first poll, retain the real 200 ms queued timeout with the only connection held, and measure its entire interval. Ordinary contract setup uses the adapters' public default acquisition policy. Native database gates remain unset: those paths compile, but this receipt does not claim they ran against PostgreSQL/MySQL.
- The formatter fixture atomically publishes its PID, then waits for an explicit go file. The stalled-child test starts the actual production collector's **200 ms** deadline after readiness, retaining production pipe workers and `ChildOwner`. Output/input failure tests exercise the complete production supervisor with the error released only after readiness. Every case still requires kill/reap and pipe joins within **1 s**, checks `kill -0` fails, and remains under the existing **5 s** parent watchdog. Fixtures stay alive until killed; reparenting exits them if the outer watchdog kills their Rust owner. Idea implementation is untouched.

The original 120 ms cleanup-delay experiment (50/50 original failures at the exact reported line, 50/50 repaired passes) remains valid historical evidence below. That diagnostic patch is absent from final source. The original ordinary CPU-only replay did not reproduce the historical event; the expanded baseline is coordinator-supplied real-load evidence, not a newly invented reproduction rate. The PostgreSQL scalar and formatter addenda each report one failure without a denominator, so no baseline rate is claimed for them.

## Final repeated proof

**30/30 consecutive complete four-package runs passed, zero failures**, with default test threads and ten continuously live Python CPU burners on ten logical CPUs; elapsed 289.266 s. Each burner repeatedly computes `sum(i*i for i in range(10000))`. Runs are sequential full suites, including doctests, each under a 300 s parent kill/reap watchdog. All burners were killed and reaped. This extends the earlier SQLite-only 30/30 loaded series (`broad-full-heavy.json`).

```sh
python3 /private/tmp/sqlite-contention/matrix-stress.py extended-full-heavy --count 30 --burns 10
# Each iteration runs:
cargo test --locked --offline -q -p stackpress-sql-sqlite -p stackpress-sql-postgres -p stackpress-sql-mysql -p stackpress-idea-transformer
```

`extended-full-heavy.json`, 30 per-run logs, and `extended-full-heavy-validation.json` record every result. Each log confirms SQLite reference scalar case 9, a passed complete shared suite, removal of the exact owned SQLite file/sidecars, and six explicit unset-native-gate reports. `extended-final-source-sha256.json` confirms all 15 code/config files remained unchanged throughout the series. All scripts/logs are under `/private/tmp/sqlite-contention/`; `run.py` records command arrays, durations and exit codes, sets the sole target directory, removes `RUST_TEST_THREADS` and native URL variables, and uses process-group watchdogs.

## Follow-up review rounds

SQLite expansion: initial complete suite passed; round 1 checked both sides of actual acquisition deadlines; round 2 centralized cleanup acknowledgement and removed redundant child wrappers; round 3 replaced callback-drop inference with native-close acknowledgement and reviewed comments/ownership. `broad-round{1,2,3-final}.log` pass. One intermediate moved-Arc compile error is retained in `broad-round3.log`; it was corrected before the passing final round and both loaded series.

Addenda: initial three-package tests passed (`extended-initial.log`). Round 1 reviewed branch/state ownership and readiness (`extended-round1.log`, pass); round 2 restored end-to-end supervisor wiring for pipe failures while retaining the independently synchronized deadline test (`extended-round2.log`, pass); round 3 reviewed bounded probes, whole queued-time measurement, naming and comments (`extended-round3.log`, all four packages pass). These reviews preserve the production cleanup owners; no ceremonial source changes or new adapter API were needed.

## Final verification and limits

Environment: `rustc 1.96.0`, `aarch64-apple-darwin`, `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`, one worktree `target/`, cached dependencies with `--locked --offline`, default test threads and native database gates unset. Final source hashes match the complete loaded series. Results are native macOS evidence, not browser/WASM or other operating systems.

- `cargo fmt --all --check`, explicit Rustfmt for the included shared files, and `git diff --check`: pass (`extended-fmt.log`, `extended-shared-fmt.log`).
- Complete four-package tests and doctests: pass (`extended-round3.log` and all 30 matrix logs). SQLite: 14 parent/unit tests + 3 doctests; Idea transformer: 90 parent/unit/integration tests + 13 doctests. Watchdog parents execute their ignored child entrypoints explicitly.
- SQLite with `maintenance`: 15 parent/unit tests + 3 doctests pass (`extended-maintenance.log`); maintenance source is unchanged.
- Clippy for all four packages/all targets with SQLite maintenance and `-D warnings`: pass; warning-clean Rustdoc for those packages/maintenance: pass (`extended-clippy.log`, `extended-rustdoc.log`).
- `cargo test --workspace --locked --offline`: pass with default test threads and doctests, 110.452 s; `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: pass (`extended-workspace.log`, `extended-workspace-clippy.log`, `extended-verification.json`).

Fresh executable production-line coverage, `cargo-llvm-cov 0.8.6`, same compiler/native target: SQLite with maintenance **568/640 = 88.75%** (without maintenance 356/371 = 95.96%); Idea transformer **1,934/2,018 = 95.84%**, including formatter implementation 140/147 = 95.24%. Both exceed the 70% development floor. No production lines are hidden: only test modules, dependencies and build artifacts are excluded. Native PostgreSQL/MySQL implementation is unchanged and gated off; this test-only shared-harness repair supplies no fresh network-driver line-coverage claim.

Replay: `python3 /private/tmp/sqlite-contention/extended-coverage.py` then `python3 /private/tmp/sqlite-contention/extended-coverage-summary.py`. Exact Cargo arrays/results: `extended-coverage-commands.json`; LCOV/per-file counts: `extended-coverage.lcov`, `extended-coverage-summary.json`. All selected unit/integration/native-watchdog tests pass instrumented; only SQLite's compile-only ownership parent is filtered (already passed in every uninstrumented matrix run and the workspace suite). Ordinary doctests pass separately. Coverage does not establish every branch or permission boundary.

## Other short-deadline fixtures

Workspace searches for PID paths/files and `getpid()` found **only the repaired formatter fixture**. `extended-pid-fixture-search.txt` and `extended-fixture-search.txt` retain the commands' results. Other short-deadline fixtures/assertions remain outside this write set; their presence alone does not establish a defect:

- Deliberately stuck watchdog controls: `examples/capability-host/tests/cli.rs:227` (250 ms), `examples/sql-profile/tests/proof.rs:53` (150 ms), `crates/lib/tests/handler_signature.rs:610` (250 ms), and protected `crates/sql/sql-sqlite/src/maintenance/tests.rs:54` (150 ms).
- CLI timing/blocked-reader fixtures: `tools/cli/src/tty/tests.rs:39` (100 ms against a 400 ms reader), `tools/cli/tests/tty_contracts.rs:106`, `:113`, `tools/cli/tests/tty_prompt_contracts.rs:118`, and `tools/cli/tests/prompt_contracts.rs:845` (250 ms).
- HTTP protocol/stop deadlines and elapsed assertions: `crates/server-http/tests/m1.rs:165-170,412,493,502,518,548-553,877,886,967,1059` and `m2.rs:591` (100-500 ms); no duplicate PID fixture.
- Existing gated native PostgreSQL/MySQL M1 cancellation/close probes also retain short deadlines; no network fixture was available or requested. Their production guarantees are not weakened or claimed newly verified here.

`cargo clean --target-dir <this-worktree>/target` completed after coverage export. `extended-target-cleanup-verification.json` confirms the sole target is absent; all CPU-load children were killed/reaped and no diagnostic adapter patch remains. Final write-set audit contains only authorized test/dev-feature files and this receipt. KB validation retains the pre-existing 146 missing sibling-source link errors and 22 warnings with no added findings (`broad-kb-before.log`, `extended-kb-after.log`, `extended-kb-comparison.json`). No unrelated KB/index/tracker file changed. Coordinator independent review is next; no implementation decision is pending.

## Initial focused investigation (historical)

The remainder preserves the original narrow receipt verbatim apart from heading depth. Statements about its final scope, unchanged 80 ms/3 s/8 s budgets, target cleanup and command results describe that earlier revision, **not the expanded final state above**. Its controlled replay requires the saved original 80 ms helper as well as the diagnostic/cancellation files; the current public-default helper intentionally differs.

## 00014 SQLite contention test receipt

Implementation completed on 2026-10-27 in `trial/3-sqlite-contention`; coordinator verification remains pending. Acceptance criteria: none; this is a native test repair with no visual output.

### Scope and intended behavior

Final changes are limited to [the cancellation test](../../../../crates/sql/sql-sqlite/src/tests/m2/cancellation.rs) and this pre-approved receipt. Production adapter code, SQL core, maintenance files, manifests and lockfile are unchanged. No installs, commits, pushes, publishing, development listeners, network databases or messages to other agents were used.

Preserve three observable properties: blocking native work leaves the current-thread executor runnable; a real competing writer returns `Busy` with its driver cause; retirement eventually returns every admission and SQLx pool slot so a fresh transaction works.

### Root cause

Classification: a test synchronization defect; no adapter ordering or permit-release defect was found. The historical failure is recorded in [the HTTP M1 receipt](00012-receipt-http-m1.md).

The shared test options impose an 80 ms acquisition deadline. A failed ordinary statement and a completed transaction both retire their connections. `SqliteSession::rollback()` acknowledges native `ROLLBACK`, then dropping its lease spawns asynchronous retirement. It does not await worker shutdown. The old test immediately called `begin()` at `cancellation.rs:454`, charging outstanding retirement and replacement connection initialization to that 80 ms deadline.

[Lease ownership](../../../../crates/sql/sql-sqlite/src/lease.rs) correctly retains both the SQLx connection and admission permit until `connection.close().await` completes. SQLx 0.9.0 also retains its pool capacity through close acknowledgement. Delaying those correctly owned cleanup tasks can therefore return the legitimate `PoolTimedOut` error before the test's unconditional success assertion. No leaked permit is needed to explain the failure.

There was a second scheduling assumption: the test released its native gate, slept 20 ms under a 150 ms timeout, and expected the contender still to be running against a 300 ms busy timeout. A descheduled executor can miss that interval. Moreover, `Hook::Progress` first gates metadata SQL before the contending INSERT, so the old hook did not identify the exact native busy-handler boundary.

CPU load alone did not reproduce the historical event in 300 original-code replays. The controlled cleanup-delay experiment below reproduces the exact original failure location and demonstrates the invalid ordering assumption; it does not claim to reconstruct the historical machine schedule.

### Fix

1. Keep the acknowledged native gate closed while the current-thread timer advances. Assert the contender remains pending before releasing the gate. The timer is a progress observation, with no narrow elapsed-time success window.
2. Keep the owning transaction's writer lock held after gate release. The contender must still return native `Busy` with its original cause.
3. Configure exactly two admission slots for the two operations. After the owner's rollback, use the existing `cleaned()` helper to await both permits. Assert SQLx pool size is zero before acquiring and rolling back a fresh transaction.

The 80 ms acquisition budget, 300 ms busy budget, 20 ms timer, three-second native gate guard, eight-second cooperative scenario watchdog and ten-second parent watchdog are unchanged. The unnecessary 150 ms timer comparison is removed. A missing cleanup acknowledgement now fails the watchdog instead of racing recovery against retirement.

### Before and after evidence

Logs, command arrays, exit codes, durations, source hashes, original source copies and diagnostic patches are under `/private/tmp/sqlite-contention/`. `run.py` owns/reaps every CPU-burn child and gives each test child an independent ten-second kill/reap deadline. Each burner repeatedly computes `sum(i*i for i in range(10000))` in a separate Python process. The host reports ten logical CPUs.

| Experiment | Load | Original test | Final test |
| --- | --- | --- | --- |
| Moderate CPU saturation | 10 burners, 2 concurrent test children | 100/100 pass | Not repeated separately |
| Heavy CPU saturation | 20 burners, 8 concurrent test children | 200/200 pass | 200/200 pass |
| Controlled cleanup delay | 10 burners, 2 concurrent test children; cleanup delayed 120 ms | 0/50 pass; 50/50 `PoolTimedOut` at original line 454 | 50/50 pass |

No stress child exceeded its parent deadline. `failure-classification.json` confirms all 50 controlled baseline failures match both the original location and error. The final delayed-cleanup series is `final-delayed-cleanup.json`; an earlier intermediate fix also passed 50/50 (`after-delayed-cleanup.json`). The final unmodified-production heavy-load series is `after-heavy.json`.

The temporary diagnostic patch adds a test-build-only 120 ms sleep inside the detached lease cleanup task, immediately before native close. It is enabled by `STACKPRESS_SQL_CLEANUP_DELAY_MS=120` and changes no release ordering. This models cleanup-task scheduling latency longer than the acquire budget. It was used for both sides of the controlled experiment, then removed byte-for-byte before final verification. It is retained only as `/private/tmp/sqlite-contention/diagnostic-cleanup-delay.patch`, together with `lease-original.rs`, `cancellation-original.rs` and `final-fix.patch` for replay.

Replay the ordinary final stress with:

```sh
python3 /private/tmp/sqlite-contention/run.py command replay-build -- cargo test -p stackpress-sql-sqlite --lib --no-run --locked --offline
python3 /private/tmp/sqlite-contention/run.py stress replay-heavy --count 200 --burns 20 --parallel 8
```

For the controlled comparison, apply only the saved temporary diagnostic patch, build, and invoke the same stress runner with `STACKPRESS_SQL_CLEANUP_DELAY_MS=120`, `--count 50 --burns 10 --parallel 2`. The saved original cancellation file supplies the baseline; the saved final patch supplies the repair. Restore the diagnostic change after replay. The runner configures the single worktree target and the build environment recorded below.

### Implementation notes and review rounds

- Initial working evidence: the synchronization repair passed 50/50 delayed-cleanup replays with load before the deliberate review rounds.
- Round 1, logic/complexity: reviewed native acknowledgement versus retirement, gate release order, error/source assertions and recovery preconditions. No further edit was justified. The complete M2 watchdog parent passed (`round1.log`).
- Round 2, responsibility/abstraction: made this two-operation scenario own its two-slot admission policy, removing its cleanup barrier's dependence on the shared default of 128. Reused the existing cleanup helper; no new abstraction or production hook. The complete M2 watchdog parent passed (`round2.log`).
- Round 3, consistency/Rustdoc/comments: reviewed the declaration and local explanations against the actual gate, timer, writer lock and cleanup sequence. No additional edit was needed. Default SQLite tests, doctests, formatting, Clippy and warning-clean Rustdoc passed (`round3-sqlite.log`, `fmt.log`, `clippy.log`, `rustdoc.log`).
- Safety review: rechecked connection/admission coupling, SQLx capacity ownership, rollback acknowledgement, runtime cancellation, and watchdog kill/reap behavior. Uncertain connections remain retired; no locks or new runtime behavior were introduced. The final delayed-cleanup experiment, heavy-load replay, workspace suite and workspace Clippy passed.

### Verification

Environment: `rustc 1.96.0 (ac68faa20)`, `aarch64-apple-darwin`; `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`. Both `CARGO_TARGET_DIR` and `CARGO_LLVM_COV_TARGET_DIR` select this worktree's sole configured `target/`. Cargo used cached dependencies with `--locked --offline`. `RUST_TEST_THREADS` and both native PostgreSQL/MySQL URL variables were removed from the runner environment.

- `cargo fmt --all --check`: pass; `git diff --check`: pass.
- `cargo test -p stackpress-sql-sqlite --locked --offline`: 14 parent/unit tests and 3 doctests pass; 9 expected ignored child/negative-control entrypoints (`round3-sqlite.log`).
- `cargo test -p stackpress-sql-sqlite --features maintenance --locked --offline`: 15 parent/unit tests and 3 doctests pass; 9 expected ignored entrypoints (`sqlite-maintenance.log`). Watchdog parents execute their child scenarios explicitly.
- `cargo clippy -p stackpress-sql-sqlite --all-targets --features maintenance --locked --offline -- -D warnings`: pass (`clippy.log`).
- `RUSTDOCFLAGS='-D warnings' cargo doc -p stackpress-sql-sqlite --features maintenance --no-deps --locked --offline`: pass (`rustdoc.log`).
- `cargo test --workspace --locked --offline`: pass with default test threads, including doctests; no serial retry (`workspace.log`, `workspace.json`; 325.428 seconds). Opt-in network database tests remain ignored, not native database evidence.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: pass (`workspace-clippy.log`).

#### Coverage

`cargo-llvm-cov 0.8.6`, native target, `maintenance` enabled: **568/640 executable production lines = 88.75%**, above the 70% development floor. This includes the unmodified maintenance module. Without that module's lines the existing adapter scope is 356/371 = 95.96%. Lease (37/37), pool entry points (63/63), session (46/46) and options (55/55) are fully covered.

```sh
cargo llvm-cov clean --workspace
cargo llvm-cov --no-report -p stackpress-sql-sqlite --features maintenance --lib --locked --offline -- --skip tests::ownership_under_process_watchdog
cargo llvm-cov report -p stackpress-sql-sqlite --lcov --output-path /private/tmp/sqlite-contention/coverage.lcov --locked --offline
python3 /private/tmp/sqlite-contention/summarize.py coverage
```

The instrumented run passes 14 tests, with 9 child entrypoints ignored and the compile-only ownership parent filtered out. Ownership compiler proofs already passed in both ordinary crate runs and the workspace run; they execute no additional adapter production behavior. Native watchdog children remain instrumented. `coverage-summary.json` and `summarize.py` retain exact per-file counts. The report selects only this crate's production `src/` files, excluding test modules, dependencies and generated build artifacts. Ordinary doctests passed separately and are not claimed as instrumented coverage. Uncovered codec/error and maintenance lines remain visible in the LCOV; line coverage does not prove every branch, and no production lines were excluded.

### Cleanup, KB validation and handoff

`cargo clean --target-dir <this-worktree>/target` completed after coverage export. `target-cleanup-verification.json` confirms the target directory no longer exists. Diagnostic source changes are absent; no build output is retained in the worktree.

The Agent Workspace validator has the same 146 pre-existing missing sibling-source link errors and 22 warnings before and after this receipt; no new findings. Logs: `kb-before.log`, `kb-after.log`, `kb-comparison.json`. Those unrelated KB files remain untouched. This coherent receipt is under 200 lines and requires no split or new Reference File; no context promotion or tracker edits were authorized.

Remaining limits: this is native macOS evidence, not browser/WASM or another operating system. A genuinely stalled native open may still legitimately exceed its configured 80 ms acquisition budget; the fix removes outstanding retirement from that recovery budget rather than changing the timeout contract. CPU-only reproduction of the historical failure remains unobserved. Coordinator independent review/replay is the next step; no additional product decision is pending.
