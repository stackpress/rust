# Task 00013: M3 owner findings receipt

Status: **M3 accepted by coordinator and merged as reported `16b687c` (2026-10-28).** This preserves historical M3 implementation evidence; M4 was subsequently released and is covered by the [rolling receipt](00013-receipt-owner-findings.md). Read with the [approved design](00013-owner-findings-design.md); owner visual acceptance remains separate.

## Scope and implementation

Baseline: branch `trial/3-s8-polish`, clean HEAD `7575697` with M2 included. Only the released Lucide re-pin, D03 and D04 are implemented here. No commits, installs, listeners, new dependencies, Cargo/lockfile changes, tracked generated-output changes, or theme/G39 changes. Frui Table/Rating sources were inspected read-only; its tooling was not run.

Lucide is pinned to **1.62.0**, commit `03f11e2d6adc319ea21e9a1d5820a99cd3767257`. Only upstream source text was fetched. Six masters are byte-identical to the previous pin; `trash.svg` replaces `trash-2.svg` and the private Rust kind is `Trash`. The renderer reproduces its rounded-arc paths and path order. NOTICE records every source SHA-256 and the complete licence as shipped by this tag. The inherited MIT section is copied from that same LICENSE into FEATHER-LICENSE; the tag explicitly lists arrow-up/down, trash, x and plus as Feather-derived, and does not list arrow-up-down/star. Runtime geometry remains static inline SVG, currentColor, decorative aria-hidden/focusable=false, 16 px actions/sort and 24 px stars. Selected stars deliberately fill currentColor. Source equality and real DOM equality cover all seven rendered masters.

D03: native header buttons look like inline header text with unsorted/ascending/descending icons and unchanged `th[aria-sort]`/keyboard activation. Select page is visually hidden but remains associated with its checkbox and native mixed-state property. `TableColumnLayout { min_width: Option<u16>, wrap: bool }` and optional `DataTable::column_layout` are additive. Missing entries preserve old defaults; extra entries are ignored; explicit rulers clamp to 1..=4096; compact data uses nowrap and explicit prose uses wrapping/line breaks. The labelled tabbable region bounds both axes and sticky headings. Controller/data/wire behavior is unchanged; generated-admin uses its separate table and its table API remains untouched.

The ordinary gallery now has 48 stable r0..r47 rows, 12 columns, page size 12: ID, Name, Email, Role, Team, Location, Status, Joined, Last active, Projects, Notes, Reference. Long email/unbroken reference, Unicode, prose/newlines, empty notes and literal zero exercise overflow. Update/shrink retain the same column shape and row keys. Early native selection adopts all 12 visible rows when Select page was checked; draft validation admits the 48-key vocabulary and rejects duplicate/unknown keys.

D04: native visually hidden radios have positive Lucide-star labels, cumulative CSS fill even before JavaScript, associated focus rings, full “N of max” names and unchanged native form values. The small numeric zero control appears only when admitted. Some(0), None/Unrated and Clear remain distinct; disabled/stale-event guards and errors remain. `RatingState` and its value contract are unchanged. **Necessary additive API detail:** `RatingInput::min: u8` defaults to 0 because the existing state stores only max. Invalid min>max renders no options without rewriting state. Generated `RatingField` derives the first admitted whole rating from existing public schema @min/ge rules, intersects multiple minima, and shares the check with generator preflight. Negative minima admit 0, fractional minima round up; an impossible minimum is rejected before conversion. No new generated-admin role prop or wire member is added. Existing no-min callers retain zero-inclusive behavior.

Affected callers/proofs: gallery breadth/client draft adoption and snapshot controls; framework-admin RatingField/catalogue/RoleProof; existing row actions only adopt the upstream trash rename. Proof-only CollectionProof adds a minimum-two rating, reusing its disabled/error controls. No M4 raw-demo removal, safety-fixture changes, role-list formatting or stream integration occurred.

## Reproduction and evidence

Artifacts: `/private/tmp/s8-polish-m3`; one owned `target`, cap 8,388,608 KiB. `owner-table-rating.py` reuses the repository Context process-group timeout/kill/reap and periodic disk-budget checks. All Cargo dependency resolution is locked/offline; installed wasm-bindgen 0.2.129 and the supplied Playwright path are used. Browser/native hosts are finite interception/pipe fixtures; no socket listener is started.

```sh
python3 examples/review-preview/owner-table-rating.py build --directory /private/tmp/s8-polish-m3
python3 examples/review-preview/owner-table-rating.py browser --directory /private/tmp/s8-polish-m3
python3 examples/review-preview/owner-table-rating.py coverage --directory /private/tmp/s8-polish-m3
```

The native gallery/proof hosts, three native admin selections (base/plugin/disabled), two gallery WASM modes and six admin WASM variants are retained outside target. Generated output is created only in disposable workspaces. Every child command/result is recorded in review/ or roles/ logs and result JSON. Initial build/failed assertion logs are retained separately; final gate numbers below refer to the final source.

| Gate | Final result |
| --- | --- |
| Collection/minimum proof | 4/4 hydrate/CSR × root/mount; exact hidden form values, names, IME/focus/disabled regressions and minimum-two rating with no 0/1 options. 16 layout cases include coarse targets and narrow/200%; real DOM masters match. |
| Gallery breadth | 4/4 mode/mount; native pre-hydration radio arrows/fill, draft/DOM identity and 12-row early page selection; sorting, filtering, page persistence, keyed replacement/shrink; 16 wide/narrow/100/200% layouts with local horizontal/vertical scroll, sticky compact headings, offscreen Reference keyboard focus, sort icons/state and no document overflow. |
| Generated consumers | 24/24 base/plugin/disabled × hydrate/CSR × root/admin × wide/narrow-200%; native radio names/Tab/arrows/Space, zero/None/Clear, fill/focus and matching schema adapter behavior. Existing collection, plugin, escaping and disposal proofs remain. |
| Full gallery | 4/4 instrumented mode/mount cases; existing controls, replay/acquisition, sanitizer parity, preferences and disposal remain correct. |

