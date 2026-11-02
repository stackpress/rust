# Shared UI source parity receipt

Status: **Merged follow-up complete; awaiting coordinator verification**. Original implementation started at 99937b0; the original sections below preserve its pre-merge evidence. Current results are in the merged verification follow-up at the end. Branch: trial/3-slice9-csrf-email. The coordinator committed/merged the original work; the implementer made no commit.

## Source and changes

Read-only Stackpress root: `/Users/cblanquera/server/projects/stackpress/stackpress/packages`, commit `a71d683051ba8350fdd12d6b5a33f268fdcc285f`. Frui root: `/Users/cblanquera/server/projects/stackpress/frui`, commit `096fd14580f0f49b6e159fa0aa5ae2a5bce8fb0e`.

| Item | Original file:line and behavior | Implementation |
| --- | --- | --- |
| Boolean filter | `stackpress-view/src/transform/filter/boolean.ts:125–156`; `stackpress-schema/src/config/attributes.ts:3271–3277,5432–5435`: hidden 0, checked 1, no presence selector. | admin suppresses presence for Boolean filters only and mirrors hidden zero in the typed draft. Native checked/unchecked submission order is preserved. Other filter/patch presence behavior remains. |
| Row actions | `stackpress-admin/src/transform/views/search.ts:304,519–531`: ordered identities, empty sticky top/right heading, trailing sticky-right cell, exact GET/full-route can check, relative detail/path; denial retains the cell. | Every generated table with identities emits the column. Additive root RowActions carries base/can and optional href adapter. Exactly one GET check precedes href selection. Public DTO keys supply paths, independently of DOM row keys. Base stays literal; number interpolation matches source zero/decimal/exponent spelling. |
| Switch | `frui/src/form/Switch.tsx:104–158`, `frui/styles/form/switch.css`: default square/checkex/smooth presentation, 52×20 track, 22px thumb, transparent checkbox and wrapping label. | Leptos retains native id/name/value, checked state, role/ARIA, required/disabled, keyboard, change events and FormData. CORE_STYLES owns the source appearance and focus track. |
| Input / URL | `frui/src/form/Input.tsx:70–74` forwards attributes; `stackpress-schema/src/config/attributes.ts:3782–3808` maps field.url to native type=url. | Additive Input::input_type defaults to text. TextField forwards it; catalog field.url fixes url. The draft remains a String; no new URL validation is added. |

G43 governs the port. G42 remains unchanged: local view callbacks introduce no route ownership, arbitration or global authorization.

## Shared API and Rust mechanics

The current admin crate has no permission Admission seam. `components/sync.rs:7` is a private scalar-validation Result alias. `pages::Snapshot` provides mount/allowed, but is not a table context. The example maps this existing projection into root `RowActions`; applications needing row-specific permissions supply their exact callback. Table and page-dispatch signatures are unchanged.

Additive APIs: RowActions::new(base, can), with_href, RowDetail; optional sticky_right props on TableHeader/TableCell; Input input_type; decorative ChevronRight. Tables keep empty heading/cells when context is absent or a row is denied. Handler authorization remains independent.

Rust-mechanics adaptations: root Leptos context carries source props without breaking generated signatures; the approved href adapter retains the existing Rust query-key host; its validated mount drops its separator when becoming the source base, while shared base is verbatim. Explicit empty AnyView satisfies the required child slot without a hydration text space. input_type is the Rust spelling for native type. Default href remains source-relative detail/path.

Numeric identity formatting follows the existing private framework-api formatter pattern; Node template vectors and route/href assertions cover integral reals, signed zero, plain-decimal and exponent boundaries. API source is untouched. A coordinated common scalar formatter could remove existing API/query/admin duplication; extracting into another crate exceeds this scope. Admin does not depend on API; generic Leptos stays plugin-free.

## Consumers and generation

