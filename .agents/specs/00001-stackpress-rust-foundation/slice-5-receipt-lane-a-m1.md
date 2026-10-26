# Slice 5 lane A — M1 implementation receipt

Implementer evidence for [task 00008](tasks/00008-slice-5-ui.md), against the accepted [verification plan](slice-5-design-verification.md). Base: `2c3189fdde48caba37c75d1590462977c510cce0`; worktree `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s5-m0`, branch `trial/3-s5-m0`. Work is uncommitted. Coordinator independent re-verification and integration remain required; no owner visual acceptance or M2–M4 completion is claimed.

## Implemented contract

- `framework-view` now supplies checked `PublicStatus`, `PageContract`, lowerable `PageLimits`, `PublicLocation`, `BasePath`, `PageAssets`, `PageRoute` and `RouteCatalog`. Page snapshots add typed `status`; the asset build ID is `profile-ui-m1-1`. Legacy missing status decodes as OK, while changed asset builds are refused before hydration.
- Native `framework-view/server` is opt-in. `ServerBuilderViewExt::page(PageSpec, renderer)` registers an ordinary GET listener and a Config-phase page-ID claim. Duplicate identities fail `build()` before publication; the ordinary server retains all-matching route semantics. Registration is not a transaction: discard a builder after registration failure.
- `PageLoader` checks its actual Send future at the invocation lifetime. A loader receives the existing request/response/invocation, delegates the generated `profile-search` operation without another lifecycle, and returns owned public data or Abort. `PageRenderer` receives only the public snapshot and returns a bounded complete document.
- Pages skip rendering after loader Abort/failure or finalized output. Rendering preserves the original host pair and normal request/response/error hook behavior. Foundation `html()` selects OK; the adapter explicitly restores the snapshot status afterward, including an actual HTTP 404 detail page.
- Public location defaults to pathname only; any query/path projection is explicit from public loaded data. Headers, cookies, arbitrary query fields, session/config/native services and developer reports are never forwarded implicitly.
- `LeptosRenderer::new` checks immutable contract/assets, provides one meta/root route context and canonical RequestUrl, renders synchronously, drops its Owner, then awaits owned Head buffers. Missing `initialize_ssr()`/Tokio startup remains a typed error. Component panics propagate with Owner cleanup; components are trusted code, not a CPU/allocation sandbox.
- Seed escaping covers `<`, `>`, `&`, U+2028 and U+2029 without changing roundtrip data. Head plain-text titles receive a final escaping formatter because pinned leptos_meta raw-injects SSR title text; attributes use checked/escaped values. Canonical, description and stylesheet tests assert one tag per render.
- Default ceilings remain one MiB logical snapshot and two MiB complete HTML; applications can lower either, including zero, but cannot increase them. Asset metadata has an independent 16-KiB ceiling. Serialization/render failures retain their native cause behind safe public Display.
- The actual app route source is `examples/profile-ui/route-contract.rs`; native registration and real Leptos path-descriptor tests consume those same definitions. Search/detail, mounted assets/canonical/form URLs and the fixed API agree at `/demo/v1` as well as root. Native encoded captures decode exactly once, preserve plus signs, reject malformed/invalid UTF-8/dot aliases, then re-encode for the public canonical location. Query fields cannot replace a detail path capture.
- The fixed POST endpoint strictly decodes its JSON body as `SearchInput` before delegation: required revision/id, unknown/duplicate fields denied, one fixed operation name, 64-KiB DTO ceiling and authoritative input validation. Query fields do not override the typed body. Successful output remains the existing `{revision,rows,total}` wire, including total zero. Generated Profile source and operation remain unchanged.

## Host error wire and coordinator disposition

The coordinator explicitly retained the existing success wire plus safe non-2xx framework errors for M1. After both UI lanes merge, a **named shared-envelope-decoder integration item** must distinguish valid host errors from transport failures/cancellation. No extra error-codec changes were made in lane B contracts.

The emitted ordinary HTTP host error envelope is:

```json
{"code":400,"status":"Bad Request","error":"Page version changed. Refresh to continue.","errors":{},"total":0}
```

`code` is the numeric HTTP/application status, `status` its public label, `error` deliberately public safe text, `errors` public field details (empty object for current failures), and `total` preserves zero. No `results`, wire revision, stack, headers/cookies, HandleReport, frames or native causes are added to this error body. The exact revision-error envelope is pinned by a real HttpAdapter test. Other lifecycle errors retain their safe code/label/message; terminal error-handler fallback is sanitized 500. Adapter output-conversion failure follows the existing separate fixed empty-500 contract. No automatic retry or transport fallback is added.

## Cross-lane changes and dependency ownership

Only the following lane B source files have authorized local compatibility edits; **lane B's own versions win at merge and these local copies are discarded**:

