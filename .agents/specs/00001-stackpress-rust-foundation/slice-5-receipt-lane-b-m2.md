# Slice 5 lane B M2 integration receipt

Task: [00008](tasks/00008-slice-5-ui.md), under [G25/G23](decisions.md) and the accepted [verification plan](slice-5-design-verification.md). Date: 2026-10-26. Worktree `trial3-s5-lane-b-ui`; branch `trial/3-s5-lane-b-ui`.

**M2 implementation is complete and ready for coordinator re-verification.** The selected shared UI inventory, real M1 integration, typed host-error decoding, detail/history/refresh/return-query behavior, missing-detail routing, initial French/dark SSR-to-hydration equality and packaged navigation are implemented and verified. Evidence now covers 00008-003 and the M2 navigation/initial-seed portions of 00008-004/005. Owner acceptance, formal refactoring rounds and later milestones are not claimed complete.

## Integration and ownership

The coordinator saved the original implementation as `5e51ac3`. The earlier authorized merge resolved lane-B conflicts against real M1 source (`d86c3e7` plus documentation updates at `8705c2b`); that completed integration was independently verified and merged by the coordinator as **70627b5**, then fast-forwarded into this worktree. Current HEAD remains `70627b5`, with no merge in progress. This correction is uncommitted and unstaged.

The coordinator explicitly assigned the entire previously requested correction to lane B, including the example route codec/web host and additive general framework changes. This patch adds the general PageRoute::captures method and its boundary test; no Profile-specific framework behavior was added. Framework-ui's generic keyed List now updates retained rows' labels/links; its translation layer also owns the locale-aware counted-phrase policy. Example-owned contracts/shared/client/web/proofs implement the remaining behavior. Framework-view policy/server and framework-ui render/routing needed no further changes.

All temporary compatibility code was removed:

- `shared/src/m1_compat.rs` and `shared/src/m1_compat/tests.rs`: replaced by real framework-ui route context and framework-view BasePath.
- `client/src/assets_compat.rs`: replaced by `PageAssets::decode(assets, BUILD)` and checked base/location agreement.
- `withM1Assets` synthetic asset-element injection in both `proofs/browser.cjs` and `proofs/browser-m2.cjs`: removed; proofs now consume the real web host's HTML metadata.
- Corresponding temporary module declarations/imports were removed. Snapshots use real PublicStatus (200 or 404 as appropriate). The approved detail wire-shape addition bumps BUILD to `profile-ui-m2-2`.

ProfileSearch reads `framework_ui::routing::use_route_context().base`; hydration provides that context from decoded assets and derives the fixed HTTP endpoint from the checked mount. There is no missing-assets fallback.

Root manifests/lockfile/facade, framework manifests/lib.rs, generated domain and lib/server/HTTP/CLI/Idea implementation remain unchanged in this correction. Earlier framework-ui lib.rs work was only the three coordinator-approved documented module declarations. No other lane's target was touched, no tools were installed and no agent messages were sent.

## Selected Now inventory

| Accepted surface | Implemented behavior and evidence |
| --- | --- |
| Button, Input, FieldControl | Typed native button/link/disabled/type behavior; controlled text; label/help/error associations; safe href checks; SSR semantics and real keyboard/Enter proof. |
| Alert, Loader, LoaderContainer | Escaped safe status/error text, announced loading and aria-busy; useful previous rows remain visible; native markup and browser transitions pass. |
| Link, List, explicit scoped conditions/iteration | Safe anchors, keyed For rows, escaped labels, visible empty state and Show branches. ProfileDetail has exact found/not-found/return presentation and hostile-text tests; catalog-driven route integration, native 404 selection and return links pass. |
| Basic layout and scoped theme | LayoutProvider/Blank/Panel and Head/Left/Main/Menu/Right semantic slots; responsive document order, neutral CSS, visible focus, reduced-motion rule; explicit light/dark root state without browser inference or persistence. |
| Immediate scoped notifications | Checked NotificationOptions, bounded store, duplicate coalescing, stable keys, dismiss/clear and root cleanup; no global singleton or timer/cookie behavior. Browser error/dismiss/retry passes. |
| Language/context/provider | Checked serializable/deserializable locale/map/placeholder, scoped replacement, missing-provider diagnostic and dictionary clearing; byte/map limits, concurrent roots and disposal are natively tested. |
| Phrase, tagged-call equivalent, string and Translate | Named phrase/template/text APIs and explicit ViewFn slots; source fallback, exact Unicode/whitespace, custom/repeated placeholders, missing-empty/excess-ignored policy, zero/false/empty preservation, escaped strings and real nodes; no raw HTML/formula execution. |
| Search/controller | Valid/invalid/empty/error/retry/loading/cancel states; invalid input revokes old publication rights; stale, duplicate, disposed and exhausted-generation completions cannot publish. Hydration performs no initial refetch; explicit packaged bootstrap performs one local read. |
| Client boundary | Native-tested seed/reply/settings/asset admission; browser-only module remains a thin DOM/fetch/IPC adapter. Actual wasm dependency trees exclude server/CLI/Idea-transformer/Tauri/Tokio/SQL/driver edges. |