Inventory: facade; framework-admin/view/session; email integration tests; generated-admin tool/shared/client/web; leptos-independent; profile-ui client/shared/web; ui-format; ui-gallery shared/web; ui-preferences shared; ui-stream shared; review-preview. Cargo normal inverse graphs and Rust-reference counts are in `/private/tmp/shared-ui-parity/consumer-inventory.json` and graphs/. Existing Input calls retain text and table slots retain non-sticky defaults.

Both generated-admin roots, shared/src/generated and generated-format, were regenerated only through the official tool and checked twice. Variant proofs restore all managed bytes. No generated Rust was hand-edited. The nullable RoleProof URL fixture uses actual generated field/span adapters; the fixture draft display subscribes to URL and exercised Switch fields.

At the original 99937b0 base, api-checkpoint retains its existing generated SQL output and has no additional generated admin UI root. Its files, framework-api and framework-session remain unchanged. The separately merged API adapters are a later coordinator cleanup/integration point. No other trial branch was consulted.

## Distinct test-verified reviews

| Round | Findings and result | Record |
| --- | --- | --- |
| Baseline / implementation | 69 baseline, then 73 selected parity tests; corrected the test expectation for Leptos's empty AnyView marker. | 28034d7bcb32; 6dcc457b774d |
| 1 complexity | Reviewed Boolean transitions, ordered identity/path construction, absent/denied context, GET-before-href and callback counts. Corrected the fixture public null expectation. 125 passed. | 5fd98faff7d7 |
| 2 responsibility / abstraction | Source policy stays in admin; generic controls/icons/table mechanics in Leptos; mount/query adaptation in the host. No shared crate extraction was justified. 125 passed. Follow-up moved Switch CSS to CORE and table CSS to ADVANCED, which the real CRUD document includes; 127 passed. | 0d9731f3ea6e; b79c66afebba |
| 3 consistency / Rustdoc / comments | Documented additive APIs, Boolean exception, context/order and provenance. Removed shared base normalization; host adapts its own mount. 126 passed. | c9046ec541b2 |
| Follow-ups / safety | Shared Boolean browser helper removes ordinary/formatted duplication. Empty AnyView fixes both lint and hydration whitespace; source number spelling is covered. 76 targeted tests plus both workspace Clippy selections passed. Reviewed native event/signal ownership, root disposal, static SVG and escaping; no unsafe/new policy. | 930badcdc983; 0dfa9c02f948; JS e3c41708ff3a |

The URL browser check exposed a missing fixture output subscription, fixed locally. The formatted fixture still expected the removed Boolean presence selector and now shares the native FormData assertions. Clippy rejected unit child syntax; the empty-text alternative revealed Leptos's hydration space, so final output uses empty AnyView. These were local verification findings, not upstream behavior changes.

## Final gate

Final native gate d379c1dc9341 passed: default workspace 932/932 (86 skipped), compatible features 979/979 (88 skipped), 965 doctests; Rustdoc -D warnings; no-default admin 2/2 and Leptos 8/8; separate hydrate/CSR Clippy -D warnings for admin/gallery and formatted admin; minimal facade check. Final fmt and both workspace Clippy selections passed in 0dfa9c02f948. Default/feature test execution: 89.212 / 100.558 seconds. Native gate recorder wall time 1289.913 s; observed Cargo build subtotal 822.340 s. Coverage: admin: 2553/2916 executable lines, 87.55%, 19 source files; features codegen,format,ssr; record 11553b8e44b5. leptos: 5404/6245 executable lines, 86.53%, 44 source files; features ssr,rich,format,stream; record 72cb5fea4ba6. Reports: /private/tmp/shared-ui-parity/{admin,leptos}-coverage.json. rustc 1.96.0, aarch64-apple-darwin, LLVM 22.1.2, cargo-llvm-cov 0.8.6. Native selected-feature production scope, ordinary tool exclusions only; no added ignore patterns. WASM interaction paths are measured separately by browser proofs. RowActions itself covers 65/68 lines (95.59%); the fallbacks left native-uncovered are assessed by the source-parity review, not claimed covered.

