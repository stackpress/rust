# Slice 7 HTTP and operations inventory

Load with [decisions](slice-7-design-decisions.md) when reviewing completeness.
`Now` means proposed Slice 7 implementation, not already implemented. `Retain`
means existing Rust behavior must remain under regression tests. Later labels
name follow-up scope, not approved new tasks.

## Provenance

Read [runtime and operational contracts](../../references/00014-upstream-runtime-and-operations.md),
[server and transport contracts](../../references/00020-upstream-server-and-transport-contracts.md),
and [operational recipes](../../references/00030-upstream-operational-examples.md).
They describe TypeScript contracts, not Rust implementation. Source corroboration:
`ingest/ingest/src/{http,whatwg}/{index,Adapter,helpers}.ts` and
`stackpress/packages/stackpress-server/tests/serve.test.ts` in read-only sibling
checkouts. Earlier exhaustive [Slice 2 adapter inventory](slice-2-design-upstream-adapters.md)
owns shared exports and terminal functionality; this file closes its network
deferrals without reimplementing CLI or startup.

## Exports, types and transport behavior

| Exact symbol / behavior | Disposition and reason |
| --- | --- |
| HTTP and WHATWG `gateway(server)(NodeServerOptions)` | Now `HttpServer` over Hyper; application owns binding and executor entry. WHATWG's Node gateway is not a serverless guarantee. |
| `handler(context, request, response, action?)`, `Adapter`, static `plug`, `.plug/.request/.response` | Retain one common lifecycle, add network input/output authority. No external event override from client input. Adapter classes become values/functions, not required plugin classes. |
| `server`, `router`, `action` factories | Retain ServerBuilder/function registration, immutable server and named Send handlers. Now separate optional network host; no duplicate routing framework or identity macro. |
| `IM`, `SR`, `HttpServer`, `HttpAction`, `NodeRequest`, `NodeResponse`, `NodeOptResponse`, `WhatwgServer`, `WhatwgAction`, `NodeServerOptions` | Now typed `http` values, owned Hyper IO/body and first-party options. Do not port Node/Web aliases or give handlers raw sockets. |
| `ServerOptions`, `ConfigMap`, `PluginMap`, `ServerConfig.host/port/process`, gateway/handler customizations | Retain typed config/services and explicit compiled plugins. Now listener options and injected IO for tests. `process`/child supervision stays S7-Ops; no dormant fields. |
| `Request`, `Response`, `Status`, `Exception`, `ReadSession`, `WriteSession`, `session`, `cookie` reexports | Retain existing lib values, errors, cookie revisions and scoped removals. No parallel transport copy or blanket JS alias catalog. |
| `ConfigLoader`, `PluginLoader` | Retain explicit bounded typed JSON/config phases from G15. Intentionally no dynamic module discovery, lazy JS import or mutable plugin reload. |
| `ActionRouter`, `EntryRouter`, `ImportRouter`, `ViewRouter`, `Router` | Retain host route registration and invocation delegation. Filesystem discovery/import/view loading belongs to generator/view tasks. Transport does not choose one winning route. |
| `Controller`, `All`, `Connect`, `Delete`, `Get`, `Head`, `mount`, `On`, `Options`, `Patch`, `Post`, `Put`, `Trace` decorators | Retain ordinary Rust registration helpers. No decorator reflection. CONNECT tunnels/upgrades are explicitly unsupported in the initial network policy, even if a raw event can carry that method name. |
| `isObject`, `objectFromQuery`, `objectFromFormData`, `objectFromJson`, `withUnknownHost`, `formDataToObject` | Retain strict lib parsers and explicit trusted origin. No unknown-MIME-to-empty-object fallback, silent coercion or malformed-URL fallback. |
| `loader`, `Request.text`, Node chunk accumulation, one-time loading | Now finite incremental network read before one strict body parse inside Input. Retain no load retry. Streaming uploads/multipart-to-disk is S7-Uploads; bounded buffered input is explicit. |
| `dispatcher`: strings, buffers, typed arrays | Retain checked buffered conversion; now stream bytes under backpressure, framing owned by transport. |
| `dispatcher`: object/array/status/error envelope | Retain public envelope, strict rich-value rejection, zero/false/null, legal native status and cookie/header append semantics. Never serialize developer causes. |
| `dispatcher`: Node/Web stream bodies | Now explicit server-owned one-shot stream extension, no Node resource bridge. Demand-driven owned byte chunks; error/disconnect drops stream and closes native output. |
| WHATWG native `Response` passthrough | Intentionally not ported. Bypasses framing/cookies/output authority. Future typed native override requires a separate reviewed contract. |
| `imToURL`, `reqToURL`, default MIME/method/URL, slash rewriting | Retain checked method/MIME, exact pathname, repeated/trailing slashes, configured origin. Reject canonicalization-changing targets; do not reproduce silent rewriting. |
| `imQueryToObject`, `reqQueryToObject` | Retain bounded URL query parse; query never chooses operation or impersonates trusted adapter context. |
| encrypted socket / `x-forwarded-proto` precedence | Now explicit listener TLS metadata and opt-in trusted-proxy policy. Intentionally reject blanket trust, comma/array-first authority and unknownhost fallback. |
| `readableStreamToReadable`, `readableToReadableStream` | Now native Rust chunk stream. Byte/empty/multiple chunks preserved; no JS interoperability claim. |
| `.listen`, `.create`, ready/bind errors, default host | Now bind only after startup/options/TLS/assets succeed, emit ready only after successful bind. Default loopback; production address explicit. All developer runs through devmetrics. |
| Node native HMR socket/Vite hooks | S7-Ops / view build tooling; not a production HTTP adapter responsibility. |

