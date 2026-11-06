# Task 00039: Leptos text editor and source HTML

Status: implemented; local code/proof gate complete. Coordinator verification and owner UI acceptance pending.

## Design note

- Add independent `components::TextEditor`, `TextEditorOptions`, and `TextEditorUpdate` to stackpress-leptos. Every frui toolbar flag is opt-in, with matching command/select options; snake_case Rust names mirror camelCase props. A shared options carrier supports source-admin metadata without duplicating editor behavior. Value/default value, change/update callbacks and arbitrary hidden-input attributes remain component inputs.
- Root-local reactive HTML and code-mode state synchronize a hidden form input, editable div and source textarea. SSR emits the initial HTML and hidden value; hydration attaches to that same structure. Subsequent native contentEditable edits retain the browser caret. Browser APIs stay in a wasm32-only adapter, using `HtmlDocument::exec_command*` for the source command vocabulary and web-sys Selection/Range for the two custom range transforms. FileReader reads images locally; no fetch/upload path.
- Preserve frui's command/update order (`input`, then command), code-toggle callbacks, HTML code round-trip, prompt-based inserts, direction, file/gallery, fullscreen, preview, print and HTML download. Preserve upstream font-size command values and its active/inactive surface synchronization instead of introducing an editor library or extra policy.
- Add `components::HTML` for `frui/view/HTML`: stored HTML is inserted verbatim. Application authors own sanitization and content security (G43); existing opt-in sanitized RichContent remains its own API.
- Source-admin `EditorField` and `HtmlView` delegate to these shared components. Register compiled imports for `field.editor` and `view.html`; retain source positional argument order and existing metadata keys. Add missing named editor props without shifting existing positional slots.
- Lucide masters use the existing pinned commit `03f11e2d6adc319ea21e9a1d5820a99cd3767257`, 24-unit viewBox, 2-unit currentColor strokes, 16px toolbar display and control-owned names. Add source hashes to NOTICE only; license files/metadata stay untouched.
- Verification: native SSR/default/prop and generated-role tests; actual hydrate/CSR Chromium proof via cached Playwright and page.route only; three narrow test-verified review rounds; one final workspace gate plus compatible renderer features, generator freshness/consumer checks and >=70% executable-line coverage per changed crate. No listener/install/commit/push.

## Source anchors and baseline

- Read-only frui `src/form/TextEditor.tsx:10-55` (config/props), `:62-454` (state, commands and effects), `:464-1297` (component and hidden-input spread).
- Read-only Stackpress `packages/stackpress-schema/src/config/attributes.ts:3406-3644` maps `field.editor` to `frui/form/TextEditor`; `:4477-4497` maps `view.html` to `frui/view/HTML`, with no props. Frui `src/view/HTML.tsx:13-17` inserts raw HTML.
- Before: `framework-admin/src/codegen/source.rs:33` explicitly refuses the editor; the schema vocabulary knows HTML but has no available adapter. Source-client emission delegates to this catalog and fails on those selected roles.
- Approval: coordinator approved exactly Selection, Range, DocumentFragment, Text, DomTokenList, HtmlTextAreaElement, HtmlAnchorElement, File, FileList, FileReader, Blob, BlobPropertyBag, Url in the existing wasm32 web-sys dependency. No other manifest/lock changes authorized. Final proof must preserve Cargo.lock bytes and its 699 registry tuples.

## Implementation notes

