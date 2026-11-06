# Task 00040: Remaining Leptos source roles

Status: implemented and locally verified; coordinator review and owner acceptance pending.

## Design note

- Work in source-role groups: simple displays, inputs, collections, rich roles. Preserve original `attributes.ts` names, positional props, fixed attributes and import identities; aliases and field/filter/span/view/list derivation remain schema-owned.
- Reuse shared `Input`, `Radio`, `TextTransform`, existing list/choice state and source overflow semantics wherever their contracts match. Source-specific components must retain upstream permissive values rather than inheriting Typed UI admission policies. Shared Leptos owns behavior; admin owns metadata/draft adaptation and generated aggregate imports.
- File/image controls expose Frui's app-supplied upload callback and URL completion; no built-in upload or added network path. Suggest's `remote` attribute is not a fetch implementation in Frui.
- Initial dependency batch sent to coordinator: cached CodeMirror/Shiki/Inputmask/markdown-to-jsx assets, exact 69,465-byte country/currency source data, and Formula's evaluator choice. G55 below resolved this batch while independent roles proceeded.
- Follow the existing pinned Lucide masters and NOTICE hashes. Reuse existing masters where possible. LICENSE/FEATHER-LICENSE and license metadata are untouched.
- Narrow recorded tests during editing; three test-verified review rounds, then one full final gate and actual aggregate source_client SSR/hydrate/CSR consumers plus intercepted Chromium in both themes at 375/1280. Per-crate coverage floor is 70%. No listener, install, commit/push or other worker-owned changes.

## Owner decision G55

- Coordinator relayed G55 (main `00573a8`): no bundled JS libraries; use locked pulldown-cmark Markdown, plain monospaced code/JSON textareas with Tab handling and plain code views; port documented common mask patterns; pure arithmetic formulas (`+ - * / %`, parentheses, unary minus, first-only templates, 0 on parse errors), with IEEE Infinity/NaN displays. Non-arithmetic JavaScript is unsupported. Exact country/currency data vendoring is approved. Record role differences with upstream anchors. No web-sys addition is approved or required so far.
- Follow-up decisions: lossless `colors.ts` is approved too; JSON passes strings unchanged, with existing source conversion for structured values and no added validation/pretty-printing. The aggregate website's root-seeded offline consumer-lock refresh is approved; the same exception applies individually to another consumer only if this exact dependency edge requires it.

## Role receipt

Base `60a18a9`; read-only upstream roots `/Users/cblanquera/server/projects/stackpress/{frui,stackpress,idea}`. A anchors are lines in `stackpress/packages/stackpress-schema/src/config/attributes.ts`; Frui anchors below are paths under `frui/src/`, named exports matching the filename. Field entries apply to field/filter/span; view entries apply to view/list (108 placements).

E = `framework-admin/tests/source/remaining.rs::source_all_44_remaining_roles_emit_and_render_in_every_placement`; every row also compiles in the actual aggregate-generated consumer. B = `leptos/proofs/source_roles/browser.cjs` interactive proof; T = named family in `leptos/src/components/source/tests.rs`.