All resolving Cargo commands use --locked --offline except the two explicitly approved standalone preparations below. Every Cargo/proof/clean command is recorder-wrapped at the outer proof boundary. Formatter/clean use supported flags. Gate: CARGO_INCREMENTAL=0, CARGO_PROFILE_DEV_DEBUG=0, CARGO_BUILD_JOBS=2, RUSTC_WRAPPER=sccache, SCCACHE_CACHE_SIZE=20G. Absolute main/coverage target: this worktree's target/slice9b. All workspace nextest runs exclude tcp_watchdog, owned by the coordinator. No renderer all-features build ran.

Compatible feature gate: csrf/server, email/server, session/auth-server, schema/providers, api/sql+codegen; admin/codegen+format+ssr; Leptos rich+stream; generated-admin web/tool format. Browser hydrate and CSR are checked independently. Main and standalone targets stay separate.

## Standalone locks and compilation

Consumer: `/private/tmp/shared-ui-parity/external-consumer/Cargo.toml`; target: target/slice9b/consumers/e4941338875b53b8fcb8. Sources and dependency artifacts remain warm. Its target began empty with warm sccache; the main tool target was warm.

Per coordinator ruling, root bytes were copied before one offline metadata resolution per manifest selection. Immediate assertions: default 190 external tuples, format 234; both subsets of root, root bytes unchanged. Evidence under private admin/crud: consumer-lock-checks.json; compile-consumer-metadata-Cargo.lock SHA 20d066e6f0149100820c109c62e4a96a56e6552440504e78ac0e2bb0a3fa310f; compile-format-consumer-metadata-Cargo.lock SHA 08a84a3b736f20140420c7a0f8fe8d6b3c46f9a1f1a058e3e4b5b5726a85c349. Later checks remain locked/offline. No update/generate-lockfile command ran.

Initial proof 7c76ed4c77d2 passed bootstrap/determinism, rename/removal, omitted/empty roles, hostile literals, format compilation, and expected mixed-revision/stale-consumer compiler failures. The native typed Input/Switch/sticky-cell/RowActions example compiles. Final current-code consumer check 3306f52c3d29 passed fmt, Clippy -D warnings, generation check and the direct API example; 234 external tuples remain a root subset, no refresh and unchanged root bytes. Retained report: private admin/external-final/receipt.json. The proof initially wrote its own logs to /private/tmp/s8-admin/external-final; 30a834f3eabe copied only those named current-run files into this task’s evidence directory. Both official roots then passed --check twice with no differences, plus final fmt --all --check.

## Browser evidence

Approved module: `/Users/cblanquera/.npm/_npx/e41f203b7505f1fb/node_modules/playwright`, used for ADMIN_PLAYWRIGHT and GALLERY_PLAYWRIGHT. Gallery reports Playwright 1.64.0 / Chromium 156.0.8078.4. Fresh contexts and intercepted .invalid requests execute finite processes without listeners. The coordinator rejected the system-Chrome fallback before execution; its override was removed. All successful runs use cached Playwright Chromium.

Matrix: gallery 4 root/mounted hydrate/CSR contexts; catalogue 24 base/plugin/disabled; CRUD 8; format 16 including replacement; transfer 8; write races 32. Admin covers both mounts and 1280px / 375px at 200%. Assertions include actual drafts, native Boolean FormData ordering, required/disabled/ARIA/Space/whole-track switch click, SSR/hydration node identity, named detail-link Enter navigation, denied empty cells and independent HTTP 403, horizontal scrolling, captured errors/unexpected requests. Final catalogue 24/24 passed (ce7f351b237b); gallery 4/4 passed (5190c6787e4e). Formatted matrix 16/16 passed (b957db076885); final CRUD 8/8 passed (1d5974607ad3); transfer 8/8 and write races 32/32 passed (d86cb0183f39); total 92 browser cases passed with no captured errors or unexpected requests. Final artifact records: roles de7b8ec59efd, format ad28d7a15b76, CRUD 026630ba912c.

