# Slice 5 lane B M2 integration receipt

Task: [00008](tasks/00008-slice-5-ui.md), under [G25/G23](decisions.md) and the accepted [verification plan](slice-5-design-verification.md). Date: 2026-10-26. Worktree `trial3-s5-lane-b-ui`; branch `trial/3-s5-lane-b-ui`.

**M2 remains incomplete at the route/detail contract boundary.** Real M1 API integration, the selected shared UI inventory, typed host-error decoding, HTTP/IPC error presentation and the late-success browser race are implemented and verified. Detail links, history/refresh/return-query preservation and integrated missing-detail presentation still require the coordinator/lane-A contract correction below. No acceptance control, formal refactoring round or later milestone is claimed complete.

## Integration and ownership

The coordinator saved the pre-integration implementation as `5e51ac3482ceb4b0464c39c1dfe00a74ec2bcb68`. The authorized `git merge --no-commit trial/3` brought in `8705c2bbc0dd3117136893fa570ff3a4a599b1bd`: verified M1 source from `d86c3e7` plus subsequent documentation-only coordinator updates. HEAD remains `5e51ac3`; MERGE_HEAD is `8705c2b`; the merge is intentionally uncommitted. No unmerged index entries remain.

Conflicts in shared/lib.rs, client/browser.rs and proofs/coverage.py were resolved with lane B's implementation plus the real upstream APIs and additive coverage support. Only those conflict resolutions were explicitly staged. Automatically staged upstream changes belong to the authorized merge; no broad staging or commit was performed.

All temporary compatibility code was removed:

- `shared/src/m1_compat.rs` and `shared/src/m1_compat/tests.rs`: replaced by real framework-ui route context and framework-view BasePath.
- `client/src/assets_compat.rs`: replaced by `PageAssets::decode(assets, BUILD)` and checked base/location agreement.
- `withM1Assets` synthetic asset-element injection in both `proofs/browser.cjs` and `proofs/browser-m2.cjs`: removed; proofs now consume the real web host's HTML metadata.
- Corresponding temporary module declarations/imports were removed. The snapshot uses real `status: PublicStatus::ok()` and M1's `BUILD = "profile-ui-m1-1"`.

ProfileSearch reads `framework_ui::routing::use_route_context().base`; hydration provides that context from decoded assets and derives the fixed HTTP endpoint from the checked mount. There is no missing-assets fallback.

No manual change was made to lane A's framework-view, framework-ui render/routing, web host, route-contract, root manifests/lockfile/facade or framework-ui manifest. A final diff against MERGE_HEAD for those paths is empty. Earlier lane-B framework-ui lib.rs work is only the three coordinator-approved documented module declarations. No generated domain, lib/server/HTTP/CLI/Idea implementation or other lane's target was changed by lane B.

## Selected Now inventory

| Accepted surface | Implemented behavior and evidence |
| --- | --- |
| Button, Input, FieldControl | Typed native button/link/disabled/type behavior; controlled text; label/help/error associations; safe href checks; SSR semantics and real keyboard/Enter proof. |
| Alert, Loader, LoaderContainer | Escaped safe status/error text, announced loading and aria-busy; useful previous rows remain visible; native markup and browser transitions pass. |
| Link, List, explicit scoped conditions/iteration | Safe anchors, keyed For rows, escaped labels, visible empty state and Show branches. ProfileDetail has exact found/not-found/return presentation and hostile-text tests; route integration remains pending. |
| Basic layout and scoped theme | LayoutProvider/Blank/Panel and Head/Left/Main/Menu/Right semantic slots; responsive document order, neutral CSS, visible focus, reduced-motion rule; explicit light/dark root state without browser inference or persistence. |
| Immediate scoped notifications | Checked NotificationOptions, bounded store, duplicate coalescing, stable keys, dismiss/clear and root cleanup; no global singleton or timer/cookie behavior. Browser error/dismiss/retry passes. |
| Language/context/provider | Checked serializable/deserializable locale/map/placeholder, scoped replacement, missing-provider diagnostic and dictionary clearing; byte/map limits, concurrent roots and disposal are natively tested. |
| Phrase, tagged-call equivalent, string and Translate | Named phrase/template/text APIs and explicit ViewFn slots; source fallback, exact Unicode/whitespace, custom/repeated placeholders, missing-empty/excess-ignored policy, zero/false/empty preservation, escaped strings and real nodes; no raw HTML/formula execution. |
| Search/controller | Valid/invalid/empty/error/retry/loading/cancel states; invalid input revokes old publication rights; stale, duplicate, disposed and exhausted-generation completions cannot publish. Hydration performs no initial refetch; explicit packaged bootstrap performs one local read. |
| Client boundary | Native-tested seed/reply/settings/asset admission; browser-only module remains a thin DOM/fetch/IPC adapter. Actual wasm dependency trees exclude server/CLI/Idea-transformer/Tauri/Tokio/SQL/driver edges. |