## Counted-phrase correction

Coordinator screenshot review identified English/French plural forms being used for one result. This is fixed in the translation layer, not by count-specific UI string edits. Language::plural_category selects integer One/Other per explicit locale: English only 1 is One; French 0 and 1 are One. Regional language keys use their base rule. Other locales use a documented English one/other fallback; fractional/additional cardinal categories are outside this initial policy.

Language::plural_phrase/plural_text and the reactive LanguageContext API share that category selection. Translate accepts an optional One phrase plus reactive count, selecting its source key before dictionary lookup and the existing safe ViewFn-slot interpolation. Ordinary non-counted callers retain their behavior. Both profile count messages use the same engine with singular/plural dictionary forms.

Native tests cover 0/1/2 in en and fr (also regional variants), text and real-node output, reactive count/locale changes, source fallback and the actual profile dictionary. The single-result browser assertions require `1 profile` in English and exactly `1 profil` / `1 profil public affiché.` in French. Packaged ordinary and instrumented proofs include the same singular assertions. Both current 375/1280 screenshots were regenerated and visually checked. Tests/doctests, Clippy -D warnings, explicit 1.96.0 check, Rustdoc, hydrate/CSR builds and WASM lints, all five browser runs, both packaged launches and coverage were rerun after this correction (`plural-tests.log` plus the normal logs below).

## Shared host-error decoder

The contracts crate owns one bounded decoder used by both HTTP and IPC. A successful SearchReply is distinct from the strict public host error shape `code`, `status`, safe `error`, `errors`, `total: 0`; unknown/private/missing/duplicate fields, invalid status/total/revision and over-budget bodies are rejected. HTTP status must agree with the decoded body; IPC uses the same decoder without an HTTP status. Safe empty-message fallback is explicit.

Typed SearchFailure preserves Host, Transport and Protocol categories. The controller retains rows, displays escaped host text with retry, and exposes a separate transport-failure state. Failed fetch/bridge acquisition does not impersonate a valid host error. NativeHost projects only the five allowed public fields from a failed lifecycle response and returns that envelope through IPC; native causes and HandleReport fields never enter that body.

Native tests pin HTTP/IPC agreement, malformed/private envelopes, safe message policy and controller categories. The browser receives an actual finite-host HTTP 400 revision envelope, displays its safe message and retries successfully. Packaged WKWebView exercises an actual selected proof listener's 422 host error with a private cause, verifies only the safe text appears, and retries through the real operation. A separate ungranted window proves IPC ACL denial produces the transport UI state. The proof listener is selected only when PROFILE_UI_PROOF is set; normal startup retains NativeHost::new().

## Behavioral proofs

- `browser.cjs` at root and `PROFILE_UI_BASE=/demo/v1`: unchanged SSR app DOM, preserved focus, zero initial calls, second typed query, disposal and mismatch recovery pass. Root also checks inert hostile script-looking/Unicode exact roundtrip. Real asset metadata is used.
- `browser-m2.cjs`: keyboard Tab/Shift-Tab/Enter, associated invalid-input error, empty results, real aborted transport, real safe host error/retry, loading with previous rows, cancellation, notification dismiss, locale/title/description/language, single style/meta owner, theme and disposal pass with no browser errors.
- Late-success race: for this case alone the proof makes transport abort noncooperative, fulfills B and then A as successful responses, and waits until both response bodies are consumed in that order. Grace/B remains visible. Normal cancellation and disposal are tested separately with real cancellation. This and the navigation proof below cover the requested M2 behavior for 00008-003.
- `browser-navigation.cjs` at root and `/demo/v1`: keyboard detail activation and visible-page focus, successful query transitions with retained row keys, correct independent return prefix, back/forward with restored rows and no history read, direct detail/search refresh, unchanged detail hydration DOM/no initial read, real native 404 and return-to-search to the exact prefix/results all pass. Initial French/dark markup, title, labels and theme remain equal through hydration with no fetch; detail/return retain locale/theme and a single title/description/style/canonical owner. The trusted finite fixture uses PROFILE_UI_DISPLAY=fr-dark to re-render the actual native-loaded public seed, not a second domain search implementation.
- Responsive browser checks pass at 375 and 1280 px with CSS zoom 1 and 2 and no horizontal overflow. Current screenshots `/private/tmp/stackpress-s5-b-m2-{375,1280}.png` were visually inspected after the final browser run. CSS zoom is not a separate OS/browser-toolbar zoom proof.
- Unsigned debug macOS ARM64 bundle: `Stackpress Profile M0.app`, 24.84 MiB, at `/private/tmp/stackpress-s5-b-m2n-package/debug/bundle/macos/Stackpress Profile M0.app`. Actual packaged launch reports `denied-ok`, `main-ok`, `in_flight: 0`, origin `tauri://localhost` and controls `bootstrap-invalid-empty-search-native-host-error-retry-denied-transport-detail-history-return-missing-locale-theme-disposal`.
- The instrumented native app also completed that real WKWebView/IPC proof for app-inclusive coverage. Finite launchers exit or kill/reap after 25 seconds; Chromium closes. No development listener or lingering GUI process is required.
- Networking was not disabled at OS level. Packaged assets/IPC use no local listener, and CSP limits network/resource access. Full networking-disabled owner acceptance remains unproved; packaged detail/history/return/missing-detail behavior passes.

