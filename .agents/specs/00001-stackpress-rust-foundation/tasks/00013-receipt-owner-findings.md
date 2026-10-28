# Task 00013: owner findings receipt

Status: **M0–M3 accepted by coordinator; M4 implemented and verified, ready for coordinator review.** The approved owner-findings scope is complete; owner visual acceptance, managed previews, commits and G39 remain separate. Follow the [approved design and owner decisions](00013-owner-findings-design.md).

## M0 result and implementation

The external async classic status asset reliably produces visible recovery after a parsed shell is interrupted, including pending/deadline delivery, in the tested Chromium. Completion-gated hydration remains intact. No dependency, manifest, lockfile, icon, gallery, generated-admin, preference or theme implementation changed in M0.

- `framework-ui::stream::StreamLeptosRenderer::with_status_script(PublicLocation)` is additive and opt-in. It freezes the mounted, checked external URL before rendering; opening-byte/chunk/aggregate limits still apply. Existing default asset/wire contracts remain unchanged.
- `stream::STREAM_STATUS_SCRIPT` exposes fixed external bytes from `src/stream_status.js`. The script edits only fixed notice text/attributes, observes the fixed inert markers, and never decodes replay, calls WASM, hydrates, evaluates text, retries or issues a network request. Completion/pagehide clears its observer, listeners and timer. A failure marker is terminal.
- Both ui-stream native registration paths share the same opt-in render helper. Its build copies the status asset. The shell retains a checked reload link and explicit static “Loading or interrupted” guidance when scripting/the asset is unavailable.
- The finite native HTTP decoder now reports the actual zero-length terminating chunk separately from socket EOF. The browser relay closes only after that terminal framing; missing terminal framing errors its ReadableStream. It no longer disguises native truncation as clean EOF.
- The independent `status-browser.cjs` proof reuses that native transport and worker. It adds no listener and does not synthesize application HTML, replay, failure or completion markers.

## What the browser evidence establishes

`/private/tmp/s8-polish-m0/status-browser.json`: **16/16 pass**, Playwright's installed Chromium **156.0.8078.4**. Each context enforces `script-src 'self' 'wasm-unsafe-eval'; object-src 'none'; base-uri 'none'`; attempted inline executable script is refused. No unsafe-inline or nonce bypass was added.

| Cases | Evidence |
| --- | --- |
| Success, root and `/demo` | Real WASM hydration, working counter, no duplicate initial Resource fetch, hidden recovery; waiting beyond six seconds does not revive a completed notice. |
| Truncation after parsed shell, both bases | Actual native missing terminal chunk, Failed delivery and capture cleanup; visible unfinished-page alert/reload, no details/completion marker/hydration. |
| Pending, tight idle deadline, late failure, both bases | Native incomplete delivery releases its root exactly once; browser shows recovery and never hydrates partial content. |
| Browser-held pending delivery, both bases | Notice appears while document.readyState is still loading, independent of parsing-end/DOMContentLoaded. |
| Delayed external status asset, both bases | Delaying the asset 2.8 seconds, past native pending termination, still installs the status updater and shows recovery. |
| Slow eventual success, root | Informational unfinished notice appears while parsing; later genuine completion clears it and hydrates once, with a working counter. |
| Blocked status asset, root | Static “Loading or interrupted” explanation and Reload link remain visible; no automatic status-update guarantee or partial hydration. |

Chromium leaves the tested erroring documents in **readyState=loading**. Thus these cases use the honest six-second **“This page hasn't finished loading”** watchdog, not an invented transport diagnosis. Typical recovery screenshots were taken at 6.4–6.7 seconds after navigation; delayed-asset cases at about nine seconds. The separate “didn't finish” branch remains for an arriving failure marker or an observable parsing end without completion; this M0 browser matrix does not claim it exercised those branches.

The first truncation run errored the worker while useful chunks were still queued: ReadableStream discarded them and there was no rendered shell. The corrected test acknowledges that the real native shell has been parsed before propagating the real terminal-framing failure. This deliberately tests the accepted **post-shell** contract. Failure before any shell reaches the browser remains browser-owned navigation failure.

This is real incremental browser parsing through an intercepted service worker, fed actual Hyper/duplex bytes with a real stream error. It is not a real TCP/TLS/browser-network preview. The coordinator's later managed preview must separately confirm the owner's exact incomplete-chunked-response scenario.

## Reproduction and native checks

