# Slice 5 lane B: refactor rounds 1–4

Task 00008; branch `trial/3-s5-lane-b-ui`; starting/current commit `307074d`. This records the coordinator-released [§5.4 rounds](../../workflows/coordinated-crate-development.md) using the [refactoring review checklist and receipt template](../../references/00037-refactoring-round-checklists.md), after the coordinator verified M3 and the owner accepted the UI. It supplements the [M3 receipt](slice-5-receipt-lane-b-m3.md), not its historical pending-on-merge status.

**Rounds complete; ready for coordinator re-verification.** Two confirmed defects were fixed with failing regressions. Other edits preserve behavior. No public signatures, exports, serialized shapes, features, dependencies or build ID changed. No commit/staging, agent message, install, dev listener, host network toggle or source-of-truth promotion occurred. Lane A's files, the root manifest/lock/facade and the shared customization source have no diff.

## Baseline and scope

Reviewed framework-ui components/state/language and examples/profile-ui shared/client/contracts/native, including their owned proofs. Baseline native tests/doctests passed in `baseline.log`. The existing M3 browser/WKWebView, cancellation/poison/watchdog and language/state tests were retained. Scope includes application behavior and fixture ownership; unrelated renderer/routes/web/gateway changes remain lane A's responsibility.

All evidence paths below use prefix `/private/tmp/stackpress-s5-b-rounds-`. Rustc 1.96.0; aarch64-apple-darwin; installed wasm32 target on stable 1.96; wasm-bindgen 0.2.129; Tauri CLI 2.12.4 / SDK 2.12.5; cargo-llvm-cov 0.8.6. `tools.json` records actual versions. Every Cargo phase used `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0` and one of the same two isolated target paths.

## Round 1 — logic and complexity

**Confirmed defect:** SearchReply admitted empty or duplicate row IDs. A duplicate violates the keyed List's explicit unique-key requirement; an empty identity cannot identify a usable detail. Such a response could pass the shared codec and enter ambiguous rendering/navigation state.

Wrote `replies_reject_ambiguous_row_identity` first; its baseline run fails (`r1-regression-before.log`). Validation now checks nonempty, unique IDs with a bounded set, alongside existing revision/count/field limits. Empty result collections remain valid. The new test covers direct validation, shared HTTP/IPC decoding and native envelope projection. A controller regression confirms an invalid acquired reply leaves prior useful rows/total intact, publishes no new success, clears pending and exposes Protocol failure.

Other reviewed decisions: input validation before admission; generation advance before cancellation; invalid-submit revocation; one accepted completion per generation; disposed callbacks; exact detail matching; independent return prefix; bounded history; count grammar; duplicate notification policy and key exhaustion; checked host/transport/protocol categories. Existing guard ordering and exhaustive matches were already clear; no cosmetic rewrites were made.

Result: affected five-package SSR tests and doctests pass in `r1.log`, including both regressions. No wire shape or public signature changed. This validation tightening is a defect fix, not a claim that malformed identities were a supported input contract.

## Round 2 — responsibility and abstraction

The native window navigation callback mixed production origin policy with proof-protocol parsing, receipt mutation, audit acknowledgement and fixture exit coordination. Moved that whole cohesive fixture operation to `Proof::record_navigation`; receipt fields are now private to Proof. The host callback now visibly consumes an explicitly selected fixture report or applies the same packaged-origin gate. The ordinary startup path still installs no proof service/listener, and no extra IPC command exists.

Reviewed the other boundaries without unnecessary splitting: contracts own checked public DTOs/decoding, native host owns admission/lifecycle/disposal, shared controller owns publication decisions, shared navigation owns checked history, locale owns the example dictionary, and browser modules adapt DOM/fetch/IPC only. The main Profile component remains coherent application composition; generic controls/state/language retain framework ownership.

Result: `r2.log` passes native tests/doctests. The actual rebuilt app passes all three modes in `launch-r2-{navigation,window-close,app-quit}.{log,json}`, testing the extracted acknowledgement and exit paths through WKWebView, not only a library mock.

## Round 3 — consistency, Rustdoc and comments

Removed the pre-merge `#[rustfmt::skip]` and stale absent-customization-source comment now that M4's real shared source exists. Added precise private state/field documentation, navigation snapshot input-versus-successful-prefix semantics, callback lease teardown ordering, the suspended-stack drop contract, and concise DOM/IPC wrapper documentation. Corrected the SearchPage description to include detail/return context. Meaningful Rust functions/methods, including both previously undocumented Drop implementations, now have purpose comments. Existing names, escaping and control behavior remain unchanged.

