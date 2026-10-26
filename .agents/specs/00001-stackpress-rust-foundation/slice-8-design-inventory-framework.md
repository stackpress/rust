# Slice 8 framework, generation and language inventory

Companion to [owner decisions](slice-8-design-decisions.md), based on references [interfaces](../../references/00015-upstream-interfaces-and-experience.md), [view](../../references/00027-upstream-view-api-contracts.md), [session and language contracts](../../references/00028-upstream-session-language-contracts.md), [component catalog](../../references/00031-upstream-idea-component-catalog.md), [exposure recipes](../../references/00032-upstream-interface-exposure-examples.md) and [package supplements](../../references/00041-framework-package-supplements.md).

## Source and completeness rule

Read-only live HEADs: Reactus `357569a4923821d9086adf565c5ed22f0e082631` (clean), r22n `48c50dd1ba57a0555c9afbd4bfa4cb41e702cb68` (three pre-existing dirty files), Stackpress `a71d683051ba8350fdd12d6b5a33f268fdcc285f` (one pre-existing dirty file). The [Slice 5 view](slice-5-design-inventory-view.md) and [language](slice-5-design-inventory-language.md) inventories are incorporated as complete export/type/member/test indexes; this file changes disposition only for the rows below. Other rows retain their accepted implementation or intentional non-port. Commit IDs are not assertions that dirty source bytes equal HEAD.

Inspected current framework-view/UI source/docs, profile-ui shared/contracts/client/web/gateway/native/proofs/check tooling, Slice 5 designs/receipts including rounds, and source language/view/admin transforms. Runtime truth comes from current Rust source, not the old proposal's “Now” column. No other trial branch was read. Per-module Frui coverage is in [the component inventory](slice-8-design-inventory-components.md).

## Reactus and current view exports

| Every inherited export/type/member family | Disposition and reason |
| --- | --- |
| Document/DocumentRender renderMarkup, renderJSX, Head/style optional metadata; their public props/output/errors and test behaviors | Existing buffered Leptos/snapshot contracts retained. M7 adds an explicit async renderer after real Resource/Head/replay proof; streaming waits for Slice 7. Keep synchronous factories unchanged. |
| dev/build/serve/engine; Builder/DocumentBuilder build outputs; manifest/page/asset IDs/config/routes and inspection helpers | Existing finite pinned builds, static catalogs and checked assets retained. Production HTTP/static hosting belongs to Slice 7. No runtime TS module import, Vite middleware or MD5/base62 ABI port. |
| DocumentLoader/ServerLoader runtime imports/fetch/fs, VirtualServer/VFS, Vite css/file/hmr/vfs hooks and aliases, Node IM/SR/Next/Rollup types, FileLoader/NodeFS, missing package ./Manifest | Remain intentionally not ported as JS/Node APIs. Any Rust watch/HMR integration is a separately proposed Developer build tooling task after Slice 8; no listener or tool migration is needed now. |
| Reactus templates, server-only duplicate exports, Exceptions, BuildStatus/Results/DocumentImport/Iterator and remaining configuration/type aliases | Existing typed shell/asset/error/catalog responsibility or intentional source-ABI exclusion retained per Slice 5. Async output gets a new typed contract; no new string template engine. |
| PageId/PageLoad/PageSnapshot/PublicStatus/PageLimits/PageAssets/PageContract/RenderedPage/ViewError, PageSpec/PageLoader/PageRenderer/ServerBuilderViewExt and route catalog/base codecs | Existing implementation retained; additive loader-selected settings projection in M1. Async/streamed page registration in M7 is gated and preserves old serde shape and current eager Head contract. |
| setViewProps; ServerProvider/Request/Response/context/useRequest/useResponse/useConfig/useServer and public props/types | Existing explicit public projection retained; M1 adds only allowlisted locale/theme/flash/allowed-action settings. Do not port source normalized header/session/config dumps or developer stack to the browser. |
| LayoutProvider/Blank/Panel, Head/Left/Main/Menu/Right and Props | Existing semantic slots retained; M1 permission-aware menus and preference controls consume authoritative public projection. No hard-coded EN/TH/account/admin routes. |
| LayoutUser/useSession, ServerSession guest/can and permission types/pattern helpers | M1 adds safe display capability projection only. Full signed identity/account/route-pattern semantics belong to proposed Signed identity and authorization task, not an invented implementation of source Session. |
| ThemeProvider/context/useTheme and Props; notifier flash/notify/unload/useNotifier/Container, cookie/config/default aliases | M1 persistence/flash with explicit cookie scope, bounded safe payload and root timer cleanup; existing immediate state retained. No global mutable singleton or exactly-once cross-tab flash promise. |

