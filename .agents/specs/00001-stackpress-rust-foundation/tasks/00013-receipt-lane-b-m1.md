# Task 00013: Lane B M1 implementer receipt

2026-10-27. Worktree `trial3-s8-lane-b`, branch `trial/3-s8-lane-b`, base `7797d94925b7f3279cfd0256428724a9c4ef8073`. This is implementation evidence for coordinator review, not owner visual acceptance. M1 only; M2 and later gates are unstarted.

## Delivered behavior

- [Locale registry and action](../../../../crates/framework/framework-language/src/lib.rs): compiled legacy/canonical/alias route prefixes, explicit underscore aliases, exact prefix > canonical cookie > configured default, shared checked location replacement, unchanged encoded mount/query/return data. Unknown or duplicate route prefixes remain for host 404s. Duplicate cookies, including malformed bare-key duplicates, fall back. Total cookie admission is now 4096 bytes for coexistence with theme/flash; preference action input remains 1024 bytes. The original locale-only M0 action/reply shape remains readable.
- [Preferences crate](../../../../crates/framework/framework-preferences/src/lib.rs): typed light/dark/system settings, exact same-origin POST admission, scoped HttpOnly cookies, strict acknowledgements and an exclusive root write slot. A fixed confirmation POST checks that both cookies actually came back before publishing. Confirmation is read-only, emits no cookies and never retries the write. Wrong/missing Origin, invalid fields/settings/path, oversized input, blocked cookies and conflicting acknowledgements fail visibly while retaining acknowledged content/location and unsaved drafts.
- [Public flash](../../../../crates/framework/framework-preferences/src/flash.rs): checked 64-byte id / 512-byte text, encoded JSON/hex/header bounds, five-minute cookie lifetime, same-scope deletion even for malformed/duplicate data, root-ID dedupe through dismissal, and saturation at 64 seen IDs. Hostile text stays escaped text. Cookie consumption is best effort: concurrent roots/tabs may both read before deletion arrives. It is not exactly-once or durable delivery.
- [Host access policy](../../../../crates/framework/framework-preferences/src/policy.rs): bounded safe layout action keys separate from authoritative admission. Absent policy denies protected exposure. Fresh host policy runs before the protected closure; a forged public projection grants nothing. No sign-in, token/role snapshot authority, default allow-all or automatic identity implementation.
- [Finite example](../../../../examples/ui-preferences/README.md): real HttpAdapter, one handle call and one request/response hook pair per valid request; compiled route catalog; shared SSR/hydrate/CSR source; host-applied theme/CSS before paint; exact public seed; no initial preference fetch or browser storage read. English/French lookup uses the existing Language/LanguageContext counted-phrase facility, preserving zero/false/empty/whitespace semantics.

Only host code emits separate Set-Cookie headers. Cookies are app-Path scoped, HttpOnly, SameSite=Lax, one-year preference max age, and Secure on HTTPS; explicit HTTP development configuration omits Secure. Personalized HTML and action responses use no-store. Error JSON stages its denial status after Response::json. Contextual errors retain concrete native causes via Error::new/with_kind/with_cause.

System mode is an exact host-seeded token whose effective colors are resolved by CSS media queries before paint. The finite example owns its shell because the existing framework-view PageContract currently admits light/dark only. No framework-view/UI hook or contract was changed, and this receipt does not claim a framework-view system-theme projector. Installed-app preference files/restart remain M6.

The example date/number strings are fixed translated phrase examples, not a new formatter or full r22n/ICU parity claim. Lane A retains formatter/specialized editing ownership. The opt-in `PREFERENCES_TEST_POLICY=explicit-fixture` is explicitly test-only; deployed apps must supply actual identity, resource authorization and web mutation CSRF enforcement.

## Verification

All logs, browser assets, screenshots and machine receipts are under `/private/tmp/s8-lane-b/`. Every compiled Cargo command uses CARGO_INCREMENTAL=0 and CARGO_PROFILE_DEV_DEBUG=0, with task-owned native `target/s8-native` and WASM `target/s8-wasm`. No listener or installation was used. Existing regression suites create/remove their own fresh disposable consumers; no stale external target or other trial branch was consulted.