1. `examples/profile-ui/contracts/src/lib.rs`: PublicStatus import plus `status: PublicStatus::ok()` in the snapshot constructor; asset build-ID bump is lane A's assigned metadata responsibility.
2. `examples/profile-ui/shared/src/lib.rs`: form action reads `use_route_context().base`.
3. `examples/profile-ui/client/src/browser.rs`: hydrate decodes `#stackpress-assets` with `PageAssets::decode(BUILD)`, provides the base context, and uses it for the fixed HTTP endpoint; CSR keeps the explicit root default.

No components/state/language declarations or lane B state/native logic were changed. No Lib/server/HTTP/CLI/Idea source changed (G23). Root default facade features remain empty. The new optional native view edge uses existing Lib/server workspace crates; no new external package/version was added. The M0 coordinator-approved `any_spawner = =0.3.0` Tokio edge remains native SSR only, for owned buffered Head assembly; its reason/failing executor log and exception are recorded in [the M0 receipt](slice-5-receipt-m0.md). It does not enter hydrate/CSR graphs.

Shared proof tooling now accepts explicit target/report paths rather than forcing old M0 build directories. The browser harness supports a mounted base. The new `non-send-page.rs` fixture proves actual PageSpec rejection; ordinary ServerBuilder rejection is also rerun. No listener/dev server was started, no tool was installed, no agent was messaged, and no commit/push was performed.

## Verification and evidence

All logs/artifacts below use `/private/tmp/stackpress-s5-a-m1-` unless an exact other path is stated. Reused targets were exactly `native` and `wasm`. Old owned M0 target directories were deleted with their logs retained. The final native and WASM target directories were also removed after all checks completed; logs, JSON coverage/merged profdata, ignored built client assets and the saved `.app` remain. Native workspace verification began after cleaning the reused native target to avoid the known rlib-selection fixture defect; no shared fix was attempted. After coverage, a normal bundle attempt reused instrumented build artifacts and emitted a profile-write warning (`tauri-final.log`); a further native clean and final bundle rebuild removed that warning. Its stray profiles were moved out of the worktree into the evidence directory.

| Check | Result / evidence |
| --- | --- |
| Workspace tests | PASS: `cargo test --workspace --all-targets --locked --offline`, `workspace-tests.log`; deliberate child fixtures remain ignored only as designed and are exercised by watchdog parents. |
| Final affected tests | PASS: native view/server, UI/ssr and web; `final-tests.log`, `encoded-route.log`, `error-contract.log`, and instrumented `coverage-tests.log`. |
| Lifecycle table | PASS: request Abort/failure; loader Abort/failure; render failure with retained cause; response Abort/failure, data/body/Response replacement without rerender; error Abort/failure and one error offer; stopped, foreign Sent, cancelled Attempted and Failed delivery; pre-normalization failure and missing route. One top-level Input stage is asserted. |
| Format / lint | PASS: `cargo fmt --all -- --check`; warning-denied workspace/all-targets Clippy and separate hydrate/CSR WASM Clippy; `final-fmt.log`, `close-clippy.log`, `{hydrate,csr}-clippy.log`. |
| Compiler floor | PASS: `cargo +1.96.0 check --workspace --all-targets --locked --offline`; `close-msrv.log`. |
| Docs | PASS: workspace doctests and `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --locked --offline`; `doctests.log`, `final-doc.log`. BasePath's public example is a real doctest. |
| Generator freshness | PASS: `cargo run -p stackpress-example-profile-codegen --locked --offline -- emit generate --check true`; `freshness-corrected.log`, empty differences. The design's bare long `--check` is rejected by the existing CLI grammar (exit 2); its log is retained as `freshness.log`. |
| WASM builds | PASS: `PROFILE_UI_TARGET=/private/tmp/stackpress-s5-a-m1-wasm python3 examples/profile-ui/build-client.py hydrate` and `csr`; `final-{hydrate,csr}.log`. No modes were combined. |
| Facade | PASS: explicit native SSR and wasm32 hydrate/CSR facade checks; `facade-{ssr,hydrate,csr}.log`. Default facade normal tree remains Lib only. |
| Dependency isolation | PASS: normal/feature trees for browser modes, contracts-only packages and facade; SSR/native app trees retained. Browser modes exclude server, SQL/drivers, CLI, Idea, Tauri and Tokio. View/contracts-only graphs contain serde/JSON dependencies, without Lib/server/Leptos/native services. |
| Send rejection / cleanup | PASS: real ServerBuilder and PageSpec reject Rc across await; `non-send-{handler,page}-result.log` and compiler logs. Tests release renderer registrations, staged reactive Owner state and cancelled native calls; panic/unwind child is deadline bounded. |
| Real Chromium | PASS: `browser.log`, `browser-mounted.log`, `browser-coverage.log`; unchanged SSR DOM/focus, zero initial calls, typed second search, abort/disposal, visible contract mismatch and root hostile-seed inert exact roundtrip; no unexpected console errors. Chromium interception uses pipes, no listener. Mounted proof skips the root-only hostile fixture; hostile escaping is covered by the root proof. |
| macOS app build | PASS: Tauri 2 debug unsigned `.app`, 30.13 MiB; final clean rebuild is `tauri-clean-final.log`. Saved reviewable bundle: `/private/tmp/stackpress-s5-a-m1-app/Stackpress Profile M0.app`. |
| Packaged runtime | PASS: `launch-final.log`, `launch-receipt.json`: `main-ok`, `denied-ok`, `in_flight: 0`, `tauri://localhost`. A prior combined edit/launch permission request was rejected; the narrower prepared launch was approved and passed. Wrapper exits or kills/reaps after 25 seconds. No GUI process is left running. |

