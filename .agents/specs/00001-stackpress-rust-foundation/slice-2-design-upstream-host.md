# Slice 2 design: upstream host inventory

Design checkpoint for [task 00005](tasks/00005-slice-2-server-and-cli.md). Load to audit Ingest hosting, routing, loading, exported types, and lifecycle breadth before reviewing the [API](slice-2-design-api.md). These are proposals pending coordinator review and owner decisions, not implemented Rust contracts.

## Authority and observation

Read on 2026-10-24 from this branch and the sibling source files linked below. Rust HEAD was `9118707db87cd41dca8b79c122eb351c44a4456c`; Ingest HEAD `477ec47f8b5dafa5713b13f9c964d06c61c22938`; lib HEAD `e2cc6b8cad77b5ad18be8a94e0d40e72e2f6183a`; Stackpress HEAD `a71d683051ba8350fdd12d6b5a33f268fdcc285f`. HEAD identifies a checkout, not a guarantee that its working files are clean. Inspection used current working files; no upstream tests were run. No other trial branch or its output was inspected.

Accepted [context](../../context/index.md) and [Slice 1 decisions](decisions.md) govern the target. Historical references [00019](../../references/00019-upstream-runtime-api-contracts.md) (runtime API), [00020](../../references/00020-upstream-server-and-transport-contracts.md) (transports), and [00024](../../references/00024-upstream-cli-and-plugin-contracts.md) (CLI/plugin conventions) were checked against source. Load them for source-language contract detail, not Rust implementation evidence.

Disposition legend: **S2** implement capability in this slice; **S3** Idea/generation/build tooling; **S5** UI/Tauri; **N** intentionally do not port the mechanism. Streaming/network serving has a named **S2-T transport follow-up**, outside task 00005, requiring its own task and owner authorization. Deferral to S2-T is a proposal for scope approval, not a completed feature or scheduled commitment.

## Ingest runtime exports and methods

Source owners: [index](../../../../ingest/ingest/src/index.ts), [Server](../../../../ingest/ingest/src/Server.ts), [Router](../../../../ingest/ingest/src/Router.ts), [Route](../../../../ingest/ingest/src/Route.ts), [Loader](../../../../ingest/ingest/src/Loader.ts), and [types](../../../../ingest/ingest/src/types.ts).

| Export or surface | Disposition and reason |
| --- | --- |
| `Server`, `server`, constructor/options | S2 `ServerBuilder` → immutable `Server`; host config, services, selected plugins and lifecycle. An empty host is valid; construction never listens. |
| `Server.config`, `.plugins`, `.plugin`, `.register` | S2 typed immutable config/service registries and plugin descriptors. Explicit `Arc<T>` sharing; duplicate registration errors instead of implicit map replacement. |
| `.bootstrap` | S2 consuming async build, one attempt, registration then config/listen/route. Upstream bootstrap itself only loads plugins; Stackpress Terminal supplies the three later phases. |
| `.props`, request/response/context and req/res/ctx aliases | S2 same three-argument handler plus `Invocation` host access. N duplicate alias fields and server subclass generic narrowing; no extra handler shape. |
| `.handle`, `.handler`, default `handler` | S2 normalized top-level handling plus narrow adapters. N an untyped identity native handler as the baseline useful implementation. |
| `.create`, `.gateway`, default `gateway` | S2-T real networking/listener ownership. N Node `createServer` in the default host. The buffered S2 HTTP adapter has no gateway. |
| `Router`, `router`, `.action`, `.emit`, `.resolve`, `.request`, `.response` | S2 host delegates to existing lib values/dispatcher, adds explicit event and method/path entry methods. Raw dispatch stays distinct from handling. No duplicate Rust Request/Response subclasses. |
| `.on`, `.route`; `all/connect/delete/get/head/options/patch/post/put/trace` | S2 named handlers and method helpers over one all-matching table. Any is a matcher; extension verbs remain supported. Priority/ties/handles retain Slice 1 rules. |
| `.listeners`, `.expressions`, `.routes`, `.entries`, `.imports`, `.views` | S2 read-only listener and route descriptors; N mutable public maps. Entry/import/view-specific maps follow dispositions below. |
| `.use` | S2 explicit function composition and handle-preserving builder contribution composition. Existing lib `extend` keeps its hook/order semantics; no cloning runtime registries. |
| `action` identity helper | S2 named `async fn` first; existing lib `handler` for borrowing closures. N a second native callback convention. |
| `Request`, `Response`, `Status`, `Exception`, `ReadSession`, `WriteSession`, `session`, `cookie` | Already lib-owned; reuse checked values, `Error`, session/cookie functions. N empty subclasses or a second exception hierarchy. S2 adapters own native resources. |
| `isObject`, `objectFromQuery`, `objectFromFormData`, `objectFromJson`, `withUnknownHost`, `formDataToObject` | Already lib-owned typed values and parsers. S2 adapter selects parser deliberately; unsupported MIME does not silently become empty parsed data. |