A real visual defect was corrected after the first passing interaction run: inherited button wrapping broke short header words and made the head too tall. Sort controls now explicitly opt into nowrap and normal word handling; the proof checks compact head height and invisible 32 px mouse/44 px coarse-pointer hit areas. Gallery embeds styles during CSR too, so its WASM modes were refreshed after this CSS change before the final replay. Explicit prose styling applies only when a layout entry was supplied, preserving omitted-layout whitespace behavior. The synthetic disabled-radio proof can toggle a native property while publication is correctly denied; it returns through real enabled controls before form/visual proofs, rather than presenting that injected property as controlled state.

## Review rounds and coverage

Round 1 (logic/complexity): reviewed minimum admission/casts, zero versus None, invalid/disabled transitions, exact form values, layout defaults/extra entries/width clamps, sorting/selection and hydration ordering. A separately named fixture test module was merged into the existing test module so the coverage helper excludes all test-only lines. Measurement was refreshed after this correction; the earlier count is archived and is not the final production result.

Round 2 (responsibility/abstraction): native radio interaction stays in framework-ui, schema minimum admission stays in framework-admin, and fixtures remain in examples. Controller/wire/generator ownership is preserved; the narrow shared browser helper contains only proof assertions. No additional abstraction or dependency was justified. Final workspace tests explicitly filter tcp_watchdog to honor this lane's no-listener rule; coordinator TCP acceptance remains separate.

Round 3 (consistency/Rustdoc/comments) and safety: public layout/min defaults and bounds are documented; private SVG geometry/names match the pin. Fmt, workspace/all-target Clippy, gallery/admin WASM Clippy in separate modes, warning-denying Rustdoc, doctests and normal-edge feature trees are checked. Browser graphs exclude native server/SQL/Tokio. No unsafe code, raw SVG/user HTML injection, implicit acquisition, hidden value coercion or new listener/timer/observer is introduced. Native checkbox/radio state remains the pre-hydration visual source. Root disposers and browser cleanup still pass.

Final counts: scoped round 1 **141 passing entries**; minimum-rule follow-up **110 passing entries**; final workspace/all-target run **737 passed / 81 ignored**, TCP watchdog tests explicitly filtered; workspace doctests **940 passed / 0 ignored**. Clean final coverage tests **93 passed / 0 ignored**. All commands exit 0. Workspace/WASM Clippy, Rustdoc, fmt and normal-edge trees pass; Node/Python syntax and diff whitespace checks pass.

Coverage uses cargo-llvm-cov 0.8.6, Rust 1.96.0, LLVM 22.1.2, native aarch64-apple-darwin. The driver cleans workspace objects immediately before its instrumented run, selects framework-admin/ssr+codegen, framework-ui/ssr and gallery-shared/ssr, tests four selected native packages, then rebuilds instrumented finite gallery/proof hosts and replays collection/breadth/full-gallery browsers before export.

| Native crate | Covered / executable physical lines | Result |
| --- | --- | --- |
| stackpress-framework-admin | 1827 / 2072 | **88.18%** |
| stackpress-framework-ui | 3482 / 3664 | **95.03%** |
| stackpress-example-ui-gallery-shared | 635 / 710 | **89.44%** |
| stackpress-example-ui-gallery-web | 237 / 259 | **91.51%** |


`review/coverage-production.json`, raw/LCOV exports and retained profdata record whole compiled src trees, exact test-only exclusions and uncovered lines. Generated shipped code is retained. Dependencies/tooling/build output and test-only modules/files are excluded; generic hits coalesce per physical executable line. Browser-only client startup/DOM code is not native implementation coverage: both rebuilt WASM modes and real Chromium behavior are separate evidence. No WASM line/branch coverage, VoiceOver/manual assistive-technology review, installed iOS/Android support or publication claim is made.

## Inspected visuals and gate

Under `gallery/screenshots/`: `collection-icons.png`; `root-hydrate-table-left-1280-zoom1.png`, `root-hydrate-table-right-375-zoom2.png`, `mounted-csr-table-left-1280-zoom1.png`; `root-hydrate-rating-375-zoom2.png`, `mounted-csr-rating-1280-zoom1.png`; `collections-mounted-csr-rating-min2-375-zoom2.png`. Under `roles/screenshots/`: `plugin-hydrate-root-rating-375-2.png` and `disabled-csr-root-rating-1280-1.png`. These show actual-size family geometry/light/dark contexts, compact headers/local offscreen focus, cumulative fill/clear controls and generated rating consumers. The final minimum-two crop shows controlled value 2 after the injected disabled-state test is reset.

Deleted only `/private/tmp/s8-polish-m3/target` at **1,657,940 KiB**, below the **8,388,608 KiB** cap. `cleanup.json` records removal and the final owned-process check. Binaries, matching WASM assets, screenshots, exact commands/results, raw coverage exports and profdata remain. KB validator output matches the baseline: **146 existing errors / 25 warnings**, no new diagnostic; retained in `kb-validation.log`.

Re-review checklist **00013-003** table headings/filter/sort/pages, **00013-010** both axes/keyboard/375 px/200% containment, **00013-001** native early rating/page selection and hydration, and the rating controls on `/generated-admin/roles` in **00013-013**, both compiled selections. Specifically exercise 0, Unrated, Clear, pointer/Tab/arrows/Space and disabled/error behavior. The coordinator owns independent verification and managed-preview rebuilding; owner acceptance is separate. **Stop after M3; M4 is not released or started.**
