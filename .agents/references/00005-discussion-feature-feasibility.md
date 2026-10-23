<!-- kb-import:start -->
> Scope: preserved discussion history. Confirmed current Rust decisions live in [project direction](../context/project-direction.md). Proposed APIs, old paths, and historical status in this source are not implementation instructions.
> Source: `/Users/cblanquera/server/projects/stackpress/rust-notes/stackpress-rust-feasibility/feature-inventory-and-feasibility.md`; SHA-256 `bd4f10728ae86d410f6c17d9addda754e0868c753b73e86a970b1b849c7db863`; captured 2026-10-23. [Exact original](../resources/conversation/2026-10-23-stackpress-rust/feature-inventory-and-feasibility.md). Prose links are rebased; source content otherwise retained.
<!-- kb-import:end -->

# Feature inventory and Rust feasibility

Research snapshot: 2026-10-23. Source-reviewed assessment; no Rust parity implementation or target tests have been performed.

## Required behavior

| Requirement | Assessment | Contract to retain |
| --- | --- | --- |
| Prioritized emitter; false aborts | Implementable directly | Await listeners sequentially; higher priority first; stop remaining listeners on false; define tie ordering |
| Express-style event routes | Implementable directly | Method/path matching, named parameters, wildcard matches, multiple matching listeners, common priority queue |
| Manually constructed Request/Response | Implementable directly | No socket needed; optional host resources; mutable data, headers, session, result/error state before dispatch |
| CLI events | Implementable directly | Parse CLI input into the same request model, invoke the same capability, format response for terminal |
| Plugins | Implementable through compilation selection | Disabled code need not be compiled/registered; enabled plugins contribute events/services/routes/transforms through a simple registration function |
| Error events | Implementable for application failures | Convert handler Result errors into response state, emit error, preserve handled/unhandled outcome |
| Idea and plugin code generation | Strong fit | Compose declarations and metadata, execute ordered transforms, generate before final app compilation |
| Server/serverless | Implementable per adapter | Share host/capabilities; explicitly adapt each runtime and prove each target |
| Reactive server-first views | Implementable with a chosen renderer | Website SSR plus hydration, selected page, serialized props; installed apps have a different rendering host |
| Event/config/plugin framework internals | Implementable directly | Framework packages obey the same extension contracts as application plugins |
| Configured auth/API/access | Implementable | Configuration selects policy; implementation and generated capabilities provide mechanisms |

Latest author decisions: Leptos + Tauri is selected, shared web/mobile/desktop source is a requirement, the foundation name is `stackpress-lib`, and an idiomatic Rust reorganization is preferred over a 1:1 conversion. The author reconfirmed rebuild/redeploy for plugin disablement. General request/response/error lifecycle events are existing behavior to carry forward. See [architecture](00006-discussion-crate-plugin-proposals.md) and [diagnostics/parser](00008-discussion-diagnostics-existing-parser.md).

Current Ingest actions receive a props object with `request/response/context` and `req/res/ctx` aliases. A strictly two-argument Rust Request/Response handler API was discussed as a possible design, not a requirement already implemented in current source.

## 1. lib

Source anchors: [exports](../../../lib/src/index.ts), [EventEmitter](../../../lib/src/emitter/EventEmitter.ts), [ExpressEmitter](../../../lib/src/emitter/ExpressEmitter.ts), [RouteEmitter](../../../lib/src/emitter/RouteEmitter.ts), [TaskQueue](../../../lib/src/queue/TaskQueue.ts).

Carry forward:

- Nested data lookup/mutation/merging and path addressing.
- Maps, sets, readonly views, and shared collection conventions where useful.
- CLI-argument, query-string, form-data, file-data, and path processors.
- Priority queues, sequential task queues, insertion helpers, before/after hooks.
- Event matching, expression matching, route matching, composition, listener removal.
- Load-once request bodies and normalized request fields.
- Response status, errors, results/rows, totals, redirects, MIME types, body forms, sent/dispatch state.
- Cookie/session revision tracking for output adapters.
- Filesystem abstraction and deterministic file resolution.
- CLI parsing, terminal output, and prompts.
- Structured exceptions, diagnostics, and status responses.
- Generator template helpers if their syntax remains useful.

Redesign boundaries:

- Callable object syntax becomes ordinary Rust methods or a limited explicit convenience layer.
- Preserve distinctions among missing values, null, scalar values, dates, bytes, and typed native values. JSON alone does not preserve arbitrary JS values.
- V8 reflection of live functions, `this`, and JS call sites has no direct native equivalent. Use Rust diagnostic/backtrace facilities for the needed purpose.
- JS module import/resolution semantics become compiled registration or explicit external tooling.
- Small-template JS method expressions need a defined replacement rather than accidental partial support.
- Equal-priority behavior needs an explicit accepted contract. Do not infer complete FIFO guarantees just from a comment or an incomplete comparator/test set.

## 2. idea

