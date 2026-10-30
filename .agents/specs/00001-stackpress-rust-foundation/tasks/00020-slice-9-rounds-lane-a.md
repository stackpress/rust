# Slice 9 Lane A rounds 1–4 implementation receipt

Status: started; all four released rounds complete, pending independent coordinator verification. Base/HEAD is `566746a` (confirmed with git log -1), branch `trial/3-slice9-session-csrf-email`. Stop after this receipt; no commit, documentation-closeout release, publication or installed acceptance is inferred.

## Task summary and implementation steps

Behavior-preserving review/refactoring of framework-session only, under G42/G43. The starting tree was clean. Preserve every upstream process choice, public API, config gate, error/redirect, callback order, storage codec and default. Record the baseline, perform complexity, responsibility, documentation/consistency and final targeted safety passes, then run the requested complete gate.

Changed seven files: src/storage/query.rs; tests/main/actions.rs; src/auth/bridge.rs; src/auth/hooks.rs; src/account/mod.rs; src/auth/pages/mod.rs; src/views/controls.rs. All paths are under crates/framework/framework-session. No consumer source, README, Cargo manifest, root facade, dependency, root/consumer lock, historical Agent File or other crate changed. This new receipt is the only additional Agent File.

## Round 1 — complexity and branch risk

Reviewed source SQL filter dispatch, scalarization, active/default behavior, CRUD/batch transaction order, auth/password/2FA precedence and account sequential writes. The SQL loop identified the equality map by pointer identity. Replaced that hidden identity dependency with `compare == Compare::Eq`; the fixed dispatch table associates only query.eq with Eq. Same filter order, flattening, values, codecs and return behavior remain.

Read-only upstream ProfileStore.ts:470–513 (especially 481–487) explicitly copies eq, removes its literal active=-1 escape, then processes each independent operator. AuthStore follows the same sequence. The new real-SQLite test `active_override_keeps_non_equality_predicates` pins missing eq.active → active=true, eq.active=-1 → both states, ne.active=-1 → source Boolean(true) comparison, and the default-active conjunction. It observes stored enabled/disabled rows rather than reproducing the compiler's branch.

Baseline: **52/52 session tests**, one externally entered child skipped, record `79aa031d4ad5`. Round 1: **16/16 SQL/record/codec tests**, record `1047fece72e2`. The original first attempt stopped at the old size guard before executing tests, as documented under resources below. No source ILIKE, positive hasnt, native Auth create-return, account removal, identifier or lifetime behavior was changed.

## Round 2 — responsibility and abstraction

Reviewed Service/Session ownership, per-instance memoization/live grants, SQL executor/transaction borrowing, native/public projection, registration and supervised hooks. Found the same fresh-request conversion loop in bridge::resolve, account::resolve_into and Hooks::State::submit. The new **private** `auth::bridge::request(Record) -> Result<Request>` owns that conversion at the existing native bridge boundary. No trait, new service, public API or request-local caller carrier is introduced.

All three callers retain their explicit response ownership and emit point. Conversion still walks source insertion order through the existing rich Date bridge and Nest budgets. The hook calls the helper inside the already-owned async job: conversion failures still become supervised outcomes when the job is polled, not synchronous signup failures. Weak host ownership, queued ordering, drain/shutdown, mutex scopes and cancellation behavior are unchanged. No mutex guard crosses an await.

Round 2: **19/19 source auth/account/composite/supervision tests**, record `434ea6a83784`. These include original invocation traces, rich Date/rejected IO projection, same-pair vs fresh-pair event use, source late/no-check branches, real SQLite OTP/link/2FA handoff, missing config, priority/FIFO/Abort and externally bounded deadlock-class proof.

## Round 3 — consistency, Rustdoc and comments

Reviewed public names/imports, feature/target boundaries, comments, module docs, README claims and shared control state. Replaced stale M3-future language in auth/pages/mod.rs with the actual native/public projection responsibility. Expanded its compressed config projection while retaining response admission before conversion and the same field order.

Expanded dense theme-toggle, password reveal/input and strength-feedback markup in views/controls.rs. Added local explanations for optional theme context, SSR attribute/live-property synchronization, type-only password reveal and empty-strength suppression. Attributes, strings, icons, handlers, conditions, reactive reads/writes, DOM order, native form behavior and CSS are unchanged. No icon, layout, policy or UI feature was redesigned. The README and consumer APIs already describe these private boundaries correctly; no change justified there.

Round 3: formatting, **18/18 shared UI/source-process tests** and **seven session doctests** passed, record `71b511a3d879`. Final hydrate/CSR checks below verify the same controls after the formatting changes.

## Round 4 — targeted follow-up, safety and soundness

Revisited each changed branch/call site: Eq-only escape vs other operators, conversion/error timing, ordered records, independent nested Request/Response ownership, hook polling/supervision, lock/await boundaries, absence of new unsafe/global state, public signatures, secret/native projection boundaries and reactivity/DOM equivalence. **No further change justified.** Keeping longer source process functions is deliberate where extraction would obscure observable guard/error/side-effect order; no global helper or policy unification was imposed.