Result: affected tests/doctests pass in `r3.log`; warning-clean Rustdoc passes in `r3-doc.log`. Final workspace Rustdoc, rustfmt and JavaScript/Python syntax checks also pass. Formatting did not change lane A's shared customization source.

## Round 4 — safety and soundness

The list below records reviewed obligations and actual evidence; coverage percentages alone do not establish these properties.

| Obligation | Review and proving evidence |
| --- | --- |
| IPC trust / ACL / capabilities | Native window identity and URL metadata are authoritative; DTOs cannot select a window/event/service. Main-only local capability grants exactly search/cancel, with no shell/filesystem/generic emit grant. Packaged ungranted-window, unknown-command and malformed/forged/oversized JSON cases pass. |
| CSP / local transport | Same explicit packaged/self and IPC allowlist; external image/WebSocket probes are blocked before acquisition. Nine final launches record zero non-IPC data fetches and one separate packaged WASM fetch. No HTTP fallback exists in CSR. |
| Admission / cancel / deadline / shutdown | Fixed operation, bounded ID/JSON/capacity, duplicate/closed rejection, window-qualified cancel and cancellation-first selection remain. Existing entry barriers, actual stack-drop counters, eight-call shutdown, real five-second deadline and all packaged close/quit modes pass with final registry count zero. |
| Poison | Operational registry APIs fail closed; Ticket cleanup removes its own entry even on unwind without clearing poison. Poison retention, disposal and acyclic ownership remain tested. No new lock or recovery policy was introduced. |
| No locks/RefCells across await | Registry and proof-receipt guards end synchronously; shutdown registers notification before map inspection. Borrowed watch values end at the statement. Browser StoredValue updates end before local async acquisition; test RefCells do not cross await. No owned unsafe code or unsafe Send/Sync assertion was added. |
| Late completion / disposal | Generation checks precede signal access, invalidation precedes adapter cancel, disposed callbacks no-op, repeated completion is ignored. Native controller tests and the browser's forced late-success A-after-B case pass. |
| Keyed row freshness | Existing memo reads current label/href for retained keys; browser navigation crosses successful prefixes and requires the updated independent return query. The round-1 identity guard now prevents ambiguous application keys. |
| Plural / interpolation | en/fr 0/1/2 and regional cases, reactive locale/count changes, exact whitespace/falsy values and text/node slots remain tested. Dynamic strings use escaped Leptos text, not raw HTML; hostile-seed browser and detail tests pass. |
| Revision / shape / budgets | Strict DTO unknown-field/revision checks, checked asset/snapshot build/base/status/page agreement and byte/row/phrase/notification budgets remain. Browser seed/build drift, missing assets and explicit recovery pass without initial refetch or reload loops. |
| Send/Sync | Added compile-time bounds for NativeHost and NativeCallContext. Tauri all-target builds enforce native command future requirements. The real registered handler and PageSpec negative compiler proofs reject Rc retained across await; logs `r4-{compiler,page-compiler}.log` and their detailed non-send logs. WASM callbacks remain local. |
| Error disclosure / lifecycle | Fixed safe native tokens, shared five-field public host errors, and separate acquisition/protocol categories remain. The ten-case real CLI/HTTP/native table proves Abort, raw nesting, fresh replacement/recovery/fallback and suppression/foreign finalization without private/partial data. Packaged host error/retry and denial UI pass. |
| Resource ownership | No detached domain task; registry stores cancellation metadata only; handler/request/response lifetimes end with the borrowing future. Root cleanup drops listener leases before callbacks; callbacks retain signal handles rather than an Owner. All final native process checks report zero lingering app executables. |

Result: affected native tests/doctests pass in `r4.log`; real compiler rejections pass, including the PageSpec proof. These reviewed paths were already sound for the accepted execution model; no ceremonial changes were introduced.

### Follow-up to rounds 2 and 4: named plugin selection defect

**Confirmed defect:** the feature-on native helper called `customization::register` directly. Visible `Demo:` names were correct, but the selected plugin list contained only `profiles`; web selected the named `profile-ui-labels` plugin after it. This contradicted the already accepted cross-adapter customization contract.