Compiler/tools: macOS ARM64, stable rustc 1.96.0 plus named +1.96.0, wasm32-unknown-unknown installed on stable, wasm-bindgen CLI/resolved crate 0.2.129, tauri-cli 2.12.4, cargo-llvm-cov 0.8.6. No installation or dependency-budget expansion occurred.

## Coverage after clean

Native executable-line coverage uses cargo-llvm-cov show-env instrumentation after `cargo clean` plus `cargo llvm-cov clean --workspace` in the one owned native target. Scoped view/server, UI/ssr, contracts, shared/ssr, web and native-library tests run; the real browser also invokes the instrumented web executable. Raw LLVM export avoids cargo-llvm-cov's blanket examples exclusion and includes the production route-contract file in web's denominator.

| Crate / native measured scope | Covered / executable lines | Result |
| --- | --- | --- |
| framework-view / server | 510 / 527 | 96.77% |
| framework-ui / ssr + routing | 152 / 154 | 98.70% |
| example contracts | 64 / 64 | 100.00% |
| example shared / ssr | 91 / 98 | 92.86% |
| example web / library, finite executable and route contract | 253 / 271 | 93.36% |
| unchanged example native / library only | 102 / 120 | 85.00% |

Evidence: `coverage.json`, `coverage-summary.txt`, `coverage-export.log` (no warnings), `coverage-env.txt`, `coverage-tests.log`, `browser-coverage.log`; helper invocation uses `PROFILE_UI_COVERAGE_TARGET` and `PROFILE_UI_COVERAGE_REPORT`. The reused generated Profile file contains only fields/derive declarations and contributes no separate executable source entry in this LLVM export; generator freshness, real native operation/SSR/HTTP/IPC serialization and both WASM builds verify that unchanged source reuse. The root facade has no new executable lines; actual feature/target compilations provide its evidence. No workspace-average substitution is used.

The WASM-only client transport/DOM adapter is excluded from the numeric gate under the coordinator's explicit disposition; it remains thin and is covered by Chromium and packaged WKWebView execution. No numeric browser coverage is claimed. Tauri app/main is unchanged and not included in this M1 native-library denominator; its packaged build/runtime evidence is separate. The app-inclusive native coverage gate belongs to lane B's native milestone, not an invented M1 percentage.

## Implementer refinement passes

1. Logic/complexity: checked early Abort/finalization and finite limits, added native/public status restoration after HTML staging, safe error causes, dot-segment rejection, duplicate parameter rejection and real stage tables. A test initially caught the 404 reset; fixed and reran view/UI/web suites.
2. Responsibility/ownership: separated client-safe policies/routes from the opt-in native adapter; loader lifetime adaptation follows the existing checked Send pattern; renderer owns only public policy and a synchronous factory; HTTP body admission and domain delegation remain separate. Named Config claims avoid a mutable renderer registry. Reran affected tests, graphs, both WASM builds and facade checks.
3. Consistency/Rustdoc/comments: documented executor startup/idempotency, trusted component panic/CPU limits, cause privacy, route/query projection and registration mutation semantics; added a compiling BasePath example. Clippy's nested-condition finding was fixed. Encoded-path review found and fixed double encoding before final verification; Unicode/slash/plus/query-override cases and actual app catalog/Leptos matching pass. Final tests, doctests, format, warning-denied Clippy and Rustdoc pass.

These are the standing implementation refinement passes; formal coordinator-gated slice rounds/acceptance are not claimed.

## Disposition

M1 lane A implementation and required proofs are ready for coordinator independent re-verification. Shared error-envelope decoding remains the explicitly assigned post-merge integration item. Complete reactive search/detail navigation, repeated client Head transitions, responsive/accessibility controls, missing-asset recovery, native admission expansion and gateway/network acquisition are later milestone scopes; no gateway was built. Generated domain source, frozen sibling slices and G23 shared crates remain untouched.

KB validation baseline has 146 pre-existing missing-sibling-link errors and 20 warnings in this isolated worktree; they are unrelated to this receipt and were not repaired. Receipt validation must add no diagnostics. The coordinator owns task/index/decision/status updates and integration; stop here without committing.
