# G47 application configuration placement receipt

Status: implementation, three review rounds and local verification complete; coordinator verification pending. Baseline: clean confirmed **8115fd4**. Authority: G47 in [decisions.md](../decisions.md). This is evidence for the scoped task, not product acceptance or permission to commit.

## Converted application and contract

Assessed sql-profile first and retained it: its existing CLI -b bootstrap and app-owned SQLite path express a meaningful environment difference without a new framework or CLI API. `config.example.json` is replaced by `config/develop.json` and `config/build.json`. **Only sqlite.path differs** (target/sql-profile-develop.sqlite versus target/sql-profile-build.sqlite); both retain the existing three-row policy. No new operation, transport, installation policy or listener was added.

The new private `src/config.rs::shared()` supplies an application fallback limit of two, above the unchanged profiles plugin default of three. The real build_application installs this shared ConfigLayers before the CLI-supplied ConfigLayers, using the existing append API. Later explicit app-authored layers win. Without a file supplying a limit, two rows are now the documented application fallback; selecting either environment restores the existing three-row result. The public builder signature is unchanged; the module is private. No dependency/manifest/lock/path exception was needed.

Precedence, lowest first: **plugin section defaults → shared src/config.rs → -b environment file → explicit app code overrides**. JSON remains data-only. Templates/code bindings and computed/environment-derived values belong in src/config.rs using existing typed APIs. No config module/file is loaded implicitly; relative paths resolve from the process working directory, not the config file. Registration remains plain functions, with the same database → profiles prerequisite/order.

## Real-builder and executable proof

Baseline: 4 existing example tests passed, 2 watchdog children skipped in normal discovery. Added a configuration scenario to the existing native watchdog rather than a new test binary. It failed RED before the builder change: the plugin default was 3 where the shared layer required 2. GREEN: all **18 SQLite-adapter/example tests passed**, 11 existing ignored child fixtures; their **4 doctests passed**. No failing production test remains.

The new scenario reads each actual checked-in JSON file, invokes the real build_application, checks typed configs and executes profile-search. It pins default 3 → shared 2 → file 3 → explicit code 1; request limit=100 cannot override this policy. Temporary storage replaces only sqlite.path for isolation, without global cwd changes. Both an out-of-range 0 and a wrong-type string from a real bad JSON file reject the build in Config. No shared layer silently masks the invalid final value.

All native scenarios run in independent child processes with a 10-second parent kill/reap deadline and an 8-second cooperative timeout. The existing synchronous-stall negative control passes. Actual CLI executable tests retain separate 10-second deadlines. Six successful native scenarios are covered by the one parent test: parity, literals, startup, setup, failures and configuration.

The four exact finite README setup/search commands ran from the repository root via /private/tmp/g47/readme.py. Both actual -b commands returned the canonical Ada/Grace/Margaret envelope with total=3. The two remaining README verification commands are the recorded 18-test/4-doctest run. Task-owned data files remain at target/sql-profile-develop.sqlite and target/sql-profile-build.sqlite; no pre-existing file was adopted, cleared or reseeded. Repeating setup deliberately fails, as documented.

## Required review rounds

| Round | Findings and proving rerun |
| --- | --- |
| Complexity | **Nothing found** warranting refactoring. Two visible appends preserve the order; validation and error propagation stay in the existing builder. The native watchdog parent, including the configuration scenario, passed again. |
| Responsibility/abstraction | **Nothing found.** Shared app values have a private module; JSON files hold only environment data; database/profile plugins retain their responsibilities. No new public API, loader, dependency or abstraction. Both real executable tests passed again. |
| Consistency/Rustdoc/comments | **No implementation change justified.** Rustdoc/comments and all four author-facing locations agree. Follow-up prose distinguishes pre-builder file acquisition errors from typed-value errors during build and says JSON cannot hold code values. All 4 example tests, the 3 exact server/CLI Markdown fences, docs links/claims and the final workspace gate pass. |

No further material finding required a code change. Review selections, findings and timings are retained in /private/tmp/g47/rounds.json; doc clarification was rechecked in the final docs run.

## Author-facing documentation and verification

Updated the example README, the server typed-config guide's Own external input explicitly section, both CLI argument/application-binary guides and aggregate getting-started's new Where config goes section. The latter maps original common.ts/develop.ts/build.ts to Rust src/config.rs/config/develop.json/config/build.json. The aggregate claims.json adds g47-application-config-placement with source, real-builder test and both environment-file anchors.

