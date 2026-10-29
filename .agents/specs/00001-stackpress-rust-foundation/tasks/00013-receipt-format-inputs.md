# Task 00013 specialized editing inputs receipt

The released M3 follow-up is implemented and ready for coordinator review. Worktree `trial3-s8-m0`, branch `trial/3-s8-m0`, HEAD `7b24106fbb01a5b9933a14d242e4bbd60310a945`; the required `git merge trial/3` fast-forwarded before implementation. All source remains uncommitted. No subsequent milestone was started.

## Scope and completion contract

Authority: the accepted [specialized editing table](../../../../examples/ui-format/README.md#specialized-editing-contract). Its table is preserved exactly. Production implementation is `crates/framework/framework-ui/src/edit/`, exported as `format::edit` under the existing `format` feature. [API entry](../../../../crates/leptos/docs/api/README.md) and the example explain usage and decisions. Existing public enums gained no variants; downstream exhaustive-match compatibility is preserved.

| Required area | Implementation and proof |
| --- | --- |
| Value versus draft | `EditState<P>` keeps the last typed value, bounded raw text, immutable SSR seed, UTF-16 selection/direction, captured draft locale, dirty/error and explicit composition lifecycle separately. Required empty errors; nullable empty clears to None. Partial sign/point, incomplete ISO and `+33` remain drafts and cannot replace a valid committed value. |
| Decimal grammar | Explicit en `.`/`,` and fr `,`/U+202F or U+00A0 grammar. Clipboard grouping is strict; mixed French spaces, mixed decimal separators, exponents, Unicode digits and line breaks fail. Native middle edits may move group boundaries. Exact ICU decimal arithmetic, explicit 0..18 scale, half-even commit, leading-zero normalization, padded scale and preserved signed zero; no floats. Rounded typed values are checked against exact inclusive/exclusive bounds. |
| Date/time | ISO text editing, Gregorian dates, local times and explicit-offset instants; no segmented widget or new date library. Leap days, missing offsets, unknown `-00:00`, offset suffixes, precision 0..9 and typed min/max are tested. Commit pads seconds/fractions, rejects excess precision and compares supplied-offset instants exactly; Z normalizes to +00:00. Named-zone/DST conversion stays out of scope. |
| Phone | Explicit metadata region for national drafts; `+` and `00` admit international input, with region IDD rules from the pinned parser. Complete metadata-valid E.164 only. Letters/extensions, Unicode digits and invalid/incomplete numbers never commit. Dirty region/policy switches preserve the previous context with a named error; clean changes revalidate. Validation makes no assignment/reachability claim. |
| Caret and selection | Checked UTF-16 boundaries and forward/backward ranges; semantic token remapping through grouping, sign and localized decimal marks. Pure tests cover Unicode/surrogates; Chromium covers backward ranges, middle insertion/deletion, replacement and end/collapsed positions. |
| Paste and undo | Trusted keyboard clipboard paste with permissions restricted to `https://format.invalid`; no fabricated trusted events. Collapsed/ranged replacement, en/fr spaces, plus/prefixes, invalid/line-break/mixed and one-past/oversized payloads. Raw text/selection/value are retained on rejected paste. Native undo/redo survives typing, paste and explicit normalization. |
| IME/event order | Intermediate composition text is untouched. Root-owned next-task settlement admits either final input ordering, duplicate events and deferred blur once; cancellation restores its original draft/selection/context. New sessions cannot inherit a previous blur. Evidence uses synthetic CompositionEvent/InputEvent sequences; real OS IME is unproved. |
| Bounds and errors | Cheap JS UTF-16 length and pure exact UTF-8/digit admission precede any Rust String copy, followed by controller budgets before draft/replacement allocation. Tests cover exact/one-past byte/digit limits, malformed surrogates, zero/negative zero, invalid bounds/configuration and recovery. Errors contain fixed names, never raw draft data. |
| Lifecycle and forms | Same source SSR/hydrate/CSR. Strict parsed app DOM equality on hydrate, original node/focus/backward selection and pre-JS partial draft retained, with zero initial commit. Dirty locale changes retain captured grammar. Only a separate canonical hidden field submits; invalid raw draft is never a successful field. Callbacks run outside reactive guards, only for changed typed values. Root disposal cancels the composition timer; detached late events cannot reach disposed state. |
| Browser/layout | Eight en/fr × root/mounted × hydrate/CSR cases, interactions at 375 px/200%, plus 1280 px screenshots. Labels/help/errors and canonical form values are asserted. Clean Escape closes the containing native dialog and returns focus; dirty Escape cancels the edit first. Action groups retain the gap horizontally and when stacked; no horizontal page overflow. |

## Four review rounds

1. **Logic/complexity:** reviewed grammar versus partial drafts, exact rounded bounds, signed zero/None, temporal offset keys and semantic UTF-16 mapping. Split strict clipboard admission from native middle-edit admission. The first focused round passed 10 tests; the final matrix has 14 pure/native SSR groups, including bridge budgets.
2. **Responsibility/ownership:** separated decimal/temporal/phone policies, shared state, selection, transport and thin DOM adapter. Replaced coupled composition booleans with private Idle/Active/Ending states and a cancellation backup. An unchanged native read now changes only selection, fixing a clean composition-cancel/region-switch regression. Round-two focused tests and WASM check passed.
3. **Consistency/docs/comments:** checked public Rustdoc, fixed field-error vocabulary, required/nullable behavior, callback ordering, SSR seed/property separation and examples. Corrected an initially invalid relative Rustdoc link to `crate::format::edit`. Native tests/doctests, warning-denied Rustdoc/Clippy and the pinned compiler passed afterward.
4. **Safety/soundness:** an expanded real Chromium assertion proved that `.value` normalization destroyed native undo history. Focused Enter normalization now uses Chromium's native `insertText` edit command with synchronous event suppression; blur commits without rewriting raw text. Undo restores raw text then the previous edit; redo restores both. Unsupported native-command hosts retain raw text instead of destroying history. Reviewed allocation admission, malformed UTF-16, no reactive guard across JS/application callbacks, root-owned timers and clean/dirty Escape propagation. The final browser sequence includes all regressions, modal integration and gap geometry.

The native edit command is a deliberately bounded Chromium adapter choice, not a claim of universal WebView support. Explicit inactive clean model/locale resets may replace presentation. The pure controller exposes commit, draft-preserving commit and cancel separately.

## Verification

All logs/artifacts are under `/private/tmp/s8-edit`; they are temporary and may disappear on restart. Exact commands, exits, durations and targets are in `commands.json`, `verification.json`, `workspace-final-verification.json` and `facade-verification.json`. All commands use owned targets, `CARGO_INCREMENTAL=0`, `CARGO_PROFILE_DEV_DEBUG=0`; serial native checks use two build jobs and one test thread.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | PASS |
| UI + ui-format tests/doctests, `--locked --features ssr,rich,format` | 90 passed, 0 failed |
| Existing framework-ui `--features ssr` | 60 passed, 0 failed |
| Selected native and hydrate/CSR Clippy `-- -D warnings` | PASS |
| `cargo +1.96.0 check` for selected crates/all targets | PASS |
| Selected Rustdoc, `RUSTDOCFLAGS='-D warnings'`, `--no-deps` | PASS |
| `cargo test --workspace --all-targets --locked -- --test-threads=1` | 656 passed, 0 failed, 64 intentionally ignored child/native-environment cases |
| `cargo test --workspace --doc --locked` | 892 passed, 0 failed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| Eight optimized hydrate/CSR × base/rich/format/both WASM builds | PASS; JS/WASM bytes, deterministic gzip and SHA-256 in `sizes.json` |
| All eight example plus both facade `cargo tree --edges normal,no-proc-macro` WASM graphs | PASS; no server/DB/CLI/Idea/Tauri/Tokio normal-edge leaks |

Hang-class Owner/Resource cleanup tests remain child-process watchdog tests with timeout-as-failure. Browser parents kill/reap process groups at 120 seconds, actions/evaluation/close have ten-second bounds and contexts/browser close in finally. No listener is started; only locally intercepted `.invalid` assets are admitted.

## Coverage

Fresh installed cargo-llvm-cov 0.8.6 with Rust 1.96.0, native `ssr,rich,format`. Reproduce after all other Cargo jobs finish:

```bash
UI_FORMAT_OUTPUT_DIR=/private/tmp/s8-edit python3 examples/ui-format/coverage.py --run
```

The driver cleans only the owned native target, runs instrumented selected tests plus the actual finite ui-format binary, then merges fresh profiles with the matching sysroot LLVM tools. It explicitly includes executed `editing.rs` and `main.rs`; production after external `mod tests;` declarations is measured. Dedicated test source and actual inline test-module ranges are excluded and recorded, not whole trailing file sections.

| Scope | Covered / executable production lines | Coverage |
| --- | --- | --- |
| framework-ui | 4140 / 4535 | 91.29% |
| ui-format | 229 / 239 | 95.82% |
| Pure controllers (separate gate) | 785 / 837 | 93.79% |

All exceed the 70% development gate. `coverage-summary.json`, full LCOV/JSON and per-file uncovered lines retain the scope. WASM-only DOM adapters/client bootstrap are outside native executable coverage and are exercised by actual Chromium hydrate/CSR. LLVM reports 130 unmatched dependency records (11 quick_xml, 119 tachys), zero first-party records; diagnostics are retained without suppressing warnings. This is development evidence, not publication or manual assistive-technology acceptance.

## Existing proofs and visual review

- `UI_FORMAT_OUTPUT_DIR=/private/tmp/s8-edit python3 examples/ui-format/check.py build`, then `browser.py`: PASS, original four cases, 177 formatter rows and 58 rich corpus rows per case. `edit-browser.py`: PASS, eight new cases; `edit-browser.json` explicitly records trusted clipboard, synthetic composition, realOsIme=false, retained initial DOM and released timers.
- Gallery build/core/breadth: PASS, four core and four breadth cases. Stream build/browser: PASS, six cases. Admin build/browser/compile proof: PASS, four browser cases; stale generated consumer intentionally fails with exit 101 and updated/hostile-label consumers pass.
- Unchanged `python3 examples/profile-ui/check.py`: PASS, including gateway/runtime/service proof, doctests/Clippy and customization reuse. Unchanged profile browser suite: PASS, eight root/mounted cases. No accepted profile source was edited.
- Playwright is taken from the installed package through `UI_FORMAT_PLAYWRIGHT`/`GALLERY_PLAYWRIGHT`. Stream/admin have hardcoded temporary paths; `/private/tmp/s8-edit/legacy.py` executes their unchanged proof logic with task-local path/target overrides only, preserving source SHA-256 records in `*-sources.json`. Profile uses supported environment overrides. No tools were installed.
- 156 screenshots retained: 16 new editor, 48 original formatter, 72 gallery, 12 stream, 4 admin and 4 profile. `artifact-manifest.json` has screenshot/source fingerprints and exact JS/WASM sizes. [French hydrate at 375 px/200%](/private/tmp/s8-edit/screenshots/edit-fr-hydrate-root-375-zoom2.png) and [English CSR desktop](/private/tmp/s8-edit/screenshots/edit-en-csr-mounted-1280-zoom1.png) show readable grayscale inputs and separated actions. Geometry asserts wrapped/same-row action gaps and zero horizontal overflow in every case.

## Dependencies and closeout

- No new package/version or mask/date/editor library. A direct WASM-only `js-sys = 0.3.106` edge (already locked/transitive, MIT OR Apache-2.0, MSRV 1.81) supports bounded JS-string admission; existing web-sys DOM feature flags were enabled. Cargo.lock changes by that one UI dependency edge only. Existing ICU 2.4.0 is Unicode-3.0/MSRV 1.88; phonenumber 0.3.10+9.0.33 is Apache-2.0/MSRV 1.76.0; Leptos 0.8.22 is MIT/MSRV 1.88. The workspace floor 1.96.0 passed.
- Root `framework-ui-format` forwards the already accepted crate feature through the facade, checked in both WASM modes. No framework-view, language/preferences, server, lib, SQL/schema, CLI/Idea, rich/stream implementation or other example source was changed. No other trial branch/output was consulted.
- Tooling: wasm-bindgen 0.2.129, Playwright 1.64.0, Chromium 156.0.8078.4. Synthetic composition is explicitly separate from real OS IME; packaged WebView and real OS IME remain unproved. Named-zone/DST conversion and reachability are outside this contract.
- Both owned compile targets and generated target metadata/empty fixtures were deleted; `target/` is absent. Workspace child logs were preserved in `workspace-child-logs/`. Final process audit found zero active task build/browser parents or owned target executables. Nothing is staged; scope/fingerprints are in `scope-audit.json` and `artifact-manifest.json`.
- KB validation retains the pre-existing 146 ERROR / 22 WARN baseline, with no added or removed diagnostics; `kb-before.log`, `kb-final.log` and `kb-comparison.json` retain the failing baseline. This receipt is the only `.agents` addition, pre-approved by the coordinator.
- No commit, push, publish, listener, mobile run or subsequent milestone. No product decision or shared-write request is outstanding. Stop for coordinator review.
