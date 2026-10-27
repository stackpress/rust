# 00013 Lane B closeout receipt

Status: **ready for coordinator review**, with the released proof-isolation safety fix and the limitations below. This is implementation/review evidence, not task acceptance or publication approval. Source is uncommitted. Android runtime remains unverified by the owner's decision; no mobile runtime work was attempted in this closeout.

## Checkout, authority and scope

- Worktree: `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s8-lane-b`, branch `trial/3-s8-lane-b`.
- Started clean at `7767838`; requested `git merge trial/3` fast-forwarded to `960233ac61e08228d2e9560c388cd65c836e0238`. HEAD remains there. No new commit, push, install, listener or consultation of another worktree/trial branch beyond the requested trial/3 merge.
- Reviewed all production source in framework-language; framework-preferences including native-file; ui-preferences shared/client/web; profile-ui contracts/shared/client/web/gateway/native and its fixed native/browser bridges. Used the Rust workflow, quality/style policies and the Rust adaptation of chrisai-coding principles.
- Coordinator explicitly released the compile-time proof-isolation fix after the initial safety stop. All remaining edits preserve product behavior: documentation/private-field comments and optional proof-output/target environment selectors.
- `examples/profile-ui/check.py` and its distinct-target guard are **unchanged**. Native and WASM used separate Cargo subdirectories of one capped root, as clarified by the coordinator.
- No public Rust API, serialized contract, contracts/web/gateway wire shape, product labels/layout/CSS or initial English/light behavior changed. No edits to framework-ui/framework-view/ui-gallery, server, SQL, root manifests or lockfile.

## Safety finding, stop and released correction

The initial blocked receipt is superseded by this completed review record; its counterexample and source provenance are retained here. At baseline `960233a`, app.rs included `mobile_proof` and its setup/navigation/script/monitor/lifecycle integrations using only the iOS/Android target cfg. The iOS constructor read `PROFILE_UI_MOBILE_PROOF` without a debug guard. Android used a runtime `cfg!` early return, which did not establish compile-time exclusion.

Concrete iOS release counterexample, established from source without launching a simulator: build with feature `app`, `target_os="ios"`, and no `debug_assertions`; launch with `PROFILE_UI_MOBILE_PROOF=first`. The constructor accepts `first`, app.rs injects `mobile-proof.js`, and the fixture runs denied-command probes, changes the live input to `UNSAVED-before-failure`, emits `mobile-proof://report`, and writes the fixed diagnostic snapshot. Fixed phase-file controls could subsequently drive preferences/search/history. The original release cfg probe confirmed iOS/unix and absence of debug assertions. This was reachable test automation from a controlled launch environment, **not** a demonstrated remote-origin or arbitrary-command authorization bypass. Work stopped and the finding was reported before the coordinator released its fix.

The subsequent desktop check found the same class of issue: `Proof::from_env`, `PROFILE_UI_PROOF`, `PROFILE_UI_PROOF_MODE`, extra fixture registration/window selection and `launch-proof.js` injection were also release reachable. Both platforms are now corrected:

- `proof` and `mobile_proof` modules have outer and inner `#[cfg(debug_assertions)]` guards; mobile integration also retains its target cfg.
- Every app.rs fixture integration is compile-time gated: factory/selection, setup state, navigation interception, initialization script, monitor, mobile window lifecycle and desktop exit receipt. The diagnostic shutdown-failure field/store are gated too.
- Android's fixed `mobile-proof-mode` selector and iOS's environment read exist only inside the excluded mobile module. The former runtime-only release check was removed.
- Release startup directly constructs the ordinary NativeHost and one main window. Production preference/search commands, authoritative caller checks and shutdown behavior remain intact. Debug bundles retain the existing opt-in proofs.

Before/after verification:

1. New `native/proofs/proof-isolation.py compile` imports the **actual** proof source modules into a dependency-free release metadata fixture with feature `app`. It passed on `aarch64-apple-darwin`, `aarch64-apple-ios-sim`, and `aarch64-linux-android`. Removing a module guard would expose its unresolved application/Tauri imports to this fixture.
2. Actual host `cargo +1.96.0 build --locked --release -p stackpress-example-profile-ui-native --features app --bin profile-ui` passed. Actual release `cargo check` of that crate with feature app passed on both mobile targets. These were compilation checks, not installed-mobile proofs.
3. `strings -a` on the actual linked host release executable found **none** of ten selectors/script markers: `PROFILE_UI_PROOF`, `PROFILE_UI_PROOF_MODE`, `PROFILE_UI_MOBILE_PROOF`, `window.profileProofWindow`, `window.profileProofMode`, `window.profileHangEntered`, `window.mobileProofMode`, `mobile-proof-mode`, `mobile-proof-control`, `PRIVATE_PROOF_CAUSE`.
4. The final instrumented debug executable contained all six expected desktop markers, validating the detector. Mobile markers were absent on that desktop target as expected; no linked mobile-binary strings claim is made.
5. The real release executable retained for the comparison had SHA-256 `a8d91e288c3fa7ccae74c502b97da8dbe2b6c841ffac6b29d6ddc14591fd24d1`, size 12,201,712 bytes. It was removed with build output after verification. Existing debug browser/native-preference/macOS proofs passed as recorded below.

The separately named `+1.96.0` toolchain lacked mobile std; the first mobile fixture attempt failed E0463. The already installed default stable toolchain has the same Rust 1.96.0 compiler and all requested targets, so the successful cross checks used it without installation. The initial iOS check also exposed an absent ignored CSR bundle; copying the already-built CSR assets into `native/gen/csr-bundle` with the existing native bootstrap resolved it. No iOS build CLI, options service, simulator or emulator was used.

## Four distinct whole-crate rounds

1. **Complexity/branch risk:** traced locale precedence/aliases/base/return encoding, preference acknowledgement/confirmation, generation ownership, bounded flash and policy paths; shared search/navigation cancellation; finite gateway startup/limits; native ticket admission/shutdown. The bounded existing guards and state transitions did not justify further restructuring. Re-ran all 11 scoped packages' tests/doctests: 78 passed, none failed/ignored.
2. **Responsibility/abstraction:** reviewed pure locale/preference/policy state versus cookie/file/browser/HTTP/native adapters; shared UI versus execution-host authority, ownership across await/disposal, fixed command surfaces, fixture isolation and duplication. No new abstraction or public API change was justified. Same full scoped tests/doctests passed again (78).
3. **Consistency/Rustdoc/comments:** reviewed every owned source module, error wrappers/public redaction, Rustdoc, state/branch explanations and target cfgs. Added private preference-controller/mobile-fixture field explanations, compile-exclusion docs, and corrected stale light/dark-only PageContract documentation. Proof path selectors remain optional and default-preserving. Scoped and app-feature Clippy with `-D warnings`, warning-clean docs, tests/doctests (78), final release builds/checks and formatting passed.
4. **Safety/soundness:** revisited actual caller authority, target origins, file metadata/bounds, preference publication/disposal, cancellation/shutdown, cookies/flash/policy and the released guard fix. No further established safety bug. Verified with freshly instrumented scoped tests/doctests (78), SQLite tests/doctests (25), native customization tests/doctest (3), real native/HTTP/CLI parity, release exclusion checks and the complete real browser/debug-app proofs below. No ceremonial refactor was added when the review found no justified edit.

Safety details: Tauri `trusted()` obtains the actual WebView label/URL, rejects explicit ports, then validates target-selected bundled origin. Android admits only main + `http://tauri.localhost`; other targets admit only main + `tauri://localhost`. DTO fields cannot supply authority. Navigation uses the same origin restriction; no token/role snapshot is authority. Android's source/check coverage does not upgrade its runtime support status.

The Unix file adapter uses only safe `OpenOptionsExt::custom_flags` constants for `O_NOFOLLOW | O_NONBLOCK`, checks metadata on the opened handle for a regular file and a 1,024-byte cap before reading, then bounds the read to cap+1 and checks again. Writes validate settings/destination, create-new the fixed private 0600 staging file, sync, rename and sync the directory. A late sync error means unconfirmed persistence, not rollback. Host-exclusive private-directory ownership and serialization are explicit preconditions, not cooperative-writer guarantees. Existing corrupt/truncated/oversized/symlink/FIFO tests remain intact.