Round 4: all **53/53 session tests**, one externally entered child skipped, plus all-target owned Clippy -D warnings passed; record `798f10d2a89a`. Compile-fail ownership/Send contracts and process-supervised negative controls remain passing. A recorded audit also confirms unchanged public declaration lines and all frozen manifest/lock/facade bytes (`ebfd50e46167`); the full diff was reviewed for unchanged signatures and behavior, beyond that mechanical check.

## Final verification process and results

Final gate uses Rust/Cargo 1.96.0, aarch64-apple-darwin and wasm32-unknown-unknown, two Cargo jobs, sccache, debug=0 and nonincremental gate builds. Narrow edit-loop runs followed each round; the full gate was not narrowed. All resolving/build commands use --locked --offline, with nonresolving fmt separate. No renderer --all-features, downloads, installs, lock refresh or new dependency was used.

| Gate | Result / recorder ID |
| --- | --- |
| Workspace fmt --all -- --check | Passed in `20261030T143959.406993Z-8aec4c93e75c`. |
| Workspace all-target Clippy -D warnings, default and eight-feature | Both passed in that gate. |
| Workspace all-target nextest, default | **928 passed**, 86 skipped; two test threads, --no-fail-fast, tcp_watchdog excluded. |
| Workspace all-target nextest, eight-feature | **993 passed**, 89 skipped; same full scope/thread policy and explicit TCP exclusion. |
| Workspace doctests / Rustdoc -D warnings, eight-feature | **966 doctests passed**; no-deps Rustdoc passed. |
| Session per-feature/native/WASM and facade normal graphs | consumers/graphs.py --check passed: default/native/sql/server/auth-server/leptos/leptos-ssr/browser native; separate hydrate/CSR plus default/auth-server on wasm32; all root forwarding leaves. `20261030T145007.004199Z-acb80af744b8`. |
| Native/default/browser isolation | Default facade remains stackpress-lib only; native identity/sql/server/auth-server exclude UI additions. WASM graphs exclude native JWT/crypto, SQL/server, CSRF/email/lettre and QR/PNG. |
| consumers/run.py | Direct, facade, plugin and disabled standalone graphs, tuple subsets, fmt, strict Clippy and executable assertions passed in separate retained targets; `20261030T145030.331748Z-a1c8885b9dd4`. No lock resolution/refresh. |
| Finite SQL account host | Fresh native SQLite-backed pages for all 16 source pages/themes/long-content variants, real JSON email event, Hooks shutdown and Engine close; no listener. `20261030T145143.950936Z-cc08d41f039c`. |
| Facade browser hydrate/CSR builds | Each separate mode's fmt, WASM Clippy -D warnings, build, wasm-bindgen, graph/secret checks and lock-subset check passed in the same artifact record. |
| Browser hydrate | **176 cases + no-JS Light** passed via cached Playwright, `20261030T145356.024697Z-3922337d249b`. |
| Browser CSR | **176 cases + no-JS Light** passed via cached Playwright, `20261030T145408.433310Z-f4f631de4e7c`. |
| Coverage | **5218/5707 = 91.43%** executable production lines; 53 tests passed, one externally entered child skipped. `20261030T145201.357247Z-a2f29ead27ab`. |

The eight-feature selection is stackpress-framework-session/leptos-ssr, stackpress-framework-session/auth-server, stackpress-framework-csrf/server, stackpress-framework-email/server, stackpress-framework-api/sql, stackpress-framework-api/codegen, stackpress-framework-api/ui-ssr and stackpress-framework-schema/providers. Exact commands and outputs are retained in the recorder logs. tcp_watchdog remains coordinator-owned.

Browser proof uses only the existing cached `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright` and the existing session proofs/browser.cjs. Requests are fulfilled in memory at synthetic URLs; no listener starts. The full matrix covers all source pages, 1280/375px, 200% zoom, long content, explicit Light/Dark/unset, no-JS Light, labels/keyboard/focus, password reveal/strength, pass/OTP/magic selection, phone submission, copy-secret, theme adoption, CSP/disposal, no horizontal overflow and no runtime/hydration errors. Fresh narrow account and dark enrollment screenshots were visually reviewed.

Coverage uses cargo-llvm-cov nextest with session/leptos-ssr (including auth-server/sql/server), LCOV and the complete first-party src scope. Only tests/examples are excluded; no production exclusion. The denominator grew as dense markup became readable lines; native coverage does not count browser-only handlers as executed. Those paths are separately exercised by the real browser matrix. The 70% development floor is met; publication is not authorized.

Raw artifacts: `/private/tmp/slice9a/rounds/` contains pages/, hydrate/, csr/, browser/{hydrate,csr}-results.json and PNG captures, coverage.lcov, coverage-summary.json, frozen.json, source-final.json and KB diagnostics. No temporary proof output was added to the repository.

## Resources, immutable contracts and limits

