# Task 00013 Lane A M2 receipt

M2 implementation is ready for coordinator review; no M3 was started. Worktree `trial3-s8-m0`, branch `trial/3-s8-m0`, HEAD `a268b9653fb7dbe81f7c2abe1b7632edcdb7e181`. The required `git merge trial/3` fast-forwarded accepted M1 before the restart pause. All source remains uncommitted. The historical WIP checkpoint describes the pause, not the final result.

## Scope and component evidence

Contracts: [accepted inventory](../slice-8-design-inventory-components.md), [M2 gate](../slice-8-design-verification.md), [public component guide](../../../../crates/leptos/docs/api/components-breadth.md). Explicit Rust controllers/slots replace React hooks/child reflection; source CSS/class/alias ABI is not claimed.

| Selected M2 family | Native and real browser evidence |
| --- | --- |
| Accordion/Details/Tabs | Checked unique enabled choices; single/multiple/required/collapsible policies; retained hidden content; native details; manual/automatic tab activation, orientation, arrows/edges/disabled focus, tab/panel associations. Browser retains tab input node/draft. |
| Breadcrumbs/Pagination/Menu/Menubar | Checked safe links/current final crumb; known/unknown totals, zero/one/many and overflow/clamping, canonical skip/limit codec; enabled-only roving focus, bounded incremental/repeated-letter search, menu selection, Tab exit and menubar transitions. Local paging never mutates location. |
| Popover/Tooltip/Drawer/Dialog/Toast | Native top-layer stack, outside click/Escape, modeless/modal distinction, nested/unmounted cleanup, deterministic fallback focus. Modeless closure preserves unrelated background focus; nested Escape closes the nearest surface only. Tooltip is noninteractive described-by text with bounded delay, hoverable surface and native hint-popover Escape even when only hovered. Toast has explicit dismissal/optional expiry and focus/hover pause. |
| Table slots/DataTable | Caption/head/body/group/foot/row/header/cell, scope/spans, native rowspan zero, invisible width ruler inside a cell, stripes/sticky/labelled keyboard overflow. Literal filter, stable lexical sort, cross-page/current-page selection, replacement/pruning/page clamp. Browser retains rows/cells and focused headers across same-column replacement. |
| CollectionFieldset/TextList/TagList/MetadataInput | Bounded atomic add/edit/remove/reorder, stable nonzero never-reused IDs, checked text/metadata budgets and typed nested Missing/Null/Value. Reentrant factory executes outside write guard. Browser preserves keyed nodes/drafts, cap disables Add and moves focus to the added row's enabled Remove, removal returns focus to Add; empty/false/zero submit distinctly. |
| Combobox/SuggestInput | Query/selected/active/status are separate, selected empty differs from None, explicit options, actual async success/failure/supersession and zero initial lookup. Tickets check controller identity plus generation; foreign same-generation completion rejected. Oversized Unicode edits clear stale machine selection with associated error, recover on valid choice. |
| Slider/RatingInput/SlugInput | Finite checked single/ordered-pair native range grid, crossing bounds, native Home/End/arrows; optional rating including explicit zero; verbatim slug draft and explicit ASCII/Unicode separator/case transform. Native form values and browser keyboard behavior proved. |
| Existing M1 presentation/input families | Existing gallery proof still passes, including raw-text security, labels/errors/controlled values, semantic presentation and mobile/modal behavior. No M3 formatter/editor/file behavior is claimed. |

Native tests include hostile escaped text, unsafe URL rejection, duplicate/missing keys, invalid/empty/disabled states, numeric overflow/NaN, capacity/key exhaustion, stale/disposed results, selected SSR checkboxes, and exact light/dark defaults. Tests are in `components/breadth_tests.rs` and existing/new inline suites, `tests/suggestions.rs`, `tests/system_theme.rs`, framework-view contracts and gallery shared/web tests.

## Hydration, theme and visual findings