| Role | Admin adapter → shared component | Original anchors | Test |
| --- | --- | --- | --- |
| field.code | RichField → SourceCodeEditor | A1272; form/CodeEditor.tsx | E, B, T code_tabs |
| field.color | ColorField → SourceColorInput | A1327; form/ColorInput.tsx | E, B, T color_conversion |
| field.country | CountryField → SourceCountrySelect | A1377; form/CountrySelect.tsx | E, B, T dataset_lookups/missing_selection |
| field.currency | CountryField → SourceCurrencySelect | A1457; form/CurrencySelect.tsx | E, B, T dataset_lookups |
| field.email | InputField → existing Input | A1697; form/Input.tsx | E, B, T input_appearance |
| field.file | FileField → SourceFileInput | A1728; form/FileInput.tsx | E, B, T file_queue/upload_tickets |
| field.image | FileField → SourceFileInput | A1806; form/ImageInput.tsx | E, B, T file_queue/upload_tickets |
| field.json | RichField → SourceCodeEditor | A1970; form/CodeEditor.tsx | E, B, T code_tabs; structured JSON contract |
| field.markdown | RichField → SourceMarkdownEditor | A2012; form/MarkdownEditor.tsx | E, B, T markdown |
| field.mask | RichField → SourceMaskInput | A2051; form/MaskInput.tsx | E, B, T common_masks/mask_keyboard |
| field.radio | RadioField → SourceRadio/existing Radio | A2511; form/Radio.tsx | E, B, T radio_visual_flags |
| field.small | SmallField → SourceSmallInput | A3052; form/NumberInput.tsx | E, B, T color_conversion |
| field.suggest | SuggestField → SourceSuggestInput/existing Input+Popover | A3132; form/SuggestInput.tsx | E, B |
| field.datelist | SourceTextListField → SourceTextList | A1568; form/TextList.tsx | E, B, T file_queue |
| field.datetimelist | SourceTextListField → SourceTextList | A1648; form/TextList.tsx | E, B, T file_queue |
| field.filelist | FileField → SourceFileInput | A1767; form/FileList.tsx | E, B, T file_queue/upload_tickets |
| field.imagelist | FileField → SourceFileInput | A1837; form/ImageList.tsx | E, B, T file_queue/upload_tickets |
| field.numberlist | SourceTextListField → SourceTextList | A2248; form/TextList.tsx | E, B, T file_queue |
| field.timelist | SourceTextListField → SourceTextList | A3733; form/TextList.tsx | E, B, T file_queue |
| field.fieldset | FieldsetField → SourceFieldset | A3816; stackpress-view/src/transform/form/fieldset/{single,multiple}.ts | E, B |
| view.chars | existing OverflowView → source_overflow | A4008; view/TextOverflow.tsx | E, T overflow |
| view.comma | SpreadView → SourceSpread | A4205; view/Spread.tsx | E, T lists |
| view.country | CountryView → SourceCountry | A4236; view/Country.tsx | E, T dataset_lookups |
| view.currency | CountryView → SourceCountry | A4307; view/Currency.tsx | E, T dataset_lookups |
| view.email | EmailView → SourceEmailLink | A4409; view/EmailLink.tsx | E, T color_and_mail |
| view.formula | FormulaView → source_formula | A4456; view/Formula.tsx | E, B, T formula_arithmetic/deep_nesting; formula_display contract |
| view.line | SpreadView → SourceSpread | A4668; view/Spread.tsx | E, T lists |
| view.relative | RelativeView → source_relative | A5025; view/DateFormat.tsx | E, T relative_thresholds |
| view.transform | TransformView → existing TextTransform | A5235; view/TextTransform.tsx | E |
| view.words | existing OverflowView → source_overflow | A5286; view/TextOverflow.tsx | E, T overflow |
| view.carousel | MediaView → SourceImageCarousel | A3913; view/ImageCarousel.tsx; base/Carousel.tsx | E, B, T carousel_boundaries |
| view.code | RichView → SourceCode | A4039; view/Code.tsx | E, T code_tabs |
| view.color | ColorView → SourceColor | A4134; view/Color.tsx | E, T color_and_mail |
| view.film | MediaView → SourceImageFilm | A4621; view/ImageFilm.tsx | E, B |
| view.json | RichView → SourceCode | A4532; view/Code.tsx | E; structured JSON contract |
| view.markdown | RichView → SourceMarkdown | A4783; view/Markdown.tsx | E, T markdown |
| view.rating | RatingView → SourceRating | A4963; view/Rating.tsx | E, T rating |
| view.list | ListView → SourceList | A4746; view/List.tsx | E, T lists |
| view.ol | ListView → SourceList | A4875; view/List.tsx | E, T lists |
| view.spread | SpreadView → SourceSpread | A5048; view/Spread.tsx | E, T lists |
| view.tabular | TabularView → SourceTable | A5087; view/Tabular.tsx | E, T lists |
| view.ul | ListView → SourceList | A5258; view/List.tsx | E, T lists |
| view.fieldset | FieldsetView → SourceFieldsetDisplay | A5351; stackpress-view/src/transform/{view,list}/fieldset/{single,multiple}.ts | E, B |
| view.rel | RelativeView → source_relative | A5002; view/DateFormat.tsx | E, T relative_thresholds |

## Public additions and behavior

