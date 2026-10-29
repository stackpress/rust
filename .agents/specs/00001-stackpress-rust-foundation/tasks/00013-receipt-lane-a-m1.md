# Task 00013 Lane A M1 receipt: ordinary components

Implementation evidence for [task 00013](00013-slice-8-more-ui.md) and the [M1 gate](../slice-8-design-verification.md), 2026-10-27. Coordinator released **Lane A only**, plus the two screenshot fixes and textarea security follow-up. Branch `trial/3-s8-m0`; required `git merge trial/3` fast-forwarded M0 commit `d5cc21d` to current base/HEAD `7797d94925b7f3279cfd0256428724a9c4ef8073`. No commit/push performed by this effort.

## Result and ownership

**Lane A M1 implementation proofs pass** in the native/Chromium scopes below. Ready for coordinator re-verification and owner rendered review; no M2 work is released or claimed. Lane B locale/theme/flash/policy projection remains separate. No edit to framework-language, framework-view, accepted profile-ui handwritten source, root Cargo/lock/facade, lib/server/CLI/Idea/schema/SQL. The gallery's own finite web host adds `img-src 'self'` for its explicitly managed raster fixture and the M1 build identity described below.

Only this pre-approved receipt is added under .agents. No index/tracker/context promotion, KB publishing, listener, tool installation, mobile/package work or new shared request. M0's accepted synchronous/async Owner/Head/replay contracts remain unchanged and their tests still pass.

## Inventory and behavior delivered

Read with [the accepted component inventory](../slice-8-design-inventory-components.md). React prop/helper aliases and CSS class snapshots are not a TypeScript ABI promise; explicit typed Rust props/slots remain the adaptation.

| Inventory families | M1 implementation and evidence |
| --- | --- |
| Existing Input / FieldControl | Additive required/readonly props with existing defaults; native label/help/error association and controlled values. Existing Button/Alert/Link/List/layout/notification contracts unchanged; profile regressions pass. |
| Checkbox, Radio, Switch | Root-owned bool/nullable selection; native name/value/required/disabled, associated labels/errors and Space/arrow behavior. False/None/selected empty/zero values remain distinct. Switch has native checkbox behavior with switch role/checked state. |
| PasswordInput | Same controlled native input node, initially concealed; named toggle/pressed/control association, return focus, no duplicate plaintext field or form key. Gallery initial password is empty and transient user text is never serialized into its page seed or inspection output (length only). Caller-owned public SSR values are not a credential store. |
| Textarea | Controlled native multiline text, required/readonly/disabled, native row bounds, verbatim native input/IME/paste semantics. Initial HTML RCDATA is explicitly escaped, including leading-newline compensation on SSR; security proof below. |
| Select | Native single/multiple, explicit mode over root signals, <=100 immutable unique options, each value/label <=1024 bytes. Disabled options, placeholder and repeated native form keys. None is different from Some(""); reserved disabled placeholder is never submitted. Unknown/disabled native-event values are not admitted. Searchable/dynamic combobox behavior stays M2. |
| Badge, Card, Progress | Explicit CardHead/Title/Description/Body/Foot and action slots; named section, escaped values. Native labelled progress has checked finite inclusive bounds and explicit indeterminate versus zero. No NaN/infinity/invalid-to-zero fallback. Progress's labelled wrapper replaces source reflection/container machinery. |
| Color, EmailLink, PhoneLink, Image | ColorView accepts #RGB/#RRGGBB and keeps a visible non-color value. Dedicated typed mail/tel links preserve generic safe_href. Image accepts declared host-managed same-origin raster assets only, with required alt (explicit empty means decorative), dimensions and metadata byte ceilings. |
| Metadata, Spread, Tabular, Tags | MetadataView uses a semantic definition list; bounded readonly values preserve empty/zero/duplicates where meaningful. Explicit inline/line separator mode; tags are a list of escaped badges. Tabular validates unique rows/columns/exact widths and renders caption, column scopes, scroll region and empty state. No editable collection, paging, inferred schema or data operation. |
| Rating, TextOverflow, TextTransform, YesNo | RatingView has finite checked bounds, <=100 stars, nearest/ceil/floor, optional remainder and visible exact numeric value. UTF-8-safe character/word overflow, explicit zero count/ellipsis; full source remains real assistive text, short visual text is aria-hidden. Fixed CSS transform enum preserves source DOM. Typed boolean display has explicit localizable true/false phrases. |