All logs/results/artifacts are retained under `/private/tmp/s8-polish-m0/`; the copied `stream-host` and `assets/` are independent of Cargo output. Builds used fresh native/WASM output, locked/offline dependencies and the installed matching wasm-bindgen 0.2.129; no installs.

The existing `examples/review-preview/review.py run --directory /private/tmp/s8-polish-m0 --name NAME --timeout SECONDS -- COMMAND...` provides the owned eight-GiB target, periodic disk checks and process-group kill/reap watchdog. Set `UI_STREAM_OUTPUT_DIR=/private/tmp/s8-polish-m0` on example commands.

| Recorded phase | Command/result |
| --- | --- |
| `clean`, `coverage-clean` | `cargo clean --offline`; `cargo llvm-cov clean --workspace`: pass before first build. |
| `build` | `python3 examples/ui-stream/build.py`: fresh native and hydrate/WASM builds plus bindgen pass; one target. |
| `native-tests` | `cargo test --offline --locked -p stackpress-framework-ui -p stackpress-example-ui-stream-web --features stackpress-framework-ui/stream`: **59 native tests + 18 doctest entries pass**. Two explicitly ignored child entry points are executed by their parent watchdogs. |
| Native watchdog matrix | Success, precommit, late, disconnect, deadline and shutdown at both bases; framing/status, one lifecycle pass, missing completion on failure, and root/resource capture release all pass. The finite CLI child is separately reaped. |
| New renderer regression | Optional status script is absent by default, mounted/HTML-escaped when selected, remains external, and does not prevent genuine completion. Runs inside the existing renderer watchdog. |
| `status-browser-retry` | `env UI_STREAM_OUTPUT_DIR=/private/tmp/s8-polish-m0 UI_STREAM_PLAYWRIGHT=/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright node examples/ui-stream/status-browser.cjs`: **16 pass**, exit 0, 116 seconds under a 150-second parent deadline. |
| `clippy` | Same packages/features, `cargo clippy --offline --locked ... --all-targets -- -D warnings`: pass. |
| `rustdoc` | Same packages/features, `RUSTDOCFLAGS='-D warnings' cargo doc --offline --locked ... --no-deps`: pass. |
| Formatting/source | `cargo fmt --all -- --check`, `git diff --check`, Node syntax checks for the new script/proof: pass. |

Chromium initially hit the sandbox's macOS Mach-port denial; the proof reran with explicit escalated execution. Native children close stdin and join/reap in finally; browser contexts/registrations close on every outcome. Long-running commands use parent deadlines/process-group termination.

Deleted only `/private/tmp/s8-polish-m0/target` after the checks: **1,138,140 KiB** immediately before deletion, below the **8,388,608 KiB** cap; `cleanup.json` records removal. Retained binaries/assets/logs/screenshots remain. KB validator output is byte-identical to baseline: **146 existing errors / 25 warnings**, no new diagnostic. No listeners, installs, commits or pushes.

## Inspected visuals and next gate

Inspected `truncated-root-recovery.png`, `pending-root-recovery.png`, `deadline-mounted-375.png`, `slow-success-root-375.png` and `blocked-status-root-375.png`. Alerts and Reload remain readable at 375 px without page-wide clipping; partial pages retain useful shell content without active hydration. Slow success shows settled content and the working counter. Hostile details text remains intentional pending F04.

Owner checklist to re-review after coordinator integration: **00013-012** late/pending recovery and eventual success; **00013-010** narrow-width recovery/readable reload. M0 does not release the full owner walkthrough or change any other finding.

At the M0 handoff, full milestone implementation, per-changed-crate coverage, the repository's test-verified refactoring/safety rounds and final regression matrix remained for the released later milestones. M0 was a feasibility gate, not full Slice 8 closeout. The coordinator subsequently accepted M0 and released M1; the M0 stop is superseded only by that release.


## M1 implementation and scope

F01 now selects static, escaped HTML for errors on the exact generated GET/HEAD document-route vocabulary. `generated-admin/web/src/host/document_errors.rs` reuses the trusted mount/model/action parser, excludes API/export/unknown or unregistered actions, and runs after fresh error-response staging. It restores the original 4xx/5xx status after `Response::html`, retains private diagnostics, marks the page no-store and includes only safe fixed copy and the compiled Return to list URL. No protected seed, WASM/bootstrap, raw query, exception or automatic write retry appears. The core HTTP JSON envelope, authorization order and download/API responses are unchanged.

