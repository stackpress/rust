# Task 00013 Lane A M2 restart checkpoint

Paused at the coordinator/owner request on 2026-10-27 before computer restart. This is an **unfinished progress note**, not the M2 receipt or proof of compiling components. Source is deliberately uncommitted. Resume only when the coordinator/owner resumes M2; do not start M3.

## Worktree, merge and boundaries

- Work only in `/Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s8-m0`, branch `trial/3-s8-m0`.
- Started clean at accepted M1 commit `a26256f`. Required `git merge trial/3` fast-forwarded to **a268b96**; no conflict. HEAD remains there. No commit/push/install/listener/browser process was started for M2.
- Lane A owns framework-ui, framework-view, ui-gallery and their root manifest/lock/facade entries. No Lane B, server, SQL/schema/Idea/CLI or accepted profile-ui edits. Only this pre-approved WIP note is added under .agents so far. The eventual M2 receipt path is separately pre-approved: `tasks/00013-receipt-lane-a-m2.md`.
- Read current AGENTS/context, programming/coordinated workflows, Rust style/quality, accepted Slice 8 component inventory/API/verification, and current component source. Reread the relevant upstream Frui files under `/Users/cblanquera/server/projects/stackpress/frui/src` for M2 (no other trial branch source/output was used). Applied the coding skill's Rust adaptation.

## Components written; proof status

**No new M2 component is compiled, tested or browser-proved yet.** Twelve prototype source files are untracked and intentionally not declared/re-exported in components/mod.rs. Their current names/imports assume that later wiring. Existing public Dialog is the only M2 change wired into the library. All fifteen touched Rust files passed direct `rustfmt --edition 2024` parsing/formatting at this checkpoint, and `git diff --check` passed. Rustfmt does not expand Leptos macros or type-check.

| File under framework-ui/src/components | Current prototype |
| --- | --- |
| choice.rs | ChoiceState: bounded immutable options, selected value separate from roving focus, enabled-only admission, arrows/edges/prefix lookup; DisclosureState: multiple/collapsible/required group rules. |
| disclosure.rs | Accordion/AccordionItem explicit slots; native Details controlled toggle; manual/automatic Tabs with orientation, disabled options, roving keyboard and associated retained hidden TabPanel. |
| paging.rs | PageWindow and PageTotal known/unknown, size 1..100, overflow/last-page clamp, strict skip/limit query codec and controlled Pagination. |
| navigation.rs | Checked BreadcrumbTrail/Breadcrumbs with current-page semantics; Menu, MenuEntry and Menubar keyboard prototypes using native popovers. |
| geometry.rs | Pure finite viewport collision policy, flip/clamp with eight-pixel inset. |
| overlay.rs | Native auto Popover, modal Drawer wrapper, bounded DelayMs, pure TooltipActivity, noninteractive Tooltip and explicitly owned dismissible/optional-expiration Toast. |
| overlay_dom.rs | Root-local native popover/listener/timer leases, measurements on resize/captured scroll, cancel/cleanup; no global registry or forgotten once_into_js timeout. |
| collection_state.rs | Generic BoundedList/CollectionRow, unique nonzero stable IDs and non-reuse, <=100 rows, add/edit/remove/reorder, FieldValue Missing/Null/Value and MetadataEntry. |
| collections.rs | Generic CollectionFieldset explicit render/create callbacks; TextList, TagList and MetadataInput. Stable keyed native fields/actions and app-owned errors. |
| table_state.rs | TableState over checked DisplayTable, literal filtering, stable lexical sorting, known-total paging, selection across pages and checked replacement/pruning. |
| table_family.rs | TableHead/Group/Foot/Row/Header/Cell/Rule, bounded spans/scopes, DataTable filter/sort/page/selection, overflow/stripe/sticky hooks. |
| combobox.rs | ComboboxState query/selected value separation, active descendant, bounded options, generation validation; async AbortLease/request output/publish boundary; Combobox and SuggestInput prototypes. |

Tracked changes: advanced.rs adds public DialogMode (Modal default, Nonmodal, DrawerStart/End), mode class mapping and adapter mode argument; dialog_dom.rs chooses native show versus showModal and suppresses modal Tab trapping for Nonmodal; mod.rs exports DialogMode only. Default modal class stays sp-dialog. Drawer classes have **no CSS yet**. The API/docs are provisional until compiled/reviewed.