Library sources: [ordinary controls](../../../../crates/leptos/src/components/inputs.rs), [checked display values](../../../../crates/leptos/src/components/display_values.rs), [presentation](../../../../crates/leptos/src/components/presentation.rs), [thin select DOM reader](../../../../crates/leptos/src/components/input_dom.rs). [Native decision/semantic tests](../../../../crates/leptos/src/components/core_tests.rs) exercise bounds, invalid input, exact empty/zero/Unicode, checked URL causes and actual SSR structure. Public constructor failures remain safe; image path failures retain concrete ViewError as source without interpolating its Display.

Intentional policies: ASCII mailbox subset with URI-sensitive local-part bytes percent-encoded (including literal %); no mail query/header injection. Phone is a canonical international token shape, not region/number-validity metadata or M3 phone formatting. Color is checked hex, not arbitrary CSS. Images require a checked local PNG/JPEG/WebP/AVIF path, declared 1..=1 MiB and 1..=4096 pixels per dimension. The host must enforce real MIME/bytes via its asset manifest: an img element does not meter a download. No external/data/blob/SVG image source is admitted. Source flags that hide a color's only text are replaced by the accepted non-color-only presentation policy.

Display collections cap 100 entries, text values 4096 bytes, table columns 20; tables reject duplicate/missing row identities, duplicate labels and inconsistent widths before rendering. Ordinary input validation remains application/host-owned; native FormData omission for unchecked/disabled controls is not an authorization rule. Text limits count Unicode scalars or whitespace-separated words, not grapheme clusters; character prefixes preserve whitespace. Full specialized decimal/date/phone/caret formatting, source Box/palette machinery, collection inputs, navigation/overlay breadth, sticky/striped/spanned table options and generated CRUD remain at their accepted later gates.

## Coordinator screenshot fixes

[CORE_STYLES](../../../../crates/leptos/src/components/core.css) is an explicit neutral opt-in alongside ADVANCED_STYLES. ActionGroup supplies wrapping layout with .75rem gap; language, primary gallery, dialog and card/form action groups now have separation. Geometry asserts at least 8 CSS pixels between adjacent language/primary actions; viewport screenshots show separated dialog actions too. The accepted base styles.css is unchanged.

The earlier fullPage capture extended below a viewport-fixed native backdrop. The new proof separates those coordinate spaces: at 375x812, computed ::backdrop is fixed with all four insets 0px, every viewport-corner hit test targets the modal, and the viewport PNG is exactly 812 pixels tall while document height is 3661 pixels. A fullPage image includes content outside that viewport and cannot establish backdrop coverage there.

The separate background-scroll issue is fixed with scoped `:root:has(.sp-dialog:modal)` / body overflow and overscroll rules in ADVANCED_STYLES; any nested modal keeps the lock. Native modal inertness already blocks background focus/interaction. Actual wheel/PageDown cannot change page scroll, attempted background focus stays in the dialog, and pointer events at viewport corners target the native modal. Closing/unmounting the final modal releases the lock. Dialog docs explicitly require ADVANCED_STYLES for document scroll locking; native focus/inertness is independent of optional styles. No global mutable counter, mutex or new listener is used.

`{root,mounted}-{hydrate,csr}-dialog-viewport-375.png` is the new viewport proof, in addition to the previous fullPage/mobile/zoom captures. Final desktop hydrate and mobile viewport-modal images were visually inspected; owner acceptance and screen-reader review remain separate.

## Textarea security fix and pinned-source audit