The review host alone defaults an absent role cookie to Editor. Explicit Denied and Reader remain exact; malformed, unknown and duplicate role cookies deny; forged actor headers still cannot choose authority. Existing review-only chrome links denial/error pages to the labelled fixture controls in a second tab.

D05 adds the opt-in `components::BASE_STYLES` constant exposing the existing stylesheet. Shared `generated_admin_shared::Admin` now owns a fixed-light `.sp-root.sp-role-catalogue`; `ROLE_CATALOGUE_STYLES` supplies scoped spacing, table overflow and field presentation. A single `render::roles_document` shell includes base/core/advanced/breadth/catalogue styles for both standalone and review hosts. CRUD rendering and theme implementation are unchanged. Existing role component signatures, generated contracts and values remain unchanged; no managed output or dependency was edited.

M1 reuses M0's stream implementation without modifying it. Coordinator additions remain queued for M4: prefer definitive late-failure wording when observable, and show list values readably (`None` for an empty list, comma-separated items otherwise). M1 intentionally retains raw list presentation and hostile proof labels until M4's approved fixtures/integration work.

## M1 proof and build provenance

Artifacts live under `/private/tmp/s8-polish-m1/`; exact argv, elapsed time, exit status and logs are in `review/*-result.json` and matching logs. The reproducer is `python3 examples/review-preview/owner-findings.py build --directory /private/tmp/s8-polish-m1 --baseline /private/tmp/s8-review`, followed by its `browser` and `coverage` actions. All long children use the existing capped-target/process-group watchdog, with native pipe/context disposal and child reaping in finally blocks. No listener is involved.

Both disabled/plugin native gateways and all four matching admin hydrate/CSR WASM bundles were freshly regenerated and built in the existing disposable workspace; tracked schema/generated output remain untouched. The standalone generated-admin executable was freshly built too. Unrelated gallery/preferences/stream/format static assets were explicitly borrowed from `/private/tmp/s8-review`; this is an admin-scoped proof, not an all-example rebuild or full preview acceptance.

`review/m1-browser.json` and `m1-coverage-browser.json`: **8/8 cases each pass**, disabled/plugin × hydrate/CSR × root/`/demo`. Fresh visitors open Profile/Project/Task lists successfully without a role cookie. Explicit Denied receives HTML 403 with next action and no seed/module/hydration; a failed next read receives safe HTML 503. The same caller's forged-header API write/export still receives JSON 403. Roles have base tokens, solid input/button borders, actual input focus with a visible three-pixel ring, no page-wide clipping at 375/1280 px and local table overflow. Paused hydrate cases retain the original SSR root node after startup. Plugin wrapper presence matches the compiled variant. No unexpected request or browser runtime error was recorded.

Native `web/tests/document_errors.rs` exercises real generated SQLite host registration at `/` and `/demo/admin/`: HTML 403/400/404/500/200, error diagnostic preservation, no-store, unknown/unregistered route rejection, API/export JSON errors and opposite Accept values proving route-authoritative representation. Attribute-breakout input is escaped. Authorization still precedes parsing a valid-URI but invalid-JSON query. Invalid raw URI percent escapes are rejected by the existing HTTP adapter before routing; that precedence was not widened here.

## M1 test-verified review rounds

| Round | Review and resulting evidence |
| --- | --- |
| 1 — logic/complexity | Checked normalized method/path/mount classification, action and soft-remove gates, error-listener order, original status/diagnostic retention and denial before query parsing. The small guarded classifier remains direct; no further refactor was justified. `m1-round1-final`: generated-admin-web plus review-preview, `--features review-preview`, **22 native/doctest entries pass**, including the two dedicated representation tests. |
| 2 — responsibility/abstraction | Checked that only host registration chooses representation; rendering receives safe public status/link, and shared catalogue structure/styles are reused by both shells. Private `route` now accepts the mount it actually needs; page/API/transfer callers are updated. No core HTTP/generator policy moved into examples or CSS. `m1-round2-tests`: framework-ui/shared `ssr`, **86 native/doctest entries pass**; fresh generated native/WASM variants and eight real-browser cases pass. |
| 3 — consistency/Rustdoc/comments | Reviewed new public constants/render functions, private classifier/listener contracts, explicit status restoration and local comments against existing Rust idioms. No material follow-up source edit remained. Four-package `--features review-preview`: **20 doctest entries**, warning-denying Clippy `--all-targets`, and warning-denying Rustdoc `--no-deps` pass. Final rustfmt, diff-whitespace, Node and Python syntax checks pass. |
| Safety/soundness | Reviewed escaped attributes, fixed public error copy, no protected seed/script on errors, unchanged diagnostics, policy-before-query ordering, spoofed/malformed/duplicate role denial, API/download separation, unchanged generated wire/value contracts, and cleanup on error. Real native/browser assertions pass. No unsafe code, new dependency or theme implementation change. |