Existing Rust implementation: `feature/rust-parser-2` at `6c289d1f3fba17c758baafb2d4dcefd87fd448ea` contains `idea_parser_core` and `idea_parser_node`. The former owns tokenizer/AST/parser/finalization and uses Serde; the latter is a thin N-API wrapper. Per-platform npm packages belong to the Node distribution design, not a requirement for using the parser in Rust. Assess this core before writing a new parser. No compile/parity tests were run. [Branch](https://github.com/stackpress/idea/tree/feature/rust-parser-2).

Source anchors: [parser](../../../idea/packages/idea-parser/src/index.ts), [Transformer](../../../idea/packages/idea-transformer/src/Transformer.ts), [language tooling](../../../idea/language/server/src/server.ts).

Carry forward:

- Lexer/parser, AST, compiled schema, source locations, and diagnostics.
- Models, reusable types, enums, props, optional/list modifiers.
- Open attributes and distributed semantic ownership.
- `use` composition, precedence, and mutable extension behavior.
- Environment-derived values with explicit generation inputs.
- Ordered plugins, plugin-specific configuration, shared schema context.
- JSON input where useful alongside Idea syntax.
- Generator CLI and inspectable output.
- Editor diagnostics, completion, navigation, and formatting.

The editor extension does not have to become Rust when the generator does. Adapt its resolution rules only where necessary.

Preserve the boundary between the Idea language and Stackpress interpretations. The parser should not absorb all SQL, auth, view, admin, or MCP semantics.

Generation may emit Rust, SQL, TS/TSX, metadata, or documentation. Leptos would change UI output to Rust components; it does not require changing Idea syntax.

## 3. ingest

Source anchors: [Server](../../../ingest/ingest/src/Server.ts), [Router](../../../ingest/ingest/src/Router.ts), [Route lifecycle](../../../ingest/ingest/src/Route.ts), [ActionRouter](../../../ingest/ingest/src/plugin/ActionRouter.ts).

Carry forward:

- Transport-neutral capability host and request/response handling.
- Named events and method/path routes, all matching listeners and their priorities.
- Named parameters, wildcard behavior, expression captures, route composition.
- Plugin/service registration, configuration, deliberate bootstrap order.
- One-time bootstrap behavior.
- Direct actions, explicit named handler entries, and named view entries.
- Lazy initialization when it has a useful Rust equivalent.
- General `request` before the selected operation, `response` afterward, and `error` on application failure, as implemented by Route. Define direct/nested event invocation separately from this top-level lifecycle.
- Native-resource escape hatches with adapter-specific typing.
- Configurable gateways and handlers.
- Controller registration conveniences where readable.

Use explicit compiled registrations instead of JS import callbacks, decorators, or inference based on function shape/name/argument count.

Do not hold a registry lock across asynchronous handler execution. Keep match metadata per invocation. Design nested dispatch and mutable borrows deliberately.

Serverless support is adapter-specific. Existing JS examples are not proof that the Rust framework works on those platforms. AWS's documented Rust HTTP integration is a candidate first adapter.

## 4. inquire

Source anchors: [Engine](../../../inquire/packages/inquire/src/Engine.ts), [builders](../../../inquire/packages/inquire/src/builder), [dialects](../../../inquire/packages/inquire/src/dialect).

Carry forward:

- Select/insert/update/delete builders.
- Create/alter/drop/rename/truncate operations.
- Inspectable builder state and generated SQL.
- Bound values separate from SQL strings.
- Joins, predicates, ordering, pagination, returning clauses.
- JSON selectors and containment operations.
- PostgreSQL/MySQL/SQLite dialect behavior.
- Schema differences: fields, primary keys, unique keys, indexes, foreign keys.
- Explicit field-rename reconciliation.
- Transactions, native connections, query hooks that can inspect/mutate/bypass execution.

Rust execution would likely use explicit `.execute().await` rather than JS thenable builders.

SQLx is a candidate execution backend. SeaQuery is a candidate builder backend. Neither was proven to replace the full Inquire contract. Keep generated SQL, JSON paths, hooks, diff behavior, and database-specific semantics visible.

PGlite remains a separate compatibility requirement: PostgreSQL protocol/dialect support is not embedded PGlite hosting. A JS host bridge or PGlite socket process could preserve a boundary, but SQLite changes semantics and is not an equivalent substitute.

## 5. reactus

Source anchors: [rendering](../../../reactus/reactus/src/DocumentRender.ts), [React helper](../../../reactus/reactus/src/helpers.ts), [build](../../../reactus/reactus/src/Builder.ts), [plugins](../../../reactus/reactus/src/plugins.ts).

Preserve the purpose of:

- Host-selected page entries.
- Page/head/document composition.
- Server props and browser bootstrap/hydration data.
- Manifests and stable page identifiers.
- Server-page, browser, and asset build separation.
- CSS/asset discovery and insertion.
- Development rendering and refresh tooling.
- Virtual/generated entry modules.
- Package-aware page resolution.
- Production rendering from built artifacts.

Literal React component compatibility requires retaining a JS React renderer. The author has selected Leptos + Tauri, so the new renderer should preserve behavioral intent while replacing React/Frui/r22n component implementations where necessary.

Do not promise that Leptos is a drop-in Reactus replacement. Establish page selection, Head behavior, props serialization, hydration, routing, and build contracts explicitly.

## 6. stackpress

Source/context anchors: [architecture](00012-upstream-architecture-and-composition.md), [modeling/generation](00013-upstream-modeling-and-generation.md), [operations](00014-upstream-runtime-and-operations.md), [interfaces](00015-upstream-interfaces-and-experience.md).

| Area | Features to preserve or explicitly assess |
| --- | --- |
| Server | Bootstrap, terminal events, emit/serve/develop, environment configuration |
| Schema | Models/columns/fieldsets, metadata dictionaries, assertions, defaults, serialization, names, relations, documentation |
| Generation | Executable model-specific output, registries, listeners, routes, components, tools, and declared exports |
| Revisions | Generated schema snapshots and change history |
| SQL | Stores/actions/events, CRUD/search/count/batch/upsert, soft removal/restoration, relational selectors |
| Database lifecycle | Install, upgrade, push, migration generation, population, purge, uninstall, rename/destructive-change handling |
| View | Server props, browser-facing wrappers, layouts, notifications, branding |
| UI generation | Separate field/filter/list/span/view roles and component dictionaries |
| Admin | Generated CRUD, copy, import/export, related-record workflows |
| Session/auth | Identity/profile records, signup/signin/signout, username/email/phone, link/OTP/2FA, passwords/account operations |
| Access | Roles, grants, event/route permissions, wildcards, configured guards |
| API | Public/app/session callers, configured event exposure, scopes, OAuth flows, CORS, webhooks |
| CSRF | Configuration-driven request protection |
| Language | Locale selection/persistence, locale-prefixed routes, phrase translation |
| Email | Send events and configured transports |
| AI | Generated/configured MCP tools, schemas, caller visibility, stdio/HTTP/SSE |
| Desktop | Optional host, menus, event invocation, build/package commands |
| Distribution | Aggregate facade, narrow imports, CLI/scaffold, templates, agent skills |

Current Studio content is planning material, not an implemented feature to count as porting parity. Reserved desktop updater events are not proof of a completed updater.

Preserve architectural distinctions:

- Internal events do not automatically authorize callers or expose public endpoints.
- Each external surface maps and validates its own identity and protocol.
- Revisions are generated history, not proof of the database's applied state.
- Population and other multi-event workflows do not automatically share a transaction.
- Disabling a plugin does not automatically undo migrations or remove existing data.
- Serialization to views is a browser disclosure boundary, not transmission of live server objects.

## Dependency candidates and exact-fit limits

| Candidate | Narrow responsibility | Stackpress still owns |
| --- | --- | --- |
| Tokio | Async runtime and I/O | Pipeline semantics, ordering, cancellation, lifecycle |
| Hyper/http | HTTP mechanics and types | Normalized requests/responses and routing events |
| Serde | Typed serialization/deserialization | Configuration composition, plugin policy, exposed fields |
| SQLx | Query execution/connections/transactions | Inquire semantics, generated stores/actions |
| SeaQuery | Potential SQL/schema builder backend | API parity, diff, hooks, selector behavior |
| Leptos | Selected reactive UI direction | Framework integration and generator contracts |
| Tauri | Selected installed-app host | Application capability mapping and platform policy |
| lambda_http | AWS HTTP event adaptation | Common host and other serverless targets |

Do not directly substitute Axum route selection for Stackpress's all-matching priority dispatcher. Axum's duplicate-route and specificity rules differ. It can host an adapter without owning the capability semantics.

Rust's common regex crate is not ECMAScript-regex compatible: it omits lookaround and backreferences. Exact JS regex compatibility needs a separately chosen strategy. Ordinary path/wildcard support is a narrower problem.

## Primary external references

- [Cargo build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html)
- [Axum Router](https://docs.rs/axum/latest/axum/struct.Router.html)
- [Rust regex](https://docs.rs/regex/latest/regex/)
- [Tokio](https://docs.rs/tokio/latest/tokio/)
- [Hyper](https://hyper.rs/)
- [HTTP types](https://docs.rs/http/latest/http/)
- [Serde](https://serde.rs/data-model.html)
- [SQLx](https://docs.rs/crate/sqlx/latest)
- [SeaQuery](https://docs.rs/sea-query/latest/sea_query/)
- [AWS Rust HTTP events](https://docs.aws.amazon.com/en_en/lambda/latest/dg/rust-http-events.html)
- [PGlite architecture](https://pglite.dev/docs/about)
- [PGlite Socket](https://pglite.dev/docs/pglite-socket)

Links describe upstream capabilities, not tested Stackpress Rust integration. Versioned behavior should be checked again when implementation begins.