This is a security fix, not only presentation. The initial corpus is `\n</textarea><script>globalThis.pwned=1</script>&"'\u{2028}` with actual LF and U+2028. Native SSR tests require escaped terminators/entities and no active script. Before JavaScript, the real browser's textarea value is exactly the corpus. It also round-trips after real hydration/refresh and CSR, with no script execution or breakout script element. CSP remains extra defense; exact value/element assertions independently establish containment.

Leptos **0.8.22** resolves **tachys 0.2.19** here. Exact cached primary-source findings are retained in `raw-text-source-evidence.txt`:

- `tachys-0.2.19/src/html/element/elements.rs`: script line 374, style 390 and textarea 406 have ESCAPE_CHILDREN=false; ordinary title at 416 is true. The element-definition macro assigns that boolean to the const at line 81.
- `tachys-0.2.19/src/html/element/mod.rs:426-450`, HtmlElement RenderHtml: supplied inner_html is inserted directly; otherwise children render with E::ESCAPE_CHILDREN. Thus ordinary text children in textarea/style/script are emitted raw. The application/json MIME attribute does not change script-child escaping.
- A passing native characterization deliberately demonstrates all three raw cases without serving their unsafe output, and verifies ordinary title children are escaped. This passing counterexample is not presented as a safe raw-text API.
- `leptos_meta-0.8.7/src/lib.rs:229-265`, ServerMetaContextOutput::inject_meta_context, gets title.as_string and pushes it directly between title tags. Stackpress's existing synchronous/async final plain-text title formatters already escape this boundary; the accepted routing test includes a hostile closing-title payload. These renderer bodies/tests were not changed.

The new Textarea escapes &, < and > exactly once for initial RCDATA, then uses controlled value-property updates for edits. No raw hostile string reaches inner_html. Native document parsing strips one initial LF after a textarea start tag; CSR fragment construction does not. Actual Chromium parsing records document="One", fragment="\nOne" for the same initial LF. SSR alone adds a compensating LF, preserving identical parsed values without adding one in CSR.

Other exposed boundaries were checked: styles in LayoutProvider/gallery are compile-time trusted CSS, not user props; ColorView style accepts only checked hex. Public JSON page/replay scripts already use escape_script_json, including script terminators and U+2028/U+2029, and actual Resource replay/hostile Title tests still pass. No generic unsafe Style/Script passthrough or expression evaluator was added. No dependency upgrade/upstream patch/report was made; the coordinator can use the retained source evidence for an upstream report.

## Real browser and build identity

[One CoreGallery source](../../../../examples/ui-gallery/shared/src/core.rs) renders all selected families. The client copies early native properties into root-owned signals before hydrate; it does not read browser locale/theme storage. All ordinary control nodes plus the focused original draft survive hydration; complete initial parsed app HTML remains identical. A deliberate Inspect state action reads live controller state without requiring SSR text summaries to change during initial hydration. Reset clears the inspection, preserves explicit false/None/empty values and demonstrates programmatic property reconciliation.

The gallery build identity is **s8-m1**, preserving revision-1 DTO shapes. The component tree changed from M0, so retaining s8-m0 would incorrectly admit stale assets. The pure decoder rejects an s8-m0 seed before creating a root or starting work. Legacy profile-ui build identity/source is untouched.

`gallery-browser.json`: **4/4 pass**, hydrate/CSR at root and /demo/v1. Native FormData proves selected empty/repeated multiple values, omitted disabled/unchecked values and retained readonly data. Password toggle retains node/value/focus. Space/radio arrows, partial/final composition events and caret, multiline keyboard edits, invalid/help/error relationships, programmatic reset and known/indeterminate progress pass. Native role/name queries cover switch, radio group, progressbar, table and card region. Image loads only the declared 68-byte same-origin PNG; mail/tel attributes and escaped visible values are exact.

Existing locale refresh/failed persistence, prefix/cookie/default, French/dark settings, zero initial hydrated Resource/preference acquisition, sanitizer parity, Resource cancellation and unmount proofs remain in the same run. There are zero unexpected console errors/requests, plus exactly one expected 403 diagnostic per mode. These are actual browser composition events, not a complete OS IME/input-method, assistive-technology or packaged WKWebView proof.