Visual inspection: actual 16px white chevron on source info-blue button; checked/unchecked Frui toggle crops. Proofs, computed styles and screenshots remain under `/private/tmp/shared-ui-parity/{admin,gallery}`. Native coverage and Chromium behavior are separate measurements.

## Page-chrome review answers and follow-up

1. The captured CRUD document is real example-host output, not stripped by the harness. `examples/generated-admin/web/src/host.rs:487` responds with render::document. `web/src/render.rs:69` inlines CORE/ADVANCED/BREADTH and minimal local CSS, omitting BASE and page-chrome stylesheet links. The harness forwards its body unchanged (`proofs/http_browser.cjs`). Body font is system-ui; the surrounding chrome is partial/basic.
2. This is pre-existing and outside the four items. Follow-up: wire BASE_STYLES from `crates/leptos/src/components/styles.css` (normally LayoutProvider), and port/wire admin page chrome. Source `stackpress-admin/src/transform/views/search.ts:635–637` links /styles/global.css plus configured styles; this CRUD document lacks that chain. The separate catalogue document includes BASE at render.rs:12. No broader chrome fix was made.
3. Narrow text masking is intended sticky behavior, not transparent bleed. Source `frui/src/base/Table.tsx:363–379,674–707` selects right-edge stacking; `frui/styles/base/table.css:34–48` sets sticky/right; source admin styles :41–55 provide opaque cells. Rust computes right=0, sticky, z=2, opaque white; header z=3. The labeled horizontal region reveals the underlying column. This preserves that mechanism with the existing neutral Rust table surface.

## Footprint, resources and records

Cap raised by coordinator from 20 to 24 GiB; guard 19.4 to 23.4 GiB. Aggregate guard covers main and consumer subdirectories. One incremental-only trim freed about 2.39 GiB; subsequent cleanup is owned first-party packages only. External dependency artifacts remain. Sampled peak 21.013557 GiB (22,034,312 KiB), final 17.472218 GiB (18,320,948 KiB); 15-second samples in private sizes.jsonl. Both are under the 23.4 GiB guard and 24 GiB cap. These are periodic samples, not a continuous high-water measurement.

Root Cargo.toml/Cargo.lock and existing external tuples remain byte-identical; final audit checks API/session, owned manifests, scope, local links, syntax, browser receipts and licensing. Lucide pin 03f11e2d6adc319ea21e9a1d5820a99cd3767257; chevron-right SHA 2758143d7b2434e4aa7307dfd34405c87909ff4052f21b5f3f40d45224b4f19b; LICENSE SHA b495047bd93a9b06913511076f504daba17d5bbeb3e0650f3bb53a4220329c57. Lucide/Feather notices and copied Frui MIT license accompany the assets.

Final audit 246e9c4b3ab0 passed: 66 changed files including this receipt; only released prefixes; all owned manifests/root Cargo.toml/root Cargo.lock unchanged; all 699 registry tuples preserved, root lock SHA c8d65c84957998174460f32bbc3e56ac0695065f394ca3700f35b17700c94c4d. API, session and api-checkpoint baseline hashes match. Eight relative documentation links, changed CJS syntax, six plugin-free/browser-format graph comparisons, 92 browser receipts and icon/Frui licences passed. The final publication run compares KB diagnostics byte-for-byte with the captured baseline (146 errors, 28 warnings). They are inherited, not a clean KB result.