The aggregate checker passes **19 pages / 11 fences / 60 claims / 7 negative controls**. All server/CLI/aggregate guide fences are byte-identical to baseline; no aggregate runnable fence was touched, so --compile is not required or claimed for this prose change. Three owned server/CLI Markdown fences were compiled directly with rustdoc --test against exact matching rlibs from the existing locked/offline example/server/CLI graph. No disposable manifest or lock resolution was needed. The async examples are compile evidence; their declared functions are not claimed as executed. Behaviour is proved by the example's native tests and real CLI runs.

Neither server nor CLI currently has a package docs/check.py. The scoped /private/tmp/g47/docs.py checks the five owned guides/README's local paths/headings, pins unchanged guide fences, runs the aggregate checker and compiles the three exact Markdown fences. Rustdoc/doctests provide the package API check. Aggregate/session proof tooling remains unchanged.

| Check / command | Result |
| --- | --- |
| cargo fmt --all --check | Passed. |
| cargo clippy --workspace --all-targets [--features F] --locked --offline -- -D warnings | Default + compatible combined passed. |
| cargo nextest run --workspace --all-targets [--features F] --locked --offline --test-threads 2 --no-fail-fast -E 'not test(tcp_watchdog)' | **933 default / 1036 combined passed**, 86 / 89 existing skips. |
| cargo test --workspace --doc [--features F] --locked --offline -- --test-threads 2 | **962 default / 967 combined passed**. |
| RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --features F --locked --offline | Passed. |
| cargo nextest run -p stackpress-sql-sqlite -p stackpress-example-sql-profile --all-targets --locked --offline --test-threads 2 | 18 passed, 11 existing child skips. |
| cargo test -p stackpress-sql-sqlite -p stackpress-example-sql-profile --doc --locked --offline -- --test-threads 2 | 4 passed. |
| python3 /private/tmp/g47/readme.py | Four exact finite README cargo run commands passed; argv/results in readme-result.json. |
| python3 /private/tmp/g47/docs.py | Five owned pages' links/heading anchors, three exact Markdown fences and aggregate claims passed. |
| python3 /private/tmp/g47/scope.py; python3 /private/tmp/g47/kb.py after | Exact file boundary, frozen locks/APIs/context/licenses, source append order and canonical KB passed. |

F = `framework-aggregate-full,framework-aggregate-ui-session,framework-aggregate-ui-api,framework-aggregate-ui-admin,framework-aggregate-ui-view,framework-aggregate-ssr,framework-api-codegen,framework-csrf-server,framework-email-server,stackpress-framework/codegen,stackpress-framework-schema/providers`.

All nextest/test invocations use two test workers. tcp_watchdog is coordinator-owned and excluded from workspace nextest and server coverage (--skip tcp_watchdog); no renderer all-features was used. Network backend opt-ins are unset, so live MySQL/PostgreSQL cases remain ignored. No listener or browser/installed-platform run was needed or started for this configuration-only example/docs change.

## Coverage and limits