## Verification

All checks below passed with locked dependencies and fresh isolated targets `/private/tmp/stackpress-s5-b-m2n-{native,hydrate,csr,tauri,package,cov,cov-app}`. Builds use `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`; no tools were installed. Main evidence prefix is `/private/tmp/stackpress-s5-b-m2n-`. The `package` target is used only for bundling; app lint/check and instrumented app targets are separate. No listener or development server was started.

| Check | Scope / log suffix |
| --- | --- |
| Formatting and diff whitespace | `cargo fmt --all -- --check`, working and staged `git diff --check` |
| Tests and doctests | framework-view, framework-ui, contracts, shared, client, native, web; shared's own ssr feature; `tests.log` (including new route/page/controller/404/settings tests) |
| Clippy `-D warnings` | Same seven packages/all targets/ssr in `clippy.log`; native app/all targets in `app-clippy.log`; actual WASM modes in `hydrate-clippy.log` and `csr-clippy.log` |
| Explicit `cargo +1.96.0 check` | Same native seven-package scope in `msrv.log`; native app/all targets in `app-msrv.log` |
| Rustdoc `-D warnings` | Same seven packages, no dependencies, `doc.log`; public decoder doctest passes |
| Hydrate and CSR builds | `build-client.py hydrate` and `csr`; `hydrate.log`, `csr.log`; actual normal/feature trees in `hydrate-tree.txt`, `csr-tree.txt` |
| Finite web fixture | `web.log`; browser root/mount/M2 in `browser-root.log`, `browser-mounted.log`, `browser-m2.log`, `browser-navigation-root.log`, `browser-navigation-mounted.log` |
| Tauri bundle and packaged launch | `cargo tauri build --debug --bundles app --features app --no-sign --ci`; `package.log`, `launch.log`, `launch-receipt.json` |

Representative native commands (set the target/debug environment above):

```sh
cargo test --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr
cargo clippy --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr --all-targets -- -D warnings
cargo +1.96.0 check --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr --all-targets
```

## Per-crate native coverage

Tool: cargo-llvm-cov 0.8.6, rustc 1.96.0, aarch64-apple-darwin. Actual shared ssr tests, production decision/state/render code and instrumented native main are included. Browser-only client/browser.rs and its thin DOM/history JavaScript adapter remain outside numeric coverage under the coordinator's M0 exception; native seed/codec/state logic is covered and actual browser/IPC behavior is separately proved.

| Crate / measured production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 703 / 709 | 99.15% |
| framework-view | 535 / 547 | 97.81% |
| contracts | 286 / 297 | 96.30% |
| shared | 429 / 466 | 92.06% |
| client, native decision/codec scope | 55 / 62 | 88.71% |
| native, library and app main | 207 / 235 | 88.09% |
| web, including finite main and route-contract | 369 / 415 | 88.92% |

Every crate exceeds the 70% development floor; publication's 90% target is not claimed. Tests, dependencies and tooling are excluded; no production suppression was added. Route-contract source is grouped with web. The actual instrumented finite web main ran root, detail/not-found, hostile seed, mounted French/dark and typed POST cases. The instrumented app completed the real WKWebView navigation/IPC proof.