Every Reactus test file listed in the Slice 5 inventory remains mapped: render/Head/error/asset/catalog tests become retained or M7 contract tests; constructor/JS/Vite/VFS/import/class ABI assertions remain intentionally not ported. Optional HMR assertions are later Developer build tooling, not acceptance evidence for async rendering. No upstream tests ran here.

## All schema component roles and aliases

UI transforms wait for Slice 6's normalized definitions. At most one relevant registered role per column is selected. Omitted roles remain omitted; component argument order is definition order; fixed attributes override Idea-supplied props. Preserve the distinction between role and runtime component.

| Complete component catalog group | Slice 8 disposition |
| --- | --- |
| field checkbox, email, input, password, radio, rating, select, slider, slug, small, suggest, switch, tags, textarea, url | M4 maps to proven M1/M2 controls after schema handshake. `small` is an explicit typed display/style option, not arbitrary CSS code. |
| field color, country, currency, date, datetime, integer, json, markdown, metadata, number, phone, price, time | M4 maps to M3 checked wrappers; JSON remains bounded text/tree editing, price is decimal plus explicit currency contract. Locale formatting is not storage coercion. |
| field file, image, filelist, imagelist | M4 may select proven bounded file/preview controls; persistent upload service remains later File and media transfer. Explicit unsupported service-backed props diagnose rather than silently emit broken upload UI. |
| field code, mask, editor | Later Media and rich-editor integrations. An explicit unsupported component is a generation diagnostic; never silently replace a rich editor with plain Input. |
| field datelist, datetimelist, numberlist, stringlist, textlist, timelist | M4 typed bounded collection wrappers over proven M2/M3 controls. Stable keys/error paths/order and null/empty distinctions matter. |
| field fieldset, relation | M4 nested fieldsets and typed relation selection; M5 uses actual bounded authorized Slice 6 relation search/details. No string-evaluated display template. |
| view capitalize, chars, comma, email, line, lowercase, phone, text, transform, uppercase, words, yesno, link, rel | M4 safe text/link/transform wrappers over M1/M2; relation output remains a display adapter. Unicode-safe clipping, explicit labels and fixed link policy. |
| view country, currency, date, number, price, relative | M4 typed M3 formatting. Relative time requires explicit reference time serialized with the page; it must not call ambient now during initial hydration. Unsupported custom format grammar diagnoses. |
| view code, color, html, image, json, markdown, metadata, rating, tags | M4 safe display/sanitized content. Code is escaped first; optional highlighting later. No arbitrary raw HTML from a metadata flag. |
| view carousel, film | Later Media and rich-editor integrations; avoid a generated runtime dependency before it is selected/proved. |
| view formula | Intentionally not ported under G25; explicit expression-language task would require owner approval. |
| view list, ol, spread, tabular, ul, overflow, fieldset, template | M4 collection/fieldset/display wrappers; template uses registered typed Rust renderer/data, never arbitrary source-string execution. Unknown template components diagnose. |
| Every filter/span name | Derive from the complete field definition registry; apply the same implemented/deferred/unsupported status. Placement differs from field. The `span` role is not a read-only view alias. |
| Every list name | Derive from view definitions with their props/fixed attributes and same status. No field-to-list inference. |

All field aliases remain registered: taglist→tags; string→input; text→textarea; float→number; boolean→switch; object/hash→json; strings/texts→stringlist; dates→datelist; datetimes→datetimelist; times→timelist; integerlist/integers/floatlist/floats/numbers→numberlist.

All view aliases remain registered: table→tabular; taglist→tags; clip→overflow; string→text; float/integer→number; boolean→yesno; datetime→date; object/hash→json; strings/texts/dates/datetimes/times/integers/floats/numbers/stringlist/textlist/datelist/datetimelist/timelist/integerlist/floatlist/numberlist→list. Filter/span inherit field aliases; list inherits view aliases. Canonical alias collisions/unknown props require generation diagnostics.