- **Selection mismatch fixed at the source; no allowance remains.** DataTable uses the native checkbox's SSR `checked` state and CSS `:has(:checked)` to express row selection, avoiding a second row attribute that diverged after an early native edit. Native SSR tests prove a preselected checkbox is present. Browser proves selection visible before bootstrap and compares complete parsed `#app.innerHTML` before/after hydrate byte-for-byte, with no attribute normalization/exception. All initial controls/rows retain node identity and the early focused field retains focus.
- Early native details, collection/metadata/nested/tag/slug drafts, rating/range values, combobox/suggestion query and table filter/selection are admitted before hydrate from the same source. Applying an early filter remains an explicit subsequent edit; it does not rebuild the initial SSR rows. Initial async Resource HTTP calls are zero in hydrate and one in CSR; suggestion calls are zero initially in both.
- framework-view accepts additive host-seeded `system` alongside light/dark. `Theme::System`, opt-in `SystemThemeStyles`/trusted `SYSTEM_THEME_STYLES`, and CSS media resolve before bootstrap and react to media changes without changing the seed. Existing base light/dark stylesheet bytes/default behavior remain unchanged. Lane B files were not edited; its adoption is separate.
- Wrapped and vertical action groups use the same `.75rem` gap. Geometry asserts consecutive action buttons have at least the token on the applicable axis: 12 physical px at 100%, 24 at 200%, including stacked popover and drawer actions.
- Popup positioning flips/clamps to the viewport, including 375 px/200%. Its close action and all content remain reachable; overlaying its own trigger is permitted. The ResizeObserver feedback counterexample at 200% was fixed by using finite computed CSS zoom factors rather than rounded offset-width ratios.
- Drawer zoom was a real geometry bug: at 375 px/200% the old viewport-unit dimensions produced rect `[-343,0,375,1624]`. Percentage bounds now produce `[32,0,375,812]`. All four width/zoom combinations are measured, every action is reachable, backdrop insets cover the viewport, and wheel/Tab/focus attempts cannot reach the modal background. A full-page screenshot extends beyond the viewport; viewport captures and geometry prove actual backdrop coverage.

## Review rounds and lifecycle safety

1. Logic/complexity: bounded controller decision tables, paging codec/arithmetic, enabled roving/typeahead, reactive retained cells/options, foreign-controller completion and computed zoom. Meaningful native/browser checks rerun after corrections; the foreign-completion regression failed before the Weak identity fix (`foreign-completion-before.log`).
2. Ownership/responsibility: pure policy/state/geometry separated from WASM DOM leases; keyed collection/row/header identity; factory outside guard; TableRule reread against source `Frui Table.tsx:795-874` and corrected to an invisible in-cell ruler. No executor/global registry/unsafe was introduced.
3. Consistency/Rustdoc/comments: explicit exports and API guide, closed roles/styles/modes, Rustdoc/errors, scope/span defaults, neutral opt-in CSS. fmt, Clippy and Rustdoc passed. Old light/dark and modal default markup remain covered.
4. Safety follow-up: strict whole-DOM adoption; oversized-query stale selection/recovery; async cancellation/Owner disposal; add-at-cap focus; nested/modeless Escape and return focus; drawer bounds; tooltip hover-only Escape. Final affected native and browser checks rerun after the last source edit.

Suggestion cancel, root disposal and unwind release loader captures after **one additional poll** following cancellation in the child fixture; each loader is admitted once. Existing async Owner/Resource fixtures remain child processes under 10-second kill/reap watchdogs. A serial libtest prefix exposed a false completion-marker rejection: the helper now accepts only the exact sentinel line or exact `test NAME ... SENTINEL` line; wrong-name/mode/substrings still fail. Timeout remains failure. Cooperative cancellation cannot interrupt CPU-bound/non-yielding loaders; loaders must yield and be bounded.

## Verification

