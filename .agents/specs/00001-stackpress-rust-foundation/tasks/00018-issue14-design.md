# Issue 14: function-based framework entry points — M0 design

2026-10-29, Asia/Manila. Branch `trial/3-issue14-plugin-entry-points`; inspected HEAD `047c8a0ee22d472d8638d132d77284fcc7ac9c89`. Status: started; design drafted for coordinator review. **M0 here means this design checkpoint only. Stop before implementation or disposable compilation proofs.** Acceptance criteria: none for this document; later UI parity evidence does not replace owner acceptance.

Authority: [issue 14 body and comments](https://github.com/stackpress/rust/issues/14), coordinator brief, current source, [G39–G41](../decisions.md), issue-15 [design](00016-issue15-design.md), [host milestone](00016-issue15-m2.md), [accepted closeout](00016-issue15-m3.md), and issue-17 [design](00017-issue17-design.md), [extraction](00017-issue17-m1.md), [independent consumer](00017-issue17-m2.md), [review closeout](00017-issue17-m3.md). Issue examples predating #15–#17 are historical proposals. The preferences/UI packages are removed; server paths are `crates/server/{server,http}`. No historical record or context file is this lane's write scope.

## 1. Capability-to-crate/plugin map

“Native” below means the execution host, including a deliberately selected Tauri Rust host; browser contracts and Tauri frontend assets use separate graphs. Installed desktop/mobile operation remains a separate proof. Every registration is explicit compiled code; features/configuration alone select no callback.

| Current capability/crate | Classification and useful registration | Required configuration/services and order | Targets and deliberate exclusions; current source evidence |
| --- | --- | --- | --- |
| `framework-view`: page IDs, snapshots, assets, catalogs, renderer contracts, access | Support contracts plus existing direct `ServerBuilderViewExt::page`; contributes pages to an application composite. No compulsory base-page plugin. | App supplies checked `PageSpec`, loader, renderer/contract, build/revision, public projection, mount and policies. Config claims page IDs; handler remains ordinary GET. Renderer executor initialization is explicit. | Contracts/browser routing work without native host features; page registration is native `server`. SSR/hydrate/CSR remain separately selected optional `leptos*` adapters. No automatic page discovery, renderer choice, protected-action grant or generic HTTP/IPC event bridge. [Page registration](../../../../crates/framework/framework-view/src/server.rs), [contracts/features](../../../../crates/framework/framework-view/Cargo.toml), [adapters](../../../../crates/framework/framework-view/src/leptos/mod.rs). |
| `framework-view`: preferences, locale routing, theme asset | Owns the existing runtime `preferences::register`; real contribution to the proposed view composite. | Required `view_preferences: preferences::Config`; Config must find explicit `preferences::Admission` supplied earlier. Validates locale registry, canonical origin, checked base and distinct literal settings/confirmation POST paths; optional immutable theme GET. Creates `preferences::Service`. | Native `web-preferences`; pure `preferences`/`cookies` remain browser-safe. Tauri local settings use explicit commands/native adapter instead of implicitly installing these web routes. No file, database, listener or implicit session policy. G39/G40 remain binding. [Host implementation](../../../../crates/framework/framework-view/src/preferences/host.rs), [migration](../../../../crates/framework/framework-view/docs/migration-issue15.md). |
| `framework-view`: flash | Owns existing runtime `flash::register`; independent contribution to the view composite. | Required `view_flash: flash::Config`; validated issuing origin/base and 1–128 unique literal page paths. Config constructs `flash::Service`; Listen installs one scoped `response` hook. No preference dependency. | Native `web-flash`; pure flash/cookie values usable in browser. Projects/clears only configured successful HTML GETs; app explicitly issues notices after success. This response hook shares a page intentionally and owns no route. [Flash host](../../../../crates/framework/framework-view/src/flash/host.rs). |
| `framework-view`: whole presentation integration | **Owns proposed runtime composite `plugin::register`**, composing the two real contributions above. New explicit `presentation` feature forwards `web-preferences,web-flash`; defaults unchanged. | Requires both existing config sections and Admission, with the same validators/order. No new umbrella config or invented prerequisite plugin ID. Calls preferences then flash once; each retains its own services and callbacks. | Native host only; no Leptos requirement. Smaller preference-only/flash-only integrations remain independently selectable. Never conditionally register according to whichever Cargo features happened to unify. [Current host composition contract](../../../../crates/framework/framework-view/docs/presentation-host.md) reserves this function for #14. |
| `framework-language` | **Support library only**, consumed by view preferences; no runtime or generation plugin. | Caller constructs checked `LocaleRegistry` from locales/default/aliases. View owns HTTP routes, cookie/action policy and persistence integration. | Dependency-free pure library, usable native/browser/Tauri frontend. No view/server dependency or new registry service merely for naming. [Registry](../../../../crates/framework/framework-language/src/lib.rs), [manifest](../../../../crates/framework/framework-language/Cargo.toml). |
| `framework-schema` | Runtime **support library**, plus separate build-time normalization/contracts transform participation. No ServerBuilder plugin. | Runtime callers explicitly use owned records, validators, public projections, clocks/providers. Build tool supplies finalized Idea config, frozen app `DefinitionRegistry`, `DiagnosticMap`, normalization limits and emitter callback. Schema transform precedes SQL/admin transforms. | Runtime contracts support native/browser/Tauri. Only `codegen` acquires parser/transformer; no runtime schema discovery or inferred access policy. [Manifest](../../../../crates/framework/framework-schema/Cargo.toml), [normalization](../../../../crates/framework/framework-schema/src/codegen/mod.rs), [contract emitter](../../../../crates/framework/framework-schema/src/codegen/emit.rs). |
| `framework-sql` | Direct stores/DataSession remain **support library**; proposed optional `plugin::register` owns useful runtime integration for one explicit generated-operation selection. Separate codegen transform. | App supplies existing `Arc<stackpress_sql::Engine>`, `operations::Policy`, and new compiled `plugin::Contributions` service. Engine/Policy exist before plugin Config validation; Contributions exists before Listen. Plugin publishes an opaque typed `plugin::Service` readiness claim; one retained local `operations::Registration` spans all selected callbacks. No owned JSON config section or default backend. | Runtime plugin native `server`; frontend never selects it. Direct SQL library/session use stays independent. Backend acquisition/config, transaction completion, migrations/install/seed and network hosting remain application/operator work. [Operations/policy/claims](../../../../crates/framework/framework-sql/src/operations.rs), [generated handler registration](../../../../crates/framework/framework-sql/src/codegen/operations.rs), [feature boundaries](../../../../crates/framework/framework-sql/Cargo.toml). |
| `framework-admin` | Public pages/roles/drafts/transfer remain **support library**; codegen contributes real native application plugins, rather than adding a server dependency to the shared admin crate. | Build tool chooses components/replacements, app field policies and explicit page models. Generated native registration consumes selected SQL integration, Engine/Policy, checked generated Mount service and existing app host policy/services. No runtime string selects an uncompiled model. | Shared admin UI supports separate SSR/hydrate/CSR. Generated native modules belong only in host graphs; generic components/roles need no host. No universal `framework_admin::plugin::register` while handlers/renderers/CSRF/origin remain app-owned. [Manifest](../../../../crates/framework/framework-admin/Cargo.toml), [page contracts](../../../../crates/framework/framework-admin/src/pages.rs), [native generator](../../../../crates/framework/framework-admin/src/codegen/pages.rs), [current app assembly](../../../../examples/generated-admin/web/src/host.rs). |
| `stackpress-leptos` (`crates/leptos`) | **Independent support library; stays plugin-free.** Components, generic rendering, state, language, async/replay/format/rich helpers retain direct use. | Explicit values, factories, callbacks, limits, language dictionaries and render mode. No Stackpress services/configuration/admission. Generic SSR initialization remains explicit at the host boundary. | Independent native SSR/stream and browser hydrate/CSR; reusable Tauri frontend. No Stackpress normal dependencies, page protocol, saved-preference acquisition or generated operation authority. No plugin, including an empty callback. [Manifest](../../../../crates/leptos/Cargo.toml), [independent consumer](../../../../examples/leptos-independent/README.md). |

Current Server evidence: [builder](../../../../crates/server/server/src/builder.rs), [PluginSpec preflight](../../../../crates/server/server/src/plugin.rs), [startup phases](../../../../crates/server/server/src/startup.rs), [typed configuration](../../../../crates/server/server/src/config.rs), [typed services](../../../../crates/server/server/src/services.rs), [route registration](../../../../crates/server/server/src/registration.rs). Selection is sealed before callbacks; all sections decode/validate before Config; Config → Listen → Route finish before immutable Server publication. Duplicate typed/named services and config claims already fail. These rules do not make registration transactional or undo external work already performed by arbitrary application callbacks.

## 2. Entry points

All **proposed** symbols below are API designs, not compiled evidence. Existing functions are marked. Paths use the real crate names; facade aliases remain optional conveniences. No plugin trait, dynamic loader, second dispatcher, filesystem convention or inferred plugin ID is introduced.

### Runtime functions and responsibilities

| Exact path and signature | Responsibilities and explicit application inputs |
| --- | --- |
| Existing `stackpress_framework_view::preferences::register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>` | Claims `view_preferences` with no fallback host config; queues Config service construction and Route registration of settings POST, confirmation POST and optional theme GET. Admission is required even for a sessionless app. No operations or build hooks. Preserve existing error/status/cookie contracts. With approved route option B, only these literal endpoint registrations gain exclusive ownership. |
| Existing `stackpress_framework_view::flash::register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>` | Claims `view_flash`; queues Config and Listen callbacks for the service and scoped response hook. No route, operation, database, file or generation hook. Requires no preferences plugin. App chooses pages and issuance. |
| Proposed `stackpress_framework_view::plugin::register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>` | Under explicit native `presentation`, calls the two functions above in order. This is substantive composition: two validated sections, two services, settings/confirmation/optional asset routes, one response hook. Adds no own service or general lifecycle wrapper. Requires both configs and existing explicit Admission; app still supplies loaders/renderers/pages, origin/CSRF/session facts and protected-operation policy. Native files/Tauri commands and SSR initialization stay explicit. |
| Proposed `stackpress_framework_sql::plugin::register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>` | Native `server` only. Config verifies Engine/Policy and publishes the opaque typed `plugin::Service` singleton; Listen validates the Contributions service and invokes its callbacks using **one** Registration marker. No routes or config section. Does not create/open an Engine or authorize any operation itself. Reuses actual generated handlers, their completion guard, policy-before-decode-before-database order and sanitized failure mapping. |
| Proposed generated `crate::generated::pages_host::register(app: &mut stackpress_server::ServerBuilder) -> stackpress_lib::Result<()>` | Emitted only for an explicit nonempty page-model selection into the existing host-only output. Config claims its singleton, requires generated Mount **and SQL plugin Service**, and publishes the generated SQL Contributions; Route installs only the existing selected page/API/export/import mappings through `crate::host::{page,api,transfer::*}`. SQL integration owns Listen registration; generated admin does not register operations twice. No new general lifecycle. No JSON section: mount and security config remain app-owned. |

The SQL contribution service is data containing compiled functions, not a new plugin interface:

```rust
// Proposed stackpress_framework_sql::plugin types; native server feature only.
pub type RegisterOperations = fn(
    &mut stackpress_server::StartupContext,
    &mut stackpress_framework_sql::operations::Registration,
) -> stackpress_lib::Result<()>;

pub struct Contributions {
    pub operations: &'static [RegisterOperations],
}
```

Listen first checks a nonempty maximum of 128 callbacks, then passes one marker through them. Existing generated `register(start, marker, prefix, operations) -> Result<Vec<ListenerId>>` functions keep their API; named application/generated wrappers choose model/prefix/operations and discard or retain handles deliberately. Invalid/duplicate operation selections fail through that marker; an earlier callback's queued listeners never become a usable partial host after a later failure. Callbacks register listeners only, with no database work. Missing Contributions fails in Listen, intentionally after every Config provider had its chance to run. Engine/Policy absence or wrong Config order fails earlier.

Generated admin additionally emits `crate::generated::pages_host::Mount::new(value: &str) -> stackpress_lib::Result<Self>`, delegating validation to the existing `framework_admin::pages::mount`; it owns checked mount text, not authorization. Application configure code derives Mount and its current host Config from the **same** checked input and registers both before generated callbacks; disagreement is a startup error. Existing origin/CSRF/transfer validation and error-document listeners stay application-owned and explicit. The generator already references these host handler paths; #14 does not move those handlers into the UI library.

Select SQL before generated admin, for example `PluginSpec::new("admin").requires(&["sql"])`. Its Config contribution supplies Contributions after SQL's Config service checks and before SQL's Listen callback. `plugin::Service` has private construction and records registrar readiness, not operation authorization; its checked lookup also rejects an omitted/late SQL integration when an app forgets the PluginSpec prerequisite. IDs are chosen by the application, not hard-coded library dependencies. The simple generated admin adapter owns one Contributions service; apps combining admin with other operation families instead provide one aggregate service and use the existing direct generated `operations`/`routes` helpers, avoiding two providers or two markers. Do not silently replace/merge an existing provider. Empty model selection emits no new register function and the application omits that plugin.

Registration helpers inside a composite inherit the composite's provenance. Independently selected features use `PluginSpec` only for actual dependencies, preserving supplied order; a helper never calls `app.plugin(...)` after selection is sealed. Service validation remains necessary even when a selected dependency ID exists. Direct contribution errors require discarding the builder; no rollback/retry promise is added.

### Separate build-tool entry points

Proposed ordinary functions, all gated by each crate's existing `codegen` feature:

```rust
// Each path has this exact signature; these functions register transforms, not hosts.
stackpress_framework_schema::codegen::register(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()>;

stackpress_framework_sql::codegen::register(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()>;

stackpress_framework_admin::codegen::register(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()>;

stackpress_framework_admin::codegen::register_pages(
    registry: &mut stackpress_idea_transformer::Registry,
    transform: stackpress_idea_transformer::ContextTransform,
) -> stackpress_idea_transformer::Result<()>;
```

These declarations are signature notation, not runnable Rust statements. `ContextTransform` is the existing `fn(&TransformContext<'_>, &mut OutputPlan) -> transformer::Result<()>`. Each helper calls the existing Registry with a real ownership/dependency declaration and the supplied compiled callback. Callbacks remain necessary because application normalization, provider/field policies, diagnostics and model selection cannot be inferred safely.

| Helper | Registered transform contract | Callback work, kept explicit |
| --- | --- | --- |
| Schema `register` | ID `framework-schema`; owns `contracts,metadata`; no prerequisite. | Normalize finalized input with the app's frozen definitions/diagnostics; call existing `emit_contracts`. |
| SQL `register` | ID `framework-sql`; owns `stores`; requires `framework-schema`. | Same normalization/diagnostics; call `emit_stores_with_diagnostics`, explicitly selecting whether to emit operations. No execution/migration. |
| Admin `register` | ID `framework-admin`; owns `ui`; requires `framework-schema`. | Role-only `emit_roles`; no invented SQL requirement for reusable roles. Existing `register_components(DefinitionRegistry, Vec<Component>) -> SchemaResult<DefinitionRegistry>` remains the separate catalog-selection API. |
| Admin `register_pages` | Same ID/partition as admin register, **alternative**, never both; requires `framework-schema,framework-sql`. | Emit roles and selected page/native registration through `emit_pages`. Tool explicitly enables SQL operation output. The stronger SQL dependency applies to this native page adapter, not the pure UI library. |

Current [generator Registry](../../../../crates/idea/idea-transformer/src/registry.rs), [workspace tool](../../../../examples/generated-workspace/tool/src/lib.rs) and [admin tool](../../../../examples/generated-admin/tool/src/lib.rs) establish these callbacks and partitions. Today the admin tool's transform declares only schema; the page-specific helper makes its actual generated-store dependency explicit. The application tool checks the completed plan's existing `files()` inventory for required `stores/operations.rs` before OutputWorkspace check/install when native pages are selected; stores-only output is a negative fixture. The helpers cannot infer the contents of an arbitrary supplied transform callback; generated-consumer compilation proves the actual handler references as a separate gate.

Application tooling still explicitly matches finalized root plugin blocks to compiled registration functions, rejects unknown selections, freezes one definitions function for all callbacks, supplies diagnostics/declared inputs, and owns aggregate/OutputWorkspace check/install. Helpers do not parse files, install outputs or run callbacks at registration. Current emitters/direct Registry APIs remain available. Final app normal dependencies omit parser/transformer/codegen; browser modules include only public contracts/UI. No generator uses the generated app as its own dependency. No `cargo generate-lockfile`, dependency upgrades or new external packages are proposed.

## 3. Composition and pilot parity

Direct framework selection and application customization use the same existing host API. These are **proposed integration sketches**, with application setup named explicitly, not compile-verified examples:

```rust
use stackpress_lib::Result;
use stackpress_server::ServerBuilder;

// Application helper: add actual trusted view_preferences/view_flash layers
// and queue the application's Admission provider before the plugin Config tasks.
fn direct(app: &mut ServerBuilder) -> Result<()> {
    application_settings_and_admission(app)?;
    app.plugin("presentation", stackpress_framework_view::plugin::register)?;
    Ok(())
}

// Selected as one application plugin; helpers add contributions under this ID.
fn customized_presentation(app: &mut ServerBuilder) -> Result<()> {
    stackpress_framework_view::plugin::register(app)?;
    app.on_listen(application_policy_listeners)?;
    app.on_route(application_pages)?;
    Ok(())
}

fn customized(app: &mut ServerBuilder) -> Result<()> {
    application_settings_and_admission(app)?;
    app.plugin("application-presentation", customized_presentation)?;
    Ok(())
}
```

`application_policy_listeners` and `application_pages` have the existing startup signature `async fn(&mut StartupContext) -> stackpress_lib::Result<stackpress_lib::Flow>`; they contribute ordinary listeners/routes. A custom app chooses locale/aliases, mount/origin, theme-asset ownership and flash page scope in its config, and supplies fresh protected-action policy at the operation boundary. A preference-only app selects the existing preference function directly. Neither example selects the framework composite and its constituent functions together. Rendering, page policy, database execution and existing core handlers remain untouched; nested work uses the current Invocation delegation and does not restart the general lifecycle.

**Recommended pilot: `examples/ui-preferences/web`, with its shared/client and composed review-host regression paths.** Its [current register function](../../../../examples/ui-preferences/web/src/lib.rs) already calls preferences plus flash consecutively, supplies explicit Admission and trusted sections, renders locale/theme/flash, and proves denied protected actions. Replace those two contribution calls with one selected framework function, retaining application-supplied setup/pages/security. This exercises the real composite with no added public capability.

Profile UI is the issue's cited assembly case, but [current web code](../../../../examples/profile-ui/web/src/lib.rs) deliberately seeds English/Light and has no web preference/flash endpoints; its [gateway](../../../../examples/profile-ui/gateway/src/lib.rs) assembles checked assets, while native uses separate fixed preference commands. Forcing the full presentation composite there would add routes and config requirements. Keep its direct page API available and run its regression parity; migrating its page assembly to an application plugin can follow only if the owner wants that additional example. No artificial base-view plugin is needed merely to register its two pages.

| Parity boundary | Required M1 evidence against current source/output pins |
| --- | --- |
| Public responses and failures | Exact successful/error JSON bodies, status codes, cookie names/scope/flags/lifetimes, read-only confirmation, HTML/seed/build/revision/assets and cache/CSP headers. Include missing/malformed/oversized input, invalid Origin, denied writes and flash projection/clear behavior. Compare public results; provenance IDs may intentionally change. |
| Locale and theme | Prefix > canonical cookie > default, aliases and exact query/return bytes; Light/Dark only, unset/null preserved, legacy saved `system` reads unset without writes. OS sampled once only when unset; no-JS/blocked bootstrap is readable Light, explicit saved Dark stays Dark. No initialization cookie/file writes, live OS tracking or early draft/focus/node loss. |
| Mounts and assets | Root and nonroot mount; every canonical/alias preferences page; unchanged `/ui-preferences` and `/ui-preferences/confirm`; optional content-addressed theme GET. Standalone host owns the asset, composed review host retains its checked bundle and selects `serve_theme_bootstrap=false`. No duplicate ownership or added path. |
| Lifecycle and authority | One request/selected-operation/response lifecycle per handled exchange; ordinary error/Abort/finalized-response behavior. Scoped flash hook runs once; unrelated/non-HTML/failed responses keep the notice. Missing/forged display hints never authorize protected work; denial has zero protected calls and no staged preference cookie. |
| Other consumers | Profile memory and SQLite public search/detail/API/mounted output, SSR/hydrate/CSR, French/Dark/unset render/seed fixtures and native preference bridge retain their current behavior. Missing/uninstalled SQLite is still a failure with the explicit setup remedy; startup never installs/seeds. Existing G41 differential safety table and independent Leptos consumer remain valid. |

## 4. Duplicate-route decision

Current [RouteSpec](../../../../crates/server/server/src/router.rs) explicitly permits independent duplicate matchers; Lib runs all matching prioritized listeners. `page` rejects duplicate **page IDs** via Config named services, not unrelated listeners at the same path. Preferences rejects its two equal POST paths and flash rejects duplicate declared pages, but neither detects a foreign host route. A flash response hook is deliberately not a duplicate route.

| Option | Consequences |
| --- | --- |
| A. Explicitly accept current route semantics | No Server changes; framework/app authors own complete inventories. Different IDs/functions at the same settings endpoint remain legal and may both run. Existing contract must be documented and tested as intentional; cannot claim cross-plugin conflict prevention. |
| **B. Recommend opt-in exclusive literal endpoints in Server** | Preserve ordinary ordered all-matching routes, with an additive ownership rule for endpoints deliberately marked exclusive. Framework settings/confirmation/theme and generated admin literal terminal routes opt in. Server can check foreign direct/plugin/startup registrations in both orders; a view-local named claim cannot. Requires owner approval of the API/behavior and coordinator release of the Server write set. |
| C. Reject duplicate/overlapping routes globally | Broad behavior change affecting middleware, aliases, replacement and pattern overlap. Reject for #14: it would change the core routing model and require a separate overlap-language design. |

Proposed option-B API: `stackpress_server::RouteSpec::exclusive(self) -> Self`, consumed by existing `ServerBuilder::route_with` / `StartupContext::route_with`. No new dispatcher/claim service. Server retains checked route metadata keyed by ListenerId: literal source path, method, exclusivity and contributor site. An exclusive route must be literal with no captures/wildcards/query. A conflicting host route with the **same literal path** and intersecting methods (equal checked method or Any), where either route is exclusive, fails through existing `stackpress_lib::Error` / startup `BuildError` before the rejected listener is inserted. Nonexclusive duplicates keep priority/FIFO behavior.

Check both insertion orders, direct contributions before plugin callbacks, Config/Listen/Route phase rules, builder `extend` preflight, and replacement/removal. `replace` preserves ownership metadata; successful `remove` releases only that handle's claim; a failed mutation preserves all existing registrations. `extend` must reject conflicts before moving either builder's contributions. Startup failures return no Server; discard a builder after registration errors, as today. Do not advertise external-resource rollback.

This is **literal endpoint ownership only**, not a proof that Express patterns cannot overlap. Capture routes, equivalent spellings, broad middleware and raw Lib listener registrations keep current matching semantics; app policy must remain enforced immediately at protected handlers. Page ID/catalog checks still cover their own inventories. No automatic HEAD/GET alias is invented. If the owner chooses A instead, M1 omits all Server edits and its cross-plugin literal-conflict guarantee; tests explicitly pin the accepted duplicate behavior. The decision must be recorded before M1, not silently resolved during implementation.

## 5. Negative-behavior and graph plan

All proposed runtime helpers only queue registrations or construct bounded in-memory services. Startup never installs, migrates or seeds a database, opens a listener, launches a background host or accesses native preference files. App-supplied Engine acquisition remains explicit and separate; arbitrary trusted application callbacks are not sandboxed by a plugin API.

| Risk/input | Failure contract, observation and prevention test |
| --- | --- |
| Missing/wrong-order/cyclic selected dependencies | Existing PluginSpec preflight returns Registration BuildError naming the plugin before **any** selected callback. Use callback counters for missing, present-optional wrong order, cycle and valid supplied order; no implicit sorting/loading. |
| Missing/wrong-type/wrong-order services | Config fails if Admission, Engine or Policy is absent/late; Listen fails for missing Contributions; generated admin Config fails for missing Mount or SQL plugin Service. Service names/types use existing checked lookup. Valid dependency IDs without the services still fail, and admin without a selected SQL registrar fails even if Engine/Policy exist. Count zero handler/database/persistence calls and assert no returned Server. Direct Config providers precede selected contributions at ordinary priorities; priority overrides are explicit application responsibilities. |
| Duplicate registrations and ownership | Duplicate selected ID rejects selection; different IDs selecting the same view composite/constituent reject duplicate typed sections. Duplicate services/named singleton and duplicate generated event selections reject build. Test two operation callbacks with one marker, duplicate prefix/operation, duplicate page ID/catalog route, and empty/excessive contribution lists. For option B add foreign direct/selected/startup literal conflicts in both orders and Any-method cases; for accepted overlapping listeners assert existing order/Abort instead of falsely promising rejection. |
| Invalid or inconsistent config | Unknown/missing view fields, invalid origin/base/locales/alias, equal settings/confirmation, nonliteral paths, excessive/duplicate flash pages reject before service allocation and routes. Validate admin Mount, host security config and mount agreement before generated routes. ConfigLayers cannot select compiled code; rejected sections never yield a partial host. |
| Protected action denial/forgery | Preferences requires fresh Admission and exact Origin on save and confirmation. Domain actions retain fail-closed HostAccess or OperationPolicy before input/DB effects. Test forged AllowedActions/query/body, no policy, changing session facts, wrong Origin/CSRF, invalid DTO plus denial, direct named invocation and HTTP/IPC fixed mappings. Assert sanitized responses, zero protected/database calls, no cookie/file writes and no private diagnostics in public output. |
| Generated model/operation absence/conflict | Nonempty explicit selection only; no inferred models or routes. Missing Engine/Policy, unsupported operation, duplicate claims, wrong transform order/partition, admin page output without SQL operations, unavailable component/replacement and invalid normalized metadata fail before host/output publication. Disabled tool/plugin/model fixtures emit/select no new host function, service, operation or route. |
| Feature enabled, plugin omitted | Configured library presence creates no services/hooks/routes and performs no acquisition/write. Test each capability alone, both libraries loaded with no selection, composite absent, optional theme route false, and supported direct-library consumers. Unknown config roots remain existing ConfigLayers errors, not implicit selection or silent activation. |
| Default graphs | Compare normal-edge graphs before/after: view defaults remain Leptos/server-free; language has no dependencies; schema has no parser/transformer without codegen; default SQL acquires no backend/server/codegen; default admin acquires no host/SQL driver/generator. New view `presentation` forwards only existing web features. External lock tuples stay identical to the approved trial/3 base. |
| Browser/Tauri frontend graphs | Separate actual hydrate and CSR builds and target-selected `normal,no-proc-macro` trees exclude server/native-file/libc/database backends/Tauri host/generation. Generic Leptos has no other Stackpress normal dependency. Host-feature misuse on WASM fails clearly; enabling unrelated facade Leptos never selects view. Preserve schema/admin shared consumers without host integration. |
| Native/generation graphs | Explicit runtime selections exclude parser/transformer/codegen and optional network/TLS/CLI hosts unless the application separately selected them. SQL plugin does not select a backend. Separate generator consumer has no generated-app dependency; missing generated output can be regenerated. SSR, hydrate and CSR are separate, not `--all-features`. Installed Tauri desktop/mobile acceptance is reported separately from native compilation/bridge fixtures. |

Use existing in-process requests, buffered HttpAdapter, duplex IO and finite browser interception. Never run listener-opening tests under this brief, including the existing TCP-listener watchdog; name any skipped target explicitly. Potential deadlock/cancellation/drop tests run in supervised child groups with finite deadlines, a required completion marker and kill/reap checks; no timing-only assertion or zero-test success. Retain renderer cleanup/backpressure and G41 shared safety tables rather than redesigning rendering.

## 6. Milestones and proof gates

| Gate | Deliverable and what it proves; stop condition |
| --- | --- |
| **M0 — this checkpoint** | Current-source map, exact proposed functions, explicit exclusions, route recommendation, pilot/parity and negative/graph plan. No code/probe/build. Coordinator reviews; owner resolves route behavior and pilot selection before implementation release. |
| **M1 — presentation pilot** | Add explicit view presentation feature/composite; preserve separate preference/flash/direct-page APIs; migrate ui-preferences selection with parity and custom-composite fixture. If B is approved, implement the narrowly released Server route ownership first, then apply it to literal preference endpoints; prove `extend`/remove/replace and ordinary overlap preservation. Run selected native suites, public baseline/parity, admission/lifecycle/disabled cases, separate SSR/hydrate/CSR and mount/first-paint/draft/browser proofs; Profile regression. Proves ordinary plugin DX without public presentation drift. Stop for coordinator independent verification before M2. |
| **M2 — useful SQL/admin and build-tool integration** | Implement native SQL selection service/registrar, generated admin host function/Mount and the separate transform registration helpers. Migrate the generated-admin host/tool and workspace generator only where those helpers replace real wiring. Keep generic admin shared and Leptos plugin-free. Regenerate via official tools, second clean generation/--check, selected/disabled/model-removal fixtures and unchanged outputs except intentional generated registration code. Prove actual generated CRUD/policy/transfer behavior and no implicit database setup, runtime/generator graph separation, explicit transform dependencies and direct-library alternatives. Stop before review rounds for coordinator verification. |
| **M3 — review and documentation** | Distinct test-verified rounds for complexity/error/effect order; responsibility/ownership/abstraction/graphs; consistency/Rustdoc/comments; safety/admission/lifetime/cancellation. Record findings, scoped fixes or justified no-change outcomes; repeat only material findings. Docs cover migration, direct use, features, typed setup/admission, composition/provenance, literal-route rule/limits, and runtime vs generation. Compile real final Markdown examples in disposable consumers. Final checks/coverage and coordinator re-verification close implementation; owner decisions/platform acceptance remain separate. |

After release, use one task-owned target root `target/issue14`, aggregate cap **8 GiB** across native/WASM/consumer/LLVM subdirectories, incremental/debug info off and bounded jobs. Before the first build recorder-wrap Cargo clean and LLVM coverage clean for that exact target; monitor cap/child deadlines, then delete the owned target at each milestone end and verify child cleanup. Never clean unrelated targets or the benchmark store. M0 creates no target, so no Cargo/coverage clean is appropriate now.

Record **every Cargo/proof/clean command**, including failed attempts, through `python3 .agents/scripts/build-benchmark.py --store /Users/cblanquera/server/projects/stackpress/rust/.agents/resources/build-benchmarks run --label ... --kind ... --scope ... --cache ... -- <command>`. Wrap an outer proof once, not nested Cargo again. Receipts include record IDs, scope/features, attempt/compilation/full-rebuild deltas and separate command/Cargo-stage times. No install, commit, push, issue write, lock generation or sibling idea modification/build; frui remains read-only. Shared root/Server files require explicit coordinator write ownership before M1/M2.

Verification after implementation: locked affected tests/doctests, formatting, warning-denied Clippy/Rustdoc, separate compatible target/feature consumers, generation checks, actual browser parity and normal graphs. Broaden to workspace only for affected contracts and with listener tests excluded under this brief. Development coverage is >=70% executable lines **per changed measurable crate**, including shipped generated code with declared accounting; preserve valuable existing coverage, disclose exclusions/uncovered critical branches and lack of numeric WASM coverage. Publication's >=90% gate is separate and unrequested.

## 7. Risks and open owner questions

| Decision/risk | Recommendation |
| --- | --- |
| Foreign route listeners can run at an owned settings endpoint | Approve **B**, narrowly for literal host endpoints; retain ordinary overlap semantics and enforce security inside handlers. If A is preferred, explicitly accept its weaker conflict guarantees. No implementation until decided. |
| Pilot could add unintended public behavior to Profile UI | Approve **ui-preferences** as the real presentation-composite pilot; keep Profile as regression/direct-page evidence. If owner specifically wants Profile migrated, use an application page composite and freeze its checked mount/module inputs first; do not add settings/flash routes to it implicitly. |
| SQL/admin convenience might become a generic host framework | Keep one explicit SQL operation selection and generated app-owned admin adapter. Do not add a universal admin crate plugin, service locator, host-policy defaults, per-request model lookup or move application handlers. Review the M2 signatures after M1 proof if source exposes a real missing capability. |
| Accidental feature/graph broadening or stale generated code | Explicit presentation feature, native-only runtime modules, unchanged defaults, generator-only functions, isolated consumer graphs/builds, official regeneration and exact external lock audit. Preserve G39/G40 and the G41 independent-library/safety boundary. |

### Implementation notes and M0 receipt

Read issue #14 with `gh issue view 14 --repo stackpress/rust --comments` and separately fetched its body; no GitHub write. Reviewed current manifests/implementations, relevant #15/#17 records and host/migration docs. Source inspection establishes this design, not executable API/parity evidence. No Cargo metadata/tree/build/proof requiring compilation was needed, no target was created and there is no task build output to delete.

Baseline KB validation: record `20261029T054040.215285Z-b4e4a3e54566`, exit 1, **146 inherited errors / 26 warnings**, 0.322175 s command time. Draft validation `20261029T054817.611048Z-a2dcabb74ad5`, exit 1, 0.292662 s: identical inherited diagnostics plus one advisory for this coherent >200-line design, below the 500-line cap. No added hard error. Keep this checkpoint together for capability/API/decision review; no historical/context record changed to suppress diagnostics.

Static audit `20261029T054856.100663Z-5d339d265c90`, exit 0, 0.126758 s: all **42 local links** resolve, all seven requested sections/fences/whitespace pass, exact inherited error/warning sets compare as above, only this design is untracked, no tracked source/manifests/lock/history changed and no task target exists. These three attempts total 0.741595 s, with zero compilation/check/documenting activity or full rebuilds; Cargo-stage time is unavailable, not zero. A final recorded audit repeats validation on the completed receipt. The audit driver lives only at `/private/tmp/issue14-m0-audit.py`.

Final audit `20261029T055124.936464Z-9eb8a0e3eb7e`, exit 0, 0.440667 s, repeats the scope/link checks and fresh raw validator comparison after the SQL readiness/output-plan clarifications: **146 inherited errors / 26 inherited warnings + one own line-count advisory**, no added hard error. Task records: four attempts (two audits passed, two raw KB validations retain inherited failures), zero compilation/full rebuilds, **1.182262 s accumulated command time**, no observed Cargo stages. This receipt annotation adds no source link or contract change.

Verification notes: coordinator design review pending. Owner route/pilot decisions are pending; this document recommends choices without treating them as accepted. **Stop at M0.**
