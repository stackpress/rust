# Task 00013: M4 and final owner-findings verification

Status: **M0–M3 accepted by coordinator; M4 implemented and verified, ready for coordinator review.** Owner visual acceptance, managed-preview rebuilding, commits and installed Part B remain separate. Read with the [whole-task receipt](00013-receipt-owner-findings.md), [approved design](00013-owner-findings-design.md) and [consolidated owner walkthrough](00013-review-preview-owner-checklist.md).

Baseline: `trial/3-s8-polish`, `f860764` plus this uncommitted M4 diff. M3 was accepted/merged as coordinator-reported `16b687c`. No listener, install, dependency/Cargo/lockfile, tracked generated-output, preference/theme/G39, frui, commit or push change. Frui remained read-only. Unchanged preferences/format browser assets were explicitly retained from accepted `/private/tmp/s8-review-m3`; changed gallery/stream and both review native/admin-WASM variants were freshly built.

## Completed scope and compatibility

**Step 0:** restored Select page checked before hydration with **zero selected rows**, alongside partial-row selection, at root and `/demo/v1`. All six mode/mount/scenario cases pass against accepted M3 and fresh M4. This was a missing test case: **no table component or early-selection adoption correction was necessary**. The restored test remains strict about all 12 visible keys, unchanged parsed DOM, native node/focus and no acquisition.

**F03/F04:** literal spaces and explicit rejected/allowed URL labels separate corpus anchors without changing sanitizer policy. Gallery Safety samples groups the exact hostile Resource, controlled textarea and HTML/Markdown corpus. Ordinary Notes, profile name, presentation values and reactive title are realistic. Stream Details uses a normal project summary; its separate Safety samples preserves the exact hostile delayed value. Native/WASM sanitizer equality, exact replay, initial textarea leading newline/quotes/Unicode, early edits, controlled state, form values and inertness remain proved.

The review builder creates an explicit isolated `schema-review.idea` changing only the profile-name label to **Full name**, via existing generator input/output options. Each bundle retains source/review SHA-256 provenance in `review-schema.json`. Normal automated schemas/outputs keep **Name </script> 🦀**; the plugin keeps **Plugin field label**. No tracked schema/generated file is rewritten. The catalogue's nullable/default/hostile proofs are labelled Safety samples.

**F05:** Typed nested rows is absent from the owner gallery in SSR/hydrate/CSR. The raw four-token fixture, codec and native tests now belong to `collection_proof.rs`; `start_collections` adopts its pre-hydration selection. Missing/null/false/true, stable form names, keyed identity, Set null and the reentrant default factory are retained. The shared `parse_presence` import stays compatible; only the example BreadthDraft's removed proof field/callers change. A nullable-boolean product control remains deferred, as the owner decided.

**D05 addition:** TextView renders an explicit empty array as **None**, other arrays as comma-separated plain escaped values. Scalar/Missing/Null presentation, input adapters, RoleContext::text, DTOs and wire values are unchanged. Native tests cover empty/populated/Unicode/hostile lists and exact unchanged snapshots; browser catalogue checks and inspected images show `tags: None`.

## Stream integration and observable recovery

The review gateway uses the accepted optional external classic async status script; the bundle now asserts byte equality with its fixed source and includes it in its verified manifest. It remains independent of the deferred/manual-start WASM entry point. No inline executable script, nonce bypass, eval/innerHTML, user text, replay decode, fetch, automatic retry or hydration was added to the status updater.

The composed-gateway proof caught the deferred bootstrap overwriting the early status message with old interruption wording. Its incomplete-document branch now uses the same definitive **“This page didn't finish loading. Reload this page.”** The completion/failure admission gate is unchanged. The independent six-second **“hasn't finished loading”** guidance remains for browsers that expose neither a failure marker nor parsing end.

New debug-only `stream-pipe`/`pipe::exchange_stream` sends actual native HTTP headers and base64 byte frames over the existing duplex host. A native body error remains `terminal=false`; the browser worker errors its ReadableStream and never converts truncation into EOF. Normal buffered pipe/API representation remains unchanged. The worker uses the gateway's manifest-served assets under strict `script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'none'`.

