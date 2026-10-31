# G46 source-behaviour decisions receipt

Status: implementation and required reviews complete; coordinator verification pending. Baseline: clean confirmed **3a95c1f**. Authority: G46 at `/Users/cblanquera/server/projects/stackpress/rust/.agents/specs/00001-stackpress-rust-foundation/decisions.md:85`, read from the coordinator's uncommitted main-checkout record as instructed. No historical decision/spec record was edited. This receipt is evidence, not acceptance or permission to commit.

## Behaviour and source evidence

The only production change is the dialect branch in the two SourceStatement adapters: PostgreSQL retains ILIKE; SQLite/MySQL retain LIKE. Both delegate checked identifier/bind rendering first, propagate errors, and keep caller strings as binds. SQLite/MySQL return the existing rendered Query directly; PostgreSQL retains the existing reconstruction. No public signature, configuration, guard or event ordering changed.

New tests `g46_search_operators_follow_dialect_for_profile_and_auth` and `g46_search_operators_follow_dialect_for_application_and_session` cover all three dialects through real Profile/Auth/Application/Session stores, q where declared and like, including a bound `A LIKE B` value that must not be rewritten. New `g46_sqlite_ascii_searches_execute_for_*` tests execute on real in-process SQLite files: mixed ASCII case, Profile/Auth join, counts, API stores and the registered application-search event. Old SQLite failure assertions now require successful rows/counts/status. The original Node records.json remains source evidence, not the G46 Rust expectation.

All ten questions are marked resolved under G46 in session/aggregate docs and stable claim/ledger IDs: account Profile retention is intended soft delete via @active (default active filtering and active=-1 escape); five items follow the original source unchanged (auth create-return, positive hasnt, optional phone on update, email sign-in skip send, Moment admin dates); PGlite is not required (MySQL/SQLite/PostgreSQL are the standard engines); license is owner-owned; SQLite purge keeps DELETE FROM "table". Nothing else was changed. Stable `question-*`, S11-PGLITE, SOURCE-LICENSE and SQLITE-TRUNCATE identities retain their references with explicit resolved status.

Baseline session test passed with its old SQLite error expectations; two corrected API baseline tests passed. All four new regressions were RED before the adapter change: both rendering tests exposed SQLite ILIKE and both execution tests hit SQLite syntax errors. Final scoped SQL suite: **110 passed / 6 existing skips**. Early authoring failures were a mistaken API test binary name (`sql` instead of `main`), a two-row fixture initially asserted as one, a four-character new auth token corrected to the unchanged five-character minimum, and a stale doc claim sentence restored. These are recorded below; no implementation policy was changed to satisfy them.

## Required review rounds

| Round | Finding / verification |
| --- | --- |
| 1 — complexity | **Nothing found** beyond the intended change. Exhaustive three-dialect match, early render error and bind preservation are clear. Four G46 regressions rerun: 4 passed. |
| 2 — responsibility/abstraction | **Nothing found** warranting extraction. A shared helper would expose a new cross-crate API in session or place source-specific policy in generic SQL. The tiny local branch keeps existing boundaries; both adapters remain identical. Store/rendering/paging selections: 7 passed. |
| 3 — consistency/Rustdoc/comments | **Nothing further found.** Both declarations/comments explain G46 and supported operators, stale open-question prose is closed, unrelated JSON formatting was retained. G46/coercion selections: 5 passed; fmt, default/combined Clippy -D warnings, doctests and Rustdoc all passed. |

No additional behaviour-preserving edits were justified; no dependencies or public APIs were added. Round commands and findings are retained in /private/tmp/g46/rounds.json.

## Commands and final results

Every Cargo/proof/clean command uses the benchmark recorder; outer scripts record their nested argv once. Cargo resolution is always **--locked --offline**; fmt has no resolution flags. Exact compatible features F:

`framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,framework-api-codegen,framework-csrf-server,framework-email-server,stackpress-framework/codegen,stackpress-framework-schema/providers`.

| Command / scope | Result |
| --- | --- |
| cargo fmt --all --check | Passed. |
| cargo clippy --workspace --all-targets [--features F] --locked --offline -- -D warnings | Default + combined passed. |
| cargo nextest run --workspace --all-targets [--features F] --locked --offline --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)' | **933 default / 1036 combined passed**, 86 / 89 existing skips. tcp_watchdog excluded, coordinator-owned. |
| cargo test --workspace --doc [--features F] --locked --offline -- --test-threads 2 | **962 default / 967 combined doctests passed**. |
| RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features F --locked --offline | Passed. |
| cargo nextest run -p stackpress-framework-session -p stackpress-framework-api --features stackpress-framework-session/sql,stackpress-framework-api/sql --test main --locked --offline --test-threads 2 --no-fail-fast | 110 passed; rounds add the recorded -E selections. |
| python3 crates/framework/framework-session/docs/check.py | 20 pages, 17 fences, 37 claims, 4 negative controls; local paths/anchors resolve. |
| python3 crates/framework/framework/docs/check.py --compile | 19 pages, 11 exact fences, 59 claims, 7 controls; 10 finite executions passed, plus native SSR / WASM hydrate / CSR compilation. |
| python3 crates/framework/framework/proofs/check.py | 24 inventory rows, 5 command rows, 15 export families, 77 declarations, R01–R20, 43 upstream hashes pass. |
| python3 /private/tmp/g46/scope.py; final KB validation | Scope/lock/example freeze passed; canonical KB 0 errors / 28 inherited warnings; raw worktree 146 inherited sibling-path errors / same warnings. |

