# Slice 11 rounds 1–4 — behaviour-preserving review

Status: implementer complete; coordinator verification pending. Clean baseline `5c129d1` confirmed before work. Scope follows the released [rounds contract](../slice-11-design-verification.md) and [Rust programming workflow](../../../workflows/rust-programming.md). Acceptance criteria: none; this is a library refactoring checkpoint, not product or publication acceptance.

## Changes and per-round findings

Exactly nine aggregate files changed: seven implementation files and two test files. No public behaviour/API, feature, dependency, manifest, root facade, lock, existing example, other crate or historical Agent File changed. All tracked files outside those nine retain their baseline SHA-256; public declaration lines and all four compile-fail snapshots remain unchanged. G42/G43, source skip/error semantics and all carried questions remain unchanged.

| Round | Finding and implementation | Verification |
| --- | --- | --- |
| 1 — complexity | [view build](../../../../crates/framework/framework/src/native/view.rs): pair each output-path key directly with its compiler; remove string-to-callback match and use an early skip. Preserve factory → catalog loading → empty-engine skip → clients/assets/pages order, failure short-circuiting and UTF-16 result truncation. | Baseline and first edit: 35 aggregate tests. Added source truthiness test with a real nonempty catalog: null/false/zero/empty string skip; empty array/object remain truthy; mixed stages retain source order. Then 36 tests pass. |
| 2 — responsibility/abstraction | [SQL orchestration](../../../../crates/framework/framework/src/native/commands/sql.rs): extract event-only native query/output work into one private async function; lifecycle/revision dispatch stays separate in the same module. No trait/module/public seam added. | 36 tests pass, including real SQLite lifecycle/query, lazy scripts, revisions, rollback and provider errors. Final proof compares extracted body to baseline after indentation adjustment: identical statements. Query still avoids client/revision acquisition and preserves argv/default/backslash/error/output behaviour. |
| 3 — consistency/Rustdoc/comments | Expand dense view props/factory, server develop options and language JSON expressions without changing expressions or field/evaluation order. Correct path/null and language absence/translation docs, add registrar Errors sections, explain packaging completion gate and fix stale M0 test module description. | 36 tests, compile-fail doctest, all-target Clippy -D warnings and Rustdoc -D warnings pass. All four trybuild expectations pass without overwrite and remain byte-identical. |
| 4 — safety/soundness | No change justified. Review owned futures/Invocation lifetimes, native SQL result ownership and rollback, provider cancellation/reentrant config, per-host services, nested locale dispatch and packaging task supervision/shutdown. | 13 real externally bounded cases, forced-kill/reap negative control, isolated API leaf and complete source ledger pass. Closing gate repeats them. |

The [new source truthiness case](../../../../crates/framework/framework/tests/cases/view.rs) tests externally observable compiler selection, not loop structure. Existing source fixtures/oracles stay unchanged. Readability-only edits also touch native/client.rs, native/commands/server.rs, native/config.rs, native/language.rs, native/view/packaging.rs and tests/main.rs; these plus view.rs, commands/sql.rs and tests/cases/view.rs are the entire implementation diff.

### Safety review and frozen integration surfaces

No synchronous task-state, service/registry or configuration lock is retained across an await or application callback. No process-global active client, locale, compiler or task group was introduced. Owned callbacks keep Send futures and source cancellation/drop behaviour.

The existing per-instance asynchronous `Packaging::joining` gate deliberately spans completion waits to serialize JoinSet's single completion waiter. It is not the synchronous task-state lock: state is released before Pending; shutdown first closes submissions and aborts jobs, then joins. The live-drain/suspended-callback and panic/failure tests prove shutdown/reaping and observable outcomes. Removing that gate would risk losing a completion wakeup; its accepted M1 behaviour is retained. Non-awaited packaging still never changes the build response.

Reviewed but did not edit server ConfigPresence (explicit keys/null/layering) and StartupContext::on_request (literal hook, original phase policy and G42 order), plus root facade guards/re-exports. No material issue found; their complete files retain baseline hashes. No exact-file grant was needed.

## Final gate