## Remaining implementation and known unfinished issues

1. Wire the twelve modules and explicit public exports in components/mod.rs (overlay_dom only cfg(wasm32)); add only necessary already-locked web-sys DOM features, notably HtmlDetailsElement. Compile before treating any prototype as real behavior. No new crate/version dependency has been adopted.
2. Finish Rating input, bounded single/multiple Slider, and explicit SlugInput/slug/camel transforms: these are M2 rows in the accepted inventory, although not repeated in the coordinator's release shorthand. Specialized M3 date/number/phone/file/editor behavior stays out.
3. Implement neutral M2 CSS, including hidden semantics, popup viewport width/height limits, menu/tab states, collection layout, table overflow/striping/sticky and drawer geometry. Existing light/dark UI regression markup/styles must remain compatible.
4. Review prototype ownership/Leptos macro type inference. Known likely compile work: closures captured by Children/For, generic CollectionFieldset Callback types, NodeRef/HtmlElement adapter coercions and Option prop builders. None has been compiler-verified. The metadata view delimiter mistake was fixed and direct Rust parsing now passes.
5. Popup close currently restores opener focus too eagerly. Correct outside-click/Tab behavior so closing never steals focus from a newly focused external control; preserve explicit close/Escape and unmount return focus. Add actual nested/unmounted popup/menu/drawer tests, including native auto-popover stack, pointer dismissal, viewport collision, scroll/resize listener disposal and timers. Details must adopt an early native toggle before hydration.
6. Menu prototype currently handles one-character prefix typeahead. Add bounded multi-character/time-window behavior with repeated-letter cycling, and finish menubar/menu keyboard transitions. Check menu Tab exit and disabled options. No arbitrary interactive tooltip children are admitted.
7. Combobox options rendered by For currently key only on machine value and capture label/index. Handle changing async result labels/indices/disabled states without stale DOM. Ensure aria-activedescendant always resolves to an enabled current option; define/loading-proof stale-option admission. Lookup currently calls edit then request calls edit again: simplify generation boundaries while preserving one actual acquisition. On close/disable/unmount cancel and reject late responses. Complete a real yielding/cancel/unwind loader watchdog proof: CPU-bound/non-yielding work remains non-interruptible, as in M0.
8. Collection focus handling currently requests up/down button focus after reorder; at an edge that button becomes disabled. Choose a connected enabled deterministic destination and prove node/draft/order retention. Validate limits/errors, empty/false/zero, typed nested fields, unknown keys, key exhaustion and duplicate IDs. Native maxlength counts code units, not bytes; constructor/host text byte policy needs explicit review.
9. Table controller projection is local only. Prove zero/one/many rows, stable sort ties, literal filter, shrinking snapshot/page clamp, cross-page selection and select-current-page semantics; no data operation/SQL authorization claim. Check selection initial SSR attributes as well as subsequent DOM properties. Finish explicit body/column-slot aliases only where useful; validate header/span structure in actual HTML.
10. Add complete native controller/security/SSR tests and a real shared BreadthGallery section; update gallery build identity to s8-m2 (decoder rejects stale s8-m1), while revision-1 DTO shape need not change. Capture early new native properties before hydrate; prove complete initial parsed DOM/node/focus retention, no initial refetch, every selected inventory family and 375/1280/200% geometry. Extend finite browser proof under the existing 120s parent watchdog and 10s actions; intercepted .invalid origins only.
11. Run at least three test-verified review rounds plus safety, per-crate production coverage >=70%, fmt/Clippy/Rustdoc/MSRV/WASM graphs, unchanged profile proof and locked workspace suite. Then write the approved M2 receipt, delete targets and stop for coordinator review. No M3.

## System-theme projection

**Not started.** framework-view and UI Theme/styles/gallery seeds are unchanged for this item. Current PageContract validation in framework-view/src/policy.rs accepts only light/dark. Add the host-seeded system token additively; keep light/dark output byte-identical. Check snapshot/decode/projector checks and both renderers. UI CSS media queries must resolve system surfaces before paint without JS media reads or initial acquisitions. A separate trusted system stylesheet/conditional projection may preserve the accepted base stylesheet bytes. Add native acceptance/rejection/snapshot tests and actual light/dark emulated-media pre-JS/hydrate browser proofs. Lane B adopts the new contract later; do not edit its crate/example.

