# Task 00051 — self-contained crate tests

Status: verified for the non-live lane; coordinator review pending. Branch `release/self-contained-tests`, base `7c014bb`. Acceptance criteria: none. Live replay is deferred below; this receipt does not authorize publication.

Initial result committed by the coordinator as `5315730`; the release follow-ups below record the current state.

Tests now read their own package inputs. No assertions, expected output, golden/oracle content or `.stderr` changes. No Cargo.toml or lock changes, dependencies, listeners, installation, commit or push. The five coverage-owned crates are untouched.

## Changes

- Server/CLI/framework-view use local lib watchdog copies; SQLite maintenance uses a local generated-workspace watchdog; SQL uses a local discovery harness. Five new helper copies prepend a provenance comment and otherwise match canonical bytes.
- Four byte-identical inputs are local: Leptos corpus, framework-schema session/API schemas, aggregate build-host source-client schema. Source paths/SHA-256 pins are in [test-copies.json](../../../../tools/release/test-copies.json).
- CLI entry fixtures use `CARGO_TARGET_TMPDIR`, retaining unique names and exact-directory cleanup.
- G59 already localized API/session/aggregate live helpers and API management shared code. Eleven existing helper copies are now freshness-guarded without editing their Rust files. Coordinator explicitly directed keeping nested unpublished consumers where they are; packaged tests cannot import their inputs.
- [Boundary checker](../../../../tools/release/check_test_boundaries.py) scans all 23 candidates' src (a conservative superset of cfg(test)), tests/examples/benches and support scripts. Escaped includes/module paths, unresolved include expressions, fixed runtime sibling reads, Python repository anchors, symlinks and nested Cargo package inputs fail.
- [Copy checker](../../../../tools/release/check_test_copies.py) merges every `test-copies*.json`, rejects duplicate destinations and checks exact bytes/source hashes. Schema and intentional-divergence requirements are in [README](../../../../tools/release/README.md). Twenty copies and eleven checker regression tests pass.

## Test-verified review rounds

1. Logic/complexity: preserved helper bodies, loader keys, fixture bytes and cleanup order. Added negative controls; corrected facade discovery because the repository's example applications are not Cargo example targets. Boundary/regression proofs pass.
2. Responsibility/abstraction: crate-local test ownership; only repository freshness policy is shared. Separate lane manifests merge without duplicate destinations. Added drift protection to G59's unchanged local helpers. Copy/regression proofs pass.
3. Consistency/comments: local path/provenance conventions and documented limitations; raw/conditional attributes, string data, nested comments and line-number preservation. Follow-ups fixed quoted manifest paths and added Python anchor scanning after finding the coverage-owned repository SQL proof. Eleven regression tests pass. No Rust behavior refactor was justified.

## Verification

Environment: sccache; `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`, two Cargo build jobs; `TRYBUILD` unset; retained `target/task00051`, separate consumer and instrumentation targets. All Cargo/proof/cleanup commands were recorded. Observed target peak 11,534,476 KiB (11.00 GiB); removed only completed task outputs at capacity gates.

Narrow tests: 336 passed / 40 gated skips (`20261111T022138.464184Z-c177043d0c4d`); source/schema 98 passed / 2 skips (`20261111T022501.599864Z-37eb5ce906ff`). Full default: 1,061 + isolated TCP watchdog passed; compatible: 1,226 + isolated watchdog passed. Main sweep skip counts 89/97 include the separately executed watchdog. No expectation diff occurred.

- gate fmt: exit 0.
- gate Clippy default: exit 0.
- gate nextest default: exit 0.
- gate TCP watchdog default: exit 0.
- gate doctests default: exit 0; 976 doctests passed.
- gate Rustdoc default: exit 0.
- gate Clippy compatible: exit 0.
- gate nextest compatible: exit 0.
- gate TCP watchdog compatible: exit 0.
- gate doctests compatible: exit 0; 981 doctests passed.
- gate Rustdoc compatible: exit 0.
- gate aggregate consumers and build-host check: exit 0.
- gate aggregate graphs: exit 0.
- gate session consumers: exit 0.
- gate session graphs: exit 0.
- gate generated workspace freshness: exit 0.
- gate generated admin freshness: exit 0.
- gate API checkpoint freshness: exit 0.
- gate checker regressions: exit 0.
- gate test boundaries: exit 0.
- gate copy freshness: exit 0.
- gate changed crate coverage: exit 0.

Boundaries pass for all 18 owned candidates. Pending coverage worker: `stackpress-sql-postgres` (3 references), `stackpress-sql-mysql` (10), `stackpress-framework-sql` (17, including repository-level `tests/proofs/lengths/prove.py`), `stackpress-framework-admin` (2), `stackpress-framework-email` (0 escaped reads found, still separately owned). Default strict checking remains available after integration; this lane uses `--pending-coverage`.