The final six composed-gateway cases cover success/late/pending at root and `/demo`. In this relay Chromium exposes `readyState=complete` without a completion/failure marker for late/pending, so reviewers see the definitive text. M0's separately retained erroring relay can leave `readyState=loading`, where its honest six-second fallback is what appears. Both behaviors are observable-state decisions, not transport diagnoses. No failed case hydrates, publishes details/success or refetches the Resource. Success hydrates once and its counter works. This is actual incremental parser/HTTP-error evidence through interception, **not TCP/TLS flush or managed-loopback owner acceptance**.

## Review rounds and material findings

| Round | Review and verified result |
| --- | --- |
| 1 — logic/branch risk | Checked exact list values, Missing/Null/false/zero, native-edit/replay identity, real HTTP framing, completion/failure ordering, fixed recovery text and no partial hydration/retry. No extra logic refactor was justified after the bootstrap fix. Selected native/doctest **43 entries** and six progressive browser cases pass. |
| 2 — responsibility/ownership | Moved the raw presence codec/tests into the proof-only collection module and textarea corpus into Safety samples; root Resource ownership/replay and public helper imports stay stable. Workspace **737 passed / 51 ignored / 26 filtered** (`--skip tcp_watchdog`), **940 doctests**, gallery/core/breadth/collection browsers pass. Ordinary label checks then exposed cross-snapshot compiler-cache mixing; see below. |
| 3 — consistency/docs/comments | Reviewed public/private contracts, per-block comments, scoped styles, native/target boundaries, matching assets and cleanup. Added stream-pipe pre-bind refusal and guarded proof JSON/stdin errors so finally still reaps children. Final nine-package native/doctest **181 entries**, warning-denying Rustdoc, fmt, workspace/review Clippy and five separate WASM mode/target Clippy+normal dependency graphs pass. Browser graphs contain no native server/SQL/Tokio edge. |
| Safety/soundness | Rechecked escaping and URL policy, no raw/private error seed, fixed status text, strict-CSP external asset, no marker/EOF invention, completion-gated hydration, value/wire fidelity, role guards and child/root disposal. No unsafe code or new dependency. Relevant real-native/WASM browser matrices pass. |

**Cache correction:** isolated source files alone did not protect one warm target from review-only label objects. A native shared-crate clean passed a cold normal build but missed cached WASM output. Both builders now clean the generated shared native output and explicitly clean **shared + client for wasm32-unknown-unknown before each browser mode**. Strict normal pre-hydration/post-CSR hostile-label assertions were retained/strengthened. All **24** normal base/plugin/disabled mode/mount/layout cases pass after a review-filled cache, and all **8** review Full-name/plugin cases pass on the reverse transition. No normal or review label expectation was weakened.

Other failed development attempts were an unquoted numeric CSS selector, a proof selector needing its original form after the second proof form was extracted, and a new unit fixture using an unapproved field under its existing deny-by-default policy. Only the fixtures/selectors were corrected. Failed benchmark attempts remain visible.

## Fresh coverage and browser evidence

`cargo-llvm-cov 0.8.6`, `rustc 1.96.0 (ac68faa20 2026-05-25)`, LLVM **22.1.2**, native **aarch64-apple-darwin**, SSR/review-preview plus admin codegen (exact feature argv retained). Native executable coverage is measured per changed scope, including shipped generated code and proof fixtures, using physical LLVM LCOV DA lines. Exact inline-test modules/test files, dependencies and build/tool output are excluded; generic instantiations coalesce by physical source line. Browser-only startup/DOM branches are verified separately, **not claimed as native or WASM line coverage**.

| Native crate | Covered / executable | Percent |
| --- | --- | --- |
| stackpress-framework-admin | 1878 / 2081 | 90.25% |
| stackpress-framework-ui | 4922 / 5207 | 94.53% |
| stackpress-example-ui-gallery-shared | 664 / 740 | 89.73% |
| stackpress-example-ui-gallery-web | 244 / 259 | 94.21% |
| stackpress-example-ui-stream-shared | 35 / 35 | 100.00% |
| stackpress-example-ui-stream-web | 272 / 277 | 98.19% |
| generated-admin-shared | 3424 / 4288 | 79.85% |
| generated-admin-web | 5376 / 6134 | 87.64% |
| stackpress-example-review-preview | 780 / 838 | 93.08% |

All nine measured native scopes exceed **70%**. `review/coverage-production.json`, raw/LCOV exports and retained profdata disclose every measured file, test exclusion and uncovered line. No branch-coverage, installed-target or publication claim is made. Instrumented ordinary-schema native hosts are paired with matching ordinary-schema WASM during coverage; the actual Full-name review bundles are backed up/restored in finally.

