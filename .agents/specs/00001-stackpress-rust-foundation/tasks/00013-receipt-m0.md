# Task 00013 M0 receipt: components, preferences and buffered async

Implementation evidence for [task 00013](00013-slice-8-more-ui.md), 2026-10-27. Read with [the accepted M0 gate](../slice-8-design-verification.md#first-proof-and-stop), G23/G25/G29/G30 and the coordinator's M0-only brief. Worktree `trial3-s8-m0`, branch `trial/3-s8-m0`, base/current HEAD `7029d831f116de607f2a7bb2bfa34cb32a341c7e`.

## Result and authority

Both released **M0 implementation proofs pass** within the native/Chromium scopes below. Stop here: M1 is not started. Coordinator re-verification and owner visual acceptance remain separate; this receipt does not mark task 00013 or all accepted visual controls complete.

The first checkpoint correctly stopped on a raw Resource cancellation counterexample. The coordinator approved this receipt's creation, then explicitly released cooperative cancellation and the independent component/persistence proof, and approved updating this file. That instruction supersedes the earlier request to reopen executor design. The original receipt is retained as `/private/tmp/s8-m0/00013-receipt-m0.md`; raw counterexamples remain executable. No new product decision or shared-write request is outstanding.

## Proof 1: shared components and persistence

[Advanced components](../../../../crates/leptos/src/components/advanced.rs) supply a semantic captioned Table with explicit thead/tbody children and a labelled keyboard-scrollable overflow region; controlled native Dialog; and Form with typed `FormState`/`FieldError`, retained drafts, error-summary links and invalid-field focus. Table row identity/data/sort/page remain application-owned. M0 does not implement an editing grid, sorting/pagination breadth or generated domain writes. Form acknowledgement in this gallery is local validation, not a fabricated database write.

[Dialog's thin DOM adapter](../../../../crates/leptos/src/components/dialog_dom.rs) uses native showModal/close, explicit initial focus, innermost-modal Tab/Shift-Tab handling, Escape policy, opener restoration and deterministic fallback. It handles nested dialogs, disabled/removed openers and unmount. No reactive/RefCell guard crosses a DOM callback. Native initially-open SSR dialogs are outside this M0 contract; the shared source starts closed.

[Rich content](../../../../crates/leptos/src/rich.rs) has one private `SafeHtml` constructor boundary and fixed policy revision 1. Approved ammonia 4.2.4 sanitizes both HTML and pulldown-cmark 0.13.5 output. Only p/a/strong/em/ul/ol/li/code/pre/blockquote/br survive; a permits href/title, fixed rel noopener/noreferrer, and explicit HTTP(S) only. Relative/protocol-relative/data/javascript URLs, generic attributes, scripts/styles, SVG/MathML, forms/frames/images and comments are excluded. Input/output ceilings are 64/256 KiB, lowerable for HTML policy. Parsing/sanitizing remains trusted bounded library work, not a CPU sandbox. Native hostile-corpus/idempotence/budget tests and exact compiled native/WASM strings pass; browser DOM contains none of the excluded active elements and causes no unexpected requests. The CSR DOM comparison removes only Leptos's empty framework comment anchors; exported sanitizer strings are compared exactly without normalization.

[framework-language](../../../../crates/framework/framework-language/src/lib.rs) provides a bounded immutable canonical/alias registry, prefix > canonical cookie > configured default, checked mounts and unchanged query bytes. Aliases target canonical locales directly; duplicates/unknown/corrupt cookies fall back, unknown route prefixes remain for host 404s. The fixed POST preference action requires exact configured Origin, checked locale and same-app location. Only host code applies Set-Cookie: HttpOnly, SameSite=Lax, Secure on HTTPS, one-year Max-Age and app-scoped Path. HTTP development origins explicitly omit Secure. Cookie data is skipped from public JSON. This preference action is not an identity/session system; session-bearing hosts retain their additional CSRF policy.

[The gallery](../../../../examples/ui-gallery/shared/src/lib.rs) uses one real Gallery source for SSR/hydration/CSR, three keyed rows, nested dialogs, typed validation, shared hostile rich corpus, and real component Resource/Suspense/reactive Title. Its finite [HttpAdapter host](../../../../examples/ui-gallery/web/src/main.rs) projects a Send top-level loader, executes one top-level request/response lifecycle and handles locale persistence. The host never listens. Native child tests cover prefix/cookie/default/alias, CSR seed, successful action and 400/403/413 denials, with no cookie on failure.

Actual hydrate runs preserve exact parsed initial app DOM, the original focused input node and text typed before JavaScript. Locale acknowledgement changes root phrases/caption/Title/document lang/history while preserving dirty input; refresh remembers the HttpOnly cookie and performs no initial preference acquisition or duplicate hydrated Resource fetch. French/dark refresh seeds, cookie legacy route and explicit English-prefix override pass. Missing-Origin denial is visible and preserves locale/draft. Full localization, theme/flash persistence and multi-action write race policy are later milestones.

## Proof 2: cooperative Resource ownership and replay

[The raw fixture](../../../../crates/leptos/tests/async_feasibility.rs) still demonstrates pinned Leptos's detached Resource behavior: any_spawner 0.3.0 uses tokio::spawn without returning its JoinHandle; root cleanup reaches the Resource child Owner once but cannot itself interrupt an admitted never-resolving await. Runtime teardown releases the raw capture. This is retained as a negative control, not represented as production cancellation support.

[The Stackpress guard](../../../../crates/leptos/src/resources.rs) registers AbortHandle cancellation synchronously inside the Resource fetcher, at that Resource's Owner, then races the real loader through futures 0.3.34 Abortable. Cleanup wakes even a never-resolving loader. Guarded success/cancel/render-unwind each invoke the loader once, observe one child cleanup, restore the caller Owner/context and release its capture within **16 current-thread scheduler yields**, under a 10-second parent process watchdog. Already-completed data survives later cleanup; missing/already-cleaned Owner cancels before polling. Native loader unwind is consumed once into a safe serializable failure; the private cause is accessible natively and omitted from replay. An Rc-across-await loader is rejected by a real compile-fail doctest.

This is cooperative cancellation: the host executor must stay alive to poll awakened tasks. The observed 16-turn bound belongs to these controlled fixtures, not a real-time guarantee for an overloaded scheduler. CPU-bound/non-yielding loaders cannot be interrupted; loaders must remain async, bounded and subject to host deadlines. Cancellation does not roll back side effects or forcibly interrupt every upstream source-change/refetch policy. Panic-abort and arbitrary panicking destructors are not recoverable.

[AsyncLeptosRenderer](../../../../crates/leptos/src/async_render.rs) implements the additive [AsyncPageRenderer/AsyncRenderFuture](../../../../crates/framework/framework-view/src/lib.rs) boundary. Existing PageRenderer/RenderFuture and synchronous LeptosRenderer keep their synchronous staging/Head/Owner contract; their tests are unchanged. The new buffered Send future owns its isolated root through actual component Resource/Suspense body collection, escaped replay and reactive Head. Each synchronous poll restores its actual caller Owner; no entered guard crosses await. Checked optional stylesheet/bootstrap mounts agree. Native retained reactive state must be Send/Sync; thread-local LocalStorage/Rc belongs in synchronous/client-only adapters.

Drop disposes assembly, cleans the root to cancel loaders and drains/discards public SsrSharedContext replay/deferred queues. This breaks the observed context -> serialization waiter -> Resource Owner -> context retention cycle. It uses only public pinned APIs; no custom executor, global mutex, unsafe Send assertion, leaked Owner or upstream patch. Assembly/Head unwind preserves a native cause in safe ViewError and is never retried.

[The real renderer exit matrix](../../../../crates/framework/framework-view/tests/leptos_async_renderer.rs) passes success, cancellation, factory error/panic, reactive Head panic, loader panic, HTML limit and replay limit, with capture release within 16 scheduler yields and caller restoration. A separate watchdog case stages/polls on one thread, migrates the actual Send future and completes on another, verifying both callers. Concurrent independent English/French gallery roots retain separate locale/Head/replay data.

[ResourceReplay](../../../../crates/leptos/src/replay.rs) enforces 64 resources, ID <=1024, unique IDs, checked logical byte budget and at-most-sixfold escaped input budget. Native JSON is emitted as inert application/json; the external bootstrap installs validated replay before actual hydrate. Hostile script terminators, quotes, ampersands and U+2028/U+2029 round-trip exactly in both body and reactive Title. Real browser hydration performs **zero initial Resource fetches**, including refresh; CSR performs its one required initial fetch. Browser disposal aborts an admitted fixed-endpoint acquisition and cannot publish a late result.

## Browser review packet

All artifacts/logs are under `/private/tmp/s8-m0/`; no listener was started. [Browser proof source](../../../../examples/ui-gallery/browser.cjs) intercepts only HTTPS gallery.invalid, serves built local assets and finite native host output, aborts all other origins, closes contexts/browser in finally and uses 10-second actions/acquisition deadlines. [The parent](../../../../examples/ui-gallery/browser.py) kills/reaps the process group at 120 seconds; timeout is failure. Hang-class native proofs use a separate [10-second child watchdog](../../../../crates/leptos/tests/support/async_watchdog.rs), exact filters, completion markers and a negative control for unmatched filters.

`gallery-browser.json` and `gallery-browser.log`: **4/4 pass**, hydrate/CSR at root and `/demo/v1`. Initial DOM/focus, replay, sanitizer parity, modal keyboard/nesting/dismissal/unmount, invalid/successful form, dirty locale change/refresh/denial, scoped cookie and native lifecycle checks pass. Each mode has zero unexpected console errors/requests and exactly one intentionally expected 403 console diagnostic. Final Resource call totals include the deliberately cancelled refetch; the zero-initial-fetch assertion occurs before that action.

Twenty screenshots are in `screenshots/`: for each `{root,mounted}-{hydrate,csr}`, `-1280.png`, `-dialog-375.png`, `-375.png`, `-375-zoom2.png`, `-1280-zoom2.png`. Geometry passes at 375/1280 CSS pixels and 100%/200% CSS zoom. Desktop hydrate and mounted CSR mobile-dialog screenshots were visually inspected; neutral grayscale, visible focus and contained table overflow are present. CSS zoom is a geometry aid, not all browser/OS magnification modes. Native screen-reader inspection and packaged WKWebView remain unproved; owner sign-off remains pending.

Versions: macOS 26.6.2 (25G83), Chromium 156.0.8078.4, Playwright 1.64.0, Node v26.2.0, Python 3.11.4, rustc 1.96.0 (ac68faa20), wasm-bindgen CLI 0.2.129. Installed tool paths are supplied through environment variables, not installed by this task. `m0-artifacts.json` records exact sizes/SHA-256. Debug proof WASM sizes: hydrate 6,028,419 bytes (gzip 1,524,793), CSR 5,626,436 (gzip 1,436,793); JS glue 43,078/41,105 bytes, bootstrap 543. These are unoptimized proof assets, not release-size claims.

Reproduce with `python3 examples/ui-gallery/build.py`, then `GALLERY_PLAYWRIGHT=<installed-playwright-package> python3 examples/ui-gallery/browser.py`. Both commands are finite; build logs are `gallery-{native,hydrate,csr}-build.log` and bindgen logs, assets in `gallery-assets/{hydrate,csr}`. No package install or development listener is implied.

## Review rounds

The original negative checkpoint had its own four verified fixture rounds (`async-round1.log`, `async-round2.log`, `ui-round3-tests.log`, `ui-round3-clippy.log`, `async-safety-round.log`). Continuation rounds cover production M0, not just those characterization tests:

1. **Logic/complexity:** typed scenario/state outcomes, lowerable input/replay/output limits, closed sanitizer/locale policies and denial paths. Fixed zero replay-limit expectation to match existing PageLimits semantics. UI/language checks pass (`m0-round1.log`).
2. **Responsibility/abstraction:** split pure seed/ack admission from thin browser DOM/fetch/history; separate async trait preserves synchronous renderer promises; scoped fetch owns AbortController. Corrected HTTP denial staging so JSON cannot reset 400/403/413 to 200. Selected tests pass (`m0-round2.log`).
3. **Consistency/Rustdoc/comments:** reviewed Rustdoc, error causes, immutable policy and each stateful boundary; added the actual Rc-across-await compile-fail witness. Selected tests/doctests/Clippy pass (`m0-round3-tests.log`, `m0-round3-clippy.log`). Final WASM Clippy found and fixed two collapsible browser-only branches; final mode checks and rebuilt browser proof pass.
4. **Safety/soundness:** reviewed Owner cleanup, queue retention cycle, native panic conversion/no retry, per-poll caller restoration, actual cross-thread migration, RefCell/DOM reentry, nested/disabled/removed focus targets and scoped browser acquisition abort. Preserved raw negative controls, added successful/failed renderer matrix and migration coverage. Tests pass (`m0-round4-tests.log`, `async-migration.log`, `loader-unwind.log`, final native/browser logs). Final optional stylesheet handling is covered by the renderer success assertion. No further material finding remained after those checks.

## Final verification

| Check | Result / evidence |
| --- | --- |
| `cargo fmt --all -- --check` | Pass, `m0-fmt.log`; final scope audit also passes. |
| `cargo test --locked -p stackpress-framework-ui --features ssr` | 33 tests + 4 doctests pass, `m0-ui-ssr.log`. |
| Selected UI `ssr,rich`, language and gallery tests/doctests | 43 tests + 4 doctests pass, `m0-final-tests.log`; fresh instrumented rerun also passes. |
| Selected native `cargo clippy ... --features ssr,rich --all-targets -- -D warnings` | Pass, `m0-clippy.log`. |
| `RUSTDOCFLAGS='-D warnings' cargo doc ... --features ssr,rich --no-deps` | Pass for changed scopes, `m0-rustdoc.log`. |
| `cargo +1.96.0 check ... --features ssr,rich --all-targets --locked` | Pass for all changed scopes, `m0-msrv.log`. |
| Actual WASM hydrate/CSR builds, Clippy `-D warnings`, extended opt-in facade checks | Pass separately, gallery build and `m0-wasm-{hydrate,csr}-clippy.log`, `m0-facade-{hydrate,csr}.log`. |
| `cargo tree --locked ... --target wasm32-unknown-unknown --edges normal,no-proc-macro` | Actual gallery and extended facade graphs pass both modes: no server/DB/CLI/Idea/Tauri/Tokio; four `m0-*-tree.log` files. |
| `cargo test --workspace --all-targets --locked` | 492 pass, 59 existing child/platform cases ignored, zero failures; `m0-workspace-tests.log`. |
| `cargo test --workspace --doc --locked` | 869 pass, `m0-workspace-doctests.log`. |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Pass, `m0-workspace-clippy.log`. |
| Unchanged `python3 examples/profile-ui/check.py` | Pass, `m0-profile-check.log`, `profile-check-final/summary.json`. |
| Unchanged profile browser proof wrapper | 8/8 at both mounts pass, `m0-profile-browser.log`, `profile-browser-final/summary.json`; installed Playwright supplied through PROFILE_UI_PLAYWRIGHT. |
| Gallery actual Chromium proof | 4/4 pass with 20 screenshots, `gallery-browser.json`/`.log`. |

Every Cargo command uses `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0`, native `target/s8-native` and WASM `target/s8-wasm`; no third compiled target tree is created inside this worktree. Invalid combined render features were not requested: framework-ui tests use `--features ssr`, additionally `ssr,rich`, never `--all-features`. Exact commands/exit codes/timings are in `final-m0-verification.json`; selected rich tests/doctests are `m0-final-tests.log` and coverage reruns. Root/facade exports add no executable implementation lines; their native/WASM builds and graphs pass.

| Native production scope | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 1036/1058 | 97.92% |
| framework-view | 504/526 | 95.82% |
| framework-language | 145/149 | 97.32% |
| gallery-shared | 135/146 | 92.47% |
| gallery-web | 203/225 | 90.22% |

Coverage uses installed cargo-llvm-cov 0.8.6, Rust 1.96.0, native `ssr,rich` with framework-view/server selected. `coverage-m0.py`, `m0-coverage-tests.log`, `m0-coverage-full.json`, `m0-coverage-full.lcov` and `m0-coverage-summary.json` retain exact scope, commands and uncovered lines. Matching LLVM export explicitly includes first-party example executables because cargo-llvm-cov's default report excludes examples. Only executable production src lines count: dedicated test source and inline cfg(test) modules are excluded with recorded line cutoffs; external dependencies are excluded. No broad production exclusion or aggregate-crate substitution is used. Fresh native cleaning plus `--no-rustc-wrapper` prevents reuse of non-instrumented artifacts. LLVM export reports 62 unmatched dependency functions (61 tachys, 1 memchr), none first-party; ordinary parallel/serial file summaries match. A diagnostic parallel `--dump` exposed an LLVM raw_ostream/malloc abort; serial diagnostic/export succeeds (`m0-coverage-diagnostics-summary.json`). This tooling failure is not presented as a passed test. Thin WASM Dialog DOM, fetch/history and client-start adapters have actual Chromium coverage evidence, not a native line percentage; pure state/policy/decoder logic remains native-tested. The WASM-only client is unmeasured natively, not 100% covered. Publication's 90% gate is not claimed.

## Dependencies and compatibility

`m0-lock-delta.json`: 19 added packages (four workspace, fifteen third-party), **zero removed/upgraded existing versions**. Leptos family remains pinned: leptos 0.8.22 / leptos_server 0.8.8 / reactive_graph 0.2.16 / hydration_context 0.3.2 (MIT, declared MSRV 1.88). Existing futures/futures-util 0.3.34 (MIT OR Apache-2.0, MSRV 1.71) supplies Abortable; no new scheduler dependency. any_spawner 0.3.0 is MIT with no declared MSRV; url 2.5.8 is MIT OR Apache-2.0/MSRV 1.63; wasm-bindgen 0.2.129 and web-sys 0.3.106 are MIT OR Apache-2.0/MSRV 1.81. Named Rust 1.96.0 checks pass; workspace crates inherit MSRV 1.96. `dependencies.json` and `m0-existing-dependencies.json` record registry-manifest metadata, not guessed MSRVs.

| New third-party package(s) | License | Declared MSRV |
| --- | --- | --- |
| ammonia 4.2.4 | MIT OR Apache-2.0 | 1.85 |
| pulldown-cmark 0.13.5 | MIT | 1.71.1 |
| pulldown-cmark-escape 0.11.0 | MIT | 1.70 |
| cssparser 0.38.0 | MPL-2.0 | 1.85 |
| html5ever 0.40.1, markup5ever 0.40.0, web_atoms 0.3.0, string_cache 0.11.0 | MIT OR Apache-2.0 | 1.85 |
| phf 0.14.0, phf_codegen 0.14.0, phf_generator 0.14.0, phf_shared 0.14.0 | MIT | 1.85 |
| string_cache_codegen 0.11.2 | MIT OR Apache-2.0 | Unspecified |
| maplit 1.0.2, unicase 2.10.0 | MIT/Apache-2.0 | Unspecified |

Only authorized framework-ui/framework-view additions, framework-language, ui-gallery, root Cargo/lock and opt-in facade changes exist. Existing synchronous render bodies/tests and accepted profile-ui handwritten source/check/proofs are unchanged. No edits to lib/server/CLI/Idea/SQL/schema. Mandatory profile checks rebuilt only ignored dist assets. No other trial branch or stale external target output was consulted; workspace regression consumer fixtures create/remove their own fresh temporary builds.

Both task-owned build trees were deleted after verification: native `Removed 2673 files, 1.0GiB total`, WASM `Removed 3814 files, 997.8MiB total` (`m0-native-clean.log`, `m0-wasm-clean.log`). Task-created regression child logs were copied and byte-compared under `regression-child-logs-final/`, then their ignored fixture directories removed; `m0-cleanup.json` records the audit. No `target/` remains. Earlier precoverage clean removed 3.1 GiB (`native-precoverage-clean.log`). No external/shared target or installed runtime was removed.

## KB and stop

The worktree validator already failed with **146 errors and 21 warnings**, chiefly missing sibling TypeScript sources outside this isolated worktree (`kb-before.log`). Postupdate diagnostics remain identical (`kb-m0-final.log`, `kb-m0-final-comparison.json`); this is a known failing KB baseline, not a green KB gate. Only this named receipt changed under .agents, with coordinator approval. No tracker/index/context or unrelated KB repair was performed. The old stop, unstarted-proof status and executor-design request are explicitly superseded above; historical counterexample/provenance and logs remain available.

No listener, devmetrics instance, tool installation, commit, push, publishing, generated CRUD, streaming integration, mobile build or packaged target claim. No new shared request remains. Controls involving generated CRUD, formatting/editor breadth, full preferences, installed targets, plugin regeneration and streaming remain at their accepted later gates. Both bounded M0 proofs are ready for coordinator review; stop before M1.