- Shared `components::source` exports are documented in [source-roles.md](../../../../crates/leptos/docs/api/source-roles.md). New admin adapters are the table's names plus `SourceUploader` and `fieldset_context`; generated fieldsets receive actual child descriptors/renderers. Existing Input gains optional class/style props and empty-association omission; existing TextListField, Radio, TextTransform, Popover, textarea SSR seeding and source overflow conversion are reused. Source definitions preserve positional argument order, fixed props and `component.from` anchors (`framework-schema/src/codegen/source_roles_remaining.rs`).
- G55 differences and exact supported mask grammar are enumerated per role in the API reference. Code/JSON have no syntax coloring or JSON validation; Markdown uses pulldown-cmark with raw HTML retained; formulas support arithmetic only and preserve first-only replacement and IEEE results. Relative native dates use the existing chrono edge (numeric milliseconds/RFC3339/date-only UTC); browser-specific spellings have no native parser. This limitation is explicit, not a claim of complete JS Date/locale support.
- Source oddities retained: ISO2 selector/full-record callbacks versus ISO3 displays; currency selector USD flag override; unused size flags; no added suggest fetch; unnamed file pickers plus URL hidden values and app-owned upload hook; captured upload queue/array behavior; stable collection tombstones and text-only date/number/time list inputs; carousel `auto` is overflow, no timer, and its frame prop is dropped upstream. Anchors: Frui form/{CountrySelect,CurrencySelect,FileInput,FileList,ImageInput,ImageList,TextList,SuggestInput,Radio}.tsx; view/{Country,Currency,ImageCarousel}.tsx; base/Carousel.tsx. Optional falsy fieldset children stay blank, matching transform/{view,list}/fieldset/{single,multiple}.ts.
- Lossless compiled-data comparison covers all 198 countries, 116 currencies and every color. Source SHA-256: countries `a14205652752d5d432aedad6a6eaec98725dc5775e5a383c9ed30d86565f12f4`; currencies `381c523ad201b3701290ba2306111ad95267cac8be00e7dbd6252d3b116eb578`; colors `0d585bef5896c1a1bff7cbdbf0ff8936f0627bb18969ecaf1c4865bc63edb72e` (`frui/src/data/*.ts`, 54,126/15,339/3,728 bytes).
- Lucide `check.svg` uses the existing pinned commit `03f11e2d6adc319ea21e9a1d5820a99cd3767257`, SHA-256 `7f33acc9a77a61659531044525fc008edebe215bf4dcf1c789c8674ad3277db0`. Geometry is decorative, currentColor, 16px, with names on native controls; light/dark browser screenshots verify it. No license files or license metadata edits.

## Refactoring rounds

1. Complexity: iterative formula stacks avoid recursive depth, captured upload tickets preserve out-of-order source mutations, and mask deletion skips literals/placeholders. Narrow 51-test close: `20261106T054608.361889Z-d2b944c9531e`.
2. Responsibility: split code/Markdown/mask adapters, centralize source prop/name/style conversion, compact source contract construction, remove unused wrapper. Native 51-test and generated data/native/hydrate/CSR/eight-browser checks: `20261106T055146.946858Z-4ce7ce397cbe`, `20261106T055151.211391Z-01f7a3fc8c21`.
3. Consistency/Rustdoc/comments: public source API, compiled doctest, existing native Input/Radio reuse, original fieldset labels, selector missing/empty/clear distinctions and late upload disposal. Clippy findings fixed. Native contracts/docs/doctests and eight-browser close: `20261106T061258.878307Z-a631f5a02124`, `20261106T061303.176109Z-5009095e11a5`. Final review scoped nested generated wrappers to preserve unrelated fixture bytes and removed an added finite-rating guard, retaining the original unbounded loop.

## Final gate

One final workspace sweep per feature lane, with focused corrections/retries and remaining gate stages resumed; no second full compatible test sweep.

| Check | Result |
| --- | --- |
| Workspace fmt and default/compatible Clippy, `-D warnings` | Passed; final affected-crate fmt/Clippy also passed |
| Workspace nextest, default | 1,071 passed, 90 skipped |
| Workspace nextest, compatible | 1,197 initial passes + five successful focused retries; 98 skipped. Later owned suite includes both added regressions |
| Workspace doctests, default / compatible | 979 / 986 passed |
| Workspace Rustdoc, default / compatible | Passed with `RUSTDOCFLAGS=-D warnings` |
| Actual aggregate source_client native models/stores/client, SSR and separate hydrate/CSR | Passed for all 108 placements; repeat-generation freshness passed |
| Actual compiled countries/currencies/colors | Every field matches upstream data |
| Cached intercepted Chromium | Eight final hydrate/CSR × light/dark × 375/1280 cases; no page/hydration errors; keyboard/forms/uploads/disposal/collections/carousel and formula host parity |
| Native consumer and separate WASM hydrate/CSR Clippy | Passed, `-D warnings`; WASM library scope excludes incompatible native-only dev-test dependencies |
| Aggregate existing build-host `--check`, seven runtime consumers, `graphs.py --check` | Passed, including direct/facade inventory parity and website hydrate/CSR graphs; existing generated fixtures unchanged |
| Generated-admin default and matching formatted fixture freshness | Both `{"differences":[]}` |
| Final instrumented owned nextest | 219 passed, one ignored process-only case |