## Shared host-error decoder

The contracts crate owns one bounded decoder used by both HTTP and IPC. A successful SearchReply is distinct from the strict public host error shape `code`, `status`, safe `error`, `errors`, `total: 0`; unknown/private/missing/duplicate fields, invalid status/total/revision and over-budget bodies are rejected. HTTP status must agree with the decoded body; IPC uses the same decoder without an HTTP status. Safe empty-message fallback is explicit.

Typed SearchFailure preserves Host, Transport and Protocol categories. The controller retains rows, displays escaped host text with retry, and exposes a separate transport-failure state. Failed fetch/bridge acquisition does not impersonate a valid host error. NativeHost projects only the five allowed public fields from a failed lifecycle response and returns that envelope through IPC; native causes and HandleReport fields never enter that body.

Native tests pin HTTP/IPC agreement, malformed/private envelopes, safe message policy and controller categories. The browser receives an actual finite-host HTTP 400 revision envelope, displays its safe message and retries successfully. Packaged WKWebView exercises an actual selected proof listener's 422 host error with a private cause, verifies only the safe text appears, and retries through the real operation. A separate ungranted window proves IPC ACL denial produces the transport UI state. The proof listener is selected only when PROFILE_UI_PROOF is set; normal startup retains NativeHost::new().

## Behavioral proofs

- `browser.cjs` at root and `PROFILE_UI_BASE=/demo/v1`: unchanged SSR app DOM, preserved focus, zero initial calls, second typed query, disposal and mismatch recovery pass. Root also checks inert hostile script-looking/Unicode exact roundtrip. Real asset metadata is used.
- `browser-m2.cjs`: keyboard Tab/Shift-Tab/Enter, associated invalid-input error, empty results, real aborted transport, real safe host error/retry, loading with previous rows, cancellation, notification dismiss, locale/title/description/language, single style/meta owner, theme and disposal pass with no browser errors.
- Late-success race: for this case alone the proof makes transport abort noncooperative, fulfills B and then A as successful responses, and waits until both response bodies are consumed in that order. Grace/B remains visible. Normal cancellation and disposal are tested separately with real cancellation. This satisfies the race portion of control 00008-003, not its navigation portion.
- Responsive browser checks pass at 375 and 1280 px with CSS zoom 1 and 2 and no horizontal overflow. Earlier screenshots `/private/tmp/stackpress-s5-b-m2-{375,1280}.png` were visually inspected. CSS zoom is not a separate OS/browser-toolbar zoom proof.
- Unsigned debug macOS ARM64 bundle: `Stackpress Profile M0.app`, 24.73 MiB, at `/private/tmp/stackpress-s5-b-m2i-tauri/debug/bundle/macos/Stackpress Profile M0.app`. Actual packaged launch reports `denied-ok`, `main-ok`, `in_flight: 0`, origin `tauri://localhost` and controls `bootstrap-invalid-empty-search-native-host-error-retry-denied-transport-locale-theme-disposal`.
- The instrumented native app also completed that real WKWebView/IPC proof for app-inclusive coverage. Finite launchers exit or kill/reap after 25 seconds; Chromium closes. No development listener or lingering GUI process is required.
- Networking was not disabled at OS level. Packaged assets/IPC use no local listener, and CSP limits network/resource access. Full networking-disabled owner acceptance and packaged detail navigation remain unproved.