Retained target is target/slice9a. The first round-1 build hit the previous **19.4 GiB** guard; no tests executed or assertion failed. Approved incremental/owned session cleanup removed 219.2 MiB of package output. The coordinator then raised the rounds cap **20 → 24 GiB**, guard **19.4 → 23.4 GiB**, and approved exact removal of browser-m3 and llvm-cov-target. Those two directories were removed in one recorded clean step; debug dependency artifacts and consumer-session-* targets were retained. The final instrumented coverage target was rebuilt (recorded cold, warm sccache). Subsequent checks passed under the new guard; measured rounds peak **21.445 GiB**. The wrapper retains its 1,800-second absolute process-group TERM/KILL/reap deadline, alongside the existing short external deadlock-class watchdog tests.

Fifteen frozen files include root/session manifests, root facade and all twelve root/standalone lock files (session, email and API-checkpoint). Every byte remains unchanged from the clean baseline. No new or changed registry tuple, dependency edge, feature or root source line occurred. New private request conversion does not broaden wire/SSR projection or serialize native services/seeds.

G42 still permits multiple handlers, priority then registration order and any-handler Abort. G43 still leaves protection to the configured app/process: missing access does not add a guard, empty effective access is permissive, and source password flags, lifetimes, errors/redirects, CSRF call sites and account partial effects are retained. No new security policy or default-deny is introduced. Existing upstream behavior questions remain at parity; no new owner decision is required.

Evidence is native macOS/SQLite and browser WASM. Live SMTP/SES/sendmail/webhook delivery, live PostgreSQL/MySQL, Linux/Windows native execution and installed Tauri desktop/mobile/device acceptance remain unverified. This behavior-preserving task did not release provider or platform acceptance work.

## Benchmark records

Shared store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs/`. Each outer Cargo/proof/clean sequence is recorded once. Completed pre-receipt counter: **18 attempts, 17 passed, 1 failed** (the single old-guard stop, recovered above). Command-time sum **1088.312s**; observed Cargo-stage subtotal **393.260s**. They overlap and are not interchangeable. Recorded window **1470.893s** includes review/approval gaps and independent parallel targets/browser work. Final receipt writing/validation has a subsequent record.

| Record | Label after slice9a rounds | Result | Command s | Cargo stages s |
| --- | --- | --- | --- | --- |
| `20261030T142957.561121Z-a83f632ee9bb` | baseline immutable locks and source inventory | passed | 1.366 | 0.000 |
| `20261030T143141.455567Z-79aa031d4ad5` | baseline session source and SQL UI regressions | passed | 16.662 | 7.850 |
| `20261030T143315.908987Z-34e77e0b82f3` | round1 explicit equality source SQL predicates | failed | 7.054 | 0.000 |
| `20261030T143422.986368Z-0c9f1c7b0cb8` | approved incremental and owned session guard recovery | passed | 0.949 | 0.000 |
| `20261030T143451.992554Z-9a9fecd3b925` | approved M3 browser and coverage target cleanup cap24 | passed | 1.206 | 0.000 |
| `20261030T143506.524761Z-1047fece72e2` | round1 source SQL predicates after headroom recovery | passed | 6.038 | 3.250 |
| `20261030T143556.132358Z-434ea6a83784` | round2 fresh request bridge and supervised hooks | passed | 8.379 | 4.720 |
| `20261030T143712.220940Z-71b511a3d879` | round3 controls comments source projection and doctests | passed | 17.279 | 9.290 |
| `20261030T143915.324258Z-798f10d2a89a` | round4 no further change ownership safety and full owned regressions | passed | 17.131 | 7.790 |
| `20261030T143959.406993Z-8aec4c93e75c` | gate default and eight feature complete workspace | passed | 568.607 | 256.690 |
| `20261030T144604.211786Z-ebfd50e46167` | gate frozen locks facade public surface scope audit | passed | 0.248 | 0.000 |
| `20261030T145007.004199Z-acb80af744b8` | gate session per feature facade and WASM isolated graphs | passed | 22.904 | 14.590 |
| `20261030T145030.331748Z-a1c8885b9dd4` | gate all standalone session direct facade plugin disabled consumers | passed | 17.041 | 10.430 |
| `20261030T145143.950936Z-cc08d41f039c` | gate finite SQL account host and browser hydrate CSR artifacts | passed | 14.605 | 8.650 |
| `20261030T145201.357247Z-a2f29ead27ab` | gate production coverage cold instrumented target warm sccache | passed | 80.887 | 70.000 |
| `20261030T145356.024697Z-3922337d249b` | gate hydrate full cached Playwright browser matrix | passed | 155.315 | 0.000 |
| `20261030T145408.433310Z-f4f631de4e7c` | gate CSR full cached Playwright browser matrix | passed | 152.141 | 0.000 |
| `20261030T145427.685538Z-b0692accda55` | gate interim recorder and exact workspace counts | passed | 0.499 | 0.000 |

## Acceptance criteria, verification notes and stop

Acceptance criteria: none for behavior-preserving library refactoring; unchanged UI has fresh supplemental browser evidence. Coordinator verification is pending. No Context File promotion or Reference File was needed. The Agent Workspace validator retains the exact inherited **146 errors / 28 warnings**, with no new diagnostic; this is not a clean-KB claim. No historical record was edited, no other agent was messaged, and no commit/push/listener/install/GitHub action occurred. Stop after rounds 1–4 for coordinator verification.