The sweep exposed one real regression: reusing Input added empty associations to the existing API source URL adapter. Optional empty-association omission restores its native markup while preserving Typed defaults and real associations; the API contract and new native unit test pass. Four existing compile-fail watchdogs exceeded 180 seconds behind compiler locks; all passed serially with warm output. The added carousel keyboard assertion initially had no overflow to scroll; an authored 240px fixture exercises it at both widths. The plugin-against-default freshness attempt correctly refused a different selection; the matching default/formatted checks pass. An incorrectly launched wrapper was interrupted before it could repeat workspace tests. Failed/interrupted records are retained.

Stable native executable-line coverage, every measured production `src/` file per changed crate, no added filename exclusions:

| Crate | Covered / executable | Coverage |
| --- | ---: | ---: |
| stackpress-leptos | 7,738 / 8,737 | 88.57% |
| stackpress-framework-admin | 3,569 / 4,364 | 81.78% |
| stackpress-framework-schema | 5,881 / 6,866 | 85.65% |

Scope: Leptos ssr/stream/rich/format/source; admin source-ssr/codegen/format; schema source/codegen/providers. WASM DOM paths have the separate browser evidence above. Reports and screenshots: `target/00040/{coverage.json,coverage-summary.json,source-roles-proof/}`.

Permanent benchmark store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks/runs`; exact commands/features, environment and stages are in each record. Local complete index: `target/00040/benchmark-index.json`.

| Final-series record | Result | Command s | Cargo stages s |
| --- | --- | ---: | ---: |
| `20261106T062105.530381Z-519ed8329eb6` | Clippy passed; compatible sweep found five failures above | 894.203 | 851.410 |
| `20261106T063854.185115Z-5d722a032ec3` | Five retries plus new association contract passed | 134.546 | 73.000 |
| `20261106T064151.324279Z-3ef943ea5431` | Compatible docs/default freshness passed; wrong plugin selection refused | 252.259 | 167.800 |
| `20261106T064726.279907Z-ac757c6c60e4` | Formatted freshness and default tests/docs passed | 584.795 | 186.490 |
| `20261106T062152.972415Z-5f4bb5d3254f` | Aggregate build-host/consumers/graphs passed | 2,010.608 | 1,916.670 |
| `20261106T064155.684468Z-69170c29635d` | Latest affected native/WASM/browser/coverage refresh passed | 366.666 | 303.410 |
| `20261106T065824.869327Z-3c7184a4dbaf` | Final affected lint/fmt and provenance invariants passed | 13.437 | 9.600 |

Task-only counters: 0 → 64 attempts (41 passed, 22 failed iterations/admission checks resolved by later checks, one interrupted wrapper; none running); 0 → 36 compilation-emitting attempts; zero declared cold workspace rebuilds. Accumulated command time 7,283.390 s; observed Cargo stages 5,405.400 s. Concurrent totals are not calendar elapsed time or compiler CPU time. First-use selected targets were declared cold; later targets were retained. The last mixed refresh includes a fresh instrumented subtarget.

Targets are task-owned under `target/00040/{build,consumer-build,coverage}`; sccache, CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, bounded jobs (four normally, two for coverage/WASM). Sampled combined logical peak 13.11 GiB; primary target briefly reached 12.13 GiB, then scoped cleanup returned it below 12 GiB (11.73 GiB at close). Only completed compiler/docs output and obsolete UI test executables were cleared; sources, bound browser assets, reports and benchmark logs remain.

Root Cargo.lock and workspace manifest are byte-identical to HEAD; all 699 registry tuples/checksums match. Lock SHA-256 `e480322a6ca7f44345a54d0a0fe6ce09daac6f46f0782d7a01b442684b6b7bb0`. No new external dependency or web-sys feature. Only approved tracked consumer lock change: `framework/consumers/website/Cargo.lock`, 433 → 436 tuples (+pulldown-cmark 0.13.5, pulldown-cmark-escape 0.11.0, unicase 2.10.0), all root-locked; immediate subset/root-byte/699-tuple assertions passed (`20261106T062146.776210Z-29ebf97d7997`). All 44 Lucide master hashes match NOTICE; license files and license metadata are unchanged.

## Gaps and acceptance

No requested canonical role remains unavailable. G55 rich-role differences, the native Date spelling/locale boundary, and literal handling of arbitrary RegExp-like formula keys outside normalized source field names are explicit in the API reference. This is local web/native compilation and browser proof, not installed Tauri, live upload/provider, publication or owner visual acceptance.

KB validator: 147 inherited errors and 39 warnings; none concern this receipt. Coordinator reviews/verifies/merges; owner decides product acceptance. No commit/push/listener/install. Aggregate/session/API/SQL source/docs, examples/blog and other trial worktrees were untouched; the website lock is the explicit coordinator-approved exception.

## Follow-up: nested locks

Coordinator-scoped fix after merge `73ce025`: each checkpoint lock was copied from the root lock and resolved exactly once with offline `cargo tree`. Immediately after each resolution, registry tuples were asserted to be a root subset, root bytes identical, and the same 699 root tuples/checksums unchanged. Each adds only pulldown-cmark 0.13.5, pulldown-cmark-escape 0.11.0 and unicase 2.10.0, plus the corresponding Leptos dependency edge; nothing was removed.

| Lock under `examples/api-checkpoint/` | Registry tuples before → after |
| --- | ---: |
| `tool/Cargo.lock` | 368 → 371 |
| `web/Cargo.lock` | 398 → 401 |
| `client/Cargo.lock` | 315 → 318 |

All requested verification passed with Cargo `--locked --offline`: tool `--check` (fresh generated source), native web build, client WASM hydrate/CSR builds, aggregate website hydrate/CSR Clippy/build/bindgen and secret-sentinel checks, session `consumers/graphs.py --check`, and session `consumers/run.py` (direct/facade/plugin/disabled, including fmt, strict Clippy and runtime assertions). All four session locks admitted unchanged; no session lock refresh was needed.

The unchanged `examples/api-checkpoint/proofs/browser.cjs` passed 13 scenarios for each standalone hydrate, standalone CSR, aggregate hydrate and aggregate CSR: **52 total**, each with zero errors and zero unexpected requests. Aggregate runs used `API_AGGREGATE_ASSET=1` and freshly rebuilt aggregate artifacts. `API_REVIEW_ONLY` was unset throughout; light/dark and responsive cases ran inside each matrix. The cached Playwright/Chromium and stdio interception opened no listener. Reports/assets/screenshots and orchestration scripts remain in `target/00040/nested-followup/`; benchmark index is `benchmark-index.json` there.

| Follow-up benchmark record (same permanent store above) | Result | Command s | Cargo stages s |
| --- | --- | ---: | ---: |
| `20261106T074931.323151Z-e18e9539bb17` | Three sequential lock refreshes and immediate invariants passed | 1.411 | — |
| `20261106T075049.978540Z-69f3081412ee` | Checkpoint freshness/builds and standalone 26 browser scenarios passed | 410.859 | 378.850 |
| `20261106T075054.036903Z-eb402b1aab1a` | Session graphs and all four consumers passed | 589.746 | 573.030 |
| `20261106T075326.932188Z-ec23211a617d` | Aggregate hydrate/CSR artifacts passed | 207.600 | 203.420 |
| `20261106T075811.679235Z-eb5046c6fc24` | Aggregate 26 browser scenarios passed | 29.862 | — |

Follow-up counters: 0 → 5 attempts, all passed; 0 → 3 compilation-emitting attempts; zero cold workspace rebuilds. Accumulated command time 1,239.476 s; observed Cargo stages 1,155.300 s (overlapping runs, not calendar elapsed time). Fresh consumer subtargets used warm sccache, incremental/debug off and two Cargo jobs. Sampled follow-up output peak 7.39 GiB; existing primary target retained at 11.73 GiB, with 90 GiB disk free at close.

Final scope is exactly the three checkpoint locks and this appended receipt; root lock/manifest and all source remain unchanged. Root SHA-256 and all 699 tuples match the original receipt. Current KB validation baseline is 146 inherited errors and 42 warnings; the appendix adds no diagnostics. No follow-up gaps, commits, pushes, listeners or installations.

## Coordinator verification

Committed as `737a5d2` and merged into main as `73ce025`. The nested-lock follow-up `347bce6` was merged as `8774730`. Reviewed:
- the G55 Rust-native roles;
- frui data copied with hashes;
- the `source` feature enabling pulldown-cmark only;
- one Lucide hash line, licence text untouched;
- the website and API checkpoint locks: +3 existing root tuples each.

My first post-merge gate stopped on the API checkpoint locks, which the follow-up fixed. Then a combined gate at `fa1095a`/`744f64e`, 2026-11-06, exit 0:
- 699 tuples identical; root lock unchanged;
- fmt; Clippy, default and compatible;
- nextest: default 1,070 passed; compatible 1,237 passed; `tcp_watchdog` run alone;
- doctests; Rustdoc with `-D warnings`, after quoting one 00041 doc comment in `744f64e`;
- aggregate consumers and build-host `--check`;
- generated-workspace/admin, models and API checkpoint freshness;
- session consumers;
- wasm32 admin/leptos;
- docs and ledger;
- `source_roles.py all`: data parity plus 8 browser cases;
- `editor.py all`.