UI persistence publishes only after a checked acknowledgement (and cookie confirmation where required). Failure retains the current root/content/draft and announces failure; generation/disposal checks revoke stale callbacks. Native settings load before first CSR mount; saved System survives a locale-only update. Flash remains bounded public text, inertly rendered, cleared in the issuing cookie scope, root-deduplicated and best-effort rather than exactly-once; HTML remains no-store. Absent policy and forged/denied operations invoke no protected handler.

## System-theme assessment — no migration made

Framework-view now accepts the host-seeded `system` token; the `SystemThemeStyles` component itself is exported by framework-ui (`components/layout.rs`, `components/system.css`).

- ui-preferences still uses its own `html[data-theme]` shell, `--bg` variables and #fafafa/#171717 palette. Shared SystemThemeStyles targets `.sp-root[data-theme=system]`, `--sp-*` variables and a different palette. It is not a drop-in replacement preserving visible output. Its existing CSS already resolves System before paint; the real matrix passed that behavior.
- profile-ui already uses framework-view/LayoutProvider. Native `start_native()` currently resolves saved System through `matchMedia` to an explicit Light/Dark **before mount**, while retaining the saved mode for persistence. Adding the shared stylesheet would be inert for those explicit seeds. Actually retaining `Theme::System` would add live OS-color tracking and change the current dark-system toggle label/next choice; hydration's existing seed decoder also only admits light/dark. A useful adoption therefore needs a coordinated behavior/seed decision rather than an inert insertion.
- Proposed follow-up only: coordinate shell/seed/effective-theme/toggle semantics, then adopt shared styles with explicit visual/first-paint regressions. Nothing was implemented and no API change is required by this receipt.

## Deferred behavior observation (not a safety finding)

With `PREFERENCES_BASE=/demo`, `PREFERENCES_TEST_POLICY=explicit-fixture`, caller `allowed`, the finite host renders `/demo/preferences` with `href="/demo/private"`, but GET `/demo/private` returns 404 and zero protected calls; GET `/private` returns 200 and one protected call. The protected fixture route is registered at the fixed root while the allowed layout link uses the mount. Default/denied behavior remains fail closed. Making this route mount-aware changes behavior and is left for coordinator review. Reproduction/results: `mounted-policy-observation.json` in the log directory. No public API change was made or silently proposed as accepted.

## Verification and inspected browser evidence

Tools: Rust 1.96.0 (`ac68faa20`), Cargo 1.96.0 (`30a34c682`), cargo-llvm-cov 0.8.6 with the installed toolchain LLVM tools; aarch64-apple-darwin host; macOS 26.6.2 (25G83); Xcode 27.1 (27A9275); Tauri CLI 2.12.4 / crate 2.12.5; Playwright 1.64.0 / Chromium 156.0.8078.4. Playwright was supplied through environment variables, never hardcoded in source.

- **Unchanged** `python3 examples/profile-ui/check.py` and `python3 examples/profile-ui/check.py --sqlite`: passed. Includes generated freshness, SSR/CSR/hydrate, facade/feature/dependency-tree checks, customization reuse, tests/doctests, Clippy, wrapper checks and finite service/runtime checks. Browser graphs did not acquire server/DB/CLI/Idea/Tauri/Tokio dependencies.
- **Unchanged** `examples/profile-ui/proofs/browser.py` memory and `--sqlite`: all eight proofs per backend passed at root and `/demo/v1` (16 total), including gateway/navigation/history. SQLite startup failures left missing/uninstalled databases untouched, followed by explicit temporary setup. No listener.
- ui-preferences `build.py` and `browser.py`: passed hydrate/CSR at both mounts, 375/1280 widths and 100%/200% CSS zoom; dirty EN/FR and remembered refresh, before-paint System/theme, persistence failure/blocked cookies, denied policy, lifecycle 1/1 and one-time hostile-text flash. All four contexts reported no browser errors or unexpected requests.
- `native/proofs/browser-preferences.cjs`: passed both OS light/dark fixtures, first-root preferences, delayed persistence-before-publication, saved System retention and failed-save node/draft/theme preservation. This fixture bridge proves the client boundary, not native filesystem/permission authority.
- `native-parity.py`: existing finite CLI/HTTP/native real-adapter G12 table passed offline, with no listener.
- Real unsigned **debug** macOS bundle: unchanged `proofs/launch.py` passed `navigation`, `window-close`, `app-quit`. Navigation reported `denied-ok/main-ok`, 2 entries/2 drops; close/quit each had 1 admitted call before shutdown and 1 entry/drop. Every result had zero remaining in-flight calls, shutdown_ok=true, `tauri://localhost`, zero non-IPC fetches and CSP image/connect probes blocked.
- New compiler/strings isolation proof passed; actual host release build and both mobile release check-only targets passed. Final `cargo fmt --all -- --check`, `git diff --check`, Python AST and changed JavaScript syntax checks passed. Prior scoped/app Clippy and Rustdoc were warning-clean; no Rust source changed after those runs.