### Handler facets and decorators

Sources: [ActionRouter](../../../../ingest/ingest/src/plugin/ActionRouter.ts), [EntryRouter](../../../../ingest/ingest/src/plugin/EntryRouter.ts), [ImportRouter](../../../../ingest/ingest/src/plugin/ImportRouter.ts), [ViewRouter](../../../../ingest/ingest/src/plugin/ViewRouter.ts), [decorators](../../../../ingest/ingest/src/decorators.ts).

| Surface / behavior | Disposition and reason |
| --- | --- |
| `ActionRouter.context`, `.createProps`, `.eventName`, `.emit`, `.route`, method helpers, `.use`, `_task` | S2 host context and route conversion over lib dispatch; preserve all matches, priority, named captures, nested mutation, before/action/after aborts. No emitter-global current event. |
| `_task` capture injection | S2 only explicit host route registrations inject named captures, before the route action. [Adapter contract](slice-2-design-adapters.md) defines literal-key precedence; raw lib emit is unchanged. Positional captures stay in Invocation. |
| `EntryRouter.action/on/route/use`, all method helpers, `.entries` | N executable string imports/default-export lookup; S2 use compiled function paths. S3 generated compiled registries can retain source-entry provenance when needed. |
| `ImportRouter.action/on/route/use`, all method helpers, `.imports` | N callback-arity/name inference and lazy JS import. Explicit compiled handlers give deterministic registration. Rust callbacks may themselves load domain resources. |
| `ViewRouter.action/on/route/use`, all method helpers, `.views`, `.engine`, `.render` | S5 page/view/render integration, because selected renderer is Leptos. N string inference that chooses view behavior in ordinary `.on`. |
| `Controller`, `All`, `Connect`, `Delete`, `Get`, `Head`, `On`, `Options`, `Patch`, `Post`, `Put`, `Trace`, `mount` | N reflection/decorator/class discovery; S2 ordinary registration functions cover grouped routes/events without macros or trait implementations. |
| `metadataOf`, `hasRoute`, `hasEvent`, `addRoute`, `addEvent`, `normalizePath`, `routeDecorator`, `assertHandler`, `controllerOf`, `registerRoute`, `registerEvent` | N decorator machinery, inheritance metadata and static-member rules. Checked route patterns and compiled Rust function signatures supply the relevant validation. |

### Loader behavior

| Source surface / behavior | Disposition and reason |
| --- | --- |
| `ConfigLoader(cwd/fs/key/extnames)`, `.load`, `.resolveFile` | S2 explicitly supplied JSON text/value and typed sections. S2 CLI explicitly named JSON file; no directory/module extension search. S3 owns general filesystem/path utilities. |
| Ordered `/plugins.js`, `/plugins.json`, `/package.json`, `/plugins.ts`, `.js/.json/.ts` search; `default`/key unwrapping | N executable module resolution or package metadata discovery at runtime. Compile-time plugin selection is already settled. |
| `PluginLoader(modules/plugins)`, `.plugins`, `.bootstrap` | S2 explicit plugin identity/function/dependencies, deterministic ordered registration, failed-build discard. N package path normalization, mutable cached string plugin list or dynamic code loading. |
| String→list, invalid→empty, defensive copy, nested array child loaders, cwd-relative/module/absolute names | N JS coercion/discovery. S2 explicit nested composition functions with validated unique IDs; malformed plugin input is a startup error. |
| Bootstrap once; retry after partial failure | S2 one consumed builder, no replay. Upstream sets `_bootstrapped` only after success and can replay preceding side effects after failure; the Rust host does not promise rollback of external effects. |

