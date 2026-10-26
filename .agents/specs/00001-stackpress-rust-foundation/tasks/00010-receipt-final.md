# Task 00010 final pass receipt: SQL, framework and examples

Evidence for [owner review task 00010](00010-owner-review-issues.md), implemented on 2026-10-26. Coordinator review and task status remain separate.

## Scope and baseline

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-issues-final`; branch `trial/3-issues-final`; base `ff7a25113c616f094beca23c4f41d9e55f24acc3`. Initial worktree was clean.
- Read the project instructions, Rust style/quality/programming workflow, task and both lane receipts, especially the [lib error policy](00010-receipt-lib.md#11-error-wrapping-policy-for-lane-2). Read issues #4, #6 and #7 with `gh issue view <n> --repo stackpress/rust`; no GitHub writes.
- Changes stay in SQL, framework, relevant example sources/proofs/docs and this separately approved receipt. No edits to lib, server, CLI or Idea; no dependency/lockfile change, commit, push, publication, listeners or messages to other agents.
- Followed the repository's Rust adaptation of chrisai-coding and chrisai-docing's API reference guidance. Toolchain: Rust/Cargo 1.96.0 on aarch64-apple-darwin. Debug info and incremental compilation disabled; four build jobs.

## Implemented contracts

- **#4:** Type-level `must_use` covers Create, Alter, Select, Insert, Update, Delete, Sql, Field, Index, Expr, Selector, TableRef, SqliteOptions and PageSpec. Method-level annotations cover Engine::with_limits and SqlError::with_cleanup. These configure/transform a returned value; useful mutating registration, notification and execution APIs remain discardable. Chaining/ownership semantics stay intact. Ten independent compile-fail doctests deny unused_must_use on discarded chains.
- **#6:** `escape_html` borrows `&str`; its Leptos Title callback retains the required owned String through a thin closure. Module paths no longer allocate an intermediate String solely to escape it. Private SQLite row decoding borrows SqliteRow, returning the same independently owned cells and preserving budget order. HeadPanic borrows the payload only while copying safe diagnostic text. Button reuses its required owned Leptos prop buffer for the class attribute. The public owned mount API remains compatible, with a local explained lint expectation; registration order/error precedence remain unchanged.
- **#7:** Reviewed public fallible SQL/framework APIs using source review and the opt-in missing_errors_doc lint. Added missing contracts for SQLite option validation and missing language/theme/notification providers, clarified renderer/page-registration failures and added the example's shared route failure contracts. Docs explain rejection state and no implicit IO/context installation where applicable. Existing SQL execution/transaction/cancellation/partial-effect contracts remain unchanged. No runtime change was made for documentation.
- **Generic List:** `data-item-id` names a stable keyed presentation item without a Profile-domain dependency. Updated the actual component, SSR assertion, crate API reference, profile-ui README and browser row selector. It carries the same key; visible text, structure, navigation and styles otherwise remain unchanged. Historical task statements describing the original issue retain their provenance.
- **#11 examples:** SQLite UI startup now displays only its distinct installation remedy and retains the original framework error/source chain. Codegen's parser wrapper displays `Profile schema could not be parsed`, retains TransformError/parser causes and preserves the exact safe public coordinates through with_public_message and the existing 400/Invalid policy. Gateway read failures name the asset operation/path, with the native IO error retained; manifest decode failures retain typed serde_json errors instead of flattening them. Native IO kinds and InvalidData remain intact. Audited sql-profile, capability-host and other gateway/codegen wrappers and tests: remaining generic from_source conversions inherit the accepted contextual lib policy, and no old copied-text assertions required changes.
- **Proof backend selection:** browser.py always runs a locked Cargo build of both finite hosts for the requested backend in the shared native target before startup/setup/browser work; SQLite also builds its explicit setup binary. Build failure invalidates stale success and prevents proofs. A new Python regression suite covers memory -> SQLite -> memory reuse and failed rebuild ordering; check.py includes it. Cargo children have a finite 600-second bound and browser scripts retain 120 seconds.

## Regression evidence observed before fixes

| Contract | Red evidence |
| --- | --- |
| SQL consuming builders | Eight new compile-fail doctests failed because their discarded chains compiled successfully (`/tmp/final-must-use-red.log`). |
| SQLite/framework consuming options | SqliteOptions and PageSpec compile-fail doctests independently failed for the same reason (`/tmp/final-options-red.log`, `/tmp/final-pagespec-red.log`). |
| Borrowed escaping | New reusable string/empty/Unicode/delimiter test failed to compile with the original owned String signature (`/tmp/final-escape-red.log`). |
| Borrowed native row | The real native-row reuse regression failed with expected SqliteRow/found &SqliteRow before the borrow change (`/tmp/final-decode-red.log`); its final execution runs inside the existing independent safety watchdog. |
| Neutral List key | SSR assertion failed because data-item-id was absent (`/tmp/final-list-red.log`). |
| Startup context | Exact Display assertion found the remedy followed by `: external operation failed` (`/tmp/final-startup-red.log`). |
| Parser context/public disclosure | Corrected fixture failed with the actual schema coordinate/diagnostic copied into native Display (`/tmp/final-codegen-red.log`). |
| Asset context/cause | Real malformed manifest failed the distinct decode-context assertion (`/tmp/final-assets-red.log`). |
| Backend proof selection | Both Python regressions observed node as the first command rather than cargo build (`/tmp/final-wrapper-red.log`). |

During regression setup, corrected a PublicError field name (`error`, not `message`), an unqualified QueryLimits in a test, canonicalized the parser fixture's macOS temporary root and normalized temporary aliases in the Python target assertion. Corrected fixtures were rerun against the old implementation before recording the red behavior. These were test-harness corrections, not unresolved product failures. Ownership-only refactors retain existing behavior and were checked against existing behavior suites rather than inventing changed behavior.

## Review rounds

1. **Logic/branch risk:** reviewed rendering validation, escaping, row budget reservation order, startup plugin selection and public/private diagnostic precedence. Retained existing error kinds/statuses, cause identities, registration order and no-setup startup behavior. The seven-crate scoped suite with SSR/server/SQLite features passed (`/tmp/final-round1.log`).
2. **Responsibility/ownership:** retained native row ownership in execution while decoding borrows; kept Leptos callback/props ownership at the framework boundary. Reused Button's owned class buffer after the optional ownership lint exposed the generated-props constraint. Kept asset operation context and typed causes in one private error representation; no new public abstraction. Relevant UI/SQLite/gateway library suites passed (`/tmp/final-round2.log`).
3. **Consistency/Rustdoc/comments:** reviewed all remaining consuming APIs, missing error docs and analogous example wrappers; updated the List/README/browser selector contract and public failure/state descriptions. Reviewed source retention, downcasts, safe public parser disclosure, finite proof ordering, error precedence and scope. No unsafe code, dependency, callback contract change or additional behavior scope was introduced. Opt-in lint, final workspace checks and both backend proofs below verify the final result.

## Final verification

All checks below passed after the final source/test changes. Commands use the environment stated above and the absolute native target `target/profile-ui-native`; client builds use the separate `target/profile-ui-wasm`.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | passed |
| `cargo test --workspace --all-targets --locked` | passed |
| `cargo test --workspace --doc --locked` | passed |
| `cargo doc --workspace --no-deps --locked` | passed |
| `cargo +1.96.0 check --workspace --all-targets --locked` | passed |
| `python3 examples/profile-ui/check.py` | passed |
| `python3 examples/profile-ui/proofs/browser.py` | passed |
| `python3 examples/profile-ui/check.py --sqlite` | passed |
| `python3 examples/profile-ui/proofs/browser.py --sqlite` | passed |

- Rustdoc used `RUSTDOCFLAGS="-D warnings"`. Final fmt and whitespace checks passed.
- Workspace test summary: **470 passed**, zero failed, 59 ignored watchdog/feature/probe entries across 103 groups. Ignored watchdog children execute through active parents. Workspace doctests: **867 passed**, zero failed/ignored across 27 groups, including all ten new discarded-chain cases.
- Opt-in audit passed for the seven changed packages with `cargo clippy ... --no-deps --all-targets --features stackpress-framework-ui/ssr,stackpress-framework-view/server,stackpress-example-profile-ui-web/sqlite,stackpress-example-profile-ui-gateway/sqlite --locked -- -D warnings -D clippy::missing_errors_doc -D clippy::return_self_not_must_use -D clippy::needless_pass_by_value` (`/tmp/final-opt-audit.log`).
- Both check.py modes passed freshness, wrapper/manifest regressions, SSR, hydrate and CSR builds, dependency/feature isolation, native/customization tests, doctests, Clippy and the finite service proof. SQLite performed explicit temporary setup. Each backend passed all eight browser proofs: four scripts at root and `/demo/v1`, including hydration identity/focus, keyboard, invalid/empty input, hostile values, loading/cancellation/newest-wins, errors/retry, locale/theme, responsive geometry, navigation and hashed gateway recovery. SQLite additionally passed missing/uninstalled startup checks without file changes.
- The first browser attempt selected a cached Playwright package requiring absent Chromium revision 1234. Selected an existing matching cached package instead: Playwright at `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`, Chromium headless shell revision 1248. The macOS sandbox then denied Chromium process rendezvous; the approved unsandboxed rerun passed over browser pipes, without installing tools or starting listeners. This was host tooling setup, not a product failure.
- Full command results: `/tmp/final-verification-results.json`; logs `/tmp/final-{workspace-clippy,workspace-tests,workspace-doctests,rustdoc,msrv}.log`; per-backend check/browser logs and summary.json under `/tmp/final-{check,browser}-{default,sqlite}/`. Browser screenshots are retained beside their proof logs. Edited documentation relative links and `git diff --check` pass.

## Coverage and limits

Tool: cargo-llvm-cov 0.8.6 with Rust 1.96.0 / native aarch64-apple-darwin. Combined instrumentation ran every selected crate's all-target suites plus their integration consumers with framework-ui/ssr, framework-view/server and both profile-ui host/sqlite features. Separate default/SQLite tests and browser proofs above provide behavior evidence; this coverage selection measures the SQLite native build. Stable line coverage, not branch coverage or doctest instrumentation.

The initial reuse of the shared native target left five crates uninstrumented. That incomplete export was rejected. Ran `cargo llvm-cov clean --workspace` in the task-owned native target, then reran the complete instrumented selection. All seven crates now have nonempty reports. No source changes were needed.

```sh
CARGO_TARGET_DIR="$PWD/target/profile-ui-native" CARGO_LLVM_COV_TARGET_DIR="$PWD/target/profile-ui-native" cargo llvm-cov --no-report --all-targets --locked -p stackpress-sql -p stackpress-sql-sqlite -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-codegen -p stackpress-example-profile-ui-web -p stackpress-example-profile-ui-gateway --features stackpress-framework-ui/ssr,stackpress-framework-view/server,stackpress-example-profile-ui-web/sqlite,stackpress-example-profile-ui-gateway/sqlite
```

LLVM raw exports include actual instrumented native executables so shipped example code is counted rather than hidden by the coverage tool's default examples exclusion. Only each declared crate's production src files count, plus the web example's shared route/customization source. Dedicated test files/directories, dependencies, generated tooling fixtures and other crates are excluded. LLVM source-file summaries still include inline tests. The separate production-only audit unions physical LCOV DA lines by canonical path and excludes exactly these two inline modules: `framework-ui/src/render.rs:317-329` and `codegen/src/lib.rs:122-145`. No production file/branch is excluded. Both reporting conventions exceed 70% per crate.

| Crate | LLVM covered/total | LLVM lines | Production covered/total | Production lines |
| --- | ---: | ---: | ---: | ---: |
| stackpress-sql | 2935/3073 | 95.51% | 2845/2974 | 95.66% |
| stackpress-sql-sqlite | 374/392 | 95.41% | 356/371 | 95.96% |
| stackpress-framework-view | 530/547 | 96.89% | 513/526 | 97.53% |
| stackpress-framework-ui | 743/754 | 98.54% | 687/691 | 99.42% |
| stackpress-example-profile-codegen | 179/187 | 95.72% | 155/158 | 98.10% |
| stackpress-example-profile-ui-web | 350/487 | 71.87% | 333/439 | 75.85% |
| stackpress-example-profile-ui-gateway | 511/618 | 82.69% | 490/578 | 84.78% |

Reports: `/tmp/final-coverage-results.json`, `/tmp/final-coverage-production.json`, `/tmp/final-coverage-raw.json`, `/tmp/final-coverage.lcov`, `/tmp/final-coverage.profdata`; commands/export logic `/tmp/final-coverage.py`; execution log `/tmp/final-coverage-run.log`. Doctests ran separately. Coverage is macOS native/selected features; it does not establish WASM executable coverage, Windows/mobile/packaged Tauri behavior, arbitrary OS stalls or uncooperative path replacement. Actual browser behavior is bounded by the finite Chromium proofs above. No release/publication readiness claim follows from this development gate.

## Cleanup and handoff

Task-owned native and WASM target trees, root test-fixture metadata and any task-created crates/target were removed after saving the reports (`/tmp/final-cleanup.json`). Neither target existed at task start. Local logs/reports/screenshots remain under /tmp, and ignored client dist assets remain available for review. Final fmt/diff checks pass. Branch HEAD remains ff7a251; no staged files, commit or push. Coordinator review is the next gate; task acceptance/status was not changed.

Before receipt creation, the KB validator reported the same **146 missing sibling-source link errors and 21 warnings** as the lane receipts. Those unchanged historical links point outside this isolated worktree (for example to ../lib/src/index.ts). Baseline: `/tmp/final-kb-baseline.log`. The receipt-write step reruns validation and appends its actual result below.

After receipt creation, `python3 .agents/scripts/validate-agent-workspace.py` exited 1: **146 errors and 21 warnings**. No findings name this receipt; the unchanged sibling-source-link limitation matches baseline. Full output: `/tmp/final-kb-validation.log`. The receipt is within the line cap and its task/policy links resolve.