## Verification

All checks below passed with locked dependencies and fresh isolated targets `/private/tmp/stackpress-s5-b-m2i-{native,hydrate,csr,tauri,cov}`. Builds use `CARGO_PROFILE_DEV_DEBUG=0 CARGO_INCREMENTAL=0`; no tools were installed. Main evidence prefix is `/private/tmp/stackpress-s5-b-m2i-`.

| Check | Scope / log suffix |
| --- | --- |
| Formatting and diff whitespace | `cargo fmt --all -- --check`, working and staged `git diff --check` |
| Tests and doctests | framework-view, framework-ui, contracts, shared, client, native, web; shared's own ssr feature; `tests-all.log` and final five-client-test rerun `client-final-tests.log` |
| Clippy `-D warnings` | Same seven packages/all targets/ssr in `clippy.log`; native app/all targets in `app-clippy.log`; actual WASM modes in `hydrate-clippy.log` and `csr-clippy.log` |
| Explicit `cargo +1.96.0 check` | Same native seven-package scope in `msrv.log`; native app/all targets in `app-msrv.log` |
| Rustdoc `-D warnings` | Same seven packages, no dependencies, `doc.log`; public decoder doctest passes |
| Hydrate and CSR builds | `build-client.py hydrate` and `csr`; `hydrate.log`, `csr.log`; actual normal/feature trees in `hydrate-tree.txt`, `csr-tree.txt` |
| Finite web fixture | `web.log`; browser root/mount/M2 in `browser-root.log`, `browser-mounted.log`, `browser.log` |
| Tauri bundle and packaged launch | `cargo tauri build --debug --bundles app --features app --no-sign --ci`; `tauri.log`, `launch.log`, `launch-receipt.json` |

Representative native commands (set the target/debug environment above):

```sh
cargo test --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr
cargo clippy --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr --all-targets -- -D warnings
cargo +1.96.0 check --locked -p stackpress-framework-view -p stackpress-framework-ui -p stackpress-example-profile-ui-contracts -p stackpress-example-profile-ui-shared -p stackpress-example-profile-ui-client -p stackpress-example-profile-ui-native -p stackpress-example-profile-ui-web --features stackpress-example-profile-ui-shared/ssr --all-targets
```

## Per-crate native coverage

Tool: cargo-llvm-cov 0.8.6, rustc 1.96.0, aarch64-apple-darwin. Actual shared ssr tests, production decision/state/render code and instrumented native main are included. Browser-only client/browser.rs remains outside numeric coverage under the coordinator's M0 exception; native seed/codec/state logic is covered and actual browser/IPC behavior is separately proved.

| Crate / measured production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 656 / 662 | 99.09% |
| framework-view, integrated dependency | 512 / 527 | 97.15% |
| contracts | 149 / 149 | 100.00% |
| shared | 238 / 259 | 91.89% |
| client, native decision/codec scope | 38 / 40 | 95.00% |
| native, library and changed app main | 211 / 235 | 89.79% |
| web, unchanged lane-A source | 190 / 271 | 70.11% |

Every crate exceeds the 70% development floor; publication's 90% target is not claimed. Web main was not run as an instrumented browser host in this measurement. Tests, dependencies and tooling are excluded; no production suppression was added. Route-contract source is grouped with web.