Logs/reports/screenshots: `/private/tmp/s8-lane-b-closeout/` (temporary; not guaranteed across restart). Inspected actual images, not just existence: `browser-memory/mounted-375.png` shows the French profile form/result without clipping; `native-preferences/native-hook-system-light.png` and `...-dark.png` show `UNSAVED-native-hook` and the visible failed-save notice; `ui-preferences/screenshots/mounted-csr-375-zoom2.png` shows the complete whole-word heading and spaced disabled actions, and `root-hydrate-1280-zoom1.png` confirms desktop spacing. These are browser evidence, not mobile screenshots. No iOS simulator/Android emulator was launched; earlier mobile limitations remain unchanged.

## Per-crate native executable-line coverage

Fresh native instrumentation followed `cargo llvm-cov clean --workspace --locked --offline`. The first warm-cache coverage attempt was rejected after inspecting its executables: most had no coverage map. It is **not** the source of these percentages. Fresh compiler artifacts were rebuilt and checked for `__llvm_covmap`; final export merged 453 fresh `.profraw` files and 39 actual instrumented test/host/debug-app objects using llvm-profdata/llvm-cov. `coverage-summary.json`, `coverage.lcov`, object/profile inventories and `export-coverage.py` are in the log directory.

Counts are the union of executable LCOV DA source lines in each crate's native compiled scope across memory/SQLite, native-file, SSR, customization and debug-app runs. Dedicated tests, inline cfg(test) modules, tool fixtures/dependencies and the debug-only proof module files are excluded; production app.rs is included. Shared path-included route/customization modules are counted in the crate into which each was compiled. WASM-only adapters are explicitly outside native numerical scope, not covered by assumption. Doctests passed; no separate stable doctest-coverage claim is made.

| Crate (names abbreviated) | Covered / executable lines | Native coverage | More covered lines for 90% at this denominator |
| --- | ---: | ---: | ---: |
| framework-language | 160 / 168 | 95.24% | 0 |
| framework-preferences, including native-file | 430 / 448 | 95.98% | 0 |
| ui-preferences shared | 92 / 104 | 88.46% | 2 |
| ui-preferences client | no native executable lines | **unmeasured (WASM-only)** | unmeasured |
| ui-preferences web | 229 / 246 | 93.09% | 0 |
| profile-ui contracts | 294 / 304 | 96.71% | 0 |
| profile-ui shared | 470 / 501 | 93.81% | 0 |
| profile-ui client (pure seed scope) | 67 / 73 | 91.78% | 0 |
| profile-ui web | 413 / 441 | 93.65% | 0 |
| profile-ui gateway | 515 / 563 | 91.47% | 0 |
| profile-ui native, including actual app binding | 369 / 436 | 84.63% | 24 |

Every measurable native crate scope exceeds the 70% development floor. There is **no** all-target/all-crate 70% claim: the installed wasm32 compiler fails a tiny `-C instrument-coverage` probe with E0463, missing `profiler_builtins`. No installation was authorized. The WASM-only ui-preferences client and profile-ui browser/JS adapters have real behavioral browser proofs but no numeric WASM coverage. Empty native scope is not 100%.