Reproduce: scoped seven-package `cargo llvm-cov` with shared/ssr, `--no-report --locked`, CARGO_TARGET_DIR set to the isolated cov target. Build the finite web binary with `PROFILE_UI_COVERAGE_TARGET=<cov>/llvm-cov-target python3 examples/profile-ui/proofs/coverage.py build-web` and exercise it with LLVM_PROFILE_FILE in that target. Build the instrumented app separately with `PROFILE_UI_COVERAGE_TARGET=/private/tmp/stackpress-s5-b-m2n-cov-app` and coverage.py build-native; run its debug/profile-ui through proofs/launch.py with LLVM_PROFILE_FILE pointing into the unit coverage target. Copy that instrumented app object to `<cov>/llvm-cov-target/debug/profile-ui` for the exporter's existing app-object selection; this combines independent target profiles without mixing build features. Finally export using coverage.py with PROFILE_UI_COVERAGE_REPORT. Logs: `cov.log`, `web-cov-build.log`, `app-cov-build.log`, `launch-coverage.log`, `launch-coverage-receipt.json`, `coverage-summary.txt`, `coverage.json`.

## Completed route/detail correction

The previous receipt's exact patch request was accepted and assigned wholly to lane B. All six assigned parts are complete:

1. **One codec:** contracts and web consume the same route-contract.rs source. detail_location(base, detail_id, search_prefix) emits `?return=<encoded prefix>`; decode_return_prefix rejects duplicate/unknown keys, empty/malformed/oversized values and raw plus aliases, preserving encoded Unicode/reserved characters once. A bare legacy detail path uses the explicit demo prefix `abc`, independently of its ID, then canonicalizes with return. PageRoute::captures derives matching from the compiled segments; browser JavaScript contains no path grammar.
2. **Detail seed:** SearchPage has optional DetailContext `{ id, search_prefix }` beside input/reply. BUILD is `profile-ui-m2-2`. The loader reads the exact path ID, stores the independent prefix, projects exact matches and keeps 404 when missing. Constructors/admission validate page/data/status agreement and canonical location.
3. **Canonical:** detail PageSpec uses detail_location, retaining the return query for encoded root/mounted paths. Search construction, forms, links, HTTP endpoint selection and browser navigation use the same codec/catalog.
4. **Page root:** ProfilePage consumes checked snapshot.page plus seed/status/locale/theme. Unknown/mismatched identities or statuses produce the safe invalid-page state. Native 404 renders ProfileDetail not-found with its return route. Explicit settings are passed to the same root on SSR and hydration.
5. **Navigation:** natively tested PageNavigation owns query/detail decisions, publication, bounded (64-entry, root-local public-only) history and restoration. Browser adapters only acquire link/pop events, write checked history/canonical URLs and move focus on page transitions. Navigation revokes old publication rights and cancels transport. History restores settled prefix/rows; an uncached direct-detail return performs the selected prefix read. UI locale/theme remain scoped across navigation. Missing/detail loading and safe errors/retry are visible.
6. **Proofs:** the final five Chromium runs and ordinary/instrumented packaged runs pass as detailed above. No M2 implementation dependency remains. The requested behavior is ready for independent coordinator review; acceptance is not self-awarded.

## Provenance, findings and gates

This receipt supersedes the pre-integration WIP saved in `5e51ac3` and the incomplete integration receipt merged at `70627b5`. Old shim instructions, pending-contract statements, build/status limitations, measurements and bundle sizes are historical. Earlier logs remain under `/private/tmp/stackpress-s5-b-m2-` and `m2i-`; current measurements use `m2n-`. The first historical coverage attempt missed shared's own ssr feature and was corrected. Earlier lane-local cleanup after disk exhaustion did not touch other lanes.

Natural implementation previously caught invalid submission failing to revoke an older completion and SSR title metadata retaining a locale closure after Owner cleanup; regressions protect both. The earlier ineffective JS bridge fixture was removed in favor of the real proof-only selected listener. In this correction, packaged navigation exposed a retained keyed-row href: after abc456 → abc, Grace's existing key kept its old return prefix. List now retains li identity while projecting current label/href values; the browser regression explicitly crosses those two successful prefixes before keyboard detail activation. The final packaged history/return run also protects this defect. The subsequent coordinator pluralization finding is resolved by the shared category policy and count/node/browser/packaged regressions above.

A diagnostic packaging run overlapped test/lint artifacts and produced a small test executable instead of the app. That result was discarded. The final 24.84 MiB app was rebuilt in the fresh packaging-only target and launched successfully; the instrumented app used another isolated target. All finite failures were exited or killed/reaped and are not represented as passing evidence.

Stop before formal rounds, per the accepted milestone gate. No M3/M4, whole-slice, owner visual/offline acceptance, publication, deployment or KB MCP indexing claim is made. No commit, staging or push was performed in this correction; HEAD remains 70627b5.

KB validator baseline is 146 pre-existing errors from unavailable sibling-source links and 20 warnings. Post-update validation is compared against that exact baseline; final diagnostics are recorded at handoff.