- Public additions: `components::{TextEditor, TextEditorOptions, TextEditorUpdate, HTML}` (and component Props builders), `source::roles::{EditorField, HtmlView}`. All 31 toolbar flags are supported, plus value/default-value, callbacks, direction, ignored error and native hidden-input attributes. Existing APIs remain available. The metadata signature has 40 props; the registry budget expands additively from 32 to 64, tested at the boundary.
- Changed files: `crates/leptos/src/components/{mod.rs,core.css,text_editor/**}`; `crates/leptos/{Cargo.toml,README.md,docs/api/README.md,docs/api/text-editor.md,proofs/editor.py,proofs/editor/**}`; 35 pinned SVG masters and hash lines in `third-party/lucide/NOTICE`; admin `src/{codegen/source.rs,source/roles.rs,source/roles/editor.rs}` and `tests/{source.rs,source/editor.rs}`; schema `src/codegen/{registry.rs,source_roles.rs}` and `tests/source_emission.rs`; this receipt. No concurrent session/API/SQL/aggregate-owned file changed.
- Before → after: source editor/HTML emission failed availability checks → `codegen/source.rs:33-57` selects compiled `EditorField`/`HtmlView` imports. `source/roles/editor.rs:10` forwards authored props into the shared editor; `:117` delegates HTML directly. Aggregate source-client emission remains unchanged and is exercised by the generated consumer (native models/stores/client, SSR roles, hydrate and CSR).
- `TextEditor` at `crates/leptos/src/components/text_editor/mod.rs:206` initializes SSR HTML, retains hidden form naming/value and exact callbacks, and uses browser-only DOM adapters. `HTML` at `:447` inserts raw stored markup; no sanitizer added (G43). Existing opt-in sanitized content is unchanged. Source metadata definitions retain their positional slots, append named props and add the no-argument HTML role.
- Review 1 (complexity): extracted numeric table admission and direction transitions; corrected omitted-radix parseInt semantics, including hexadecimal dimensions. Preserved native execCommand input-event multiplicity. Tests: 10 narrow contracts plus eight real browser cases, records `20261106T040019.436056Z-809ebc8fb6ab` / `20261106T040015.402023Z-9d44820f0b41`.
- Review 2 (responsibility): separated Selection/Range transforms and local FileReader/download lifecycle from DOM command dispatch. Retained shared toolbar/API and source-only prop adaptation. Corrected Boolean source dir to show controls without an initial direction attribute; first toggle now matches frui. Ten contracts and eight browser cases pass, records `20261106T040222.494511Z-e4f61fb69427`, `20261106T040218.198106Z-5ebefca59b23`, `20261106T040335.998244Z-7fd978b7243b`.
- Review 3 (consistency/docs/comments): conventional formatting, explanatory mutation/selection/file comments, raw-HTML Rustdoc and API reference with a compiled example. Added finite proof-process cleanup and real late FileReader completion after root disposal. Eight browser cases, 30 doctests and 10 narrow contracts pass; owned Clippy is warning-clean. Records `20261106T040636.456253Z-c97e5c947446`, `20261106T040818.861498Z-4abab2871628`, `20261106T040904.580869Z-31090987639b`, `20261106T040632.256707Z-4517d0358131`.
- Safety/soundness review: no unsafe code, new networking, external dependency or global editor state. File readers release their once callbacks on terminal completion; late completions refuse disposed/code-mode roots. Blob URLs are revoked. Raw insertion is intentionally source-faithful, documented caller policy. Local native/Chromium evidence is not installed-host or cross-browser acceptance.
- Targeted follow-up: the final gate's stricter callback-time form read failed (record `20261106T040953.567026Z-24334e881965`): the reactive hidden value lagged onChange. Added a root-local hidden NodeRef and immediate form write. Closing code mode now preserves Frui's distinction between authored form source during the callback and browser-normalized callback/currentValue HTML; empty direction uses the original `|| 'ltr'` fallback. Eight browser cases and 10 native contracts pass (`20261106T043203.757951Z-a6b8061534f6`, `20261106T043254.303539Z-3da9de039110`). Both source adapters leave missing/null content empty instead of inheriting Typed text's dash placeholder; a regression covers all four empty carriers in editing and display. The gate continued from its failed boundary; final owned native/WASM/browser/docs/coverage checks were refreshed after that correction (`20261106T045518.657829Z-2123318e9549`). No second full workspace sweep.

## Final gate

| Check | Result |
| --- | --- |
| Workspace fmt; default and compatible Clippy, `-D warnings` | Passed; final owned fmt/Clippy also refreshed |
| Workspace nextest, default / compatible | 1,053 passed / 1,209 passed; 90 / 98 skipped |
| Workspace doctests, default / compatible | 978 passed / 983 passed |
| Workspace Rustdoc, default / compatible | Passed with `RUSTDOCFLAGS=-D warnings`; final owned docs/doctests refreshed |
| Native generated source client/stores/SSR and hydrate/CSR consumers | Passed, including repeat-generation freshness |
| Cached intercepted Chromium | Eight final cases passed; no page/hydration errors |
| Hydrate / CSR Clippy | Passed separately, `-D warnings`; no all-features renderer build |
| Existing Typed workspace/admin freshness and aggregate build-host consumer/check | Passed |
| Final instrumented nextest | 196 passed, one ignored process-only case |

Stable native executable-line coverage, every measured `src/` file in each changed crate, no added filename exclusions:

| Crate | Covered / executable | Coverage |
| --- | ---: | ---: |
| stackpress-leptos | 6,215 / 6,988 | 88.94% |
| stackpress-framework-admin | 3,027 / 3,692 | 81.99% |
| stackpress-framework-schema | 5,455 / 6,439 | 84.72% |

Scope: Leptos ssr/stream/rich/format; admin source-ssr/codegen/format; schema source/codegen (including providers). WASM-only DOM code is outside native line coverage and has the separate real browser evidence above. Development's 70% floor passes per crate; no publication or all-platform claim.