| Check | Result / evidence |
| --- | --- |
| Native selected tests and doctests | 15 tests + 1 doctest pass; `final-selected-tests.log`, final visual follow-up `visual-tests.log`. |
| fmt | Pass; `final-fmt.log`, `visual-fmt.log`. |
| Selected Clippy, all targets, -D warnings | Pass; `final-selected-clippy.log`, final `visual-clippy.log`. |
| Rust 1.96.0 selected check, all targets, locked | Pass; `final-msrv.log`, final `visual-msrv.log`. |
| Warning-denied selected Rustdoc | Pass; `final-rustdoc.log`. |
| Actual WASM hydrate/CSR builds and bindgen | Pass; `build.log`, `preferences-{hydrate,csr}-{build,bindgen}.log`. |
| WASM hydrate/CSR Clippy -D warnings | Pass; `wasm-{hydrate,csr}-clippy.log`, final `visual-wasm-{hydrate,csr}.log`. |
| Client and extended facade normal runtime graphs | Both modes pass, no server/DB/CLI/Idea/Tauri/Tokio; `{client,facade}-{hydrate,csr}-tree.log`. Facade target builds pass in `facade-{hydrate,csr}-check.log`. |
| Workspace all-target tests, locked | 549 pass, 75 existing child/platform cases ignored, zero failures; `workspace-tests.log`. |
| Workspace doctests, locked | 884 pass, zero ignored/failures; `workspace-doctests.log`. |
| Workspace Clippy, all targets, locked, -D warnings | Pass; `workspace-clippy.log`. |
| Unchanged Profile UI checker | Pass; `profile-check.log`, `profile-check/summary.json`; handwritten accepted example source/checker unchanged. |
| Final Chromium matrix | 4/4 pass at root and `/demo/v1`, hydrate/CSR; `browser.log`, `browser.json`. |
| Scope/lock/artifact audits | `scope-audit.json`, `lock-delta.json`, `artifacts.json`. |

`verification.json` records commands, exit codes and timings for the broad checks. The coordinator's two later visual fixes affect only this example: responsive whole-word heading size and a half-rem protected-action gap. After them, selected native tests/doctests/MSRV/fmt/Clippy and both WASM Clippy modes passed again (`visual-check.log`), and the rebuilt browser matrix/screenshots passed again. The unchanged Profile check and broad workspace checks are separate passing regression evidence.

Browser evidence covers initial parsed SSR DOM equality, the same focused input node and pre-JavaScript draft; dirty EN/FR switching; consistent title/lang/links/counted phrases; exact encoded query/return preservation; refresh/legacy-cookie selection/explicit-prefix override and Back/Forward; no initial preference acquisition; scoped HttpOnly flags; dark refresh with no light transition; system dark before JS and live CSS light/dark media changes; visible/inert flash, dismissal and refresh consumption; failed Origin, blocked-cookie confirmation and hostile acknowledgement without draft/content/location loss; absent-policy protected controls and denied deep link/forged mutation with zero protected handler calls.

There are 28 final screenshots: 16 at 375/1280 CSS pixels and 100%/200% CSS zoom, plus each mode/mount's visible flash, French dirty dark form and failed save. Geometry asserts no page-wide clipping, every heading word has one text rectangle, and the action gap is 8 CSS pixels. Final mounted CSR 375/200% and root hydrate French/dark screenshots were visually inspected after the coordinator fixes. Earlier versions exposed mid-word heading wrapping and touching actions; both were corrected. CSS zoom is a geometry aid, not every OS magnification mode. Owner acceptance and native screen-reader/WebView/packaged-target inspection remain separate.

The browser intercepts only HTTPS preferences.invalid, blocks other origins, has 10-second action/acquisition/native-child deadlines, and closes contexts/browser in finally. Its Python parent kills/reaps the process group at 120 seconds; timeout is failure. Native adapter tests also kill/reap child processes at 10 seconds. Initial sandbox Chromium startup was denied by macOS; the finite proof then ran outside the sandbox with approval. Final runs report zero unexpected console errors or requests. Expected preference denial HTTP diagnostics are filtered by endpoint; denial status/body and retained state are independently asserted.

Tools: macOS 26.6.2 (25G83), rustc 1.96.0 (ac68faa20), cargo-llvm-cov 0.8.6, wasm-bindgen 0.2.129, Chromium 156.0.8078.4, Playwright 1.64.0, Node v26.2.0, Python 3.11.4. Installed Playwright is supplied through PREFERENCES_PLAYWRIGHT; no machine-specific package path is committed.

## Four test-verified review rounds