## Precise upstream lifecycle findings

These are source observations from `Route.ts`, checked against [Route tests](../../../../ingest/ingest/tests/Route.test.ts); they correct two tempting but inaccurate parity assumptions.

1. `emit()` calls `prepare`, `process`, `shutdown` in order and returns false at the first false result. It does not dispatch native output.
2. `prepare()` emits `request`, catches a thrown error through `_catch`, and tests the returned status for ABORT. Missing request listeners do not themselves stop it.
3. `process()` awaits the selected string event **without assigning its returned status**. It also ignores the direct callback's returned false. Thus normal selected-operation abort is lost here; only the error path or later empty-result error event can set the local ABORT status. The tests cover abort after an error, not ordinary selected-event abort propagation.
4. The empty-result check is JavaScript truthiness: `!body && !code`, not an Option/empty-body check. It creates 404 exception state and directly emits `error` outside the operation try/catch. A throw there escapes.
5. If code or status text is missing, processing sets the response status to OK; this can overwrite partially supplied status. Rust already represents code/text as a single Status.
6. `shutdown()` emits `response`, catches through `_catch`, and recognizes ABORT. It is not a finally hook.
7. `_catch()` upgrades the original error, writes response exception state, then emits `error`. A throw from `error` escapes; there is no dual-failure terminal fallback. Errors handled during request preparation can permit the operation to run afterward.
8. HTTP/WHATWG `plug()` loads the request before `Route.emit`; load failures bypass that route error pipeline. The adapter ignores the boolean returned by `Route.emit`, then dispatches whenever response is not sent. Abort does **not** inherently mean suppress output.
9. Upstream Stackpress Terminal emits raw command events; the `emit` script does another raw resolve. It does not share Route's whole lifecycle today. S2 proposes one deliberate top-level boundary for CLI and HTTP.

The [lifecycle design](slice-2-design-lifecycle.md) makes each divergence explicit for Q05 approval: reliable operation abort, failed-request operation skip, bounded nonrecursive error fallback, typed emptiness, and common CLI/HTTP handling.

## Exported type inventory

All Ingest-owned types in `types.ts` and its index are covered below. Inherited exports are grouped by their already established semantic owner, not presumed new work.