## Every relevant upstream test behavior

Sources are `ingest/ingest/tests/whatwg.Adapter.test.ts` and `helpers.test.ts`,
plus Stackpress `serve.test.ts`. Families below enumerate individual cases;
existing parser tests stay in lib rather than being duplicated mechanically.

| Test identity / cases | Disposition and replacement evidence required |
| --- | --- |
| WHATWG plug through route lifecycle | Retain + network duplex test proving exactly one Input/request/operation/response/output sequence and raw nested dispatch. |
| Request normalized URL and parsed cookies | Retain exact path/cookie parsing; deliberate rejection of source slash rewrite. Add proxy-spoof regression. |
| Response with WHATWG dispatcher attached | Replace with private transport authority; handler replacement cannot obtain another native attempt. |
| Parse urlencoded body | Retain parser; test body arriving in split network frames and budget overrun. |
| Skip loader on cached request | Retain one-attempt loading; network reader cannot be retried. |
| Plain string with status/headers | Retain; stream and buffered representations share header legality/cookies. |
| Object body as JSON | Retain public envelope; network output equals existing buffered adapter bytes. |
| Typed arrays and Node streams | Retain exact bytes; now multi-chunk/empty stream, bounded backpressure and cancellation tests. |
| Existing native Web Response unchanged | Intentionally not ported; foreign/finalized output must be Unavailable/closed, never guessed or retried. |
| Error response and cookie revisions | Retain G12 fresh response, original error identity and multiple scoped Set-Cookie values. |
| Helpers JSON, urlencoded, multipart | Retain strict parse behavior and limits, exercised through network Input. |
| Helpers unknown MIME empty object | Intentionally preserve raw bytes per accepted Rust contract. |
| Web→Node stream ordinary, empty, binary | Now ordinary/empty/binary byte stream with demand and drop tests. |
| Node→Web stream ordinary, empty, multiple | Same Rust stream contract, each case tested; JS API bridging not ported. |
| Web request URL and URL without query | Retain trusted origin/exact URI; no normalization rewrite. |
| Web query-to-object and query absent | Retain strict repeated-field query semantics; operation still method + path. |
| Node default protocol | Now explicit listener scheme or configured public origin. |
| Node forwarded proto scalar, array, comma-separated | Replace all three with trusted-peer + single-authority validation; untrusted values ignored. |
| Node forwarded protocol versus socket | Trusted public-origin policy is explicit, never automatic header precedence. |
| Node invalid URL fallback | Intentionally reject before callbacks (G17). |
| Node query present and absent | Retain bounded parse and empty query behavior. |
| Serve default 127.0.0.1 and ready log | Now default loopback and ready-after-bind; listener behavior awaits separately authorized devmetrics smoke test. |
| Serve configured host | Now explicit address and bind failure handling; test injectable binder plus supervised smoke test. |

## Operational examples and scope ledger

| Reference 00014 / 00030 responsibility | Disposition |
| --- | --- |
| config → listen → route startup; same host for CLI and pages | Retain builder-time startup once. Transport receives a built Server; CLI emit never starts a listener implicitly. |
| Compile-time plugins, common/develop/build config, host/port/mode | Retain typed owned sections; now `http`/database selection sections, secrets explicitly supplied. Environment is not implicit plugin discovery. |
| Register database during config | Now PostgreSQL/MySQL service opens eagerly; same Engine service as SQLite, no migration or seed on open. |
| Generate before importing generated client | Retain G20 tooling; Slice 6 owns real stores/actions, Slice 8 generated UI. |
| `generate/install/push/upgrade/migrate/populate/purge/uninstall` | Slice 6 migration/tooling design owns these operations and destructive policy. Slice 7 provides adapters and backend capability facts, not command parity by name. |
| Idea source, schema config, revision history, migration SQL, actual DB state | Retain separation. A backend URL change neither migrates existing data nor proves applied schema. |
| Generated events exposed through API/MCP/desktop | Retain explicit exposure/authorization per surface. No public `emit arbitrary-event` HTTP endpoint. |
| build/serve preview, static assets, SSR/hydration verification | Now production asset serving and streaming transport; Slice 8 owns Leptos HTML/hydration correctness and visual acceptance. Build pipeline is not copied into socket host. |
| Rename safety, force policy, transaction boundaries, population order | Slice 6; MySQL DDL autocommit must not inherit a false rollback claim from SQLite/Postgres. |

No omitted terminal/export family becomes a Slice 7 commitment: inherited
[Slice 2 inventory](slice-2-design-upstream-adapters.md) remains its owner.