Corrections during proof development were to fixtures/expectations: avoid Tokio macros absent from the existing feature set; use valid URI syntax with invalid JSON to reach page parsing; accept Response::html's existing MIME spelling; omit an empty JSON GET body header; and inspect the actual inner framework table scroll region. No core routing, authority or HTTP input behavior was relaxed to pass a test.

## M1 coverage and final gate

`cargo-llvm-cov 0.8.6`, Rust **1.96.0**, LLVM **22.1.2**, native **aarch64-apple-darwin**. Command: `cargo llvm-cov -p stackpress-framework-ui -p generated-admin-shared -p generated-admin-web -p stackpress-example-review-preview --features review-preview --offline --locked --no-report`, followed by an instrumented disabled pipe host replay of the eight-case browser matrix and LCOV export. **121 native tests pass**; the one ignored stream child entry is deliberately invoked by its passing watchdog parent. The separate doctest result is above.

| Crate | Covered / executable physical lines | Result |
| --- | --- | --- |
| stackpress-framework-ui | 4769 / 4964 | **96.07%** |
| generated-admin-shared | 3424 / 4288 | **79.85%** |
| generated-admin-web | 5376 / 6134 | **87.64%** |
| stackpress-example-review-preview | 737 / 794 | **92.82%** |

All exceed the development floor of 70%. `review/coverage-production.json` lists measured files, exact excluded inline-test ranges and uncovered lines; `coverage-lcov.log`, `coverage-raw.log`, `coverage.json` and retained profdata preserve exports. Scope is each crate's complete compiled `src/` implementation, including generated contracts/UI/stores and native path-included source. Only test-only files and exact inline test-module ranges are excluded, alongside out-of-scope dependencies/tooling/build output. Physical lines coalesce generic instantiations. Browser-only WASM branches are not claimed as native coverage; fresh hydrate/CSR builds and real Chromium behavior provide the separate M1 target evidence. No branch coverage, installed-target support or publication preparation is claimed.

Tooling now runs `cargo llvm-cov clean --workspace --offline` immediately before measurement: ordinary builds can otherwise leave fresh non-instrumented objects reusable under the coverage wrapper. The incomplete first measurement and its logs are retained in `review/coverage-attempt-uninstrumented-cache`; it is not a coverage result. The clean replay produced the table above, with no exclusion changes.

Inspected screenshots under `review/`: `disabled-hydrate-root-denied-1280.png`, `disabled-hydrate-root-denied-375.png`, `disabled-hydrate-root-failed-375.png`, `disabled-hydrate-root-roles-1280.png`, `disabled-hydrate-root-roles-375.png`, `plugin-hydrate-mounted-roles-focus-375.png`, `plugin-csr-mounted-roles-1280.png` and `plugin-csr-mounted-denied-375.png`. The latest focus screenshot shows the actual input ring; the earlier weak selector had focused review navigation and was replaced by explicit active-element/:focus-visible/outline assertions.

Deleted only `/private/tmp/s8-polish-m1/target`, **3,310,620 KiB** immediately before deletion, below the **8,388,608 KiB** cap. `cleanup.json` records deletion; binaries, matching admin assets, logs/reports and screenshots remain. No listener, install, commit or push; no Cargo/lockfile, theme source or tracked generated output changes. KB validator output is byte-identical to baseline: **146 existing errors / 25 warnings**, with no new diagnostic; retained in `/private/tmp/s8-polish-m1/kb-validation.log`.

Coordinator/owner re-review: **00013-003** Failed next read and Denied; **00013-005** direct denied document routes/recovery; and the `/generated-admin/roles` page in **00013-013**, both compiled plugin variants, including narrow inputs/focus/table scrolling. The coordinator owns managed preview rebuilding and independent verification. At the M1 handoff, M2 remained gated. The coordinator subsequently accepted/merged M1 into trial/3 (reported reference `d5f7564`, 2026-10-28), independently reran its wider checks, and released M2.