1. **Complexity/logic:** reviewed precedence, encoding, acknowledgement publication, write admission and cookie ambiguity. Replaced locale-cookie collection with explicit bounded duplicate detection including bare keys; made completion validate the currently pending request rather than accepting a caller-constructed reply. Native tests/doctests pass (`round1-tests.log`); final WASM modes and browser proof cover the adapted publication path.
2. **Responsibility/abstraction:** separated pure seed/catalog/phrase admission into shared/model.rs; isolated host fixture policy and finite shell assembly; kept browser fetch/history separate from reusable native-tested state and persistence admission. Native tests/doctests pass (`round2-tests.log`); subsequent target checks fixed a missing browser-only BasePath import after the split.
3. **Consistency/Rustdoc/comments:** reviewed public names, facade opt-in, error causes, typed theme wire values, scope flags, state/async comments and platform promises. Added the compiled crate-level write/readback example and explicit finite-example limitations. Tests/doctests and Clippy pass (`round3-tests.log`, `round3-clippy.log`, final mode checks).
4. **Safety/soundness:** reviewed bounded serialization, hostile flash, policy absence/forgery, mutation ordering, owner-local signals, no signal guard across DOM/await, request-local rendering, actual fetch abort/deadline ownership, and uncertain persistence. Fixed escaped flash JSON expansion so issuance cannot exceed its admitted hex/header ceiling, and tightened seed route admission. Tests/doctests pass (`round4-tests.log`). Native coverage and final browser proofs pass. No unsafe code, new unbounded registry or process-global preference state was introduced.

Follow-up visual review: replaced forced heading word breaks with clamp(1.25rem,4vw,2rem), grouped protected actions with an 8-pixel gap, and added whole-word/gap geometry assertions. `visual-check.log`, `browser.json` and final screenshots pass. No further material in-scope finding remained.

## Per-crate coverage

Fresh native instrumentation uses cargo-llvm-cov show-env --no-rustc-wrapper, normal cargo test --locked --all-targets with shared SSR, and the matching Rust LLVM tools. `coverage.py`, `coverage-env.txt`, `coverage-tests.log`, `coverage-objects.json`, `coverage.lcov`, `coverage-summary.json` and `coverage.profdata` retain commands/scope/counters. Explicit export includes all six first-party test/host objects, including the real spawned example binary, rather than relying on the tool's default example-file filtering. Serial LLVM export succeeds with no warning (`coverage-export.log`).

| Native production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-language | 160/168 | 95.24% |
| framework-preferences | 332/339 | 97.94% |
| preferences-shared | 92/104 | 88.46% |
| preferences-web | 224/246 | 91.06% |

Each measured crate exceeds the 70% development floor. Dedicated tests and inline cfg(test) modules are excluded with source-line cutoffs; external dependencies and target-only thin browser adapters are not native production percentages. Client startup is entirely WASM-only, unmeasured natively, not 100% covered. Its seed decoder/controller policy remains natively tested in shared/preferences, and actual Chromium exercises hydration/CSR/fetch/history/DOM behavior. Root facade has no executable implementation lines; its target builds/graphs pass. Publication's separate 90% gate is not claimed.

Remaining native uncovered lines are recorded individually: language Display diagnostics; a few checked helper/error exits and notice id access; UI click closures exercised in Chromium; finite bridge/error/confirmation branches exercised in browser or outside the scoped native fixture set. Line coverage is not a branch/authorization proof; denial and state assertions above provide that evidence.

## Compatibility, cleanup and stop

Lock delta: four new workspace packages, zero new third-party packages, zero removed/upgraded versions. Existing Leptos/bindgen/runtime versions are retained. Only framework-language, new framework-preferences, new ui-preferences, root Cargo/lock/facade and this permitted receipt changed. No edits to UI/view/gallery/Profile source, lib/server/HTTP, CLI/Idea/SQL/schema, context or trackers.

The KB validator baseline has 146 errors and 22 warnings, chiefly missing historical sibling TypeScript sources and existing file/link diagnostics in this isolated checkout (`kb-before.log`). Post-receipt diagnostics are compared in `kb-comparison.json`; this failing baseline is not code verification and is not repaired by this lane.

Cleanup evidence is recorded in `cleanup.json` and the final clean logs. Both task-owned compiled target trees are removed after validation; generated regression child logs are preserved under the receipt output before their task-created directories are removed. Review screenshots/browser assets and coverage evidence remain under `/private/tmp/s8-lane-b/`.

Reproduce the finite proof with `python3 examples/ui-preferences/build.py` and `PREFERENCES_PLAYWRIGHT=<installed-package-directory> python3 examples/ui-preferences/browser.py`. No new toolchain/package install, devmetrics/listener, commit, push, publish, deployment, MCP indexing, generated CRUD, native file persistence, packaged target or streaming claim. An unconfirmed preference write can still be saved and become visible on refresh; failure is not rollback. Coordinator review is next; stop before M2.
