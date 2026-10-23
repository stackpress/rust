# Foundation source and behavior map

Owner: [source map](../context/source-map.md). Load for detailed ownership and compatibility boundaries. This documents existing source behavior, not working Rust APIs.

## Lib

[Exports](../../../lib/src/index.ts) define the public primitive families. [Tests](../../../lib/tests) are the contract starting point alongside source, especially EventEmitter, ExpressEmitter, Router, Request/Response, TaskQueue, Nest, and FileLoader tests.

- `Nest`, callable nest/map/set controllers and readonly views provide shared nested state vocabulary. Preserve the purpose while using Rust methods/borrowing rather than callable-object syntax.
- `ArgString`, `QueryString`, `FormData`, `FileData`, and `PathString` processors convert boundary representations into nested state. Preserve precedence and shape intentionally, including missing versus null values.
- `ItemQueue` sorts priorities; `TaskQueue` awaits sequentially and returns status. Before/action/after false aborts the queue. No listener returns not-found; complete iteration returns OK. Do not infer complete equal-priority stability from stated FIFO intent alone.
- `EventEmitter` owns named listeners, matches/tasks, before/after hooks, composition, removal, and dispatch metadata. `ExpressEmitter` adds `:params`, `*`, `**`, and regex expressions. `RouteEmitter` adds method/path events. Keep match collection distinct from conventional single-winner routing.
- `Request` owns normalized data/query/post/headers/cookies/session, URL/method/MIME/body and optional native resource, with load-once behavior.
- `Response` owns body/status/errors/results/rows/totals/data/headers/session revisions, redirects, stop/sent state, and native dispatcher. Mutable framework state exists before output; native send constraints remain adapter-specific.
- `Session` records cookie changes/deletions for later dispatch; it is distinct from the higher-level signed identity Session.
- `NodeFS`/`FileLoader` abstract filesystem/module resolution; JS executable import behavior needs a Rust registration/build adaptation.
- `Terminal`, control, and input own CLI parsing/prompt/output behavior. Terminal events can use manually constructed framework requests/responses.
- `Exception` carries status/type/errors/source spans and parses V8 stack strings; preserve diagnostic purpose with Rust Error/source/report/tracing rather than copy stack-string assumptions.
- `Reflection` is V8-specific. Small `Template` helpers are build-oriented; method-expression JS semantics are not automatically portable.

Use [runtime API detail](00019-upstream-runtime-api-contracts.md) for exact methods/data shapes and [diagnostic source assessment](00008-discussion-diagnostics-existing-parser.md) for Rust error-report strategy.

## Idea

[Parser source](../../../idea/packages/idea-parser/src) owns lexer definitions, tokens/trees, compiler, and type/schema config. [Transformer](../../../idea/packages/idea-transformer/src/Transformer.ts) owns loading/composition and ordered plugins. [Parser tests](../../../idea/packages/idea-parser/tests) and [transformer tests](../../../idea/packages/idea-transformer/tests) provide current fixtures. [Editor code](../../../idea/language/server/src) is a separate surface.

- Declarations include enums, types, models, props, plugin configuration and `use` composition; modifiers and open attributes are language contracts.
- JSON and environment-derived values are supported in existing workflows; build inputs and host environment need deliberate Rust treatment.
- Transformer composition preserves parent/child precedence and mutable type/model rules. Do not replace these with generic object merging without proving behavior.
- Enabled transform plugins run sequentially with schema/config/cwd/extra generation context. Plugins may cooperate on shared output, so order is observable.
- Stackpress schema adds the meaning of attributes. Parser openness is not permission for every unknown attribute to have runtime effects.
- Editor completion, diagnostics, navigation, and formatting can remain independently implemented while parser/runtime internals change.
- Existing Rust parser branch is a reuse candidate; no completeness claim or Node binding requirement follows from its existence.

Load [the full language catalog](00022-upstream-idea-language-catalog.md) for syntax, declarations, merge semantics, and examples; [schema contracts](00021-upstream-schema-api-contracts.md) for normalized semantic lenses; [component catalog](00031-upstream-idea-component-catalog.md) for role/type-specific UI generation.

## Ingest

[Server/Router/Route](../../../ingest/ingest/src) and [tests](../../../ingest/ingest/tests) define the host. Handler facets live in `plugin/ActionRouter`, `EntryRouter`, `ImportRouter`, and `ViewRouter`. HTTP/WHATWG adapters have their own resource/stream contracts.