The external documentation workspace is `/private/tmp/slice11-docs-6hnn1ri8`. Its five full/headless/facade/website/build-host locks are byte-identical to the checked-in consumer locks, all registry tuples are root subsets, and the root lock is unchanged. No Cargo resolution or refresh was needed. All current documentation fence bodies are byte-identical to baseline; extra compilation verifies their dependency compatibility. Session fences were statically rechecked and session doctests ran in the workspace; no new session external-example compilation is claimed.

Coverage command per package: `cargo llvm-cov -p PACKAGE --features FEATURES --locked --offline --fail-under-lines 70 --json --output-path /private/tmp/g46/PACKAGE-coverage.json -- --test-threads 2`. Tool: installed cargo-llvm-cov/LLVM, native aarch64-apple-darwin, Rust/Cargo 1.96.0. No new exclusions or threshold manipulation. Independent first-party src denominators:

| Crate / features | Executable line coverage |
| --- | --- |
| session / sql,auth-server,leptos-ssr | 5222/5710 = **91.45%** |
| API / sql,codegen,ui-ssr | 3708/4146 = **89.44%** |
| aggregate / full,codegen | 1043/1078 = **96.75%** |

Each changed adapter file is 11/14 = 78.57%; all three dialect cases execute in the regression tests. Whole-crate coverage exceeds the 70% development floor. Reports include owning tests; the table isolates first-party implementation source. Aggregate implementation itself is unchanged. Native coverage does not establish browser/installed or live backend behaviour.

Live PostgreSQL/MySQL execution is **skipped**: existing backend harness opt-ins STACKPRESS_TEST_POSTGRES_URL and STACKPRESS_TEST_MYSQL_URL are unset. The two source-store suites have no network fixture; PostgreSQL/MySQL evidence here is rendering/bind preservation. SQLite ASCII execution is real. No server was started or probed, no listener or browser runtime was launched. MySQL matching depends on the configured collation; G46 relies on default _ci. UI/browser behaviour was not changed; documentation WASM compilation is separate from browser replay. G42/G43 remain binding.

## Exact changed files

**23 files including this receipt**; only two production Rust files and two test files. All **3547 other baseline tracked files** remain byte-identical. Root/consumer locks, all manifests, facade, license files, historical .agents records and unrelated source remain unchanged. Root registry set/count stays **699**, SHA-256 `8687d9b425a56b3865918ba79bf211caa9bb4a9f17c40fdbbc64673a2574653e`. Index empty; benchmark runs were never staged.

| File |
| --- |
| [.agents/specs/00001-stackpress-rust-foundation/tasks/00024-g46-source-questions.md](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00024-g46-source-questions.md) |
| [crates/framework/framework-api/src/sql/adapter.rs](../../../../crates/framework/framework-api/src/sql/adapter.rs) |
| [crates/framework/framework-api/tests/sql.rs](../../../../crates/framework/framework-api/tests/sql.rs) |
| [crates/framework/framework-session/README.md](../../../../crates/framework/framework-session/README.md) |
| [crates/framework/framework-session/docs/account.md](../../../../crates/framework/framework-session/docs/account.md) |
| [crates/framework/framework-session/docs/api/account.md](../../../../crates/framework/framework-session/docs/api/account.md) |
| [crates/framework/framework-session/docs/api/auth.md](../../../../crates/framework/framework-session/docs/api/auth.md) |
| [crates/framework/framework-session/docs/api/storage.md](../../../../crates/framework/framework-session/docs/api/storage.md) |
| [crates/framework/framework-session/docs/api/views.md](../../../../crates/framework/framework-session/docs/api/views.md) |
| [crates/framework/framework-session/docs/auth.md](../../../../crates/framework/framework-session/docs/auth.md) |
| [crates/framework/framework-session/docs/claims.json](../../../../crates/framework/framework-session/docs/claims.json) |
| [crates/framework/framework-session/docs/concepts.md](../../../../crates/framework/framework-session/docs/concepts.md) |
| [crates/framework/framework-session/docs/guides/authentication.md](../../../../crates/framework/framework-session/docs/guides/authentication.md) |
| [crates/framework/framework-session/docs/source-behavior.md](../../../../crates/framework/framework-session/docs/source-behavior.md) |
| [crates/framework/framework-session/docs/storage.md](../../../../crates/framework/framework-session/docs/storage.md) |
| [crates/framework/framework-session/docs/verification.md](../../../../crates/framework/framework-session/docs/verification.md) |
| [crates/framework/framework-session/src/storage/statement.rs](../../../../crates/framework/framework-session/src/storage/statement.rs) |
| [crates/framework/framework-session/tests/main/actions.rs](../../../../crates/framework/framework-session/tests/main/actions.rs) |
| [crates/framework/framework/docs/api/sql.md](../../../../crates/framework/framework/docs/api/sql.md) |
| [crates/framework/framework/docs/claims.json](../../../../crates/framework/framework/docs/claims.json) |
| [crates/framework/framework/docs/source-behavior.md](../../../../crates/framework/framework/docs/source-behavior.md) |
| [crates/framework/framework/docs/verification.md](../../../../crates/framework/framework/docs/verification.md) |
| [crates/framework/framework/proofs/coverage-map.json](../../../../crates/framework/framework/proofs/coverage-map.json) |

