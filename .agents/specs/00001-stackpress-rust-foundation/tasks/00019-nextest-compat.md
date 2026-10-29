# 00019: nextest compatibility and deterministic SQLite safety

2026-10-29, Asia/Manila. Branch `trial/3-nextest-compat`, based on `trial/3`. Status: started; implementation complete, coordinator verification pending. Acceptance criteria: none (test tooling only).

## Task summary and scope

The coordinator's owner-approved brief authorizes seven custom-harness protocol fixes, deterministic SQLite cancellation/completion evidence, shared watchdog output placement, section 5 of the fast-build workflow, and these 00019 records. Production APIs, dependencies, manifests, Cargo.lock and historical KB records are unchanged. No installs, commits, pushes, listeners, GitHub writes or live PostgreSQL/MySQL execution.

Implementation steps: inspect the current harness/fixture contracts; share pure discovery/selection; preserve each runner's fixture, timeout, cleanup and unset-gate reporting; add native suspension barriers without removing the existing sweep; fix the shared output path; perform review rounds and all requested gates; remove the task Cargo target after recording evidence.

Verification process: seven-executable protocol proof, targeted nextest rounds, three workspace nextest passes without package exclusions, touched-package cargo-test parity, workspace doctests/fmt/Clippy/Rustdoc, assertion/source audit and KB diagnostics comparison. Every Cargo/proof/clean invocation uses the required recorder; the ledger below includes failures.

## Implementation notes

- [Shared harness](../../../../crates/sql/test-support/harness.rs) handles `--list --format terse`, ignored-only listing, exact/substring filters, skip filters and capture flags. Lists contain only `name: test` lines; discovery never opens a fixture. Ignored status reflects the existing environment gate. Ordinary execution still reaches the original `ignored (STACKPRESS_TEST_..._URL unset)` reporting branches. The network runner retains its existing non-Unicode-URL skip behavior; native runners retain `var_os` gating.
- Renderer network cases retain their ordered shared fixture on a full cargo-test run. Exact later cases replay missing prior fixture stages inside their uniquely owned database before reporting only the selected test. Full runs do not replay anything, including after a failed selected case. This dependency setup is source-reviewed; live DB execution is deliberately unverified.
- Native M0 exposes its lifecycle case; M1 exposes values/outcomes/reuse/lifecycle; M2 exposes contract/stress. PostgreSQL M2's offline SQLite reference is independently selected. MySQL M2 preserves its existing opt-in `--g37-ddl` registry. Private watchdog-child switches dispatch before public argument parsing; parent CREATE/DROP authority, independent child deadlines, error redaction and cleanup failure status remain intact.
- [SQLite rounds](../../../../crates/sql/sql-sqlite/src/tests/rounds.rs) retain all 64 real Pending-response cuts and add one native barrier cut plus one uninterrupted completion per phase. [Native hooks](../../../../crates/sql/sql-sqlite/src/tests/transitions.rs) are installed on the single connection before leasing, remain unarmed during setup, then arm immediately before the tested future is polled. The child waits for native entry, drops the public future, then releases the worker. No new sleeps, timing assumptions, unsafe code or production hook API.
- [Shared watchdog](../../../../crates/lib/tests/support/watchdog.rs) uses the integration target's compile-time temporary directory when present, otherwise the OS temp directory for unit-test inclusions, with a `stackpress-watchdog` subdirectory. The first workspace compile caught the unit-test inclusion lacking `CARGO_TARGET_TMPDIR`; the optional-variable fallback resolves it. The existing `crates/target` tree was not deleted or edited.
- Section 5 of [fast-build-and-test](../../../workflows/fast-build-and-test.md) removes both fixed exceptions and describes the remaining explicit live-DB gates and separate doctests. Other sections are unchanged. No `.config/nextest.toml` was needed: these hang-class tests retain existing process watchdog deadlines, and the new ordering uses native handshakes.