## Admin exports, routes, types and test behaviors

| Upstream family | Disposition and behavior to preserve |
| --- | --- |
| Admin plugin/config/types; model/pages/views/routes/package transforms; generated registration | M4/M5 native function transforms and compiled route registration through G20. Selected output only; generated existence does not expose an event automatically. No TS module/package rewrite ABI. |
| SearchQuery/Scalar; filter, order, paginate | M5 typed URL state: null removes filter, false/zero remain values; order cycles absent→desc→asc→absent; paging uses validated skip/limit. Strict bounded one-decode codec, no global window mutation in shared controllers. |
| LayoutAdmin and AdminConfig/Props/LayoutProps/PageProps | M5 explicit layout slots/action projection and host policy, no full session/auth claim. Deep links enforce the same server policy as visible links. |
| create/search/detail/update/copy/remove/restore | M5 actual typed operations, errors, total, nullable/default/optional fields, draft preservation, explicit remove/restore, copy omits identity. Generate restore only when data layer supports it. |
| Related search/create/import/export/detail views | M5 bounded related search/create first; relation import/export in M6. Preserve parent identity/foreign scope; data layer owns integrity/transaction behavior. |
| csvToFormData, CSVParseError/Results; batchImportSend, batchAndSend, BatchSendResults/Response; ErrorWithErrors | M6 bounded CSV parsing/preview/typed row errors/progress/cancel and explicit partial outcomes. Use per-row/transaction capabilities provided by Slice 6; do not invent all-or-nothing or automatic retry. |
| All generated field/filter/span/list/view wrapper Props and aliases | M4 compile tests of complete registered prop/role rules and output ownership. No `any`/arbitrary string interpolation fallback. Test fixed-prop collisions, omissions, aliasing, stale/remove/rename and real runtime accessibility. |

## r22n, language, session and other surfaces

- Existing Rust Language/LanguageContext/Provider/Translate and all source r22n exports/types (`Language`, context/provider props, useLanguage/changeLanguage, template/t/_/node slots) retain their Slice 5 disposition. Zero/false/empty and exact whitespace stay preserved, unlike upstream truthiness. Keep tested en/fr counted phrases; this is not a full ICU MessageFormat/plural API.
- Source Language static configure/locales/key/language/load and instance locale/label/translations/save/update/translate become M1 immutable registry/selection/explicit persistence; pure phrase interpolation remains existing UI. No mutable process-global language class. Locale routing no longer reconstructs a native Request and resolves a second lifecycle.
- Source Session configure/create/token/load/authorize/data/authorization/guest/can/permits/save, all identity/JWT/auth/profile types, signup/signin/signout/TOTP/account actions and wildcard permission matchers are later Signed identity and authorization. UI must not copy default seed `abc123` or empty-access allow-all. M1 requires explicit authoritative app admission, not a token substitute.
- Source CSRF plugin clear/generate/valid behavior is a boundary requirement for website mutations; implementation belongs to the app's explicit policy or a separately approved security package. Generating a hidden field alone does not establish enforcement.
- Electron window/preload/local-token/HTTP host is not ported. M6 extends Tauri capabilities/platform origins/prefs; native menu contribution ordering can be designed independently, but signing/notarization/updater/device integrations remain later Signed application distribution and platform integrations pending owner credentials/policy. Other-desktop support needs its own real target receipt.
- Reference 00032 page/event, API/OAuth and MCP recipes preserve separate caller/validation/disclosure responsibilities. This slice generates page/admin exposure only. Generic configured API/OAuth/MCP transports and their credentials/scopes are later named framework tasks; UI generation never exposes all model events to them.
- Email, migrations, SQL dialects and domain validation remain their owning packages/Slices 6/7. Studio is planning-only upstream; no schema editor or low-code studio is implied by generated admin pages.

Read source tests at each implementation milestone; source absence of tests (including r22n and PhoneInput in the reviewed inventory) creates a new Rust verification obligation. Every later/unsupported row is a visible closeout deferral, not omitted breadth.
