# Issue 17: independent Leptos library design checkpoint

2026-10-29, Asia/Manila. M0 design accepted; **M1 released** under G41. Branch `trial/3-issue17-leptos-library`, inspected HEAD/base `8ee2bdfa9feb24d5b864f1756696db68067cebaf`. The source inventory records M0; the M1 receipt records executable evidence and final signatures. The coordinator reviews and independently verifies; G41 resolves the two owner choices below. Acceptance criteria: none for this design-only task; implementation and visual parity require later evidence.

## Contract and authority

Read [issue 17 and its implementation comment](https://github.com/stackpress/rust/issues/17), [issue 14 and its coordination comment](https://github.com/stackpress/rust/issues/14), the current source/manifests, the Rust context/style/quality and coordinated/programming/benchmark workflows, all issue-15 task Markdown receipts/ledgers, and view's [migration](../../../../crates/framework/framework-view/docs/migration-issue15.md) and [host composition](../../../../crates/framework/framework-view/docs/presentation-host.md). Source observations below concern this HEAD, not the issues' older `74319ac` baseline. [G39/G40](../decisions.md) and the coordinator brief supersede historical System-theme sketches.

- Extract reusable UI into `crates/leptos`, package `stackpress-leptos`, Rust name `stackpress_leptos`. Upstream `leptos` remains a distinct dependency with its existing pins.
- View owns page/route/asset/status/seed contracts, renderer-trait implementations, presentation persistence and host integration. Its default contracts must acquire no Leptos or server dependency. Library imports and Cargo features never perform registration.
- No `framework_view::plugin::register` here: issue 14 owns that future composite. Existing `framework_view::{preferences,flash}::register` and their explicit admission/configuration remain unchanged. Issue 16's server paths are `crates/server/server` and `crates/server/http`.
- Preserve HTML, JSON/replay bytes and admission, escaping, eager/reactive Head behavior, cancellation, bounded buffering, cleanup and opt-in streaming. No redesign, dependency/toolchain upgrades, listeners, installs, commits, GitHub writes or implementation in M0.

## Inventory by source file

Paths in this section are relative to `crates/framework/framework-ui/`. **L** means move into the same relative module in `crates/leptos`; **V** means move under `framework-view/src/leptos/`; **S** means split as described. Every listed public item's variants, fields, trait implementations, associated constants, constructors and methods follow that item, preserving signatures except the explicit migrations below. Leptos-generated component `*Props` types and builders follow their component; private implementation modules do not become new public modules.

| File | Class and complete public surface / disposition |
| --- | --- |
| `Cargo.toml` | S: replace old member/dependency; generic dependencies/features to L; page integration and native host edges to V. Preserve every external package tuple. |
| `src/lib.rs` | S: upstream `leptos`, `leptos_meta`, `leptos_router` re-exports and generic module exports to L; render-mode compile errors remain there. V owns its optional `leptos` adapter module. No backward edge or plugin callback. |
| `src/state.rs` | L: `Theme`, `ThemeState`, `use_theme`, `NoticeKind`, `Notice`, `Notifications`, `NotificationOptions`, `use_notifications`; explicit Light/Dark and root-local state only. |
| `src/state/tests.rs` | L: theme isolation, notification limits/IDs/coalescing/disposal; no persistence fixture belongs here. |
| `src/language.rs` | L: `PluralCategory`, `Language`, `LanguageContext`, `use_language`, `LanguageProvider`, `Translate`. Keep checked dictionaries/interpolation/plural policy; no locale registry/cookie/host dependency. |
| `src/language/tests.rs` | L: phrase/slot/locale/budget and wire admission tests. |
| `src/resources.rs` | L (`async`): `ResourceFailureKind`, `ResourceFailure`, `cancel_on_cleanup`; preserve safe serialized failures, native cause and cooperative cancellation. |
| `src/replay.rs` | S: `ResourceReplay`, `MAX_RESOURCES=64`, `MAX_RESOURCE_ID=1024` to L (`async`), returning `ReplayError`; V supplies a page-policy/error adapter. Replace `PageLimits`/`ViewError` imports; use library-local script escaping, with differential parity tests. |
| `src/owner.rs` | L (`ssr`, native): private `RestoreOwner` and native Owner guards shared by extracted kernels; no public protocol type. |
| `src/render.rs` | S: `initialize_ssr`, executor admission, Owner staging/cleanup, eager Head handling and escaping to L's generic `render`; `LeptosRenderer`, `PreparedPage`, their constructors/`prepare`/`finish`, `PageRenderer` implementation and complete Stackpress shell to V's `render`. |
| `src/async_render.rs` | S: per-poll Owner restoration, `OwnedRender` lifetime, body/Head consumption and serialization-queue cleanup to L's `async_render`; `AsyncLeptosRenderer`, `AsyncPageRenderer` implementation, snapshot/assets/settings validation and page shell to V. |
| `src/stream.rs` | S: inseparable worker/runtime/LocalSet/supervisor/two-slot bridge, demand gating, cancellation/join, generic byte/deadline budgets to L's `stream`; `StreamLeptosRenderer`, page status, `StreamPage`, mapped `StreamBody`, `with_status_script`, page opening/recovery/replay/trailer protocol to V's `stream`. `StreamLimits` is generic and defined in L, re-exported by V. |
| `src/stream_replay.rs` | S: private `StreamContext`, pending-serializer ownership and cumulative replay admission to L; `<template data-stackpress-replay>` formatting belongs to V. Split `ReplayBudget::append` into checked newly ready data and view-owned framing. |
| `src/stream_status.js` | V: `STREAM_STATUS_SCRIPT` belongs with the unchanged Stackpress recovery/failure/completion element identities, never the independent renderer. |
| `src/stream_tests.rs` | S: worker/drop/backpressure/deadline/queue-cleanup tests to L; page opening, settings, status, marker/recovery and total framed-byte tests to V. Keep supervised child execution. |
| `src/routing.rs` | V in full: `RouteContext`, `provide_route_context`, `use_route_context`, `CatalogPath`; `BasePath`, `PageRoute`, `RouteCatalog`, `RouteSegment` re-exports resolve to view's existing root definitions. Generic users use upstream Leptos routing directly. |
| `src/components/mod.rs` | L: all existing component/value re-exports below; `BASE_STYLES`; preserve optional/target gates and `SuggestionCompletion`'s `async` gate. |
| `src/components/controls.rs` | L: `ButtonType`, `safe_href`, `Button`, `Input`, `FieldControl`, `Alert`, `Loader`, `LoaderContainer`, `Link`, `ListItem`, `List`, `NotifierContainer`. Keep the existing navigation policy and disabled-anchor semantics. |
| `src/components/display_values.rs` | S: all values remain generic in L: `DisplayError`, `ProgressValue`, `ProgressAmount`, `RatingRound`, `RatingValue`, `HexColor`, `EmailAddress`, `PhoneNumber`, `ImageSource`, `DisplayValues`, `DisplayRow`, `DisplayTable`, `TextLimit`, `overflow_text`, `MetadataValues`. `ImageSource` must use the library-local checked local location; `DisplayError::Location` changes from `ViewError` to `LocationError`. |
| `src/components/inputs.rs` | L: `ControlError`, `SelectOption`, `SelectOptions`, `SelectValue`, `Select`, `Checkbox`, `Switch`, `Radio`, `Textarea`, `PasswordInput`. |
| `src/components/input_dom.rs` | L: private WASM select-value adapter; retain target gate. |
| `src/components/presentation.rs` | L: `CORE_STYLES`, `ActionGroup`, `Badge`, `Card`, `CardHead`, `CardBody`, `CardFoot`, `CardTitle`, `CardDescription`, `Progress`, `ColorView`, `EmailLink`, `PhoneLink`, `Image`, `RatingView`, `YesNo`, `TextCase`, `TextTransform`, `TextOverflow`, `MetadataView`, `SpreadSeparator`, `Spread`, `Tags`, `Tabular`. |
| `src/components/layout.rs` | L: `LayoutProvider`, `LayoutBlank`, `LayoutPanel`, `LayoutHead`, `LayoutLeft`, `LayoutMain`, `LayoutMenu`, `LayoutRight`; retain explicit state/slots and `initial_theme_pending`, with no OS/storage acquisition. |
| `src/components/advanced.rs` | L: `ADVANCED_STYLES`, `Table`, `FieldError`, `FormState`, `Form`, `DialogMode`, `Dialog`. |
| `src/components/dialog_dom.rs` | L: private focus/trap/scroll/return-focus adapter; cleanup/stack ownership unchanged. |
| `src/components/choice.rs` | L: `ChoiceMove`, `ChoiceState`, `DisclosureState`, `TypeaheadState`. |
| `src/components/collection_state.rs` | L: `FieldValue`, `CollectionRow`, `BoundedList`, `MetadataEntry`, `CollectionValue`; retain stable identity and null/empty distinctions. |
| `src/components/collections.rs` | L: `CollectionFieldset`, `TextList`, `MetadataInput`. |
| `src/components/combobox.rs` | L: `SuggestionStatus`, `ComboboxState`, `SuggestionCompletion` (`async`), `Combobox`, `SuggestInput`; stale completion/draft/IME semantics unchanged. |
| `src/components/disclosure.rs` | L: `Accordion`, `AccordionItem`, `Details`, `TabActivation`, `Tabs`, `TabPanel`. |
| `src/components/geometry.rs` | L: crate-private `place_popup`, `effective_zoom`, unchanged target/test gates. |
| `src/components/navigation.rs` | L: `Crumb`, `BreadcrumbTrail`, `Breadcrumbs`, `Menu`, `MenuEntry`, `Menubar`; generic hrefs, not Stackpress catalog lookup. |
| `src/components/overlay.rs` | L: `PopupRole`, `Popover`, `Drawer`, `DelayMs`, `TooltipActivity`, `Tooltip`, `Toast`, `BREADTH_STYLES`. |
| `src/components/overlay_dom.rs` | L: private placement/focus/dismissal/delay/expiry helpers; listeners/timers remain Owner-cleaned. |
| `src/components/paging.rs` | L: `PageTotal`, `PageWindow`, `Pagination`; generic bounded paging/query syntax, no schema/SQL dependency. |
| `src/components/table_family.rs` | L: `TableHead`, `TableGroup`, `TableFoot`, `TableRow`, `HeaderScope`, `TableHeader`, `TableCell`, `TableRule`, `TableColumnLayout`, `DataTable`. |
| `src/components/table_state.rs` | L: `SortDirection`, `TableState`; presentation state, not generated data authority. |
| `src/components/tag_list.rs` | L: `TagList`. |
| `src/components/bounded_inputs.rs` | L: `RangeScale`, `RangeValue`, `RangeState`, `Slider`, `RatingState`, `RatingInput`, `SlugStyle`, `SlugAlphabet`, `slugify`, `SlugInput`. |
| `src/components/icons.rs` | L: private Lucide `IconKind`/`icon`; no new icon family or public API. Preserve accessible labels/decorative handling and licenses. |
| `src/components/styles.css` | L: exact `BASE_STYLES` bytes, including frozen initial-theme selectors. |
| `src/components/core.css` | L: exact `CORE_STYLES` bytes. |
| `src/components/advanced.css` | L: exact `ADVANCED_STYLES` bytes, including scroll lock. |
| `src/components/breadth.css` | L: exact `BREADTH_STYLES` bytes. |
| `src/components/tests.rs` | L: existing component SSR/escaping/provider tests. |
| `src/components/core_tests.rs` | L: existing core control/value/accessibility tests; adapt only the location-error cause type. |
| `src/components/breadth_tests.rs` | L: existing keyboard/state/collection/overlay/table tests. |
| `src/format.rs` | L (`format`): `DisplayLocale`, `FormatErrorKind`, `FormatError`, public `edit` module and number/phone/time re-exports. Preserve deterministic explicit-locale behavior and inline tests. |
| `src/format_number.rs` | L: `MAX_DECIMAL_BYTES`, `DecimalValue`, `NumberFormat`; preserve exact decimal grammar/precision. |
| `src/format_phone.rs` | L: `E164Phone`; pinned metadata, no region guessing/reachability claim. |
| `src/format_time.rs` | L: `CalendarDate`, `TimeOfDay`, `TimePrecision`, `OffsetDateTime`; explicit offsets, no ambient clock/zone. |
| `src/edit/mod.rs` | L under `format::edit`: `EditError`, `EditPolicy`, `EditState`, child re-exports, private `check_bounds`. |
| `src/edit/decimal.rs` | L: `DecimalPolicy`. |
| `src/edit/phone.rs` | L: `PhoneRegion`, `PhonePolicy`. |
| `src/edit/temporal.rs` | L: `TemporalKind`, `IsoValue`, `TemporalPolicy`. |
| `src/edit/selection.rs` | L: `SelectionDirection`, `EditSelection`; preserve UTF-16/UTF-8 checks and selection remapping. |
| `src/edit/input.rs` | L: `FormattedInput` and generated props. |
| `src/edit/dom.rs` | L: public WASM-only `adopt_native` and private DOM/undo/IME adapter. |
| `src/edit/transport.rs` | L: private UTF-16 transport admission. |
| `src/edit/tests.rs` | L: draft/commit/locale/selection/IME/bounds/transport tests. |
| `src/rich.rs` | L (`rich`): `POLICY_REVISION=2`, `MAX_INPUT_BYTES`, `MAX_OUTPUT_BYTES`, `RichError`, `SafeHtml`, `RichContent`; unchanged closed sanitizer/Markdown policy and inline tests. |
| `tests/executor.rs` | S: generic executor/idempotence checks to L; view error mapping assertions to V. |
| `tests/cleanup.rs` | S: generic Owner/unwind/Head lifecycle checks to L; `PageSnapshot`/`PreparedPage`/`ViewError` assertions to V. |
| `tests/async_feasibility.rs` | L: upstream feasibility/counterexamples and guarded-resource lifetime proofs, retaining child completion markers. |
| `tests/async_renderer.rs` | S: Send migration/Owner/resource cancellation/unwind matrix to L; page output/contracts to V. |
| `tests/routing.rs` | V: catalog path matching, mounted Head/router/assets/limits. |
| `tests/system_theme.rs` | V: despite historical filename, current test covers saved Light/Dark and unset bootstrap; do not restore System. |
| `tests/suggestions.rs` | L: yielding suggestion cleanup/stale completion. |
| `tests/support/async_watchdog.rs` | S: reusable finite child supervision support follows both suites without depending on the other production crate. |
| `third-party/lucide/{arrow-down,arrow-up-down,arrow-up,plus,star,trash,x}.svg` | L: copy/move exact assets with `NOTICE`, `LICENSE`, `FEATHER-LICENSE`; verify include paths. No icon artwork change. |
| `docs/**` | S: component/state/language/format/rich/direct-library guides to L; page renderer/prepared-page/catalog-routing and Stackpress bootstrap recipes to V. Rebase API indexes, links, examples and feature names; retain generic route/Leptos guidance in L. |

## Accepted decisions

G41 (owner, 2026-10-29) removes `stackpress-framework-ui` outright, with no shim, including old member/facade features/aliases/lock entry. All callers migrate in M1. No shared safety package or move into `stackpress-lib` is authorized.

Each library owns its location validation and HTML/script escaping. Leptos receives local `navigation::{LocalLocation,LocationError}` retaining the exact strict `PublicLocation` admission policy needed by images, plus its own escaping helpers. View keeps `PublicLocation` and page/script escaping unchanged. Image raster/suffix/dimension/byte checks remain; broader `safe_href` is a separate policy.

A single normal view-integration test table, gated by `leptos`, runs identical valid/invalid location cases against both implementations and compares script/HTML escape outputs. Cover valid/malformed percent escapes, dot segments, Unicode, query and exact length limits, script terminators and Unicode separators. This differential guard is required; neither implementation may weaken.

## Renderer and replay seams

Proposed public L modules are `render`, `async_render`, `stream`, `replay`, `resources`, `navigation`, `components`, `state`, `language`, `rich`, `format`; existing gates remain. View exposes adapters at `stackpress_framework_view::leptos::{render,async_render,stream,routing,replay,presentation}`. Generic kernels have **no** page IDs, statuses, snapshots, assets, preferences, Stackpress DOM identities or `ViewError` in signatures, errors or implementation imports.

| Path | Generic execution and local result | View's policy and page result |
| --- | --- | --- |
| Sync | `render::{initialize_ssr,Renderer,PreparedRender,RenderLimits,RenderError}`. A factory runs synchronously under a fresh Owner; stage owned body/Head; clean Owner before returning; finish eager Head against an explicitly supplied trusted shell. No reactive Owner in the Send assembly future. | `leptos::render::{LeptosRenderer,PreparedPage}` retains old entry signatures and `PageRenderer<T>`. Validate executor/assets/snapshot/settings, serialize seed/assets, join mounts, provide route/RequestUrl, add canonical/style/theme metadata, assemble shell and call `RenderedPage::with_limits`. |
| Buffered async | `async_render::AsyncRenderer` owns the existing `OwnedRender`/SSR shared-context lifetime through bounded body, replay and reactive Head assembly. Enter/restore Owner per poll; clear serialization/deferred cycles on every drop. Return one owned buffered result through a Send future. | `leptos::async_render::AsyncLeptosRenderer` implements `AsyncPageRenderer<T>` and supplies public data, contexts and shell. Keep its distinct contract; synchronous rendering must not silently become async component rendering. |
| Stream | `stream::{StreamRenderer,StreamLimits,StreamBody}` owns exactly one existing supervisor thread/runtime/LocalSet and the two-slot bridge. A staged generic opening/body has no HTTP status. Preserve first-demand start, reserve-before-poll, finite deadline, failure/explicit-completion distinction, close-before-cancel and synchronous join on drop. | `leptos::stream::{StreamLeptosRenderer,StreamLimits,StreamPage,StreamBody,STREAM_STATUS_SCRIPT}` preserves status, failure/recovery/completion markers, checked status asset, mounted module/theme/CSS and inert replay templates. Wrapper maps errors/bytes without spawning any additional worker. |
| Replay | `replay::{ResourceReplay,ReplayError,MAX_RESOURCES,MAX_RESOURCE_ID}` validates IDs/count/uniqueness/logical bytes and expanded input before JSON decoding. Generic stream bookkeeping admits cumulative replay, then returns data for caller framing. | `leptos::replay::ResourceReplay` is a thin checked wrapper retaining old `new/decode/script_json/entries -> ViewError` behavior and `PageLimits` ceilings. Use it in Stackpress clients rather than scattering `map_err` rules. |

The cross-crate seam uses explicit trusted factory/context/assembly closures and owned output, not a new rendering trait hierarchy. The generic staging operation accepts a fallible component, conceptually `try_prepare<V,E>(factory: impl FnOnce() -> Result<V,E>) -> Result<PreparedRender,RenderError<E>>`, `V: IntoView`; only the existing synchronous construction phase may retain `V`. A fallible shell closure receives rendered body and validated replay while the appropriate Head/Owner lifetime still exists. It returns the caller's document before Head injection/final byte admission. The infallible convenience uses `Infallible`.

Public use sketches (proposed paths, not compile evidence):

```rust
// Independent UI: values and callbacks, with no page protocol.
use stackpress_leptos::{components::Input, state::{Theme, ThemeState}};
use stackpress_leptos::leptos::prelude::*;
let owner = Owner::new();
owner.with(|| {
    let theme = ThemeState::new(Theme::Dark);
    let value = RwSignal::new("Ada".to_owned());
    theme.set(Theme::Light);
    let _control = view! {
        <label for="independent-name">"Name"</label>
        <Input id="independent-name" name="name" value=value/>
    };
});
// Stackpress page integration keeps the existing checked constructor shape.
use stackpress_framework_view::leptos::render::LeptosRenderer;
// LeptosRenderer::new(contract, assets, |page: &PageSnapshot<Data>| Ok(view! { ... }))
// implements view::PageRenderer<Data>; it does not select/register a plugin.
```

`RenderError<E>` keeps component/assembly `E` separately from generic executor/policy/output/replay/panic causes. V unwraps an existing `ViewError` unchanged; maps executor to `ExecutorUnavailable`, output to `HtmlLimit`, replay invalid/count/byte/policy cases to their existing `InvalidSnapshot`/`SnapshotLimit`/`InvalidPolicy`, and retains native panic/I/O causes via `ViewError::from_source`. Match these mappings and old error precedence in tests. Replay's own `ReplayError` never mentions page policy; V validates that policy first. Zero budget still denies output, never means unlimited.

For streaming, move the mature supervisor and its ownership as a unit. Trusted framing closures produce opening/fragment/trailer and late-failure bytes inside that same worker; the generic byte budget includes **all** framed bytes and reserves caller-supplied failure bytes before publication. V supplies unchanged markers and settings. Do not map a generic fragment into larger unchecked HTML after the generic budget check. Component/closure bounds must retain current Send/Sync staging guarantees while constructed views stay local. Prove these bounds before completing the wider move; if a seam cannot preserve them, stop for coordinator review rather than adding detached tasks or unsafe assertions.

Preserve evaluation order: sync/async admit executor, validate assets then snapshot/settings, bound/escape seed/assets and mount paths before evaluating the component; stream keeps its existing constructor validation, executor/snapshot/seed admission before worker launch and precommit Head/opening checks before publishing. Keep each path's existing metadata order (including sync canonical Link and async/stream behavior), rather than harmonizing their output during extraction. Synchronous Head stays eager; async Head stays Owner-live until completion; stream Head freezes at precommit. None of the byte ceilings bounds arbitrary trusted component CPU/allocation or makes blocking destructors preemptible.

Replay compatibility pins the exact JSON array-of-`[id, encoded-string]` shape, IDs, codecs, escaped delimiters/U+2028/U+2029, logical versus sixfold-expanded byte accounting, and cumulative stream limits. V/browser bootstraps retain seed/asset identities, `data-stackpress-replay` removal, completion-before-hydration, and globals required by pinned upstream hydration. No executable replay, second ID allocator or parser is introduced.

## Target dependencies and features

Arrows mean normal dependencies. G41 keeps safety code local to each library; no additional package is introduced. All external dependency versions/features are retained unless an edge moves with its existing responsibility; no re-resolution or upgrades.

```text
independent consumer -> stackpress-leptos -> upstream leptos/meta/router + generic helpers
Stackpress app -> framework-view --optional leptos integration--> stackpress-leptos
                       |--existing optional server--> stackpress-lib + stackpress-server
                       |--existing preferences/flash/native-file--> their current owners
framework-admin -> stackpress-leptos + framework-schema; codegen remains opt-in build tooling
stackpress facade -> each explicitly selected library; no library -> facade edge
```

No feature of L depends on view, server, facade, framework-language/preferences, SQL/schema/admin, Idea or Tauri. Normal graph audits include transitive edges and inspect selected target features, not just the top-level manifest. A shared workspace lock may contain unrelated packages; that is not evidence of a selected graph edge.

| Package/feature | Exact intended feature forwarding / resulting graph |
| --- | --- |
| L `default=[]` | Components/state/language/navigation and upstream Leptos/meta/router with defaults off, serde, local safety; no renderer runtime/server or optional async/rich/format dependency. Zero selected render modes is valid for helpers. |
| L `async` | `dep:futures`, `dep:serde_json`; generic resources/replay and async suggestions, with no native host. |
| L `ssr` | Upstream `leptos/ssr`, `leptos_meta/ssr`, `leptos_router/ssr`, `async`, `dep:hydration_context`, `dep:serde_json`, native-only `dep:any_spawner`, `dep:tokio`; **no** view/server forwarding. Preserve existing pins and executor initialization. |
| L `hydrate` | `leptos/hydrate`; WASM browser DOM dependencies stay target-specific. Add `async` explicitly for replay/resources, as current clients do. |
| L `csr` | `leptos/csr`; same browser isolation; add optional helpers explicitly. |
| L `stream` | `ssr`, `tokio/sync`, `tokio/macros`, `tokio/time`; native opt-in only. Default/ordinary `ssr` never selects it. |
| L `rich` / `format` | Existing ammonia/pulldown-cmark and ICU/phone dependency lists respectively; preserve existing compiled-data choices. No host edge. |
| V `default=[]` | Existing page/access/theme contracts + serde/serde_json with its existing local safety; no L/upstream Leptos/server/language/cookies/files. `PageRenderer`/`AsyncPageRenderer` remain engine-independent. |
| V `leptos` | `dep:stackpress-leptos`, dependency `path="../../leptos", optional=true, default-features=false`; routing/presentation adapter namespace, no server. Upstream Leptos/meta/router dependencies are also optional on V: upstream component macros resolve those crate names directly. They use the same pins and defaults-off flags; no unconditional Leptos edge is added. |
| V `leptos-async` | `leptos`, `stackpress-leptos/async`; page replay wrapper available to native/browser admission, no server. |
| V `leptos-ssr` | `leptos`, `leptos-async`, `stackpress-leptos/ssr`, existing `server`; native sync + buffered async page adapters. |
| V `leptos-hydrate` | `leptos`, `stackpress-leptos/hydrate`; no native services; replay requires explicit `leptos-async`. |
| V `leptos-csr` | `leptos`, `stackpress-leptos/csr`; no native services. |
| V `leptos-stream` | `leptos-ssr`, `stackpress-leptos/stream`; no implicit selection by preferences/flash or future base plugin. |
| V existing features | Keep `cookies`, `preferences`, `flash`, `web-preferences`, `web-flash`, `native-file`, `server` forwarding and target gates unchanged. None enables `leptos*`. Preference/flash mapping helpers require both their existing feature and `leptos`; no new persistence feature. |

L rejects SSR+hydrate, SSR+CSR and hydrate+CSR together, and SSR on WASM, as today. V transitively preserves those failures; its server/native-file WASM refusals remain. Hydrate and CSR artifacts must be built separately, including any workspace feature-unification effects. Never use `--all-features` for either renderer crate after this change.

Facade proposal: `leptos`, `leptos-ssr`, `leptos-hydrate`, `leptos-csr`, `leptos-async`, `leptos-stream`, `leptos-rich`, `leptos-format` expose `stackpress::leptos` and forward only to L. `framework-view-leptos` enables `framework-view`, `leptos` and V/`leptos`; suffixes `-ssr`, `-hydrate`, `-csr`, `-async`, `-stream` add the corresponding V feature. Existing `framework-view*` preference/flash/server/file aliases stay. The default facade remains unchanged. There is no old `framework-ui-stream` facade feature today; do not describe it as an existing API. L's `leptos` re-export is upstream, so `stackpress::leptos::leptos` is deliberately distinct from the normal upstream `leptos` import.

## Public migration and caller changes

| Old public path / feature | Proposed canonical destination |
| --- | --- |
| `stackpress_framework_ui::{components,state,language,resources,rich,format}::*` | `stackpress_leptos::{components,state,language,resources,rich,format}::*`; preserve all listed nested paths, including `format::edit`. |
| `stackpress_framework_ui::{leptos,leptos_meta,leptos_router}` | Same re-exports under `stackpress_leptos`; upstream imports may remain direct. |
| `stackpress_framework_ui::render::initialize_ssr` | `stackpress_leptos::render::initialize_ssr`; V's `leptos::render` provides a thin wrapper retaining the old `ViewError` return type. |
| `stackpress_framework_ui::render::{LeptosRenderer,PreparedPage}` | `stackpress_framework_view::leptos::render::{LeptosRenderer,PreparedPage}` / V `leptos-ssr`; retain old public methods and page result types. |
| `stackpress_framework_ui::async_render::AsyncLeptosRenderer` | `stackpress_framework_view::leptos::async_render::AsyncLeptosRenderer` / V `leptos-ssr`. |
| `stackpress_framework_ui::stream::{StreamLeptosRenderer,StreamPage,StreamBody,STREAM_STATUS_SCRIPT}` | Same names under `stackpress_framework_view::leptos::stream` / V `leptos-stream`; no HTTP status in L's different generic types. |
| `stackpress_framework_ui::stream::StreamLimits` | `stackpress_leptos::stream::StreamLimits`, re-exported by V's stream adapter. |
| `stackpress_framework_ui::replay::{ResourceReplay,MAX_RESOURCES,MAX_RESOURCE_ID}` | Stackpress callers: `stackpress_framework_view::leptos::replay` / `leptos-async`, preserving ViewError. Independent callers: L's `replay`, using `ReplayError`. Constants re-exported by V. |
| `stackpress_framework_ui::routing::{RouteContext,provide_route_context,use_route_context,CatalogPath}` | `stackpress_framework_view::leptos::routing` / V `leptos`; keep mount/context semantics. |
| `stackpress_framework_ui::routing::{BasePath,PageRoute,RouteCatalog,RouteSegment}` | Prefer existing `stackpress_framework_view::{BasePath,PageRoute,RouteCatalog,RouteSegment}`; adapter module may re-export these same types. |
| `components::DisplayError::Location(ViewError)` | L `components::DisplayError::Location(LocationError)`; downcasts/pattern matches intentionally migrate. `ImageSource::new` retains its args and rejection table. |
| `stackpress::framework_ui` / `framework-ui{,-ssr,-hydrate,-csr,-async,-rich,-format}` | Generic use: `stackpress::leptos` / matching `leptos*`. Page use: `stackpress::framework_view::leptos` / matching `framework-view-leptos*`; select rich/format directly on L. Old aliases removed if owner accepts removal. |

`framework-admin/Cargo.toml` replaces the UI dependency with L; `format/ssr/hydrate/csr` forward to L only. Migrate `src/components.rs`, `components/{catalog_inputs,collection_inputs,formatted,formatted_tests}.rs` and `codegen/table.rs` imports, preserving role validation and safety restrictions. The generator's emitted imports must change before regenerating `examples/generated-admin/shared/src/{generated,generated-format}` with the official generator; run a clean second generation/`--check`. Audit template strings, not only Rust use statements. Preserve schema/SQL/codegen ownership and opt-in boundaries.

Migrate root workspace/dependencies/lock/facade/Rustdoc; all old UI-owned tests/docs; generated-admin `{tool,shared,client,web}` manifests, render/docs/proofs; `profile-ui/{shared,client,web,native,gateway}` plus contract references/proofs; `ui-gallery/{shared,client,web}`; `ui-preferences/{shared,client,web}`; `ui-stream/{shared,client,web}`; `ui-format`; and `review-preview`. Not every listed manifest has a direct UI edge: inspect forwarding features, generated imports and dependent proof drivers as well. Component-only callers select L; page/catalog/seed callers additionally select explicit V integration. For example, Profile shared's routing context requires V/`leptos`; gallery/stream replay decoders require V/`leptos-async`; native web renderers require V/`leptos-ssr` or `leptos-stream`.

Update finite build/browser/graph/coverage/proof scripts, package-name selectors, source/include paths, asset manifests and active READMEs/Markdown examples together. Generated-admin's custom document renderer uses generic components/initialization, so do not force a new page-protocol dependency there. Profile codegen and other generators must still pass `--check`. Retired-name scanning includes hidden active configuration as well as crates/examples/root; explicit historical golden input and old receipts are excluded with a named allowlist, never a broad directory exclusion. Historical decisions/status/context/task records are not this lane's write scope; record proposed live-KB promotions in the issue-17 closeout for coordinator/owner handling.

Preferences bridge: V optionally maps `Some(ThemePreference::Light/Dark)` to L's `Theme::Light/Dark`. `None` remains absence in saved/page state; it is **not** a third UI variant. SSR/hydration start with the same Light tree and pending marker when unset, then adopt the already frozen document color; CSR/native adapters acquire their value at the existing boundary. No-JS/blocked bootstrap stays readable Light. Keep the 248-byte one-shot asset, no live tracking or initialization save, exact existing acknowledgements and early draft/focus/node retention. Dictionary, flash-to-notice and allowed-action display adaptation consume explicit public values/callbacks; host authorization remains outside UI state.

## Compatibility recommendation and consequences

**G41 accepts outright removal of `stackpress-framework-ui`, with no shim**. Nothing is published; migrate every in-tree caller in M1, delete the member/old package lock entry and old facade features/alias, and document deliberate Rust path/error-payload changes. G41 explicitly decides UI removal. No wire/HTML behavior break is proposed.

No shim or old aliases remain. Existing docs become migration guidance, not evidence that old APIs compile.

## Independent consumer and proofs

Create a small maintained fixture at `examples/leptos-independent` after release, with its **own `[workspace]`**, explicitly excluded from the root workspace so workspace feature unification cannot hide an integration edge. Its direct normal dependencies are upstream `leptos = =0.8.22` and path `stackpress-leptos` with defaults off; WASM ABI/browser and native test-runtime dependencies are target/dev-only and use existing locked versions. It has no direct or transitive view/server/facade/preferences/language-integration/SQL/schema/admin/Idea/Tauri edge. No shared safety package is added.

Fixture features: `default=[]`; `ssr=[leptos/ssr,stackpress-leptos/ssr]`; `hydrate=[leptos/hydrate,stackpress-leptos/hydrate]`; `csr=[leptos/csr,stackpress-leptos/csr]`; `async`, `rich`, `format` forward to L; `stream=[ssr,stackpress-leptos/stream]`. Reject mixed modes. Seed its disposable lock from current approved workspace tuples and permit only local-edge changes; never `cargo generate-lockfile`. Record an audit of every external name/version/source/checksum tuple for both locks. If the already installed offline cache cannot resolve the fixture without a new external tuple, stop and report rather than install/upgrade.

Use one actual root component containing explicit theme/language state, `Input`, a counter/action, `Popover`, and notifications, plus optional async `Resource`/Suspense and format/rich samples. Pass initial values directly; use a small fixture-owned inert seed/shell/bootstrap with its own identities, rather than importing/copying PageSnapshot or application examples. Use L's real generic renderer, then the exact same component for hydrate/CSR. This proves reusable rendering as well as imports.

| Proof | Required observable evidence |
| --- | --- |
| Native SSR | Real escaped component/document output, title/Head, theme/language isolation across two roots, checked images and byte-limit failures; Owner release on success/error/unwind. Exercise buffered async with success/cancel/panic and generic stream slow-consumer/drop/deadline without page/status types. |
| WASM hydrate | Separate locked `wasm32-unknown-unknown` build and matching installed bindgen; hydrate real native-rendered HTML with stable node identity, early input/focus retention and working counter/popover/theme/language/notification interactions; no warnings, duplicate mount or protocol globals imported from view. Async replay uses checked generic data. |
| WASM CSR | Separate locked build/artifact; mount the same components from explicit initial values, exercise identical interactions and disposal/repeated-root cleanup; no SSR/hydrate feature leakage. |
| Optional helpers | Rich hostile-content/closed-policy fixtures; format/IME/selection/locale/draft fixtures; async stale completion/cancellation; stream ownership/aggregate framing/deadline. Native coverage and browser execution evidence reported separately. |
| Graphs | Fixture default/SSR/SSR+stream/hydrate+async/CSR+async and helper selections; L default/modes; V default and isolated leptos modes; admin modes; facade aliases. Inspect target-selected normal edges (excluding build/dev where appropriate), assert forbidden package sets, assert default V has no L/upstream Leptos. Browser graphs exclude native server/SQL/Tauri/Idea/libc/runtime services. |

Browser proofs use finite request interception and real generated artifacts, never a listener. A gallery/profile application is parity evidence, not the independent consumer. No installed desktop/mobile, TCP/TLS acknowledgement, numeric WASM coverage or owner visual acceptance is inferred from these proofs.

## Milestones and verification process

| Gate | Work and what it proves |
| --- | --- |
| M0 — stop here | Source inventory, proposed APIs/features/safety ownership, migration/compatibility and proof plan only. Coordinator accepted the design; G41 resolves removal and local safety ownership. No implementation/build proof claimed. |
| M1 — extraction, adaptation, migration; stop | First prove fallible factory/Head lifetimes, Send futures/body, worker/drop/error translation and safe location parity with production types under watchdogs. Then extract L/local safety, add optional V adapters and migrate facade/admin/templates/all callers together. Existing component tests and pinned before/after page/seed/HTML/replay/error/route/theme outputs prove behavioral parity; generators prove current/idempotent output. Run relevant native tests/doctests/lint/docs/format and separate actual WASM builds/browser regressions, measure changed production crates; stop before M2. |
| M2 — independent consumer, graphs, docs; stop | Implement/execute the fixture above; audit all selected default/native/browser normal graphs, invalid-mode failures, feature-on/integration-absent behavior, lock tuples and source migration completeness. Produce L's README/getting-started/concepts/guides/API docs and V's integration/migration docs using chrisai-docing; compile-check Markdown examples in the isolated consumer. Proves independent use and optional integration, not release/platform acceptance. |
| M3 — reviewed rounds; stop | Four distinct test-verified passes: complexity/branch/error-order risk; responsibility/abstraction/ownership/coupling; consistency/idiomatic Rust/Rustdoc/block and non-obvious-line comments; safety/soundness at validation/unwind/cancel/Send/stream/CSP boundaries. Refactor only justified findings, rerun meaningful affected tests after each pass, and repeat targeted rounds while material findings remain. Coordinator independently re-verifies before closeout. |

Parity inventory includes existing UI integration tests above and real gallery, Profile, preferences, stream, format and generated-admin proofs. Retain root and mounted routes, encoded query bytes, hostile seed/title/replay values, exact HTML-limit boundaries, saved/unset theme and strict-CSP/no-JS behavior, delayed hydration/early drafts, keyboard focus and overlays, responsive/zoom/overflow/accessibility and disposal checks. No plugin registration, authorization or route-uniqueness policy is added for #14.

Deadlock-class proofs run in supervised child process groups with finite deadlines, a required completion marker and kill/**reap** assertions; no zero-test filtered success. Include never-polled/drop-before-precommit, drop while capacity-blocked, pending/failed/panicking Resources, consumer disconnect, repeated polling after completion/failure, upstream effect-panic deadline and caller Owner restoration. Verify no live worker/runtime/resource queue survives. Preserve the existing limitation that non-yielding trusted work/blocking Drop cannot be forcibly preempted by Rust cancellation.

Per released milestone use one task-owned target root, e.g. `target/issue17`, aggregate **8 GiB** across native/WASM/consumer/LLVM subdirectories, incremental/debug info off and bounded jobs. Before its first build recorder-wrap Cargo clean and LLVM coverage clean for that root. Recorder-wrap every Cargo/proof/clean command with the persistent store `/Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks`, explicit milestone/mode/cache labels, finite watchdog and cap. Delete only that owned target at milestone end; retain records/review evidence and verify child cleanup. M0 creates no target and requires no cleanup/build.

Applicable final commands include locked selected/workspace tests and doctests, `cargo fmt --all -- --check`, warning-denied Clippy/Rustdoc, separate SSR/hydrate/CSR and stream/rich/format selections and facade/independent compile-fail consumers. Avoid invalid all-feature unification; enumerate actual scopes. Development requires >=70% executable-line coverage **per changed production crate** (including any changed production scope), preserving valuable existing coverage and documenting exclusions/critical branches; publication >=90% is a separate unrequested gate. Never use empty-scope or native coverage as browser/mobile coverage.

## Risks and owner decisions

| Risk | Prevention and proving test |
| --- | --- |
| Hidden protocol coupling or cycle | Audit signatures/errors/template strings and all selected normal graphs; no view/server edge anywhere in L. Default V negative graph and independent fixture must fail if an integration edge leaks. |
| Lifetime/backpressure/error drift at a cross-crate seam | Move existing Owner/worker supervision intact; preserve staging order and generic error passthrough; watchdog success/cancel/panic/slow-consumer/teardown and exact page-output/error parity. |
| Safety or hydration/first-paint regression | Library-local safety plus a shared differential table, differential old/new valid/invalid tables including percent/dot/Unicode/query limits, raster bounds and script terminators; real delayed hydration, CSP, replay and G39 proofs. |
| Incomplete migration or dependency drift | Generator-before-output migration, second clean generation, active retired-path audit, explicit feature matrix and exact external lock tuple comparison; no generate-lockfile or installs. |

G41 resolves the two owner decisions. No theme, plugin, renderer-default or platform behavior choice is reopened. Coordinator can resolve routine implementation signatures against the accepted graph; any required public-behavior deviation returns for review.

## M0 implementation and verification notes

Only this issue-17 design is a worktree change; no production code/manifests/lock/history/status/context changes, Cargo build/tree/metadata, target, listener, install, commit or GitHub mutation. GitHub issue bodies and comments were read with `gh issue view`; source reads establish the inventory, not executable correctness. Baseline `20261028T231342.700365Z-659521e9dff8` and document validation `20261028T232140.428382Z-51c579ee9e5a` both exit 1: the same **146 inherited errors / 25 inherited warnings**, plus one new advisory line-count warning for this coherent **265-line** checkpoint (146 errors / 26 warnings total; no new hard errors or removed diagnostics). Keep it intact for combined inventory/API/graph review; it is below the 500-line cap. Static audit `20261028T232154.273121Z-54c40d028080` passes: **all 60 source files classified**, exact one-file change scope, unchanged tracked production/lock/historical files, clean whitespace, balanced fences and diagnostic-set comparison. These three recorder attempts total **0.813464 s command time**, **zero compilation/check/documenting events and zero rebuilds**; Cargo-stage time is unavailable, not zero. A disposable read-only audit script lives only under `/private/tmp`; no target exists to delete.

Verification notes: coordinator review pending. Acceptance notes: no visual artifact or product acceptance is claimed at M0. M0 was accepted and M1 explicitly released; the M1 receipt will record implementation evidence.