## SQLite old/new assertions and observation points

No existing assertion expression or failure message was removed or weakened. All original 64 suspension cutoffs are retained; its aggregate count is now 66 because two deterministic observations are added.

| Assertion/observation | Old | New | Strength |
| --- | --- | --- | --- |
| Cancellation entry | nth actual `Poll::Pending`, cutoffs 1–64 | Same 64 cuts, plus worker ready/go barrier before acknowledgement | Superset; at least one real cancellation regardless of scheduler speed |
| Acknowledged completion | Only if a future finishes before a cutoff | Same sweep, plus `Cut::Complete` awaiting the real future | Completion guaranteed unless the operation fails/hangs, which fails the test/watchdog |
| Begin barrier | No deterministic seam in this sweep | Native VM progress after arming | Future cannot finish BEGIN before cancellation observation |
| Run barrier | No deterministic seam in this sweep | Native row update, after the prior write finishes | Drops the borrowed operation during actual mutation, retaining rollback-only ownership |
| Commit barrier | No deterministic seam in this sweep | Native commit before its acknowledgement | Preserves the lost-acknowledgement all-or-nothing check |
| Rollback barrier | Timing could produce zero cuts | Explicit native rollback before acknowledgement | Guarantees the formerly flaky phase is cancelled |
| Engine plan barrier | Only opportunistic Pending cuts | Native commit following the plan's DDL | Adds an actual lost-plan-commit observation |
| Borrowed plan barrier | Only opportunistic Pending cuts | Native VM progress after caller BEGIN setup | Adds a real in-flight borrowed-plan cancellation |
| Public future destruction | Drop before returning a cut outcome | Same; native gate additionally releases only after drop | Same ordering plus explicit worker evidence |
| Lease retirement | `settled(&admission)` before persistence inspection | Unchanged, now after all 66 observations | Same permit-retirement/no-orphan guarantee, more observations |
| Run result | `affected == 1` on completion | Unchanged | Exact same result requirement |
| Run/borrowed-plan cancellation | `commit().unwrap_err().kind() == RollbackOnly` | Unchanged | Exact same poison/retirement requirement |
| Commit/rollback completion | `result.is_ok()` | Unchanged | No error swallowed |
| Both completed plan forms | Returned query-result length `== 2` | Unchanged | No statement-count relaxation |
| DDL atomicity | Schema rows empty or length `== 2` | Unchanged | No partial table/index result allowed |
| Borrowed plan persistence | Schema rows empty | Unchanged | Caller rollback/cancellation still forbids persistence |
| Completed Engine plan | Schema length `== 2` | Unchanged | Completion still requires all effects |
| Lost commit | Row count `== 0 || == 2` | Unchanged | Neither one-row result nor replay accepted |
| Acknowledged commit | Row count `== 2` | Unchanged | Completion still requires both rows |
| Non-commit phases | Row count `== 0` | Unchanged | No leaked prior/write/rollback data |
| Sweep result | `completed > 0`, `cancelled > 0`, original failure messages | Unchanged | Deterministic endpoints satisfy the same assertions by actual execution |
| Reopened database | Count `== 0`; fresh insert and close must succeed | Unchanged | Clean reopen and usable writer still required |
| Parent watchdog | Same ten scenarios; `success && !expired`, 10 s each | Unchanged | Executor/native hangs remain killable and failing |

The bounded sweep still explores real cancellation ownership across prepare/run/accounting/completion awaits. Barriers augment it rather than replacing those observations. All native hooks remain test-only.

## Review rounds