Browser wrapper retains a **120-second process-group kill/reap watchdog**, 10-second action/acquisition/native child deadlines and finally cleanup. All traffic is intercepted on gallery.invalid, with all other origins rejected. Native hang-class Owner/Resource/panic fixtures remain under their existing 10-second child watchdog. No localhost listener or devmetrics process is involved.

Artifacts are under `/private/tmp/s8-m0/m1/`, including **24 screenshots**: four modes/mounts times -1280, -375, -dialog-375, -375-zoom2, -1280-zoom2, plus the four new -dialog-viewport-375 images. Browser: Chromium 156.0.8078.4 / Playwright 1.64.0. Rust 1.96.0 and wasm-bindgen CLI 0.2.129; no installed tool changed. `artifacts.json` records current asset/screenshot byte sizes, gzip sizes and SHA-256. Debug WASM/glue sizes are proof artifacts, not optimized release budgets.

Reproduce with `GALLERY_OUTPUT_DIR=/private/tmp/s8-m0/m1 python3 examples/ui-gallery/build.py`, then the same output variable and `GALLERY_PLAYWRIGHT=<installed-playwright-package> python3 examples/ui-gallery/browser.py`. The default output path remains compatible with the existing M0 proof wrapper; no machine-specific Playwright path is committed.

## Verified review rounds

1. **Logic/complexity:** checked finite numeric/URL/option/collection types, explicit modes and decision tables; removed a ceremonial option-clone helper; corrected character-versus-word whitespace and zero-count behavior. Native round1-tests.log passes. First semantic/security tests caught raw textarea children; escaping was fixed before claiming success.
2. **Responsibility/abstraction:** pure data admission stays in display_values/inputs; the thin DOM select reader only copies values. CoreGallery is separate from the gallery's transport/locale adapters. Inspection reads actual adopted root state while initial SSR DOM remains intact; URL causes stay concrete. round2-tests.log and round2-clippy.log pass.
3. **Consistency/Rustdoc/comments:** explicit exports, native associations, constructor/error/platform docs, typed examples and full accessible overflow text; checked private-field and state-transition comments. Native round3-tests.log, final doctests/Rustdoc and native/WASM Clippy pass. New optional styles preserve the accepted base stylesheet.
4. **Safety/soundness:** initial RCDATA breakout/leading-LF corpus and pinned raw-text audit; same-node password reveal; enabled-option admission; no credential seed; viewport/backdrop/scroll/nesting/unmount; actual Send async regression cases; stale-gallery identity rejection. round4-tests.log plus the final native/browser/coverage checks pass. No further material finding remains in this released scope.

## Final verification

All 17 final-driver steps pass on the final s8-m1 source:

| Check | Result / retained log |
| --- | --- |
| cargo fmt --all -- --check | PASS; m1-fmt.log |
| framework-ui --features ssr tests and doctests | 38 native tests + 5 doctests, no failures; m1-ui-ssr.log |
| Selected native ssr,rich Clippy / Rustdoc | -D warnings, PASS; m1-clippy.log / m1-rustdoc.log |
| cargo +1.96.0 check, selected native crates/all targets | PASS; m1-msrv.log |
| Actual WASM hydrate / CSR Clippy | -D warnings, both PASS; m1-wasm-{hydrate,csr}-clippy.log |
| Extended root facade WASM hydrate / CSR check | Both PASS; m1-facade-{hydrate,csr}.log |
| Actual frontend + extended facade normal,no-proc-macro trees | All four PASS; no server/DB/CLI/Idea/Tauri/Tokio runtime; m1-{wasm,facade}-{hydrate,csr}-tree.log |
| python3 examples/profile-ui/check.py, unchanged | PASS; m1-profile-check.log / profile-check-final/summary.json |
| cargo test --workspace --all-targets --locked | 542 passed, 0 failed, 75 explicitly ignored; m1-workspace-tests.log |
| cargo test --workspace --doc --locked | 884 passed, 0 failed; m1-workspace-doctests.log |
| cargo clippy --workspace --all-targets --locked -- -D warnings | PASS; m1-workspace-clippy.log |
| Finite gallery build + actual intercepted Chromium | All build steps and 4 mode/mount proofs PASS; build.log / gallery-browser.json |
| Unchanged profile-ui intercepted browser regressions | 8/8 PASS; profile-browser.log / profile-browser/summary.json |