## M2 implementation and scope

D01: TextList and MetadataInput now render a shrink-safe inline field area with a nonwrapping right-hand action group. The inherited visible input labels become associated `sp-sr-only` labels; IDs, names and error relationships remain exact. Arrow-up, arrow-down and trash-2 replace action text, with row-qualified button names. Narrow rows scroll locally and keyboard focus reveals offscreen actions; generic CollectionFieldset content stays application-owned, with wrapping generic actions. Post-add/remove/reorder focus and the reentrant factory-outside-write-guard policy remain unchanged. Boundary checks now borrow the list instead of cloning its complete contents.

D02: TagList is a dedicated keyed chip renderer with text/X remove buttons, a visible Add tag label and an Add button with plus/visible text. Existing empty strings display `(empty)`. Enter/Add append exact nonblank strings; accepted text is never trimmed, comma-split or deduplicated. Blank/whitespace drafts stay unchanged with no new error; byte/capacity rejections preserve draft and values. Conditional signal publication prevents no-op/rejected drafts from promoting a generated adapter's Missing/Null presentation into a supplied empty array. Hidden values keep stable `id[key]` form names in collection order; the draft has no form name. Min/max, disabled/error admission and existing BoundedList/value contracts remain unchanged.

Owner decision implemented: **TagList has no in-place chip editing or reorder arrows.** Remove/add changes a tag. TextList/MetadataInput retain up/down/remove actions, and BoundedList's programmatic edit/reorder API remains available. Empty Backspace focuses the last remove button without deleting; removal focuses the next neighbour, previous final chip, or add input. Composing Enter, composition state and legacy keyCode 229 do not append. TagList adopts its native add draft before hydration internally, preserving its public props. The add region wraps in narrow nested containers so both controls remain usable at 200% zoom.

Private `components/icons.rs` renders five decorative inline SVGs at 16 px: arrow-up/down, trash-2, x and plus. Geometry matches Lucide tag 0.468.0, commit `f12b0de177fbc2a6795e99be065887e72b237123`; seven exact masters, SHA-256 inventory, complete ISC and inherited Feather MIT licences live in `framework-ui/third-party/lucide/NOTICE` and adjacent files. Arrow-up-down/star masters are reserved for M3; no table/rating behavior was implemented. SVGs use currentColor, 24×24 viewBox, two-unit round strokes, aria-hidden and focusable=false; buttons own their accessible names. Targets are 32 px, 44 px for coarse pointers, with existing visible focus. The contact sheet covers 16/24 px, light/dark, enabled/disabled controls, family anchors and enlarged geometry. No icon crate or dependency was added.

Callers: framework-admin's TextListField tags branch and MetadataField, generated project/RoleProof field/span wrappers and AccentTags keep their Rust signatures and generated bytes. Their real disabled/plugin consumers were regenerated/built in isolation and browser-tested. Gallery adoption reads existing hidden tag values; only the separate add draft is editable. Gallery/admin browser selectors/assertions now use chip removal/addition. The explicit collection_proof module, native example and WASM start_collections export provide adversarial proof controls outside owner routes. Ordinary Typed nested rows remains in the gallery pending M4; this fixture does not implement F05 early. Crate component docs and Lucide attribution links are updated.

## M2 proof and review rounds

Artifacts: `/private/tmp/s8-polish-m2/`. Reproduce using `python3 examples/review-preview/owner-collections.py build --directory /private/tmp/s8-polish-m2`, then its `browser` and `coverage` actions. Every child uses the existing process-group kill/reap watchdog and eight-GiB target checks. Builds are locked/offline, using the installed wasm-bindgen 0.2.129 and installed Playwright Chromium. No listener, install or commit. Fresh native gallery/proof hosts, two gallery WASM modes, three native admin selections (base/plugin/disabled) and six matching admin WASM outputs are retained outside the target. Admin generation never changed tracked output.