Installed cargo-llvm-cov/LLVM, native aarch64-apple-darwin / Rust-Cargo 1.96.0. Command per crate: `cargo llvm-cov -p PACKAGE [--features FEATURES] --locked --offline --fail-under-lines 70 --json --output-path /private/tmp/g47/PACKAGE-coverage.json -- --test-threads 2`; add --skip tcp_watchdog for server. The example's first test run passed all 4 tests, but the report was empty because the tool's default filename regex excludes every examples directory. A recorded verbose report confirmed the cause. The report-only correction uses `cargo llvm-cov report -p stackpress-example-sql-profile --locked --offline --no-default-ignore-filename-regex --ignore-filename-regex REGEX --fail-under-lines 70 --json --output-path ...`, reusing those profiles. REGEX is the actual generated default regex with only `(tests|examples|benches)` changed to `(tests|benches)`; all other dependency, toolchain, target and test exclusions are preserved in /private/tmp/g47/example-report-filter.txt. All seven example src files are included. No production exclusion or threshold reduction was introduced. This is the [tool's documented override](https://github.com/taiki-e/cargo-llvm-cov#exclude-file-from-coverage). The table isolates each owning first-party src denominator; native/browser/feature limits remain separate.

| Crate / features | Executable line coverage |
| --- | --- |
| sql-profile / default | 191/194 = **98.45%** |
| server / http | 1921/2085 = **92.13%** |
| CLI / default (tty) | 1306/1402 = **93.15%** |
| aggregate / full,codegen | 1043/1078 = **96.75%** |

All exceed the 70% development floor. Server/CLI/aggregate implementation is unchanged; their measurement confirms the requested touched-crate floor. New example/shared code is covered by the real builder and executables. Coverage does not establish additional hosts, live databases, runtime template support or environment-file code execution.

## Exact changed-file list

**15 paths including this receipt:** 9 existing files modified, 1 old example JSON deleted, 4 new example files and this receipt added. All **3560 other baseline tracked files** remain byte-identical. All manifests/root and consumer locks, .agents/context, license files, server/CLI/framework implementation and historical .agents records are frozen. Root registry tuples remain exactly **699**, root lock SHA-256 `8687d9b425a56b3865918ba79bf211caa9bb4a9f17c40fdbbc64673a2574653e`. Nothing was staged; benchmark runs were never staged.

| Status | File |
| --- | --- |
| `A` | [.agents/specs/00001-stackpress-rust-foundation/tasks/00025-g47-config-placement.md](../../../../.agents/specs/00001-stackpress-rust-foundation/tasks/00025-g47-config-placement.md) |
| `M` | [crates/framework/framework/docs/claims.json](../../../../crates/framework/framework/docs/claims.json) |
| `M` | [crates/framework/framework/docs/getting-started.md](../../../../crates/framework/framework/docs/getting-started.md) |
| `M` | [crates/server/server/docs/guides/typed-config.md](../../../../crates/server/server/docs/guides/typed-config.md) |
| `M` | [examples/sql-profile/README.md](../../../../examples/sql-profile/README.md) |
| `D` | `examples/sql-profile/config.example.json` |
| `A` | [examples/sql-profile/config/build.json](../../../../examples/sql-profile/config/build.json) |
| `A` | [examples/sql-profile/config/develop.json](../../../../examples/sql-profile/config/develop.json) |
| `A` | [examples/sql-profile/src/config.rs](../../../../examples/sql-profile/src/config.rs) |
| `M` | [examples/sql-profile/src/lib.rs](../../../../examples/sql-profile/src/lib.rs) |
| `A` | [examples/sql-profile/tests/cases/configuration.rs](../../../../examples/sql-profile/tests/cases/configuration.rs) |
| `M` | [examples/sql-profile/tests/cases/mod.rs](../../../../examples/sql-profile/tests/cases/mod.rs) |
| `M` | [examples/sql-profile/tests/proof.rs](../../../../examples/sql-profile/tests/proof.rs) |
| `M` | [tools/cli/docs/guides/application-binary.md](../../../../tools/cli/docs/guides/application-binary.md) |
| `M` | [tools/cli/docs/guides/arguments.md](../../../../tools/cli/docs/guides/arguments.md) |

## Benchmark/resources and final KB

Every Cargo/proof/clean command is recorded once at its outer command. Cargo uses --locked --offline (fmt has no resolution flags). Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; runs/<id>.json/log retain exact argv, results and environment. Fresh target/g47 was initially cleaned with separate recorded cargo clean and cargo llvm-cov clean --workspace before the first build; prior targets were retained. First-build cache label is cold with warm sccache; subsequent target outputs stay warm. Two jobs, debug=0, incremental=0; Cargo stayed offline. The user's separate connectivity check succeeded in browsing and in a recorded local HTTPS HEAD request to rust-lang.org (HTTP 200); no package was fetched.

Standing cap **24 GiB**, guard **23.4 GiB**. Sampled completed-command peak **9.40 GiB**, current target **9.40 GiB**; no guard/deadline trip or mid-effort cleanup. Effort-filtered counter change: **20 attempts**, **9 with observed compilation**. Wall and observed Cargo-stage subtotal stay separate; unavailable is not zero. The expected RED attempt is retained; no package install, fetch, lock resolution or new tuple occurred.

| Record ID | Stage | Status / exit | Wall s | Cargo-stage s |
| --- | --- | --- | ---: | ---: |
| 20261101T084603.794508Z-238e7c4557f2 | baseline snapshot and fresh bounded target preparation | passed (0) | 0.54 | unavailable |
| 20261101T084621.402598Z-5773df22afcf | fresh target initial clean | passed (0) | 2.04 | unavailable |
| 20261101T084633.313523Z-29ae5e3f9ecc | initial coverage clean | passed (0) | 2.05 | unavailable |
| 20261101T084646.643190Z-b27e5642cc5c | edit-loop baseline sql-profile warm sccache empty target | passed (0) | 38.47 | 33.38 |
| 20261101T084834.187733Z-316c354ab168 | edit-loop RED shared-before-file precedence before builder change | failed (1) | 4.10 | 1.53 |
| 20261101T085003.076555Z-050c90e97a14 | edit-loop GREEN converted real example and SQLite adapter README tests | passed (0) | 57.22 | 9.26 |
| 20261101T085149.240704Z-036ec324277e | proof exact finite README setup and environment commands | passed (0) | 10.33 | 7.86 |
| 20261101T085320.517820Z-0446b72a619d | three required post-GREEN refactoring rounds | passed (0) | 10.36 | 3.11 |
| 20261101T085357.624531Z-58e3df4c316d | gate docs links claims and exact server CLI Markdown examples | passed (0) | 14.48 | 10.88 |
| 20261101T085513.130203Z-3d8bb0ff3051 | gate full workspace default and compatible features two test workers | passed (0) | 1164.59 | 557.18 |
| 20261101T085624.184593Z-e95fdfd23034 | proof exact scope frozen locks and layer order | passed (0) | 2.17 | unavailable |
| 20261101T085919.411876Z-be018b4deeb5 | proof canonical KB and inherited worktree baseline | passed (0) | 2.24 | unavailable |
| 20261101T100055.386114Z-3434785ee000 | gate final docs links claims and exact server CLI examples | passed (0) | 7.63 | 0.57 |
| 20261101T101144.585639Z-a98e2f5e970c | gate independent example server CLI aggregate coverage floor 70 | failed (1) | 54.11 | 45.31 |
| 20261101T101954.299757Z-6f315f1c0391 | requested local HTTPS connectivity check | passed (0) | 2.71 | unavailable |
| 20261101T102018.085979Z-0b30ba666f47 | coverage report filter help inspection | passed (0) | 2.48 | unavailable |
| 20261101T102043.257606Z-afd4fe4cbff0 | diagnose existing example coverage report verbose | passed (0) | 2.48 | unavailable |
| 20261101T102138.680748Z-5bff24ad869c | corrected example report include owned examples source | passed (0) | 2.52 | unavailable |
| 20261101T102203.705468Z-c2dc981e4a33 | gate independent example server CLI aggregate coverage floor 70 | passed (0) | 116.75 | 88.10 |
| 20261101T102454.554725Z-ee62f577e334 | final receipt exact scope and KB verification | passed (0) | 5.22 | unavailable |

Canonical-layout KB: **0 errors / 28 inherited warnings**. Raw worktree: **146 inherited sibling-path errors / 28 warnings**; exact error/warning delta zero. Receipt links resolve; the deleted file is listed without a broken link. Final audit passed after writing/validation; its completed status and timing are transcribed above. No commits, staging, pushes, installs, listener starts or GitHub writes. Stopped for coordinator verification.

## Coordinator verification

Committed as `591b0e9` on `8115fd4`; 15 files (sql-profile example, its tests and README, typed-config guide, two CLI guides, aggregate getting-started and claims, this receipt). No manifest, lock, license or framework API change. Reviewed `src/config.rs`, both environment files and the `build_application` wiring (shared layer installed before caller layers). Independent fresh-target gate `/private/tmp/g47-verify/run.sh` on 2026-11-01, exit 0: Cargo.lock identical to `8115fd4`; fmt; example clippy -D warnings; example nextest 4 passed / 2 skipped; example doctests; aggregate docs check; both README environments set up and searched as written (develop and build each returned the three canonical rows), scratch databases removed. With the owner's local servers running, the live `stackpress-sql-postgres` and `stackpress-sql-mysql` suites passed (M0/M1/M2 each printed matching `created owned database` / `drop confirmed` pairs).

Merged into trial/3 as `fb43538`.