## Evidence safe across restart

- M2: initial task target cleans and cargo-llvm-cov clean completed. No build/test/browser was started after the merge; no M2 screenshot, coverage or passing-component claim exists. Only direct Rust parse/format and diff whitespace passed.
- KB baseline validator exits 1 with 146 existing errors and 22 warnings after the required merge. Its temporary log will disappear. Re-baseline after restart before adding the M2 receipt; do not repair unrelated missing TypeScript sibling links.
- Accepted M1 evidence already survives in tasks/00013-receipt-lane-a-m1.md and the coordinator independently verified/merged it. Coordinator reported UI ssr,rich 44 passed, gallery build/Chromium and unchanged profile check PASS. M1 screenshots show spaced actions and a viewport backdrop. Earlier native textarea security proof escaped </textarea><script> plus &, quotes and U+2028; actual hydrated value exact, no execution. Raw tachys textarea/style/script child emission and leptos_meta title boundary source locations are preserved in that accepted receipt, not only /private/tmp.
- M1 viewport conclusion: 375x812 backdrop fixed at all insets zero; fullPage document height 3661 exceeded viewport, explaining its apparent cutoff. Background wheel/PageDown/focus/interaction protection was actually proved. Do not repeat this as new M2 evidence.
- Temporary /private/tmp/s8-m0/{m1,m2} artifacts, scripts and screenshots may all vanish. There is no essential M2 result only in those directories. Installed wasm32 target, wasm-bindgen 0.2.129 and Playwright package `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright` were provided; verify availability after restart, do not install tools.
- No task build/browser session remains running. A process listing showed other lanes' work; do not terminate unrelated agents/processes. Initial clean left only Cargo cache/lock metadata; that was removed. target/ is absent at this checkpoint. Source and this note are uncommitted in the worktree.

## Exact next commands after explicit resume

Read this note, AGENTS/context/style/quality and the accepted inventory, then inspect the dirty source. Do not merge over dirty work without coordinator direction.

```sh
cd /Users/cblanquera/.paseo/worktrees/342ftjlj/trial3-s8-m0
git status --short
git rev-parse HEAD
mkdir -p /private/tmp/s8-m0/m2
python3 .agents/scripts/validate-agent-workspace.py > /private/tmp/s8-m0/m2/kb-before.log 2>&1
export CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0
export CARGO_TARGET_DIR="$PWD/target/s8-native"
export CARGO_LLVM_COV_TARGET_DIR="$PWD/target/s8-native"
cargo clean --target-dir target/s8-native
cargo llvm-cov clean --workspace
cargo clean --target-dir target/s8-wasm
```

After wiring modules/exports and fixing the known items, begin with these separate modes; redirect logs under the new temporary M2 directory and use finite deadlines:

```sh
cargo test --locked -p stackpress-framework-ui --features ssr,rich
cargo test --locked -p stackpress-framework-view
cargo clippy --locked -p stackpress-framework-ui --features ssr,rich --all-targets -- -D warnings
cargo +1.96.0 check --locked -p stackpress-framework-ui --features ssr,rich
GALLERY_OUTPUT_DIR=/private/tmp/s8-m0/m2 python3 examples/ui-gallery/build.py
GALLERY_OUTPUT_DIR=/private/tmp/s8-m0/m2 GALLERY_PLAYWRIGHT=/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright python3 examples/ui-gallery/browser.py
cargo test --workspace --all-targets --locked
cargo test --workspace --doc --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

The gallery wrapper already uses exactly target/s8-native and target/s8-wasm, CARGO_INCREMENTAL=0 and CARGO_PROFILE_DEV_DEBUG=0. Rebuild a verification/coverage driver after restart if useful: final per-crate scope includes framework-view now, UI, gallery shared/web, with thin browser adapters explicitly covered in Chromium. Run actual hydrate/CSR + extended facade cargo tree --edges normal,no-proc-macro, deny Stackpress server/SQL/DB/CLI/Idea/Tauri/Tokio runtime dependencies. Never use --all-features for UI. Existing hang-class tests must remain child/watchdog/kill/reap; timeout fails. Do not start a listener or reinstall anything.