Added a watchdog-covered registry assertion before fixing it. `r4-plugin-before.log` records the actual failing comparison: left `["profiles"]`, expected `["profiles", "profile-ui-labels"]`. Native now uses `builder.plugin("profile-ui-labels", customization::register)` after generated registration, matching web. Feature off asserts only `profiles`; feature on asserts both exact IDs in order. No public API or shared plugin source changed; visible data remains the same.

Targeted repeat: both native app feature sets pass tests/doctests, all-target Clippy `-D warnings` and explicit +1.96.0 checks in `r4-plugin-{off,on}-{tests,clippy,msrv}.log`. The final ordinary app and both instrumented configurations were rebuilt. All final feature-off and feature-on packaged modes pass; customized receipts contain `customization: true` and exact `Demo: ` names. No material finding remains in this repeated responsibility/safety pass.

### Execution-model probe and residual limit

A bounded exploratory probe attempted simultaneous writers against one notification root. Even with Owner context installed and a freshly cleaned target, Leptos's underlying nonblocking signal read guards report contention through the same diagnostic as a disposed signal. Installed reactive_graph/guardian source confirms `Plain::try_new` uses a try-read guard. This does not establish that the application disposes live state or that a concurrent queue is implemented.

The accepted UI uses synchronous actions and marshals completions into its owning root; concurrently rendered server roots are separate and remain tested. Documented that existing boundary in state.rs. Did not expand the frozen API into a cross-thread queue or add a mutex/retry scheme. The exploratory test is not part of the green supported suite; its source and failure evidence remain in `notification-concurrency-probe.rs` and `r4-regression-before.log`. Arbitrary concurrent writers to the same UI signal root are not claimed as supported behavior.

## Final verification

| Gate | Result / evidence suffix |
| --- | --- |
| Format / patch | `cargo fmt --all -- --check`, standalone parity formatting, working/staged diff checks; pass. Staged set is empty. |
| Full workspace tests | `cargo test --workspace --all-targets --locked --features stackpress-example-profile-ui-shared/ssr`; 435 Cargo test cases pass plus the native watchdog child execution; `workspace-tests.log`. Follow-up native suites above pass after the feature-on fix. |
| Full workspace doctests | Same SSR feature with `--workspace --doc --locked`; 855 pass; `workspace-doctests.log`; follow-up native doctests also pass. |
| Full workspace Clippy | Same SSR/all-targets scope, `-D warnings`; `workspace-clippy.log`; native app off/on follow-up Clippy also passes. |
| Explicit compiler | `cargo +1.96.0 check --workspace --all-targets --locked` with shared/ssr; `workspace-msrv.log`; native app off/on follow-up checks also pass. |
| Rustdoc | Workspace/SSR/no dependencies, `RUSTDOCFLAGS='-D warnings'`; `workspace-doc.log`. |
| Wasm modes | Clean hydrate/CSR builds and their target-specific Clippy `-D warnings`; `hydrate.log`, `csr.log`, `hydrate-clippy.log`, `csr-clippy.log`. |
| Client dependency proof | Actual wasm32 `--edges normal,no-proc-macro` trees exclude gateway/server/CLI/Idea-transformer/Tauri/Tokio/SQL/Hyper/SHA runtime edges; `{hydrate,csr}-tree.txt`. Existing build-time proc-macro dependencies are not runtime claims. |
| Assets | Both current manifests match actual rebuilt bytes/current contracts; two asset drift/symlink tests pass; `assets-tests.log`. |
| Browsers | All seven pass: browser.cjs root and `/demo/v1`; browser-m2.cjs; browser-navigation.cjs root and `/demo/v1`; browser-gateway.cjs `/demo` and `/demo/v1`. Logs `browser-{root,mounted,m2,navigation-root,navigation-mounted,gateway,gateway-mounted}.log`. |
| G12 / compiler proofs | Ten real-adapter cases pass in `parity.log`; handler/PageSpec non-Send compile rejections above pass. |
| Ordinary packaged app | `cargo tauri build --debug --bundles app --features app --no-sign --ci`; 25.10 MiB; `package.log`; all three modes pass in `launch-packaged-{navigation,window-close,app-quit}.{log,json}`. |
| Instrumented apps | Both off and `app,demo-customization` builds pass all three modes; `package-coverage.log`, `package-customized.log`, `launch-{coverage,customized}-{navigation,window-close,app-quit}.{log,json}`. |

No listener was started: Chromium uses request interception/pipes; gateway proof requests use its finite no-port probe. App launchers kill/reap after 25 seconds, compiler/parity tests are finite, and Chromium closes in finally. A process check using actual executable names after all launches finds zero remaining rounds app processes. An earlier argument-substring check matched llvm-cov's app-object argument; it was not a running GUI app.