The workspace total includes watchdog fixture child results printed into their parent logs; 75 ignored entries include child-only, platform and external SQL gates. The separately selected rich-feature tests pass again during final coverage; the workspace default feature run is not substituted for them.

Leptos common client packages leptos_server/server_fn remain in the frontend tree; the forbidden-runtime check concerns Stackpress server, SQL/DB, CLI, Idea, Tauri and Tokio execution dependencies.

Exact command arguments, exits and timings are in verification.json; verify.py is the finite driver. Every compiling Cargo invocation uses CARGO_INCREMENTAL=0 and CARGO_PROFILE_DEV_DEBUG=0, native target/s8-native or WASM target/s8-wasm. Render modes are separate; no framework-ui --all-features. External PostgreSQL/MySQL gate variables are removed for the workspace run: their native integration cases remain explicitly ignored, with no database listener/setup/connection implied.

Native executable production-line coverage, per changed native crate (development gate >=70%):

| Crate | Covered / executable lines | Coverage |
| --- | --- | --- |
| framework-ui | 1703 / 1735 | 98.16% |
| gallery-shared | 221 / 246 | 89.84% |
| gallery-web | 203 / 225 | 90.22% |

Installed cargo-llvm-cov 0.8.6, Rust 1.96.0, native ssr,rich. coverage.py uses --no-rustc-wrapper and fresh profiles, then matching LLVM export to include executed first-party example binaries. Dedicated tests.rs/core_tests.rs and inline cfg(test) modules are excluded; no production branch is excluded to raise a number. m1-coverage-full.json/.lcov and m1-coverage-summary.json retain per-file counts/uncovered lines. The thin wasm-only client, select DOM/event reading, early DOM adoption and browser fetch/history modules have actual Chromium evidence, not a native 100% claim; policy/decoders remain native-tested. Publication's 90% gate is not claimed.

LLVM emits 78 unmatched dependency function records (77 tachys, 1 memchr) when merging the independently executed test/binary objects. A serial --dump --num-threads=1 diagnostic export succeeds and names zero first-party mismatches; retained in coverage-diagnostics.log/.json. This is reported as a tooling limitation, not suppressed or described as a warning-free coverage export.

No dependency or version was added: Cargo.lock/root manifest/facade are unchanged. Only already-locked web-sys 0.3.106 feature flags for select/option/collection/textarea DOM and style declaration APIs are added (MIT OR Apache-2.0, declared MSRV 1.81). Leptos/tachys/leptos_meta remain 0.8.22/0.2.19/0.8.7 (MIT; Leptos declared MSRV 1.88); named workspace MSRV 1.96 checks pass. No ICU/phonenumber/editor or other integration was adopted.

Both task-owned compiled targets were deleted with cargo clean after the final browser/coverage export: Removed 2661 files, 1.1GiB total; Removed 3611 files, 845.5MiB total. The 34 finite workspace regression child logs were copied byte-for-byte to regression-child-logs/ before removing the now-empty target/m0 and CLI fixture directories. cleanup.json records sizes/hashes and confirms target/ no longer exists. No target outside this worktree was read, reused or deleted.

## KB and handoff

The agent-workspace validator exits 1 both before and after this receipt: 146 existing missing TypeScript sibling links and 22 warnings. The final sorted ERROR/WARN set is exactly identical, with no added or removed diagnostic; kb-before.log, kb-final.log and kb-comparison.json retain the comparison. No unrelated KB repair is included. All new receipt links resolve locally; only the pre-approved receipt is added under .agents.

This receipt does not complete Lane B, M2, full Slice 8 or all accepted visual controls. No new shared change or product decision is requested. Coordinator review/owner rendered acceptance are next; stop at Lane A M1.