| Exported type names | Disposition / replacement |
| --- | --- |
| `ConfigLoaderOptions`, `PluginLoaderOptions` | S2 `ConfigLayers`, `PluginSpec`, `ServerBuilder`; N JS filesystem/modules/extension options. |
| `ActionRouteProps`, `ActionProps`, `AnyActionProps`, `ServerProps` | S2 borrowed Request, Response, Invocation plus typed `ServerInvocationExt`. Note `ActionRouteProps`/`AnyActionProps` exist in types even where the index omits them. |
| `ActionRouterArgs`, `ActionRouterMap`, `ActionRouterAction`, `ActionRouterListener`, `ServerAction`, `AnyRouterAction` | S2 existing `Handler`/`Result<Flow>` plus typed builder methods; N tuple-map plumbing and string/function unions. |
| `EntryRouterTaskItem`, `ImportRouterAction`, `ImportRouterTaskItem` | N dynamic import metadata; S3 static generated registration/provenance if required. |
| `ViewRouterTaskItem`, `ViewRouterEngine`, `ViewRouterRender` | S5 view/render contracts. |
| `ConfigMap`, `PluginMap`, `Infer`, `KnownPlugin` | S2 typed config/services/descriptors; N TS conditional/inference markers or unchecked cast-based lookup. |
| `ServerHandler`, `ServerGateway`, `ServerOptions` | S2 normalized handle and buffered adapter options; S2-T native listener/runtime ownership. |
| `NodeServer`, `NodeServerOptions`, `NodeRequest`, `NodeResponse`, `NodeOptResponse`, `IM`, `SR` | N Node/global-Web object types; S2 typed HTTP envelopes, S2-T streaming/native integration. |
| `HttpRequest`, `HttpResponse`, `HttpRouter`, `HttpServer`, `HttpServerOptions`, `HttpAction` | S2 optional HTTP Request/Response<Vec<u8>>, adapter options, same host/Handler. No duplicate native-generic foundation. |
| `WhatwgRequest`, `WhatwgResponse`, `WhatwgRouter`, `WhatwgServer`, `WhatwgServerOptions`, `WhatwgAction` | N JS aliases; S2 buffered HTTP equivalence, S5 browser/Tauri adapter, S2-T network/serverless follow-up separately. |
| `ControllerProperty`, `ControllerHandler`, `ControllerRouteDefinition`, `ControllerEventDefinition`, `ControllerMetadata`, `ControllerInstance`, `ControllerClass`, `ControllerMountable` | N decorator/class reflection types; S2 explicit registration functions/descriptors. |
| `RouteMap`, `RouterContext`, `RouterArgs`, `RouterMap`, `RouterAction` | Slice 1 deferrals resolved in S2 by adapter envelopes, typed host context and the existing Handler; N compatibility tuples/subclass-dependent aliases. |
| `TypeOf`, `Key`, `NestedObject`, `UnknownNest`, `Scalar`, `Hash`, `ScalarInput`, `FileMeta`, `CallableSet`, `CallableMap`, `CallableNest` | Already inventoried in [Slice 1](design-upstream-inventory.md); typed Value/Nest/collections replace JS syntax. |
| `ResponseStatus`, `Trace`/`StackTrace`, `ErrorResponse`, `SuccessResponse`, `StatusResponse` | Existing Status/Error/InvocationFrame/ResponseEnvelope. S2 adds host diagnostic report, not another wire schema. |
| `Item`, `TaskResult`, `TaskAction`, `TaskItem`, `EventMap`, `EventName`, `EventData`, `EventMatch`, `Event`, `EventHook`, `EventExpression` | Existing queue/event contracts. S2 composes them; no new dynamic variadic event typing. |
| `Body`, `ResponseDispatcher`, `ResponseOptions`, `Headers`, `Data`, `Query`, `Post`/`PostData`, `Session`, `LoaderResults`, `RequestLoader`, `CallableSession`, `RequestOptions`, `Revision`, `CookieOptions`, `Method`, `Route`, `RouteAction` | Existing lib buffered values, attempts, parsers, sessions and matchers; S2 native mapping; streaming S2-T. |
| `FileRecursiveOption`, `FileStat`, `FileStream`, `FileSystem`, `CallSite` | S3 filesystem, S2-T response streams; N V8 call-site reflection. No unused compatibility aliases. |

## Tested host behavior coverage

This maps assertions/scenarios, not an assertion that the upstream suite passed in this session.

| Test file | Behaviors and disposition |
| --- | --- |
| `Server.test.ts` | S2 route/event invocation, plugin registration/retrieval, function/async/object plugin bootstrap analogues, config access, all method helpers, request/response initializer values, canonical context. Custom gateway/handler construction/setters and default native factories → S2-T; JS generic narrowing/object-as-plugin → N, explicit service registration instead. |
| `Route.test.ts` | S2 request abort; selected string/function calls; empty→404→error; selected failure→error→response; shutdown abort; full-flow abort after operation error; static helper equivalence; request/response failures offered to error. Callback props aliases → N. Add ordinary-operation abort and error-listener failure tests absent upstream. |
| `Router.test.ts` | S2 direct/action method helpers, Any, params, wildcards, priority, composition; direct typed handlers. Lazy import and mixed string-view/anonymous-import dispatch → N; view behavior → S5. |
| `RouterExtensions.test.ts` | S2 method helper consistency, direct handler priority/composition metadata; entry/import execution/maps → N; configured/missing view engines and render getter → S5; empty subclass wrappers → N. |
| `Loader.test.ts` | S2 equivalent deterministic composition, missing/failed selected plugin and once-only successful startup; default/custom constructor options, ESM/default wrapping, invalid/empty/string/list/package config, recursive arrays and relative/module/absolute resolution → N runtime discovery; explicit JSON data loading replaces config input, general paths → S3. |
| `Decorator.test.ts` | N class/instance/multiple-controller mounting, method decorators, helper metadata, normalization, standard initializer path, invalid member/static target checks. Rust signatures and explicit grouped route functions are the replacement. |

Transport and terminal exports/tests, including every Slice 1 deferral, continue in the [adapter inventory](slice-2-design-upstream-adapters.md); load it before approving scope breadth.