Final ordinary unsigned debug app: `/private/tmp/stackpress-s5-b-rounds-final.app`. Additional instrumented customization evidence: `/private/tmp/stackpress-s5-b-rounds-customized.app`. Their inherited bundle filename/product identifier uses M0; they contain this final source and fresh CSR assets. They are not signed releases or other-platform proofs. Existing owner visual acceptance remains with the coordinator; no new design acceptance is claimed.

Screenshots produced by the browser proof are `ui-375.png` and `ui-1280.png` under the same log prefix, without overwriting lane A's evidence. Interaction/geometry proof includes CSS zoom 1/2; it does not claim a separate OS zoom test.

## Per-crate coverage

Tool: stable LLVM instrumentation/export via installed cargo-llvm-cov/LLVM tooling. Features: real view/server, UI/shared SSR, default web/gateway, native library and actual native app off/on; target aarch64-apple-darwin. Scoped tests, adapter parity, the finite web/gateway browser runs and real packaged runs contribute actual profiles. The exporter includes application examples, shared route/customization source, the instrumented native mains and the parity executable.

| Crate / measured production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 703 / 709 | 99.15% |
| framework-view | 535 / 547 | 97.81% |
| contracts | 300 / 311 | 96.46% |
| shared | 430 / 466 | 92.27% |
| client, native seed/codec scope | 55 / 62 | 88.71% |
| native, library and app off/on | 467 / 501 | 93.21% |
| web, route contract and shared customization | 389 / 437 | 89.02% |
| gateway, measured library/probe scope | 515 / 569 | 90.51% |

Every changed crate exceeds the 70% development floor; no workspace-average substitution. Whole-slice publication preparation at 90% is not claimed. Dependencies, dedicated tests and non-Rust tooling are not production totals. The native opt-in proof module remains counted. Browser-only client/browser.rs and thin DOM/IPC adapters remain outside native numeric coverage under the coordinator's approved M0 condition and are exercised by real Chromium/WKWebView proofs. No WASM line-coverage, mobile or other desktop claim is made.

LLVM reports 57 mismatch records: a dump audit identifies 13 distinct third-party symbols from Tokio, tachys and brotli, with no first-party mismatch. Full diagnostics are retained; no warning suppression or new production exclusion was added. `coverage.json` is the full export, `coverage-summary.txt` has the per-crate totals, and `coverage.profdata` retains merged data.

Reproduction: use fresh phases in the two target paths below; run the workspace checks above. For numeric coverage set `RUSTFLAGS='-C instrument-coverage'`, `LLVM_PROFILE_FILE=<evidence>/%m-%p.profraw`, run the eight-package SSR test scope (view, UI, contracts, shared, client, native, web, gateway), build the finite web/probe binaries and execute all seven browser cases. Run instrumented native tests, parity and all three packaged modes off/on. Before cleaning a phase, archive its executables under `<evidence>/debug/deps`; put finite mains in `<evidence>/debug`. Both native app configurations must be retained (the off object uses a `stackpress_example_profile_ui_native_` prefix). Export with `PROFILE_UI_COVERAGE_TARGET=<evidence> PROFILE_UI_COVERAGE_REPORT=<report> python3 examples/profile-ui/proofs/coverage.py`.

## Cleanup and stop

Only `/private/tmp/stackpress-s5-b-rounds-native` and `/private/tmp/stackpress-s5-b-rounds-client` were Cargo target directories. Completed UI feature phases were cleaned before reuse. Both target directories and temporary raw/object archive `rounds-evidence` are now deleted; no other lane's output was touched. Logs, final apps, screenshots, merged profile, full JSON export and probe evidence remain. No GUI process or dev listener remains.

The network proof retains its explicit M3 limit: zero non-IPC **data** fetches plus one local packaged WASM acquisition and blocked external probes; the host network was not disabled, and no OS packet capture or audit of arbitrary future native plugins is claimed.

KB baseline is 146 errors / 21 warnings, chiefly existing sibling-source references and document-size notices. Post-write validation retains exactly the same diagnostics (146 errors / 21 warnings), with no new receipt diagnostic; unrelated KB repair is outside scope. Coordinator owns verification/acceptance/task-state changes, any further scoped findings and commits. This receipt does not claim a crate-doc publication phase or accepted context promotion.