Coverage is LLVM-mapped native `src/**` for each changed Rust crate, including mapped in-source tests, without extra exclusion patterns. Separate tests/examples/consumers and dependencies are not attributed as implementation. Exact selected features, counts and benchmark records are in [verification evidence](../../../resources/self-contained-tests-00051/verification.json).

| Crate | Covered/total lines | Coverage |
| --- | --- | --- |
| stackpress-server | 2040/2173 | 93.88% |
| stackpress-cli | 1306/1402 | 93.15% |
| stackpress-sql | 3224/3394 | 94.99% |
| stackpress-sql-sqlite | 727/808 | 89.98% |
| stackpress-leptos | 7232/8683 | 83.29% |
| stackpress-framework-view | 1216/1520 | 80.00% |
| stackpress-framework-schema | 5811/6840 | 84.96% |
| stackpress-framework | 1176/1231 | 95.53% |

All eight meet the requested 70% lane floor; this is not a waiver of the final publication floor.

Live replay: **not run (servers unavailable)**. Both approved PostgreSQL/MySQL endpoints refused connections; no databases created. Coordinator directed finishing without waiting and will replay live suites in the combined gate.

Root lock bytes and all 699 registry tuples remain identical. Agent Workspace validation retains 146 inherited missing-link errors and 44 warnings; none concern this receipt. The initial false-positive checker failure was fixed; the generated-workspace freshness invocation was corrected to the documented `emit generate --check true` syntax. Both failed records are preserved. Initial sandbox writes to the shared benchmark store/receipt were denied before execution; approved escalation completed the requested writes.

Benchmark store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Snapshot through `20261111T035215.053608Z-cf3109250bfa`: 60 lane attempts, 17 emitting compilation; statuses {'passed': 56, 'failed': 4}. Accumulated command time 5400.712s, observed Cargo-stage time 3705.220s across 21 records. These are distinct, potentially overlapping accumulated durations, not elapsed time or compiler CPU time. Full IDs/commands are in the evidence. First narrow/fresh consumer/coverage targets began empty despite the wrapper's warm label; no cold-rebuild or performance comparison is claimed.

No context promotion: lane evidence awaits coordinator verification.

## Publish-order follow-up — 2026-11-11

Base: `5315730`. Historical selected-crate verification; the standard-workspace publication follow-up below supersedes this measurement and the example location. The coordinator approved the existing unpublished `generated-workspace-app` lane. The original verification above describes the initial commit; the manifest/lock and proof locations below supersede those specific initial claims.

- Moved the server lifecycle proof to `examples/generated-workspace/app/tests/sqlite_host.rs`. All nine original host assertions remain. Recovery observes a public healthy read; the two private all-permit cleanup acknowledgements stay in `sql-sqlite/src/tests/contention.rs`, driving the same held-transaction/time-out path directly. No public API or production code change.
- Moved `m0-host` to that application's examples, changing only its documented Cargo command. Moved `non_send_handler`/`non_send_service` into its existing `tests/ui` tree; all four source/`.stderr` files are byte-identical. Its existing compile-fail glob runs both cases in the workspace suite and coverage. Repaired the historical M0 receipt link to the relocated example; its past commands remain historical.
- The repository host lane retains the original five-second pool budget. It uses real time because this package has no Tokio test-util feature, with a thirty-second independent process bound. SQLite's direct proof retains the controlled before/after-deadline checks.
- Sole manifest edit: removed `stackpress-server = { path = "../../server/server" }` from SQLite dev-dependencies. The sole lock graph change removes that workspace edge; all 699 registry tuples remain identical.
- Audited all 18 owned candidates. SQLite → server was the only later-crate dev edge; no other owned candidate needs a move. The boundary checker now rejects used later/unpublished dev-dependencies, supports renamed/target dependencies, permits normal dependencies and ignores comments/string data. Fourteen regression tests pass; all 18 owned boundaries and 20 copies pass. Five coverage-owned candidates remain pending; none was edited. PostgreSQL's schema/SQL/generated-app and email's schema/SQL/session references are reported for that worker.

Review rounds: (1) logic — retained the nine host assertions, both private cleanup acknowledgements and deterministic held-lease ordering; four narrow moved/compiler proofs pass; (2) responsibility — private adapter checks versus public repository lifecycle; independent SQLite suite 14/14 and checker regression suite 14/14 pass; (3) consistency/docs — grouped imports, documented clock/process budgets and checker limits; final affected checks below pass. No further behavior-preserving refactor was justified.

Affected gate: fmt; default/compatible Clippy `-D warnings`; nextest 26/26 and 28/28 (nine ignored child/control tests in each sweep); four doctests per feature lane; Rustdoc `-D warnings`; moved example; generated-workspace freshness; boundary/copy/regression checks. Coverage runs SQLite + repository app + server: 92 passed, 12 ignored. Exact commands/features and all 18 audit rows are in [follow-up evidence](../../../resources/self-contained-tests-00051/followup-verification.json). Aggregate/session/browser and unrelated generators were not rerun because no API or generated source changed; their original full-gate evidence remains above. Live replay belongs to the coordinator's combined gate; no live harness was changed.