All exact commands, outputs and machine-readable receipts are under `/private/tmp/s8-m0/m2/` (temporary; source proofs are retained in the worktree). Native/WASM targets are only `target/s8-native` and `target/s8-wasm`; all commands use `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`. Later checks use `CARGO_BUILD_JOBS=2`, `RUST_TEST_THREADS=1`.

| Check | Result / evidence |
| --- | --- |
| `cargo fmt --all -- --check` | PASS; `local-fmt.log`. |
| UI tests/doctests `--locked --features ssr` and separately `ssr,rich` | PASS: ssr 60 and ssr,rich 61 (each includes six doctests), zero failures; `local-ui-ssr.log`, `local-ui-rich.log`. Never `--all-features`. |
| Selected view/gallery native tests, UI rich tests | PASS; final fresh instrumented execution in `coverage-tests.log`. |
| Clippy selected crates `--all-targets -- -D warnings`; Rustdoc `RUSTDOCFLAGS=-D warnings`; `cargo +1.96.0 check` | PASS; `final-verification.json` and final affected `local-verification.json` include commands/features. |
| Client WASM hydrate/CSR Clippy, builds and matching bindgen | PASS; `local-{hydrate,csr}-clippy.log`, `gallery-*-build.log`, `gallery-*-bindgen.log`. |
| Client and opt-in facade WASM checks/graphs | PASS four `cargo tree --locked --target wasm32-unknown-unknown --edges normal,no-proc-macro` graphs; `m2-{wasm,facade}-{hydrate,csr}-tree.log`. Deny Stackpress server/DB/SQL/CLI/Idea/Tauri/Tokio runtime; no leaks. Leptos common `leptos_server`/`server_fn` packages are not the Stackpress execution host. |
| Original gallery browser and new breadth browser | PASS: original four cases and breadth four cases, root/mounted × hydrate/CSR; `gallery-browser.json`, `breadth-browser.json`; 120-second parent process-group watchdog, 10-second action/native request deadlines, close in finally, intercepted `.invalid` only. |
| Unchanged `python3 examples/profile-ui/check.py` | PASS; `m2-profile-check.log`, `profile-check-final/summary.json`. Accepted source untouched. Its unchanged browser proof also passes 8 cases (`profile-browser-final/summary.json`). |
| `cargo test --workspace --all-targets --locked -- --test-threads=1` | PASS: 601 passed, 0 failed, 62 ignored; `final-workspace-tests.log`. |
| `cargo test --workspace --doc --locked` | PASS: 891 passed, 0 failed; `final-workspace-doctests.log`. |
| Workspace Clippy `--workspace --all-targets --locked -- -D warnings` | PASS; `final-workspace-clippy.log`. |

The first workspace run hit its 600-second outer deadline under machine contention, without an observed assertion failure. Per coordinator direction it was stopped/reaped and rerun serially with two build jobs and an 1800-second deadline; the retry passed in 222 seconds. No unrelated SQL fix was attempted. Workspace checks preceded the last small M2-only follow-ups; all affected crates/targets/browser proofs were rerun afterward.

## Coverage

Fresh native `ssr,rich` instrumentation with installed cargo-llvm-cov 0.8.6 / Rust 1.96.0; the actual finite gallery web executable is included alongside test objects. Command: `cargo llvm-cov --no-rustc-wrapper --no-report --locked -p stackpress-framework-ui -p stackpress-framework-view -p stackpress-example-ui-gallery-shared -p stackpress-example-ui-gallery-web --features ssr,rich`, followed by matching sysroot LLVM profile merge/export. Reproduction driver: `/private/tmp/s8-m0/m2/coverage.py`.

| Crate | Covered / executable production lines | Coverage |
| --- | --- | --- |
| framework-ui | 3305 / 3391 | 97.46% |
| framework-view | 504 / 526 | 95.82% |
| gallery-shared | 481 / 554 | 86.82% |
| gallery-web | 205 / 229 | 89.52% |