Reproduce: scoped seven-package `cargo llvm-cov` with shared/ssr, `--no-report --locked`, CARGO_TARGET_DIR set to the isolated cov target; then `PROFILE_UI_COVERAGE_TARGET=<cov>/llvm-cov-target python3 examples/profile-ui/proofs/coverage.py build-native`; run that instrumented native binary through proofs/launch.py with LLVM_PROFILE_FILE inside the same target; finally export using coverage.py with PROFILE_UI_COVERAGE_REPORT. Logs: `cov.log`, `native-cov-build.log`, `launch-coverage.log`, `launch-coverage-receipt.json`, `coverage-summary.txt`, `coverage.json`.

## Exact remaining dependency and patch request

The merged source does not contain the return-query codec/detail seed described in the coordinator's integration message. This is verified against actual files, not inferred from an earlier receipt:

1. `examples/profile-ui/route-contract.rs` defines search/detail/catalog, search_location(base,id), search_api and decode_parameter. It has no detail location carrying an originating prefix and no return-query decoder.
2. `contracts/src/lib.rs` SearchPage contains only input/reply. `web/src/lib.rs::load_detail` overwrites the input with the exact path ID, then narrows rows. The originating search prefix is lost.
3. The detail PageSpec canonical-location closure emits only the detail pathname, dropping any hypothetical return query. The renderer always calls `<ProfileSearch seed/>`, ignoring snapshot.page/status and initial locale/theme, so even a native HTTP 404 currently displays search presentation rather than ProfileDetail.

**Coordinator/lane-A patch request:** provide one shared route codec with `detail_location(base, detail_id, search_prefix)` and a strict corresponding return-prefix decoder; retain that prefix and the exact detail ID distinctly in the detail seed; preserve the return query in PageSpec's canonical location; render a shared page root selected from checked snapshot.page and feed its seed/status/locale/theme. A concrete seed option is an optional detail context `{ id, search_prefix }` alongside input/reply; agree that serialized addition before changing the owned contracts. The native loader must populate it, and a missing exact match must retain 404 plus the same return destination. Search result/history snapshots must retain the originating prefix and rows. Do not add a second route convention or infer the origin from the exact detail ID.

Lane B has not edited the A-owned route/web files or invented that absent wire contract. Once this correction is supplied, remaining lane-B work is to wire catalog-driven result/detail links and shared page selection, history/back/forward/direct refresh and return-to-search, missing-detail routing, then rerun root/mounted/browser and packaged navigation proofs. The approved French/dark snapshot already passes native admission, and interactive locale/theme changes pass; an actual French/dark initial SSR-to-hydration equality proof still needs the renderer to pass those metadata props. Therefore 00008-003 and the full navigation/initial-seed portions of 00008-004/005 remain open.

## Provenance, findings and gates

This receipt supersedes its pre-integration WIP text saved in `5e51ac3`: old shim instructions, old build/status limitations, old target prefix, old three-crate coverage and old 24.55 MiB bundle size are historical. Earlier logs remain under `/private/tmp/stackpress-s5-b-m2-`; the corrected current measurements use `m2i-`. The first historical coverage attempt missed shared's own ssr feature and was corrected; no failed measurement is represented as passing. Earlier lane-local output cleanup after disk exhaustion did not touch other lanes.

Natural implementation caught invalid submission failing to revoke an older completion and SSR title metadata retaining a locale closure after Owner cleanup; regression tests protect both. During the packaged integration proof an attempted JS bridge fixture proved ineffective and timed out; it was removed entirely and replaced by the real proof-only selected listener. The final ordinary and instrumented packaged runs pass. These are implementation fixes, not the coordinator-gated formal refactor rounds.

Stop before rounds, per the accepted milestone gate. No whole-slice, owner visual/offline acceptance, publication, deployment or KB MCP indexing claim is made. No new commit or push was performed; the authorized merge remains uncommitted.

KB validator baseline is 146 pre-existing errors from unavailable sibling-source links and 20 warnings. Post-update validation must have the identical diagnostics and no receipt-specific error; final validation is recorded at handoff.