| Gate | Result |
| --- | --- |
| Collection proof | **4/4** hydrate/CSR × root/mount; **16** 375/1280 px × 100/200% layouts, mouse/coarse targets, plus a nested eight-rem container. Exact empty/zero/Unicode/duplicate values, atomic byte/capacity rejection, no-op publication counter, hidden form names, IME guards, Backspace/remove focus, parent-form submission, disabled stale events, field errors, keyed node retention, both reorder directions and keyboard-reachable local overflow pass. Rendered primitives equal pinned SVG masters. |
| Ordinary gallery breadth | **4/4** mode/mount cases pass, including early unsubmitted tag draft and input identity, existing nested factory/typed values, resource behavior, no page overflow, overlays and root cleanup. |
| Generated roles/plugin consumers | **24/24** base/plugin/disabled × hydrate/CSR × root/admin mount × wide/narrow-200% cases pass. Exact draft arrays, keyed text rows/metadata, plugin field/span aliases, escaping, hydration identity and unique names remain correct. |
| Full gallery core replay | **4/4** instrumented native/browser mode/mount cases pass: ordinary controls, replay/acquisition, language/preferences, dialogs, sanitizer parity, layout and disposal. |
| Round 1 — logic/complexity | Reviewed byte/row/identity boundaries, no partial mutation, exact strings, keyboard/composition and focus transitions. Extracted adjacent-row movement; preserved ordering and factory admission. Conditional publication fixes the real generated-adapter no-op risk. Six-package `m2-round1-final`: **138 passing native/doctest entries**. The later clean instrumented pass and browser matrix reverify final behavior. |
| Round 2 — responsibility/abstraction | Kept generic typed slots separate from chip interaction, private icon geometry and proof-only fixtures. Shared APIs/wire values stay unchanged. Local type erasure at icon/fieldset/tag boundaries resolves generated async-hydration type-depth overflow without changing recursion limits or adding dependencies. Workspace `cargo test --workspace --offline --locked -- --skip tcp_watchdog`: **734 native log pass entries + 940 doctest pass entries**, exit 0. Its 81 ignored entries remain disclosed; TCP is explicitly filtered to respect no-listener scope. |
| Round 3 — consistency/docs | Workspace/all-target Clippy, gallery/admin WASM Clippy in each separate mode, and six-package warning-denying Rustdoc pass. Rustfmt, diff whitespace, Python and changed Node syntax pass. Let-chain/assignment-spacing lint fixes preserve behavior; public/meaningful private declarations and state assumptions are documented. Native/UI and both browser-mode normal-edge dependency trees pass; browser graphs contain no native server/SQL/Tokio dependency. |
| Safety/soundness | Reviewed untrusted tag/metadata text and names, hidden-input fidelity, currentColor/decorative SVG semantics, disabled and mutation admission, Missing/Null preservation, stale identities, factories/locks, root disposal and compile-feature boundaries. Real native/WASM assertions pass. No unsafe code, raw SVG HTML injection, network icon source, new dependency, theme source or API/permission change. |

Material integration findings fixed: generic no-wrap actions plus inherited button margins widened the old nested proof at 375 px/200%; generic wrapping and explicit zero icon margins restore page containment. Nested tag add controls now wrap rather than press outside the field. Empty SSR error text uses Leptos's whitespace placeholder, so the proof checks semantic emptiness without trimming any user value. Attribute order is not assumed by the native label test. The first type-depth/owner-gallery-overflow logs are retained; no test weakened the public value, focus or overflow contract.

## M2 coverage, visuals and gate

`cargo-llvm-cov 0.8.6`, Rust 1.96.0, LLVM 22.1.2, native aarch64-apple-darwin. The helper cleans workspace objects immediately before measurement, then runs `cargo llvm-cov -p stackpress-framework-ui -p stackpress-example-ui-gallery-shared -p stackpress-example-ui-gallery-web --features stackpress-framework-ui/ssr,stackpress-example-ui-gallery-shared/ssr --offline --locked --no-report`: **69 tests pass**, no ignored entries. It builds instrumented finite native/proof hosts and replays the collection, breadth and full gallery browsers before LCOV export.

| Native crate scope | Covered / executable physical lines | Result |
| --- | --- | --- |
| stackpress-framework-ui | 3438 / 3617 | **95.05%** |
| stackpress-example-ui-gallery-shared | 605 / 680 | **88.97%** |
| stackpress-example-ui-gallery-web | 237 / 259 | **91.51%** |

All measured native scopes exceed 70%. `review/coverage-production.json`, LCOV/raw exports and retained profdata disclose source files, exact inline-test exclusions and uncovered lines. Counts include complete compiled src trees, conservatively including the shared proof fixture; dependencies/tooling/build output and test-only files/ranges are excluded. Generic instantiations coalesce per physical executable line. Browser-only client startup/DOM branches have no native implementation coverage; both modes are built/linted and exercised in real Chromium separately, and no WASM line-coverage, branch-coverage, installed-target or publication claim is made.