Store: `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`; short IDs identify runs/*-ID.json and matching logs. Completed command times below are recorder wall seconds and observed Cargo build subtotals. They overlap and must not be added. A dash means no observed Cargo build stage. Failed historical attempts remain visible; the final reruns above passed.

| Record | Result / label | Wall s | Cargo build s |
| --- | --- | ---: | ---: |
| 20261030T111500.710059Z-28034d7bcb32 | passed: baseline edit loop admin leptos | 67.554 | 48.220 |
| 20261030T111740.387456Z-b9c3f4bc182f | passed: implementation format | 8.968 | — |
| 20261030T111833.577577Z-aa4278850cf4 | passed: parity edit loop format | 5.843 | — |
| 20261030T111844.195550Z-35f60cf778e5 | failed: parity edit loop admin leptos | 47.277 | 36.570 |
| 20261030T112017.423783Z-6dcc457b774d | passed: parity edit loop empty-view assertions corrected | 9.117 | 2.110 |
| 20261030T112147.449248Z-761aac39d58d | passed: edit loop regenerate both admin outputs and check twice | 124.521 | 106.370 |
| 20261030T112931.006760Z-0f8f0cf5e013 | failed: edit loop generated consumers and parity | 152.771 | — |
| 20261030T113236.522981Z-bd63116c2304 | passed: trim owned incremental after headroom guard | 4.324 | — |
| 20261030T113329.004317Z-83c7f4f08feb | failed: round 1 logic and generated consumer verification | 115.302 | 72.000 |
| 20261030T113712.776702Z-5fd98faff7d7 | passed: round 1 verify URL fixture projection updated | 115.886 | 75.000 |
| 20261030T114002.871175Z-0d9731f3ea6e | passed: round 2 responsibility and abstraction verification | 92.735 | 42.580 |
| 20261030T114331.921812Z-c9046ec541b2 | passed: round 3 Rustdoc consistency and source base interpolation | 129.854 | 86.000 |
| 20261030T114635.691948Z-028879d62599 | passed: gate gallery native hydrate csr builds | 248.115 | 236.110 |
| 20261030T114727.967591Z-fc38c1ae7805 | failed: consumer preflight frozen local lock edge | 0.347 | — |
| 20261030T114756.976468Z-a616b60a12ca | failed: consumer full frozen metadata aggregate guard | 1.607 | — |
| 20261030T114939.408190Z-37cc675a2a52 | failed: consumer frozen original package identity | 1.940 | — |
| 20261030T115118.049151Z-8271a1b75dea | failed: consumer frozen identity manifest corrected | 3.729 | — |
| 20261030T115131.433222Z-1a771350064f | failed: gate gallery listener-free browser | 4.823 | — |
| 20261030T115437.067908Z-eed87378a439 | passed: gate admin native hydrate csr artifacts | 148.361 | 135.510 |
| 20261030T115503.645615Z-7cf576b3df6d | passed: gate gallery cached Playwright Chromium | 32.943 | — |
| 20261030T115918.261957Z-559300a365f6 | failed: gate admin cached Chromium source parity | 7.701 | — |
| 20261030T120821.181869Z-b79c66afebba | passed: followup stylesheet ownership tests | 160.892 | 65.000 |
| 20261030T121034.340262Z-a4c6c408decc | passed: test process diagnostic | 0.408 | — |
| 20261030T121142.326258Z-1ecd5da3363f | passed: gate admin CRUD native hydrate CSR artifacts | 68.953 | 55.380 |
| 20261030T121309.359428Z-86d573b32fd6 | passed: gate gallery final native hydrate CSR artifacts | 97.597 | 84.770 |
| 20261030T121315.027791Z-a308de8ee915 | passed: gate admin CRUD cached Chromium browser matrix | 113.286 | — |
| 20261030T121503.628973Z-622845114ea0 | passed: gate admin base plugin disabled artifacts | 186.695 | 119.530 |
| 20261030T121521.293207Z-5190c6787e4e | passed: gate gallery final cached Chromium matrix | 33.201 | — |
| 20261030T121709.750288Z-1a25c1c8646c | passed: gate admin sticky opacity browser matrix | 96.054 | — |
| 20261030T121850.175886Z-797dc601be54 | passed: gate admin formatted base plugin artifacts | 299.662 | 252.330 |
| 20261030T121856.056307Z-9f246ce7358a | failed: gate admin base plugin disabled browser matrix | 12.087 | — |
| 20261030T122019.980114Z-149377e27c43 | passed: gate admin transfer and write race browser regressions | 97.522 | — |
| 20261030T122312.722936Z-7793629b01d2 | passed: KB diagnostic baseline | 0.550 | — |
| 20261030T122401.636992Z-02ad0f41950c | passed: gate admin final catalogue artifacts | 146.691 | 91.220 |
| 20261030T122647.654502Z-4d6e7af8dfe2 | passed: gate final catalogue URL browser matrix | 84.303 | — |
| 20261030T122653.161946Z-f4355e2a83d4 | passed: gate final formatted base plugin artifacts | 119.826 | 76.580 |
| 20261030T122920.893555Z-4c6fa2db4848 | failed: gate final Boolean form and formatted browser matrices | 137.651 | — |
| 20261030T122920.893851Z-7c76ed4c77d2 | passed: gate standalone consumer cold target warm sccache | 267.241 | 225.610 |
| 20261030T123140.452306Z-a0fb160c9e48 | passed: gate unchanged browser dependency graphs and consumer inventory | 11.093 | — |
| 20261030T123239.139779Z-57b759642b5d | passed: gate shared Boolean form helper and format browser matrices | 244.852 | — |
| 20261030T123410.533126Z-db79b1d2b089 | failed: gate workspace fmt and Clippy selected compatible features | 60.837 | — |
| 20261030T123555.515569Z-83053778bf6c | passed: gate lint-clean generation both roots and twice checks | 38.540 | 19.000 |
| 20261030T123718.231763Z-7bf404589d50 | failed: gate lint-clean workspace fmt and Clippy | 6.151 | — |
| 20261030T123824.254078Z-907ebb569557 | passed: gate final workspace fmt and Clippy | 57.984 | 52.880 |
| 20261030T124026.696360Z-c855e91e0c90 | passed: gate final header catalogue artifacts | 146.050 | 79.410 |
| 20261030T124340.100508Z-55af8b94a872 | failed: gate final header switch URL catalogue browser matrix | 10.499 | — |
| 20261030T124340.101617Z-244b192deeb1 | passed: gate final header formatted artifacts | 117.112 | 72.760 |
| 20261030T124609.383140Z-5cf2bdb026ca | passed: gate empty AnyView generation and double checks | 63.787 | 17.150 |
| 20261030T124751.906609Z-930badcdc983 | passed: gate empty view unit and both workspace Clippy configurations | 190.657 | 154.610 |
| 20261030T125344.381005Z-0dfa9c02f948 | passed: gate source numeric identities targeted tests and Clippy | 67.051 | 39.680 |
| 20261030T125438.201109Z-e3c41708ff3a | passed: source JS numeric template reference | 0.167 | — |
| 20261030T125726.211717Z-d379c1dc9341 | passed: gate final workspace tests docs minimal and WASM checks | 1289.913 | 822.340 |
| 20261030T140540.858014Z-de7b8ec59efd | passed: final roles artifacts | 112.834 | 68.890 |
| 20261031T000342.634791Z-ce7f351b237b | passed: final roles browser | 64.990 | — |
| 20261031T000342.634848Z-ad28d7a15b76 | passed: final format artifacts | 78.985 | 51.740 |
| 20261031T015813.583039Z-b957db076885 | passed: final format browser | 116.812 | — |
| 20261031T015815.048020Z-026630ba912c | passed: final CRUD artifacts | 40.881 | 28.920 |
| 20261031T020019.400536Z-1d5974607ad3 | passed: final CRUD browser | 72.987 | — |
| 20261031T020020.879758Z-3306f52c3d29 | passed: final external consumer | 26.139 | 8.130 |
| 20261031T020520.262708Z-d86cb0183f39 | passed: final transfer and race browsers | 61.837 | — |
| 20261031T020521.531720Z-30a834f3eabe | passed: final generation checks | 21.848 | 4.870 |
| 20261031T021013.555119Z-11553b8e44b5 | passed: gate admin coverage | 56.386 | 43.540 |
| 20261031T021044.730897Z-246e9c4b3ab0 | passed: final footprint and evidence audit | 1.246 | — |
| 20261031T021121.786824Z-72cb5fea4ba6 | passed: gate Leptos coverage | 42.097 | 35.210 |

Closing receipt/scope/KB audit record: 20261031T021429.253729Z-8dab600494bf. Its final status and timings are in the store; publication fails if either audit changes the baseline. The ledger above contains 64 completed runs before that closing run.

Earlier failed attempts were resolved before the final gate: old size guard; initial assertion/nullable fixture expectations; frozen consumer metadata and inherited package-version trials before the approved one-off offline preparation; missing cached Chromium in the first selected runtime; misplaced CSS bundle ownership; fixture draft subscriptions; removed Boolean presence assertions; Clippy unit-child syntax; and empty-text hydration whitespace. No failure was waived.

## Merged verification follow-up

Coordinator committed the original work as 883126e, then merged 16f5255 into this branch as 443dfc0e4e36bb770766d6f5a24a3f25b1b34978. This follow-up starts at that merged HEAD; G42–G48 were read. No implementer commit or branch merge was made.

Failure: shared catalog field.url (stackpress.ui.field.url) was imported before API registered api.source.field.url. API definitions() rejected the second canonical registration before normalize(). Focused full-feature baseline 26004e9813ae reproduced the coordinator's failure. The initial selected-package command rejected unrelated workspace features (509007d04a07); the workspace-filtered retry used the exact accepted feature string.

Decision: use the expressly authorized fallback. API codegen now excludes field.url alongside its existing input/switch/date replacements. Shared TextField adds a nullable presence selector (including Clear (empty)), sp-role wrapper and aria-describedby links that API InputField lacks. Both are native URL/String controls, but their SSR structures differ, so replacement does not meet the requested SSR/hydration equivalence. The comparison is native SSR; it is not claimed as an API browser proof. API's local URL registration/import/fixed type remain intact, as do SwitchField and its detail-link adapter. No renderer, runtime adapter or shared catalog changed.

Regression: tests/url_roles.rs compares the actual normalized API logo/website descriptors against the shared URL binding obtained through the real shared registry/normalizer. Missing, Null, empty String and escaped URL String seeds retain their draft values; type/id/name/value attributes match; presence/wrapper/association differences are pinned. The final test plus the original normalization test fail under the old registration and pass under the fix (be01c2d8c250). Earlier fixture identity and HTML-attribute-order test helpers were corrected; no failure was waived.

Distinct reviews: round 1 checked canonical registration and optional draft branches (red/green proof); round 2 kept composition API-owned and test helpers local, with no justified shared-crate extraction (95 API tests passed, 881eaee93896); round 3 checked comments, feature guards and Rustdoc, with no further edit justified (API Clippy -D warnings, six doctests and Rustdoc -D warnings, 2bde47a91d39). Safety review: owned Leptos test roots and safe Rust only; no registration-policy relaxation, global gate or runtime behavior change.

Merged native gate d970dd109102 passed: fmt; both workspace Clippy selections -D warnings; default 941/941 (86 skipped), compatible features 991/991 (88 skipped), with tcp_watchdog excluded and nextest --test-threads 2; 966 workspace doctests; Rustdoc -D warnings; hydrate/CSR plus formatted browser Clippy -D warnings; no-default admin 2/2 and Leptos 8/8; minimal facade check. No-default Leptos retains its existing returns_focus dead-code warning; the strict Clippy selections are clean. The compatible feature string is the original gate selection documented above. API separately passed all 95 sql+codegen+ui-ssr tests and its six doctests/docs/lint check.

Merged browser proofs: CRUD 8/8 (c5e03ccea419) and gallery 4/4 (1f123d1f3ead), both freshly rebuilt on this tree. Playwright 1.64.0 / cached Chromium 156.0.8078.4, fresh headless contexts, finite request processes and intercepted .invalid URLs; no listener or system Chrome. No captured errors or unexpected requests. The first gallery run timed out on the native corpus startup before any browser case (15587a1de209); the unchanged retry passed within the existing deadlines. Artifacts: CRUD d7ce99020b5a, gallery 69dad3077e80.

Coverage 42b0120e86d9 passed separately for each crate: admin 2553/2916 (87.55%, 19 files), features codegen,format,ssr; leptos 5404/6245 (86.53%, 44 files), features ssr,rich,format,stream; api 3708/4146 (89.44%, 32 files), features sql,codegen,ui-ssr. Native executable-line scope, default tool filtering and no added exclusions; reports are private merge-followup/{admin,leptos,api}-coverage.json. rustc 1.96.0 / aarch64-apple-darwin / LLVM 22.1.2 / cargo-llvm-cov 0.8.6. Coverage tests use two threads. Native coverage and the 12 browser cases are separate evidence.

Standalone formatted consumer compiled and passed Clippy -D warnings and generation check (4f64bc466d0c), including the direct Input/Switch/sticky-cell/RowActions example. Its existing Cargo.lock passed --locked --offline without refresh (ab09bda8144a): 234 registry tuples remain a root subset, SHA 08a84a3b736f20140420c7a0f8fe8d6b3c46f9a1f1a058e3e4b5b5726a85c349. Root bytes are unchanged. No update/generate-lockfile command or offline resolution was needed. The retained consumer target remains consumers/e4941338875b53b8fcb8. Both official admin roots were regenerated without differences and checked twice; current browser normal graphs are checked against this merged HEAD.

Protected-scope audit f6f53067d0be passed and is repeated at closeout: root manifests/lock, framework-session, licences and .agents/context are unchanged against 443dfc0. All 699 registry tuples remain intact; root lock SHA 8687d9b425a56b3865918ba79bf211caa9bb4a9f17c40fdbbc64673a2574653e. Only API codegen/tests and this receipt changed. Final target 18,826,660 KiB (17.954502 GiB); overall periodically sampled effort peak 21.013557 GiB includes the original runs. Cap 24 GiB, guard 23.4; dependency artifacts retained, with only owned-package cleanup. KB is compared byte-for-byte with the merged baseline: 161 inherited errors, 28 warnings, not a clean KB result.

The real generated-admin document still omits BASE and the original /styles/global.css/configured admin page-chrome chain (examples/generated-admin/web/src/render.rs:69). This file is byte-identical to merged HEAD; the separate follow-up remains outside this fix.

Follow-up recorder ledger: 23 attempts including this closeout; full commands, failed attempts, IDs and timings are in the same persistent store and private merge-followup/benchmarks.json. Below: wall / observed Cargo build seconds (overlapping measures). Other record IDs: 509007d04a07:failed, 26004e9813ae:failed, 13001a3c4a31:failed, 8eb96598f628:failed, ccdbebebda2f:failed, 4190f67f0e4c:failed, d9bf033fda24:failed, ad25b34f94b8:passed, d7ce99020b5a:passed, 69dad3077e80:passed, 15587a1de209:failed, ab09bda8144a:passed, f6f53067d0be:passed.

| Record | Evidence | Wall s | Cargo build s |
| --- | --- | ---: | ---: |
| be01c2d8c250 | URL final regression red green and round 1 | 22.979 | 13.850 |
| 881eaee93896 | URL round 2 API responsibility regression suite | 4.724 | 0.520 |
| 2bde47a91d39 | URL round 3 API lint Rustdoc and doctests | 70.284 | 58.080 |
| d970dd109102 | full closing native gate | 692.390 | 364.120 |
| c5e03ccea419 | CRUD browser | 91.786 | — |
| 1f123d1f3ead | gallery browser retry after corpus startup timeout | 25.920 | — |
| 4f64bc466d0c | standalone consumer compile | 30.793 | 16.260 |
| ddc261cba39e | generation and browser graphs | 24.097 | 6.250 |
| 42b0120e86d9 | admin Leptos API coverage gate | 62.766 | 43.480 |

Closing receipt/scope/KB record: 20261102T222842.963806Z-22d9d1d8806c; its final status and timings are in the store after completion. No commits, listeners, installs, license changes or .agents/context edits were made.