| Crate | Covered/total src lines | Coverage |
| --- | --- | --- |
| stackpress-sql-sqlite | 727/808 | 89.98% |
| generated-workspace-app | 3421/4026 | 84.97% |
| stackpress-server | 2008/2173 | 92.41% |

Published-crate figures retain the tool's standard implementation scope (`src/**`, default test-file omissions). Its default filter also omits all repository examples, so the same collected profiles/objects were exported again without that filter for app `src/**` and proof files. The moved host proof is 150/156 lines (96.15%); private contention proof 48/48 (100%). No added exclusions. These values meet the requested 70% lane floor; they do not waive the final publication floor. Native aarch64 macOS only.

Benchmark snapshot through `20261111T044310.123041Z-bf5cf04a8948`: 29 follow-up attempts, 9 emitting compilation, statuses {'failed': 3, 'passed': 26}; accumulated command time 357.683s, observed Cargo-stage time 193.760s across 13 records. Timings are distinct accumulated durations, not elapsed time or compiler CPU time. Failed attempts remain: initial `--locked` before obsolete-edge removal; report assertion detecting the omitted app; unsupported report `--features`. All resolved, with profiles reused for export. Report completion and receipt validation are subsequent recorded checks. Target observed at 2,214,408 KiB, below 12 GiB; no cleanup was needed. The shared benchmark store is unchanged from the initial receipt.

Agent Workspace validation: 146 inherited missing-link errors and 44 warnings, with no Task 00051 diagnostics. No expectation/golden edits, extra manifest changes, dependencies, listener, installation, commit or push. No context promotion; coordinator review remains the next step.

## Publication floor — standard workspace follow-up

Base `5315730`; final affected gate green, commit-ready and uncommitted. This supersedes the earlier selected-crate/special-export measurement as release evidence.

- `generated-workspace-app` explicitly declares `[[test]]` targets `sqlite_host` and `sqlite_service`. The concurrent/nested lifecycle proof was already auto-discovered under tests; the separate `m0-host` seed/startup example is now `tests/sqlite_service.rs`, with its body unchanged and a ten-second parent watchdog. Both execute in ordinary workspace nextest. The historical M0 receipt links to the final test location while preserving its past commands.
- Existing host assertions, direct private-admission checks and all UI/`.stderr` bytes are preserved. No public API, production source, new dependency, exclusion or expectation change. Final manifest edits are the obsolete SQLite server dev line removal and the two approved repository-app test declarations. The lock still changes only that removed workspace edge; 699 registry tuples are unchanged.
- **SQLite: 740/808 executable lines = 91.58%, meeting the 90% publication floor.** Measured with standard `cargo llvm-cov nextest --workspace --all-targets --locked --offline`, the complete owner-provided compatible feature set, one test thread, JSON output, and no custom export or added exclusions. Summary [ 493.103s] 1229 tests run: 1229 passed (1 slow), 96 skipped. Both native target PASS entries are in that recorded run. No additional SQLite tests were necessary.
- Affected checks after that measurement: fmt; selected default/compatible Clippy `-D warnings`, nextest (27 default / 29 compatible; nine ignored child/control entries per sweep), doctests and Rustdoc `-D warnings`; generated-workspace freshness; all 18 owned boundaries, 20 copies and 14 checker regressions. Exact commands/features, per-file SQLite counts and record IDs: [publication-floor evidence](../../../resources/self-contained-tests-00051/publication-floor-verification.json).

Review rounds: logic — unchanged seed/startup/dispatch/shutdown body plus two passing native tests; responsibility — explicit repository-only targets, no dependency additions, checker/copy/lock proofs pass; consistency/docs — clarified historical measurement and updated source links, final affected checks pass. No further refactor was justified.

Benchmark snapshot through `20261111T050851.187372Z-161747ee7f42`: 21 publication-floor attempts, 4 emitting compilation, statuses {'passed': 21}. Accumulated command time 999.944s; observed Cargo-stage time 284.030s across 12 records. These are distinct accumulated timings, not elapsed time or compiler CPU time. Snapshot excludes subsequent report completion and final receipt checks. Target observed at 9,144,452 KiB, below 12 GiB. Commands retain the same sccache/incremental/debug/jobs settings and shared benchmark store as above.

Agent Workspace validation retains 146 inherited missing-link errors and 44 warnings; no new receipt diagnostics. Five coverage-owned crates are untouched/pending. No live harness was changed or live database URL enabled; combined live replay remains with the coordinator. No development listener, installation, commit, push or context promotion.