Inspected current screenshots: `gallery/screenshots/collection-icons.png`; `root-hydrate-text-rows-1280.png`, `root-hydrate-text-rows-375.png`, `root-hydrate-metadata-rows-375.png`, `mounted-csr-tag-rows-375.png` in that gallery screenshot directory; `collections-root-hydrate-375-zoom1.png` and `collections-mounted-csr-375-zoom2.png` there; and `roles/screenshots/plugin-hydrate-project-field-tags-375-2.png` plus `disabled-csr-proof-field-metadata-1280-1.png`. The final plugin crop shows the complete wrapped add input/button; the zoomed proof shows keyboard focus reaching the locally scrolled remove action.

Deleted only `/private/tmp/s8-polish-m2/target` at **3,013,496 KiB**, below the **8,388,608 KiB** cap; `cleanup.json` records removal. Retained binaries/assets permit port-free browser reruns without the target. KB validator output is byte-identical to this milestone's baseline: **146 existing errors / 25 warnings**, no new diagnostic; retained in `kb-validation.log`. No Cargo manifest/lockfile, tracked generated output, preference/theme or M3/M4 implementation changed.

Provenance: current branch `trial/3-s8-polish`, verified HEAD `f14cb53b59a40a9a5288c5b3d13cba9abbdc8c33` plus this uncommitted M2 diff. M1 source is present. The coordinator-reported merge reference d5f7564 is not an ancestor of this HEAD; no merge or history action was performed by the implementer.

Re-review the gallery Text rows/Metadata rows/plain tags and generated project tags/RoleProof collections: **00013-001** early draft/SSR behavior; **00013-004** field edits/removal and native keyboard/form behavior; **00013-010** narrow/200%/coarse-pointer layout; **00013-013** plugin/disabled collection consumers. Owner visual acceptance and coordinator managed-preview rebuilding/verification remain separate. **M2 stops here; do not start M3 until released.**


## M3: table/rating and Lucide 1.62.0

The coordinator accepted/merged M2 into trial/3 (reported `f46ee0a`) and released M3. M3 re-pins all seven Lucide masters/licences, implements the 48×12 gallery table and inline sortable headings, and renders keyboard/native-radio star ratings with the minimum-zero rule. The optional UI minimum is additive; generated callers derive it from existing public @min/ge metadata, with no new admin role prop or wire member.

Load the [M3 evidence receipt](00013-receipt-owner-findings-m3.md) for exact implementation/API boundaries, commands, three review rounds, final tests/coverage, inspected screenshots, retained artifacts and cleanup. This rolling receipt retains the prior milestones without changing their historical evidence. The coordinator accepted/merged M3 as reported `16b687c` and released M4. Earlier milestone stops in this receipt are historical handoff records.

## M4 and whole-task verification

Load the [final M4 receipt](00013-receipt-owner-findings-m4.md) for completed implementation, cache-isolation findings, three review rounds, fresh per-crate coverage, exact benchmark IDs/counter changes, retained evidence, cleanup and limitations. The [consolidated walkthrough](00013-review-preview-owner-checklist.md) supplies the owner's updated steps and expected results.

| Findings | Final disposition |
| --- | --- |
| F01 | M1: document HTML denial/error pages and review-only default Editor; API/export representations unchanged. |
| F02 | M0/M4: fixed external status asset, completion-gated bootstrap, visible observable-state recovery and real truncated-HTTP review proof. |
| F03/F04 | M4: spaced labelled links, Safety samples, normal ordinary fixtures and isolated review Full-name label; normal proofs retain hostile labels. |
| F05 | M4: raw Typed nested rows removed from owner gallery, fully retained in the dedicated proof fixture; nullable-boolean product control deferred. |
| D01/D02 | M2: inline icon actions and owner-approved plain text/X chips; TagList editing/reordering dropped only from that UI, exact values/limits/forms retained. |
| D03/D04 | M3: Lucide 1.62.0 pin, table headers/48×12 overflow fixture, native star rating and minimum-zero rule. |
| D05 | M1/M4: styled scoped role catalogue and readable list display without wire/value changes. |

Owner re-review: **001, 003, 004, 005, 009, 010, 012, 013**. Installed Part B and G39 remain separately gated. This implementation receipt does not claim owner acceptance or a commit.