## Resources and recorder evidence

Fresh target/g46; initial recorder-wrapped `cargo clean --locked --offline` and `cargo llvm-cov clean --workspace --locked --offline` passed before the first build. Existing target/slice11 was retained. Warm sccache, two jobs, debug=0, incremental=0, offline. Cap 24 GiB / guard 23.4 GiB; sampled completed-command peak **14.53 GiB**, current size **14.55 GiB**. No deadline/headroom trip or later cleanup. Disposable scripts/reports live under /private/tmp/g46; documentation copies are identified above.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; runs/<id>.json/log hold full argv/results. This effort adds **16 recorder attempts**, **9 with observed compilation**; these are effort-filtered counters, not unrelated coordinator runs. Wall time and observed Cargo-stage subtotal are separate; unavailable is not zero. The final audit passed; its completed status and timing were transcribed from the recorder after validation.

| Record ID | Stage | Status / exit | Wall s | Cargo-stage s |
| --- | --- | --- | ---: | ---: |
| 20261031T032002.060314Z-b626501bf641 | baseline and bounded target preparation | passed (0) | 0.61 | unavailable |
| 20261031T032018.927310Z-63b9d4509a94 | initial clean and current SQLite search baseline warm sccache | failed (1) | 71.13 | 65.00 |
| 20261031T032922.292851Z-deeef0a49c32 | API baseline corrected integration binary | passed (0) | 43.14 | 37.08 |
| 20261031T034818.814271Z-c3dfebbd90bb | RED regression tests before dialect change | failed (1) | 22.68 | 15.41 |
| 20261031T035218.644206Z-5a3b4fc5a029 | GREEN dialect-aware session and API SQL regressions | failed (1) | 10.34 | 5.78 |
| 20261031T035252.752373Z-6e0f5fb38c30 | GREEN complete session and API SQL suites | failed (1) | 96.19 | 3.72 |
| 20261031T035327.097420Z-bb0bce6ab1d1 | resolved docs claims and coverage ledger checks | failed (1) | 2.12 | unavailable |
| 20261031T035359.027694Z-dd584a89d60a | SQL GREEN after valid five-character token fixture | passed (0) | 29.83 | 2.39 |
| 20261031T035417.559205Z-1e48d8e5f2a6 | docs links claims and complete ledger validation | passed (0) | 2.17 | unavailable |
| 20261031T035453.326243Z-1dd4b6041d42 | full compatible workspace gate default and combined two test workers | passed (0) | 1067.04 | 568.26 |
| 20261031T041412.608649Z-72ee4f734f51 | frozen lock manifest license scope and resolved-claim proof | passed (0) | 2.75 | unavailable |
| 20261031T041437.114531Z-62ed8cffc621 | three required post-GREEN review rounds | passed (0) | 4.95 | 0.92 |
| 20261031T041502.900778Z-80684d83f823 | independent session API and aggregate coverage floor 70 | passed (0) | 261.27 | 239.71 |
| 20261031T042901.065655Z-771115d219bd | workspace counts and timing evidence extraction | passed (0) | 2.74 | unavailable |
| 20261031T042920.344257Z-3d0f579c356d | aggregate exact docs fences native SSR hydrate CSR and final session docs | passed (0) | 672.72 | 662.16 |
| 20261031T044113.547063Z-cd166103a125 | final receipt scope and KB validation | passed (0) | 6.19 | unavailable |

No commits, staging, pushes, installs, dependency downloads, listener starts or GitHub writes. Stopped for coordinator verification of G46.

## Coordinator verification

Committed as `c734147` on `3a95c1f`; 22 tracked files plus this receipt, no manifest, lock or license change. Reviewed the two adapter diffs (exhaustive dialect match; PostgreSQL alone rewrites to ILIKE; SQLite/MySQL return the rendered query unchanged; binds untouched, proven by the `%A LIKE B%` bind assertions) and the test changes (former SQLite-failure assertions now assert results). Independent fresh-target gate `/private/tmp/g46-verify/run.sh` on 2026-10-31, exit 0: Cargo.lock identical to `3a95c1f`; fmt; clippy session+API (sql, all targets, -D warnings); nextest session+API sql 113 passed / 6 skipped (network opt-ins unset); aggregate full,codegen 36 passed; doctests; session and aggregate docs checks and aggregate proofs ledger check green. Live PostgreSQL/MySQL execution not exercised; those dialects are proven at the rendered-SQL level only.

Merged into trial/3 as `70da23f`.