1. Logic/complexity: reviewed exact misses, filter/skip ordering, enabled/disabled gates, discovery purity, selected-case fixture prerequisites, cancellation drop-before-release and uninterrupted acknowledgement. Shared parser unit checks plus the seven-executable proof pass; selected native watchdog and offline reference pass. Replaced unused test-module imports exposed by harness=false compilation with qualified names, removing warnings.
2. Responsibility/abstraction: reviewed the pure helper versus each runner's fixture authority, environment ownership, redaction, deadlines, and cleanup; reviewed test-only native hooks and shared watchdog inclusion sites. No further abstraction or production dependency was justified. Targeted harness/SQLite suite reran successfully.
3. Consistency/Rustdoc/comments: aligned the safety module/function descriptions and sync imports, documented handshake setup/arming/destruction order and fixture prerequisites, and ran the workspace gates. The first broad compile found the missing temp variable for unit tests; the follow-up uses `option_env!` plus OS-temp fallback. Subsequent full passes, cargo tests, formatting, lints and docs are recorded below.
4. Safety/soundness follow-up: retained every safety assertion and every original suspension cutoff; reviewed single-connection hook ownership, one-shot callbacks, permit retirement, unknown-commit observations, child kill/reap and cleanup-on-unwind. No unsafe or production behavior changes. Live DB gates remained unset for execution throughout.

## Verification notes

The required workspace command was run with no package exclusions; the only explicit filter is the coordinator-owned listener test:

```sh
cargo nextest run --workspace --all-targets --locked --no-fail-fast -- --skip tcp_watchdog
```

| Workspace pass | Record | Runner summary | SQLite safety runtime |
| --- | --- | --- | --- |
| 1 | `20261029T100052.695094Z-85e41b7332a9` | 71.905s; 739 tests run: 739 passed (1 slow), 82 skipped | 1.456s |
| 2 | `20261029T100515.804811Z-77587d11941f` | 69.205s; 739 tests run: 739 passed (1 slow), 82 skipped | 2.047s |
| 3 | `20261029T100649.431240Z-372a3a36ae8b` | 77.634s; 739 tests run: 739 passed (1 slow), 82 skipped | 1.825s |

Each pass runs 739 tests across 213 binaries. The 82 total skips comprise 30 unset-URL custom cases, 51 existing ignored cases (including watchdog children/controls), and `stackpress-server-http::tcp::tcp_watchdog`. No listener test was run. The generated-profile consumer is slow (about 63–71 s) but passes; no timeout/config relaxation was added.

- Touched-package Cargo gate: `cargo test -p stackpress-lib -p stackpress-sql -p stackpress-sql-postgres -p stackpress-sql-mysql -p stackpress-sql-sqlite -p stackpress-framework-view --features stackpress-framework-view/leptos-ssr --all-targets --locked`: **218 passed, 0 failed, 46 ignored** across reported summaries, including the existing unset-URL messages and the actual render-unwind case.
- `cargo test --workspace --doc --locked`: **942 passed, 0 failed, 0 ignored**.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, and `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`: **passed**. These workspace scopes include every touched crate.
- [Protocol proof](00019-nextest-protocol.py): **7 harnesses, 31 discovered cases, 62 individual unset/offline invocations**, enabled-gate discovery only, exact misses, ignored variants, isolated SQLite reference and opt-in generated-DDL selection. No configured-gate execution. The final serial proof is the isolated confirmation; an earlier final-proof submission overlapped the doctest command, so its wall time may include Cargo lock waits.
- [Source/output audit](00019-audit.py): **35 existing safety assertion expressions/messages unchanged**, all 64 original cutoffs retained, byte-identical Cargo.lock, only authorized paths, workflow outside section 5 unchanged, real render-unwind output in the task target, existing `crates/target` output preserved. Replay the audit after rebuilding the test target, before cleaning it.

Observed cancellation/completion counts from the explicit `cargo test ... tests::rounds::safety_under_watchdog -- --exact --nocapture` record `20261029T101450.859222Z-62cec59a5c84`:

| Phase | Cancellation cuts | Acknowledged completions |
| --- | ---: | ---: |
| begin | 4 | 62 |
| run | 7 | 59 |
| commit | 2 | 64 |
| rollback | 2 | 64 |
| plan | 17 | 49 |
| transaction_plan | 13 | 53 |