For the 90% publication floor: exercise ui-preferences shared preference event handlers/DOM draft transport (12 native-uncovered lines, currently 2 needed numerically); add meaningful native Tauri preference-command admission/ack/error integration and remaining shutdown/error/customization cases (67 native-uncovered lines, currently 24 needed). Native-file alone is 98/109; remaining paths include error diagnostics, late failures and bounds, despite its containing crate passing 90%. Existing native percentages above 90% do not remove branch-risk gaps, e.g. gateway stdin main is 0/32, shared event paths and native late-error handling. Publication also needs an authorized instrumentable WASM strategy, applicable release/target integration, and the still-open mobile runtime/visual limitations. This closeout does not establish publication readiness.

## Output, ownership and cleanup

Used only `target/lane-b-closeout/{native,wasm}` plus a small release-evidence copy inside that root. `CARGO_INCREMENTAL=0`, DEV/TEST/RELEASE debug information disabled, jobs=2. A parent watchdog sampled the complete root every 15 seconds and failed on an 8,000,000-KiB cap; maximum sampled usage was 4,902,996 KiB (about 4.68 GiB). Child groups/actions had deadlines and kill/reap on timeout/failure. All final proof/build phases completed without a timeout. No build service/listener was needed for these desktop/cross-check commands.

Earlier removed root output: 538,472 KiB after correcting the initial target layout; release/cross-check caches: 1,491,544 KiB. Workspace coverage-clean also removed first-party artifacts/profiles before fresh instrumentation; these sequential figures are not a claim of unique cumulative allocation. Final removal: capped root 4,865,328 KiB; generated profile-ui/dist 9,144 KiB; generated native/gen/csr-bundle 4,072 KiB. All three are absent afterwards. Installed toolchains/caches, pre-existing platform project text and owner processes were preserved. No Python bytecode directories remain in either example.

The final process audit found no process referencing this capped root/owned debug app or runner. All launchers returned/reaped; no app, build, simulator/emulator/adb or options-service process was started and left running by this closeout. No owner process/server/device was controlled. Source and this receipt remain uncommitted; staging is empty.

KB validation has the unchanged baseline **146 errors / 24 warnings** (existing stale external-TypeScript/example links and other baseline issues); final comparison adds/removes no issue. Only this pre-approved receipt changed under `.agents`. No task/context/decision promotion or KB publishing was performed.

Stop boundary: coordinator review of the safety fix, native-scope coverage limitation, optional theme adoption and mounted fixture route observation. No Android retry, commit, M2/other-lane implementation or publication started.

## Mounted link fix

Coordinator-released follow-up, completed for review on 2026-10-27. The requested `git merge trial/3` fast-forwarded this clean worktree from `dcc3738` to `6949256`; HEAD remains there. No commit/push. This section resolves the deferred mounted fixture-route observation above. The System-theme assessment/adoption remains untouched.

The finite host now registers **both GET and POST** protected routes using the same normalized base as the layout link: `/private` at the root, `/demo/private` with `PREFERENCES_BASE=/demo`. It does not retain an unprefixed protected route under a non-root mount. Existing fresh host-policy/Origin/CSRF checks and handler admission order are unchanged; absent policy still fails closed. No public Rust API, serialized wire-shape, CSS, theme, persistence or other example changes.

Changes are limited to `examples/ui-preferences/web/src/main.rs`, `web/tests/finite.rs`, `browser.cjs`, `README.md`, plus this authorized receipt append. The test helper now accepts the mount explicitly and optionally uses the existing `PREFERENCES_LOGS` output selector, preserving its previous default. No manifest, dependency, lockfile or installation change.

Regression evidence: the new/extended protected-route tests were run against the **unfixed** host first. They reproduced both failures: the mounted route returned 404 instead of the expected policy denial, while an unprefixed allowed `/private` request under `/demo` returned 200 instead of 404. The projection test already passed. Baseline: 1 passed / 2 failed (expected), no timeout. Baseline-only profiles were discarded before fixed-run coverage.