- Server extends routing with config, plugin state/loader, configurable gateway/handler, native-resource adaptation, and one-time bootstrap.
- Bootstrap discovery alone does not resolve Stackpress's config/listen/route phases; the application/framework coordinates those.
- Canonical current handler props expose request/response/context and req/res/ctx aliases. Rust signature ergonomics remain open.
- Actions can be direct, lazy module imports, entries, or named views. Rust should use explicit compiled function registration, not inference from function shape/argument count.
- General handled-invocation stages are request, selected processing, response. Exceptions are converted to response state and sent to error. Early abort may skip later stages.
- Direct emit/resolve is a lower-level capability protocol, not automatic top-level lifecycle or caller authorization.
- Native Node HTTP and WHATWG request/response types remain escape hatches. Existing gateways still create Node servers; WHATWG does not itself promise an edge deployment.
- Body buffering/parsing, limits, cookies, streams, envelope serialization, forwarded URL behavior, and dispatch differ by adapter. Reprove them on the selected Rust hosts.

Load [server and transport detail](00020-upstream-server-and-transport-contracts.md) and [CLI and plugin detail](00024-upstream-cli-and-plugin-contracts.md) when implementing host edges and lifecycle composition.

## Inquire

[Engine](../../../inquire/packages/inquire/src/Engine.ts), [builders](../../../inquire/packages/inquire/src/builder), [dialects](../../../inquire/packages/inquire/src/dialect), adapter packages, and their tests are authority. [Examples](../../../inquire/examples) demonstrate deployment combinations.

- Build visible parameterized SQL; retain values separate from raw clauses.
- Query builders cover CRUD, joins, predicates, JSON paths/containment, order/pagination/returning. Schema builders cover create/alter/drop/rename/truncate, fields/keys/relations.
- Builders expose structured state and dialect rendering; JS thenability can become explicit Rust execution.
- Engine hooks can mutate/inspect or bypass execution; connection hooks observe the final formatted request. Preserve their distinct responsibilities.
- Schema diff compares create definitions and produces add/remove/change operations. Rename intent requires deliberate reconciliation; names alone do not prove a semantic rename.
- Transactions require connection affinity. Native result/error/lastId behavior differs among MySQL, pg, PGlite, and SQLite.
- SQLite package naming is historical; current native resource is better-sqlite3. PGlite uses PostgreSQL SQL but is embedded/WASM and has separate hosting/persistence/concurrency limits.
- Raw connection/results remain explicit nonportable escape hatches.

Load [SQL contracts](00025-upstream-sql-api-contracts.md) and [all database adapters](00026-upstream-database-adapter-contracts.md) before choosing a Rust backend or promising dialect parity.

## Reactus

[Source](../../../reactus/reactus/src), [tests](../../../reactus/reactus/tests), and framework integration examples define existing rendering/build behavior.

- Host routing chooses page entries; documents compose Head/body/props/browser entry/styles.
- React server renderToString produces initial markup; the browser entry hydrates matching serialized props under React StrictMode.
- Document/loader/builder/render classes separate entry resolution, markup, assets, and emitted source.
- Manifest provides stable entry identifiers and saved output metadata; VirtualServer holds in-memory generated modules.
- Vite resource/plugins manage dev transforms, HMR, CSS/package resolution, virtual files, and client/page/asset builds.
- Production serving loads built page modules, with aligned entry/asset routes and templates.
- These are behavior goals to reestablish with Leptos; no React code/component or Vite runtime compatibility is implied.

Load [view contracts](00027-upstream-view-api-contracts.md) for exact server snapshot, provider/hook/layout, generated component, and build behavior.

## Frui and r22n

- Frui supplies granular React UI behaviors rather than application theme/layout/brand policy. Generated form/filter/list/span/view wrappers depend on its input/value/formatting contracts. Its notification/flash behavior is also used by shared layouts.
- r22n uses source phrases as translation keys and `%s` positional interpolation; React-node interpolation preserves embedded nodes. Wording changes can change translation identity. Locale persistence is a Stackpress concern rather than implicit provider behavior.
- Reimplement intended contracts in Leptos where selected. Do not describe existing React components as native/shared Rust UI or assume a string-only interpolator preserves node interpolation.

Load [session and language detail](00028-upstream-session-language-contracts.md), [component catalog](00031-upstream-idea-component-catalog.md), and [view contracts](00027-upstream-view-api-contracts.md) when rebuilding localization and generated UI.