Permanent benchmark store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs`. These records retain exact commands/features and Cargo stages; local per-step detail is `target/00039/{gate.json,final-source-parity.json,benchmark-index.json}`.

| Final-series record | Outcome | Command s | Cargo stages s |
| --- | --- | ---: | ---: |
| `20261106T040953.567026Z-24334e881965` | Workspace sweeps passed; new DOM regression stopped browser stage | 1,268.342 | 437.930 |
| `20261106T043454.050749Z-6cc72105ec17` | Remaining checks and instrumented tests passed; report selected no packages | 504.984 | 440.080 |
| `20261106T044443.800607Z-317eab6dd811` | Final owned checks passed; reporter rejected its help-listed features flag | 7.761 | 3.290 |
| `20261106T044548.181802Z-452568fb5c93` | Correct package-selected export and per-crate floors passed | 3.298 | unavailable |
| `20261106T045518.657829Z-2123318e9549` | Latest owned native/WASM/browser/docs/coverage refresh passed | 125.357 | 72.680 |

Task-only recorder counters: 0 → 42 attempts (31 passed, 11 resolved failed iterations; none running), 0 → 22 compilation-emitting attempts, zero declared cold workspace rebuilds. Accumulated command time 3,002.956 s; observed Cargo stages 1,629.110 s. Concurrent command totals are not calendar elapsed time or a speed comparison. Initial compile/consumer labels inherited warm from the wrapper despite first-use outputs; later stages reuse retained outputs. Sampled post-step target peak: 10.75 GiB. Near the 12 GiB cap, only completed task compiler outputs were cleared (records `20261106T041921.364546Z-0e867c44b905`, `20261106T042949.725866Z-40838b301392`); sources, browser artifacts and benchmark logs were retained.

Cargo.lock is byte-identical to HEAD and the initial baseline, SHA-256 `e480322a6ca7f44345a54d0a0fe6ce09daac6f46f0782d7a01b442684b6b7bb0`; all 699 registry tuples/checksums are identical. Exactly the approved 13 web-sys feature names were added under the existing wasm32 dependency. All 43 Lucide master hashes (35 added) match NOTICE. LICENSE, FEATHER-LICENSE and other license metadata are unchanged. Native target: `target/00039/build`; independent consumer: `target/00039/editor-consumer-build`; sccache, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, four bounded build jobs. No dev listener or install was used.

## Unavailable source-role inventory

With the existing optional `format` adapters selected, 44 canonical roles remain unavailable. The following field names are unavailable in **field, filter and span** placements (20 names):

`code`, `color`, `country`, `currency`, `email`, `file`, `image`, `json`, `markdown`, `mask`, `radio`, `small`, `suggest`, `datelist`, `datetimelist`, `filelist`, `imagelist`, `numberlist`, `timelist`, `fieldset`.

The following display names are unavailable in **view and list** placements (24 names):

`chars`, `comma`, `country`, `currency`, `email`, `formula`, `line`, `relative`, `transform`, `words`, `carousel`, `code`, `color`, `film`, `json`, `markdown`, `rating`, `list`, `ol`, `spread`, `tabular`, `ul`, `fieldset`, `rel`.

Thus 108 canonical placement/name pairs remain unavailable before aliases. Without `format`, additionally unavailable: field/filter/span `number`, `price`, `integer`, `date`, `time`, `datetime`, `phone`; view/list `number`, `price`, `phone` (54 canonical roles / 135 placements). Aliases normalize through `registry.rs::canonical_role` to these same entries. Inventory anchors: `registry.rs` FIELD_NAMES/VIEW_NAMES, admin `codegen/{catalog.rs,formatted.rs,source.rs}`. None of these other roles was implemented.

Existing partial-adapter limits remain: source select styling/object options, nondefault source date formats and metadata style objects require application adapters. Metadata-only schema definitions and the separate Typed catalog retain their established behavior.

## Gaps and evidence limits

- Coordinator verification and owner visual acceptance remain pending. No commit/push, publish/deploy, development listener or new installation.
- Browser proof uses only the required cached Playwright and headless Chromium with intercepted requests, eight hydrate/CSR × light/dark × 375/1280 cases. It exercises real editor commands, keyboard/value/form round-trip, selection transforms, prompts, local images/gallery, native save/fullscreen and queued reader disposal. Preview/print host calls are observed in-process. No other browser or installed Tauri target is claimed.
- Agent Workspace validation has inherited **147 errors / 39 warnings**, including upstream-relative missing links and the pre-existing 513-line task 00036. None concerns this receipt; unrelated KB files were preserved. This is not clean KB validation.
- Other branches/worktrees and the actual blog checkout were not inspected or run. The supplied Article role shape was proved through local normalization, actual aggregate emission and compiled consumers. Browser JSON/screenshots are under `target/00039/editor-proof/`; rendered 16px toolbar icons were inspected in light/dark and mobile/desktop layouts.

## Verification notes

Coordinator: I reviewed and committed this as `60a18a9`, then merged it into main as `bc03c06` (no-ff). The review covered:
- the registry prop budget, raised from 32 to 64;
- the editor/HTML emitter mapping;
- the 13 approved web-sys features;
- Lucide NOTICE hash lines, added with the licence text untouched;
- raw HTML documented under G43.

Fresh-target gate `/private/tmp/editor-verify/run.sh`, 2026-11-06:
- 699 registry tuples identical; Cargo.lock unchanged from `a802483`;
- fmt; default and compatible Clippy with `-D warnings`;
- nextest: default 1,052 passed / 91 skipped; compatible 1,209 passed / 99 skipped;
- compatible doctests; Rustdoc with `-D warnings`;
- wasm32 Clippy for leptos and admin source, hydrate and CSR;
- aggregate graphs, eight consumers and build-host `--check`;
- generated-workspace, generated-admin and API checkpoint freshness;
- aggregate docs and proof ledger;
- `crates/leptos/proofs/editor.py all`: generated consumers plus 8 browser cases.