All exceed the 70% development gate. `m2-coverage-full.json`, `.lcov` and `m2-coverage-summary.json` retain measured scopes/per-file uncovered lines. Coverage compilation/execution passed; there are no failing tests. LLVM reports 118 mismatched dependency function records (117 tachys, 1 memchr). Serial `--dump --num-threads=1` export succeeds with zero first-party mismatches; exact diagnostics are retained in `coverage-diagnostics.log` and `coverage-diagnostics-summary.json`. The warnings are reported, not suppressed; publication readiness is not claimed.

Native instrumentation excludes dependency sources, dedicated test source and actual top-level inline `cfg(test)` modules only. WASM-only DOM/fetch adapters and client entry are outside native executable coverage and are exercised by actual hydrate/CSR Chromium cases, including cleanup/unmount/focus. Native pure admission/controller/geometry/lifetime logic remains measured. Native coverage is not a claim about WKWebView or manual assistive technology.

## Screenshot and artifact review

72 gallery PNGs plus the unchanged profile evidence are retained. SHA-256/source fingerprints and JS/WASM byte/gzip sizes are in `artifact-manifest.json`. Review captures: [stacked popup at 375 px/200%](/private/tmp/s8-m0/m2/screenshots/root-hydrate-m2-popup-375-zoom2.png), [drawer at 375 px/200%](/private/tmp/s8-m0/m2/screenshots/root-hydrate-m2-drawer-375-zoom2.png), [complete desktop inventory](/private/tmp/s8-m0/m2/screenshots/root-hydrate-m2-1280-zoom1.png). All four root/mounted hydrate/CSR cases have 375/1280 px at 100%/200% full-page, popup and drawer captures. These files are temporary and can be lost on restart; the measured conclusions above are durable in this receipt.

At 375 px/200% popup actions are clearly separated and reachable; drawer/backdrop fills the viewport with all actions in bounds. Full-page captures retain complete component inventory; popup/drawer captures are viewport screenshots. Automated role/name/relationship/native keyboard assertions are bounded Chromium evidence; manual screen-reader review and owner visual acceptance remain separate.

## Dependencies, boundaries and closeout

- No dependency version, lockfile or root facade change. Only already-locked web-sys DOM feature flags (`HtmlDetailsElement`, `ResizeObserver`) were enabled; web-sys 0.3.106 is MIT OR Apache-2.0, rust-version 1.81. Existing Leptos 0.8.22 is MIT, rust-version 1.88. Workspace/compiler floor 1.96.0 verified. Installed wasm-bindgen 0.2.129, Playwright 1.64.0 and Chromium 156.0.8078.4 were used; no tools installed.
- No edits to streaming lane (`async_render.rs`, `replay.rs`, `resources.rs`, `stream*.rs`), formatting lane (`rich.rs`, `format*.rs`), framework-language/preferences, server/lib/SQL/schema/CLI/Idea, or accepted profile-ui. Root manifest/lock/src/lib.rs remain untouched. No other trial branch/output consulted.
- No listener, devmetrics instance, global scheduler, commit, push, publish, mobile build or M3 work. Packaging/mobile/WebView support, streaming integration, generated CRUD and formatter/editor breadth remain outside M2.
- Both compiled target roots were deleted after exporting coverage; `target/` is absent. Finite workspace child logs were copied to `workspace-child-logs/`; empty fixture directories/cache metadata were removed. The final read-only process audit found no task build/browser processes (`cleanup.json`, `process-audit.log`). Other lanes were left alone.
- Final KB validator exits 1 with the **unchanged baseline of 146 errors and 22 warnings** (mainly unavailable TypeScript sibling links and existing size/link-text warnings). Exact diagnostic counters match, with no added/removed diagnostic: `kb-final.log`, `kb-comparison.json`. No unrelated KB repair was attempted.
- New shared requests: none. Lane B can adopt the additive framework-view system token later through its own released work.