| Check | Result |
| --- | --- |
| Workspace fmt; default + compatible combined all-target Clippy -D warnings | Passed. |
| Workspace nextest default / combined | 931 / 1032 passed; tcp_watchdog excluded as coordinator-owned; 86 / 89 existing skips. |
| Default / combined workspace doctests; combined Rustdoc -D warnings | 962 / 967 passed; Rustdoc passed. |
| Foundational server default / http nextest; http Clippy -D warnings | 36 / 61 passed; Clippy passed; server files unchanged. |
| Aggregate isolation | No-feature + eleven leaves (including native/codegen), WASM no-feature, crate-alone no-feature normal graph pass; isolated API-only registrar test passes. |
| Facade isolation | All 32 facade features and native/browser graphs pass; default first-party graph remains root + stackpress-lib; both intentional compile-error guards pass. |
| Consumers | All eight fmt/Clippy/build/finite executions pass: full, direct, headless, API-disabled, disabled, facade, website, build-host. Full/direct/facade ordered phase/event/response inventories match; generated freshness passes. |
| Browser | Separate hydrate/CSR builds; 352 session cases (176 each), two no-JS Light cases and 12 aggregate smoke cases pass. Four unset/wrong Playwright path controls pass. |
| Deadlock watchdog | 13 real cases + SIGTERM-ignoring forced SIGKILL/reap control pass. Each actual test has an external 45 s bound. Consumer/matrix executable also has a finite external runtime bound. |
| Aggregate executable-source coverage | 1043/1078 = 96.75%, full,codegen; no source exclusions. |
| Server executable-source coverage | 1921/2484 = 77.33%, http; unchanged integration helper crate reverified. |
| Root re-exports | Compile/graph/diagnostic evidence only; no executable denominator or 100% claim. |
| Ledger/source | 24 inventory rows, five command rows, 13 disposition groups / 15 export families, all 77 declaration rows and R01–R20 resolve; 43 upstream hashes and source command/order oracles unchanged. |
| Docs / locks / scope | Session docs checker passes. Server has no package docs checker; unchanged API docs and workspace Rustdoc checked. Root + all eight consumer lock bytes and all manifests unchanged; exactly 699 external registry tuples, no resolution or new tuple. |
| Links / KB | Receipt links resolve. Canonical trial/3 layout: zero errors / 28 inherited warnings. Raw worktree: 146 inherited missing sibling-path errors / same warnings, unchanged from M3. |

Exact combined argv is in the recorder: aggregate full + four UI families + SSR, API codegen, CSRF/email server, aggregate codegen and schema providers. No renderer all-features. Coverage uses cargo llvm-cov --fail-under-lines 70 plus independent owned source-file line sums; native coverage and browser evidence remain separate.

All browser proofs use exactly `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`, its own cached headless Chromium, fresh contexts and in-memory request interception. No executablePath/channel, system Chrome or listener. The existing checker confirms session replay assertions are unchanged; 16 real SQLite/Auth/Profile/TOTP pages supply Light/Dark, narrow/desktop, zoom, long-text, form/focus/copy/phone/disposal/overflow checks. Native signing/database/SMTP sentinels remain absent from SSR seed/WASM graphs.

## Accepted limits and open questions

Named M3 evidence gaps are unchanged: API-MATRIX (26 generated-management/consent/native-navigation browser cases need an aggregate finite transport seam), API-BUILD-HOST (full API regeneration is not replayed by the Note build-host), EDGE-REPLAY (not every owning-crate edge case is re-invoked through aggregate). Owning tests and compatible compile/SSR/graph evidence remain in place; these are not claimed as aggregate browser proof.

All ten owner questions remain open, with current behaviour following upstream or the accepted lower crate:

| Question | Current behaviour retained |
| --- | --- |
| PGlite | S11-PGLITE: no connector/dummy feature; PostgreSQL dialect reuse only. |
| MIT/Apache metadata | SOURCE-LICENSE: declarations unchanged, no copied upstream source assets or publication claim. |
| SQLite purge/TRUNCATE | SQLITE-TRUNCATE: source Sqlite.js:404–408 emits SQLite-rejected TRUNCATE … CASCADE; accepted stackpress-sql SQLite DELETE mapping remains. |
| Account removal keeps Profile | Open question — current behaviour follows upstream. |
| SQLite ILIKE | Open question — upstream / accepted backend error retained. |
| Auth create-return | Open question — current behaviour follows upstream. |
| Positive hasnt | Open question — current behaviour follows upstream. |
| Optional phone on update | Open question — upstream retained; G44 optional signup phone unchanged. |
| Email sign-in skips send | Open question — current behaviour follows upstream. |
| Moment admin date tokens | Open question — current behaviour follows upstream, including literal tokens. |

S11-WHATWG, Node-only packaging, live SMTP/SES/sendmail/Postgres/MySQL, real app-owned command/compiler IO and installed Tauri remain explicit limits/follow-ups. No TCP/TLS flush, remote acknowledgement or installed-platform acceptance is inferred. The aggregate adds no global guard, default-deny or route ownership arbitration.

## Benchmark and resource evidence

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Every Cargo/proof/clean command is recorded; nested argv is printed. Same warm target/slice11; sccache, two jobs, debug=0, incremental=0, resolving Cargo commands --locked --offline. No builds/resolutions/edits outside the released scope.

Cap 24 GiB / guard 23.4 retained; sampled peak 18.15 GiB, final 16.42 GiB. Only completed owned consumer outputs were removed when needed; parent debug dependency artifacts were retained. No deadline/guard trip. Fresh private outputs under /private/tmp/slice11-rounds keep historical evidence untouched. A staging-only old private output path was caught and corrected before browser/coverage execution; both preparation records are retained.

| Record ID | Stage | Status / exit | Command seconds | Cargo-stage seconds |
| --- | --- | --- | --- | --- |
| 20261030T222127.307612Z-f1adaf6bfb69 | baseline and aggregate source contract tests | passed (0) | 7.63 | 0.94 |
| 20261030T222207.989764Z-0f2e7228d9d3 | 1 complexity build callback selection | passed (0) | 8.77 | 3.47 |
| 20261030T222311.324697Z-a2a2e7a383c3 | 1 source path truthiness branch proof | passed (0) | 8.69 | 2.77 |
| 20261030T222408.156773Z-7321888b9b74 | 2 SQL query and lifecycle responsibility | passed (0) | 8.77 | 3.28 |
| 20261030T222619.612390Z-9f423bc80b02 | 3 projection readability Rustdoc and comments | passed (0) | 20.99 | 13.45 |
| 20261030T222826.921576Z-8fbc0681d3de | 4 ownership cancellation and supervised shutdown | passed (0) | 5.81 | 2.86 |
| 20261030T222919.148777Z-4de07a2f11f0 | final gate staging | passed (0) | 2.88 | 0.00 |
| 20261030T223006.155367Z-e1b5c9cab28f | final workspace and foundational server gate | passed (0) | 210.73 | 18.75 |
| 20261030T223032.004610Z-377e8de4a934 | isolated output path staging correction | passed (0) | 3.34 | 0.00 |
| 20261030T223441.824320Z-1ba9191fd1cd | all standalone consumers and graph isolation gate | passed (0) | 281.45 | 245.44 |
| 20261030T223658.479897Z-0768aff24b20 | receipt preparation | passed (0) | 0.02 | 0.00 |
| 20261030T223936.700545Z-a7eb9dda1964 | full session matrices and aggregate smoke browser gate | passed (0) | 416.01 | 93.02 |
| 20261030T224645.370722Z-8f6488359714 | closing isolation watchdog coverage docs and locks | passed (0) | 31.67 | 10.43 |

Final scope/link/KB/receipt assertion: 20261030T224751.201043Z-062332bba03c (outcome in recorder). 13 prior completed attempts (0 failed), plus this final assertion. Prior completed command total 1006.75 s; observed Cargo-stage subtotal 394.41 s. These overlap and are reported separately. End-to-end elapsed 1586.66 s includes edits/review/waits. No test failure was hidden; generated snapshots were never overwritten in these rounds.

Stop after rounds 1–4 for coordinator verification. No commits, pushes, installs, GitHub writes, listeners, other-agent messages, lock changes, historical-record edits or later-stage work.