Fresh final coverage remeasurement followed the three test-verified rounds after a recorded cache clear. Final relevant matrices: gallery **4**, extracted collections **4**, breadth **6** (including both early-select cases), native progressive stream **6**, status/CSP **16**, ordinary generated roles **24**, review label/denial/roles **8**, composed review streaming **6**, review CRUD/preferences/format smoke **3**. Instrumented coverage replays the relevant matrices; final owner-label proof uses the actual review variants separately. Exact argv, outcomes and elapsed times are in `review/*-result.json` and corresponding logs. Watchdogs cap every long child; contexts, workers, native pipes and processes close/reap in finally.

## Benchmark records and counters

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`. Every M4 Cargo/proof/clean command is captured by its outer recorder attempt once; nested Cargo commands are not double-recorded. Exact labels, commands, cache/scope, metadata, terminal statuses and logs are in `runs/<ID>.json` / `.log`.

Global attempts **15 → 41 (+26)**; M4-owned **26** (19 completed passed, 6 failed development attempts, one final metadata validation in progress at write time); other new attempts **0**. Observed-compilation attempts **3 → 3**; successful declared cold workspace rebuilds **0 → 0**. Summed completed outer seconds **2236.735913 → 5266.012403**; recorder Cargo stages **3 → 3**, stage seconds **111.86 → 111.86**.

Kind: build 5→9, check 1→2, clean 0→3, clippy 1→2, coverage 0→1, test 8→24.

Scope: other 0→9, selected 11→23, workspace 4→9.

Cache: cold 3→6, unknown 1→10, warm 11→25. The final metadata-validation record is included in the attempt counts below; its terminal status/duration finalize when its writer exits. Outer Context proofs capture Cargo output in their own logs, so the recorder's null Cargo-stage timing/unchanged observed-compilation count does **not** mean no compilation happened. Inner command durations and compiler logs remain retained.

| Record ID | Kind / scope / cache | Outcome at write time | Phase |
| --- | --- | --- | --- |
| `20261028T060305.965731Z-d6ce9d98e192` | test / other / unknown | passed | step 0: page-only and partial native selection, retained M3 hosts |
| `20261028T060814.828298Z-0ea77be1c34e` | build / selected / cold | passed | initial matching gallery and stream assets, review-only label variants; retained M3 preferences and format |
| `20261028T061104.871648Z-45612623e762` | test / other / unknown | failed | gallery safety, extracted collection presence, breadth and strict-CSP stream browser matrix |
| `20261028T061120.942590Z-c20ef0ca3c76` | test / other / unknown | failed | browser matrix retry after quoting numeric corpus selector |
| `20261028T061218.253118Z-3f6206aed568` | test / other / unknown | passed | collection and remaining browser proofs after scoping native FormData to its original form |
| `20261028T061350.008787Z-b8b761cfe144` | test / other / unknown | failed | composed-gateway progressive stream, review label, role lists, denial and smoke browser proofs |
| `20261028T061412.792524Z-c2803d5c5788` | test / selected / warm | failed | initial native tests for changed scopes and streaming/admin integration |
| `20261028T061503.044900Z-cb1402dbe904` | test / other / unknown | passed | refreshed stream bootstrap definitive failure text and composed review browser retry |
| `20261028T061620.346217Z-97e4169b6c39` | test / selected / warm | passed | native test retry with an explicitly admitted list field in the deny-by-default fixture |
| `20261028T061749.957513Z-8d6ec40fdfc7` | test / selected / warm | passed | round 1: logic and branch-risk review, exact lists, replay, stream framing and recovery |
| `20261028T061900.927497Z-492c0b8437bb` | build / selected / warm | passed | round 2: fixture ownership refactor, fresh gallery and ordinary hostile-label admin variants |
| `20261028T062053.788108Z-8f83dbc1c757` | test / workspace / warm | failed | round 2: workspace except TCP listener tests, doctests and final gallery/ordinary-admin browser regressions |
| `20261028T062740.788005Z-6e2486ea3dc9` | clean / workspace / warm | passed | release 6990196 KiB cache before round-three compilation; copied browser artifacts retained |
| `20261028T062924.147809Z-766b86a6f9fc` | test / selected / cold | passed | round 2 follow-up: clean shared dependency at schema boundaries, prove all ordinary hostile-label variants |
| `20261028T063412.216206Z-91737c4dbc8b` | clippy / workspace / warm | passed | round 3: consistency, comments, fmt, workspace/review/WASM Clippy and dependency boundaries |
| `20261028T063704.804607Z-d766eef1e347` | test / selected / warm | passed | round 3: final native contracts, CLI refusals, warning-denying Rustdoc and fmt |
| `20261028T063930.899940Z-fe14d2150190` | build / selected / warm | passed | final review-only Full name native and WASM bundles after consistency round and shared-cache fix |
| `20261028T064105.720168Z-ce238bc1945d` | test / other / unknown | passed | final review labels, role lists, HTML denial, progressive recovery and composed smoke browsers |
| `20261028T064109.461324Z-cc12ea58211b` | test / selected / warm | failed | warm review-to-normal generated-label isolation with all hostile-label and plugin browser cases |
| `20261028T064427.916275Z-5f30ae3c3316` | test / selected / warm | passed | warm label isolation retry with explicit native and WASM shared/client cleanup |
| `20261028T064920.157073Z-4529c1714494` | build / selected / warm | passed | reverse warm normal-to-review label transition with explicit WASM cleans in final bundle builder |
| `20261028T065128.491410Z-c98b7a939dc3` | test / other / unknown | passed | final reverse-transition review labels, denial, stream recovery and smoke proofs |
| `20261028T065132.294011Z-835cbfe60325` | clean / workspace / warm | passed | clear ordinary compiler output before fresh native coverage; copied review/browser artifacts retained |
| `20261028T065407.298362Z-4d29eea1de06` | coverage / selected / cold | passed | fresh native coverage of nine scopes with instrumented finite hosts and full relevant browser matrices |
| `20261028T070317.577339Z-67aa06bd8e0f` | clean / workspace / warm | passed | final owned target cleanup, restored owner-artifact hashes and filtered fixture-process/scratch audit |
| `20261028T070436.457121Z-9921badbf4bc` | check / other / unknown | running | final records, source checks and KB validation |

## Artifacts, cleanup and re-review

All M4 evidence is under `/private/tmp/s8-polish-m4/`: `gallery/`, `stream/`, `roles/`, matching `bundle/{disabled,plugin}/`, `review/`, coverage/provenance, `benchmark-before.json` and `benchmark-after-at-write.json`. Private bounded drivers there reproduce each named phase; standard review.py build/bundles and the tracked browser proofs include the permanent changes. Normal and instrumented assets remain separated and the owner bundles are restored.

Deleted only the maintained `/private/tmp/s8-polish-m4/target` at **2,829,136 KiB** after coverage. Its largest observed size was **6,990,196 KiB**, below the **8,388,608 KiB** cap; `cleanup.json` records removal, no remaining fixture processes and no matching consumer scratch tree. **Exception:** the full workspace run invoked the existing `independent_consumers_compile_and_remove_stale_runtime_outputs`, which creates its own temporary consumer target. It completed successfully and its TempDir cleanup was checked; this is disclosed rather than claiming one target throughout. Subsequent selected runs do not execute it. TCP listener watchdogs were explicitly filtered; the coordinator's earlier full baseline covered them. No unrelated process/service/target was touched.

Inspected current screenshots (relative to that M4 evidence root):

- `gallery/screenshots/root-hydrate-375.png`
- `gallery/screenshots/root-hydrate-safety-375.png`
- `gallery/screenshots/mounted-csr-safety-1280.png`
- `review/review-stream-late-root-375.png`
- `review/review-stream-stream-root-375.png`
- `review/review-stream-pending-mounted-375.png`
- `review/disabled-hydrate-root-roles-1280.png`
- `review/disabled-hydrate-root-project-lists-375.png`

Consolidated owner re-review: **001, 003, 004, 005, 009, 010, 012 and 013**, following the updated walkthrough's steps and expected results. It includes inline actions/plain chips, table overflow/selection, stars/0/clear, HTML denial, readable role lists, ordinary versus Safety samples, removed raw nested demo, Full-name/plugin labels and stream recovery. Installed 011/Part B and G39 remain separate.

Final source/KB validation is executed after this record is written; successful completion of the enclosing benchmark proves Node/Python syntax, whitespace, scope invariants and byte-identical KB diagnostics against **146 existing errors / 25 warnings**. Its exact terminal outcome is in the final metadata record above. No owner visual acceptance or commit is implied by these proofs. The coordinator can review this diff and rebuild its managed preview next.