Every phase has 66 observations. The guaranteed native cut and full completion are construction properties, independent of how many opportunistic cuts land in a particular run.

KB validator `20261029T095801.256651Z-78c878e247a8`: **146 inherited errors / 27 warnings**, raw validator exit 1; recorded command exit 1. KB validator `20261029T101910.023927Z-336743e33b73`: **146 inherited errors / 27 warnings**, raw validator exit 1; recorded command exit 0. The final recorded command passes by proving exact diagnostic parity and cleanup; the raw validator remains nonzero. Historical diagnostics are preserved; no unrelated repair is in this task.


Scope/limits: native host/default workspace features plus the selected framework-view SSR cargo-test lane. No new browser/WASM/installed-target behavior changed or is claimed. No production implementation line changed; this test-tooling-only task does not establish a new production coverage percentage. Live adapter branches are intentionally unexecuted and not claimed verified. Independent coordinator verification and the TCP listener proof remain coordinator-owned.

## Benchmark ledger

Environment for every Cargo/proof/clean record: `RUSTC_WRAPPER=sccache`, default `SCCACHE_DIR` (unset), `SCCACHE_CACHE_SIZE=20G`, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_JOBS=2`; one `CARGO_TARGET_DIR=<this worktree>/target/nextest-compat`. Both live DB URL variables are removed for execution. All commands use `--locked` where supported. Initial clean and LLVM coverage clean were separate recorded commands. The edit-loop labels describe narrow scope; this brief explicitly uses the gate environment even there.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Rows report end-to-end command seconds separately from observed Cargo Finished-stage subtotals; `n/a` means unavailable, not zero. Outer protocol proofs capture Cargo output, so their build-stage subtotals are unavailable. Warm means the target was retained; it does not imply all feature artifacts were reusable or compiler-cache hits.

| Record ID | Result | Command s | Cargo stages s | Kind/cache | Label |
| --- | --- | ---: | ---: | --- | --- |
| `20261029T094800.012671Z-63c57f7fb598` | passed | 0.637084 | n/a | clean/cold | gate initial clean |
| `20261029T094808.555901Z-126b76e8479a` | passed | 1.362401 | n/a | clean/cold | gate initial coverage clean |
| `20261029T095114.259170Z-2c7922242e54` | passed | 2.373938 | n/a | other/cold | edit loop format |
| `20261029T095125.079858Z-abddb4a79206` | passed | 156.345320 | 71.0 | test/cold | edit loop custom harness and SQLite |
| `20261029T095453.990062Z-f37bd6aae831` | passed | 15.017537 | 8.12 | test/warm | round 1 logic and selection |
| `20261029T095525.799395Z-98251a7a677f` | passed | 44.589409 | n/a | test/warm | round 1 seven harness protocol proof |
| `20261029T095626.538545Z-28f5d33f46d3` | passed | 2.761085 | 0.33 | test/warm | round 2 responsibility boundaries |
| `20261029T095721.492399Z-f7d775f32320` | passed | 2.431653 | n/a | other/warm | round 3 consistency format |
| `20261029T095728.544490Z-5d8c13172758` | failed | 182.289929 | n/a | test/warm | gate workspace nextest 1 round 3 TCP watchdog skipped |
| `20261029T095801.256651Z-78c878e247a8` | failed | 0.385914 | n/a | other/warm | KB validation baseline |
| `20261029T100052.695094Z-85e41b7332a9` | passed | 237.798464 | 40.21 | test/warm | gate workspace nextest 1 retry temp fallback TCP watchdog skipped |
| `20261029T100515.804811Z-77587d11941f` | passed | 72.552991 | 0.61 | test/warm | gate workspace nextest 2 TCP watchdog skipped |
| `20261029T100525.696749Z-8edb374c92ff` | passed | 0.307084 | n/a | other/warm | safety assertion and scope audit |
| `20261029T100601.264313Z-9ac999bb1d48` | passed | 4.911906 | n/a | other/warm | gate format check |
| `20261029T100649.431240Z-372a3a36ae8b` | passed | 81.427412 | 1.4 | test/warm | gate workspace nextest 3 TCP watchdog skipped |
| `20261029T100828.986378Z-6017cea826e1` | passed | 82.128268 | 45.93 | test/warm | gate cargo-test touched packages plus view SSR unset DB |
| `20261029T101014.281916Z-7c277c9622b5` | passed | 52.100122 | 4.28 | test/warm | gate workspace doctests |
| `20261029T101022.695298Z-5ac270ceb621` | passed | 15.768344 | n/a | test/warm | final seven harness protocol proof |
| `20261029T101138.397499Z-1b980e89dc2a` | passed | 98.214109 | 97.0 | clippy/warm | gate workspace Clippy warnings denied |
| `20261029T101341.699503Z-d45927ff7127` | passed | 43.911893 | 43.85 | doc/warm | gate workspace Rustdoc warnings denied |
| `20261029T101450.859222Z-62cec59a5c84` | passed | 12.517977 | 11.12 | test/warm | gate SQLite observed cancellation and completion counts |
| `20261029T101634.712103Z-336db8311a24` | passed | 2.822276 | n/a | test/warm | final serial seven harness protocol proof |
| `20261029T101733.770855Z-f1dd6b5351c3` | passed | 0.269160 | n/a | other/warm | final safety assertion scope and output audit |
| `20261029T101741.478108Z-a81e08c20344` | passed | 3.671280 | n/a | clean/warm | gate final task target clean |
| `20261029T101910.023927Z-336743e33b73` | passed | 0.432060 | n/a | other/cold | KB validation final inherited-diagnostic and cleanup proof |

Task-filtered counter delta: **+25 attempts** (23 passed, 2 failed), **+8 attempts emitting Compiling**, **+0 successful declared-cold full-workspace rebuilds**. Accumulated command time **1117.027616 s**; observed Cargo-stage subtotal **323.85 s**. These are accumulated timings, not elapsed calendar time; the earlier doctest/protocol overlap is visible in timestamps. Initial narrow cold build and later feature rebuilds do not count as declared-cold workspace rebuilds.

Observed target peak, sampled every 2 s: **4,916,064 KiB (4.688 GiB)**, below **8 GiB**. The task target is deleted; the persistent benchmark store remains. The compile failure was the resolved mandatory-temp-variable inclusion; KB validator failures are the preserved inherited diagnostics. No failed test was hidden by retries/exclusions.


## Acceptance notes

Acceptance criteria: none. This receipt is implementer evidence, not owner acceptance or coordinator verification. No public-behavior decision is outstanding. The next step is coordinator source review and independent checks, including the explicitly skipped TCP watchdog. No historical status/acceptance record was changed.

## Coordinator verification

Independently reran on the committed tree at gate tier (`sccache`, non-incremental), recorded as `n19-verify gate *`. All passed:
- Cargo.lock and Cargo.toml are unchanged.
- fmt; workspace clippy `-D warnings`.
- `cargo nextest run --workspace --all-targets --locked --no-fail-fast` twice with **no exclusions**: 740 passed each, including `tcp_watchdog`. The second run took 46.9 s.
- Correction (2026-10-29): the separate cargo-test lane targeted `stackpress-server` instead of `stackpress-server-http` and ran no tests. The nextest runs above did execute `tcp_watchdog`. It was rerun alone in the correct package during the issue 14 M2 verification and passed.
- Workspace doctests: 942.
- `cargo test` on the four SQL packages: 135 passed, 41 ignored, with the unset-URL behaviour preserved.
- The seven-harness protocol proof.

The coordinator removed the stale `crates/target/` folder (gitignored) after merge. Merged into trial/3 on 2026-10-29.