The fixed suite contains five real-adapter integration tests. Its protected decision table runs at root and `/demo`: allowed reads and fully valid mutations return 200 with exactly one protected call; absent policy, denied callers, missing/hostile Origin and forged CSRF return 403 with zero calls. Layout projection emits a protected href only when allowed, while invoking zero handlers. Wrong-mount GET/POST requests return 404 with zero calls even with otherwise valid fixture admission, including `/private` under `/demo` and `/demo/private` at the root. Every finite request retains its kill/reap watchdog, lifecycle `1/1` and no-store assertions.

The browser proof still starts with no policy and verifies disabled controls/default denial; its denied/forged probes now use `${base}/private` rather than accidentally probing an unrelated root path. A separate final phase explicitly selects the host fixture policy and caller, renders a fresh allowed projection, then **clicks the visible anchor**. All four hydrate/CSR × root/`/demo/v1` contexts reached the correct destination, returned HTTP 200, the expected fixed public body and `x-protected-calls: 1`. No browser errors or unexpected requests. Existing dirty-draft/persistence/refresh/theme/flash/geometry/disposal checks still passed.

Verification (installed Rust/Cargo 1.96.0; Playwright 1.64.0 / Chromium 156.0.8078.4 supplied via environment):

- Scoped `cargo test --locked` for ui-preferences shared/client/web: **6 tests passed** (1 shared + 5 adapter), none failed/ignored; applicable doctest harnesses passed with no doctests defined. Fixed baseline and final round-three runs both passed.
- Round 1, logic/branch risk: checked empty/non-root base construction, both methods, matched-route permission failures versus wrong-mount 404, and exact handler-call counts. No additional refactor justified; all 5 adapter tests passed again.
- Round 2, responsibility/abstraction: route registration owns the normalized base; host policy remains authoritative and separate from layout projection. Test fixtures select policy only in the explicit final browser phase. No new shared helper/public API justified; all 5 adapter tests passed again.
- Round 3, consistency/docs/comments/safety: reviewed route/link parity, method/error sequencing, native child cleanup and browser phase isolation; added the local registration explanation and README contract. Final scoped tests passed; scoped all-target Clippy `-D warnings`, Rustdoc `-D warnings`, `cargo fmt --all -- --check`, changed-JS syntax and `git diff --check` passed. No source edits followed this verification.
- Existing `python3 examples/ui-preferences/build.py`: native host plus separate hydrate/CSR WASM builds and bindgen steps passed offline with already installed tools. Existing browser parent wrapper passed with the new link-click assertions (120-second parent, 10-second action/native deadlines); no listener.
- Fresh native coverage of the changed web crate: **230/247 executable lines = 93.12%**, including the new route construction, above 70% development and 90% native publication line floors. Merged 258 successful-run native profiles and 4 verified instrumented objects; dedicated tests/tool fixtures excluded. Shared measured scope remained 92/104 (88.46%); WASM-only client remains numerically unmeasured, not 100%. Original broader publication/target limitations above are not superseded.

Evidence: `/private/tmp/s8-lane-b-mounted-link/` contains baseline/fixed/round logs, build/Clippy/docs/browser reports, `coverage-summary.json`, LCOV and inventories. Inspected `screenshots/mounted-csr-protected-link.png` (visible allowed link) and `screenshots/mounted-hydrate-375-zoom2.png` (whole-word heading and spaced disabled actions). Browser reports record successful navigation to `/private` and `/demo/v1/private`; screenshots were taken before clicking, not claimed as screenshots of a separate protected-page UI.

Cleanup: one capped root `target/lane-b-mounted-link/{native,wasm}`, incremental/debug information disabled, jobs=2; highest observed size **1,341,676 KiB** versus an 8,000,000-KiB cap. Deleted that root and 4,780 KiB of generated browser assets; retained temporary logs/reports/screenshots only. Build/test/browser commands were finite and watchdog-supervised; the only nonzero result within that suite was the intentional regression baseline. Final process audit found no owned build/host/runner process; browser/context cleanup completed. No listeners, mobile runtimes, owner-process control, installs or commits. Only ui-preferences source and this receipt changed; staging remains empty. KB validation issue-set comparison is unchanged from this follow-up's baseline (146 errors / 24 warnings, no added/removed issues). Stopped for coordinator review.
