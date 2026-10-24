# Slice 2 design: transport and terminal inventory

Companion to the [host inventory](slice-2-design-upstream-host.md). Load to audit HTTP/WHATWG, Stackpress command tooling, lib terminal exports and tests, and all Slice 1 host deferrals. Dispositions are proposed scope, with the same S2/S3/S5/S2-T/N meanings as that inventory.

## HTTP and WHATWG surfaces

Sources: Ingest [HTTP](../../../../ingest/ingest/src/http/index.ts), [HTTP Adapter](../../../../ingest/ingest/src/http/Adapter.ts), [HTTP helpers](../../../../ingest/ingest/src/http/helpers.ts), [WHATWG](../../../../ingest/ingest/src/whatwg/index.ts), [WHATWG Adapter](../../../../ingest/ingest/src/whatwg/Adapter.ts), [WHATWG helpers](../../../../ingest/ingest/src/whatwg/helpers.ts).

| Export / behavior | Disposition and reason |
| --- | --- |
| Both `Adapter`, static `.plug`, constructor, `.plug/.request/.response`; `handler`, `server`, `router`, `action` | S2 one optional `HttpAdapter` using `http::Request<Vec<u8>>` → `http::Response<Vec<u8>>`, explicit host entry and same handlers. No arbitrary external event override. |
| `gateway` in both entries | S2-T real network/runtime ownership. WHATWG gateway also creates a Node server; its presence does not prove serverless support. |
| Shared exports (`Status`, `Exception`, `Request`, `Response`, session/cookie, router facets, loaders, helpers, decorators) | Follow the [host inventory](slice-2-design-upstream-host.md); do not create transport copies or silently re-export all JS names. |
| `loader` | S2 bounded buffered body decode through lib `load_with`/`parse_body`. S2-T streaming/backpressure; N unlimited WHATWG `.text()` and HTTP character-count limits as security policy. |
| `dispatcher` | S2 checked buffered wire conversion, native status legality, repeated headers and scoped cookie revisions. S2-T streaming/native writer output. No duplicate send or retry on partial output. |
| Native request resources on framework Request; native response passthrough | S2 adapter-owned envelopes. No native socket/JS resource field in foundation Request/Response; S2-T native response override can be designed separately. |
| Method/default MIME, query/URL/cookies/headers normalization | S2 standard/extension methods and lib query/session values. Preserve pathname and repeated slashes; do not rewrite a path based on query data. Reject invalid UTF-8/header inputs rather than guess. |
| `imToURL`, `imQueryToObject`, `reqToURL`, `reqQueryToObject` | S2 explicit URI/base conversion and query parse. N blindly trusted `x-forwarded-proto`, fallback on malformed authority, repeated/trailing slash rewriting outside the accepted matcher grammar. A caller can explicitly supply a trusted base URL. |
| `WhatwgRequest`, `WhatwgResponse` global aliases | N JavaScript runtime aliases. Typed HTTP request/response values are the S2 boundary. |
| `readableStreamToReadable`, `readableToReadableStream`; raw Node/Web streams, binary chunks | S2-T streaming/cancellation/backpressure. S2 owns bounded raw byte buffers, never claims stream equivalence. |
| Object/array results and status-only error JSON; text/buffer passthrough | S2 existing public ResponseEnvelope conversion for structured/status-only output, raw text/bytes for explicit bodies. Preserve zero totals/null/false and reject rich unsupported wire values. |
| Set-Cookie/header iteration with repeated `.set` calls | S2 append each value; intentionally fix source header replacement that loses multiple cookies. Cookie deletion uses retained original scope from Slice 1. |
| Load → request → selected route → response → output | S2 common host lifecycle, including load failures offered to error where normalization permits. See [lifecycle](slice-2-design-lifecycle.md). |

## Stackpress server package exports

Sources: [index](../../../../stackpress/packages/stackpress-server/src/index.ts), [Terminal](../../../../stackpress/packages/stackpress-server/src/Terminal.ts), [plugin](../../../../stackpress/packages/stackpress-server/src/plugin.ts), [events](../../../../stackpress/packages/stackpress-server/src/events/index.ts), [scripts](../../../../stackpress/packages/stackpress-server/src/scripts/index.ts), [types](../../../../stackpress/packages/stackpress-server/src/types.ts), [HTTP facade](../../../../stackpress/packages/stackpress-server/src/http.ts), [WHATWG facade](../../../../stackpress/packages/stackpress-server/src/whatwg.ts).

| Export / behavior | Disposition and reason |
| --- | --- |
| `Terminal`/`StackpressTerminal`, `.server/.verbose/.force/.config/.brand/.cwd`, `.bootstrap/.run` | S2 `stackpress-cli::Terminal`, owned argument parse/settings, host startup once, one handled target event and writer-injected output. No live terminal in immutable server services. |
| `--verbose/-v`, `--force/-f`, `--bootstrap/-b`, `terminal.label` | S2 typed operator options before the `emit` separator; `-b` selects an explicit JSON data file, never compiled code. `force` has no global destructive effect. Brand belongs to terminal options. |
| Terminal registers itself as plugin `terminal` | N mutable I/O controller in shared host state; S2 request-local `CliContext` metadata and explicit writer handles in CLI adapter. A capability must not acquire stdin ownership implicitly. |
| Missing-command verbosity; errors rethrow except successful error event; serve/develop bypass | S2 deterministic envelope plus exit mapping and diagnostics on stderr. N ad hoc dispatch-status 200 recovery and silent unknown command. Serve/develop are deferred, so their exception special case is unnecessary. |
| `plugin`, events factory/default `listen`, named `events::{develop,emit,serve}` | S2 compiled function plugin registration pattern; CLI `emit` is adapter syntax, not a recursively exposed framework event. S2-T `serve`; S3 development/build tooling for `develop`. |
| `events/emit` guards existing non-200, checks process argv length, progress, calls script then sets 200 | S2 parser requires event, selects it once, preserves operation response/abort/failure. N nested command wrapper that can hide the target's status. |
| `scripts::emit(server, skip)` | S2 reusable `run` over supplied argv/sinks, no global process.argv access or secondary parse/join. |
| `events/serve`, `scripts::serve`, create/listen, defaults host 127.0.0.1/port 3000, ready/error/close IPC, graceful HTTP+Vite shutdown | S2-T managed listener integration, separate task. No listener can be exercised in task 00005. |
| `events/develop`, `scripts::develop`, `DevelopmentServer` | S3 build/development tooling, only when a real watch/build workflow exists. Do not import chokidar/Node child-process policy into the host. |
| `DevelopmentServer::{terminal,shutdown,start,close,restart,watch}`, `Promiser`, `DevelopmentSpawner`, `DevelopmentWatcher`, `DevelopmentOptions` | S3 supervision contract: ready handshake, early-exit/error, restart debounce/queue, extension/path ignore rules, timeout, forced close and shutdown cleanup. N JS Promise/IPC object types. |
| `DEVELOPMENT_READY`, `DEVELOPMENT_ERROR`, `DEVELOPMENT_SHUTDOWN` | S3 protocol concept; exact Node message literals need not become Rust public API. |
| `Transformer` | S3 Idea assessment/transform integration; not a host dependency. |
| `StackpressServerException`, `IngestException`, `Exception` aliases | S2 existing Error plus structured startup/handle reports; N parallel empty exception subclasses. |
| `control` | S2 CLI-owned writer/brand/color abstraction; details below. |
| `TerminalConfig`, `TerminalPlugin` | S2 typed `TerminalOptions`, request-local CLI metadata; N naming a mutable terminal object as an arbitrary plugin value. |
| `ServerConfig` (`build/cwd/mode/develop/host/port/process`), `DevelopmentConfig` | S2 host/input limits and explicit config; cwd only in CLI file adapter. S3 build/develop fields; S2-T host/port; S5 rendering mode consumer. No dormant config claims. |
| `RouteProps`, all forwarded Ingest loader/router/action/server/native/HTTP/WHATWG types and `Body` | Exhaustively classified in the [host inventory](slice-2-design-upstream-host.md); S2 uses one handler/value contract and narrow native envelopes. |

Aggregate composition `server → schema → language → csrf → sql → view → session → api → admin` remains an upstream ordering reference, not a requirement to scaffold those plugins here. Only selected real plugins register. Generation, SQL, auth and UI are owned by their later tasks.

## Lib terminal exports and types

Sources: [Terminal](../../../../lib/src/terminal/Terminal.ts), [control](../../../../lib/src/terminal/control.ts), [input](../../../../lib/src/terminal/input.ts), [terminal types](../../../../lib/src/types.ts). These were explicitly deferred from Slice 1; S2 must implement the CLI behavior, including prompt logic, rather than quietly dropping it.

| Surface | Disposition and reason |
| --- | --- |
| `Terminal(command,args,brand)`, `.command/.args/.brand/.control/.data`, `.expect`, `.run` | S2 owned command/args/Nest/options, typed alias lookup that preserves false/0/empty, explicit `run(&Server, ...)`. Existing lib parser is reused. N shallow aliasing copies, truthiness lookup and argument joining. |
| `control(brand)`, `.output/.error/.info/.success/.system/.warning/.input` | S2 `Control` with injected writer, level/color policy, branded placeholder replacement, required/default prompt forwarding. Machine JSON exclusively stdout; progress/prompt/diagnostics stderr. ANSI only for selected terminal output. |
| Default `input(config, context)` | S2 pure prompt state machine plus explicit native CLI I/O driver. No host-global stdin mutation, no blocking prompt inside capability async dispatch. |
| `createAbortError` | S2 typed `PromptError::Aborted`; cancellation is distinct from invalid answer, EOF and I/O failure. |
| `validateInput` | S2 required check, Rust-regex pattern check/error text, sync/async validator with exact success/invalid result. N JavaScript global regex cursor reset; regex is immutable. |
| `formatValue`, `renderPrompt`, `clearRender` | S2 transformation/final-state rendering and terminal erase policy, with injected event/output tests. No escapes for pipes. |
| `TerminalInputConfig` | S2 `PromptOptions`: message/default, Tab/Editable prefill, required, bounded answer length, optional pattern and validator/transformer. |
| `TerminalInputContext` | S2 explicit input/output driver, cancellation input, clear-on-done. Own the raw-mode guard and restore prior state, not an assumed false state. |
| `TerminalTTYInput`, `TerminalTTYOutput` | S2 native driver adapter over terminal events/TTY detection; no Node streams. Platform support limited to measured targets. |
| `TerminalPromptState`, `TerminalInputTheme` | S2 state/current/default/error/final fields and Keep/Clear invalid policy as enums. Unicode-safe backspace; no byte slicing through UTF-8. |

The prompt driver runs outside host dispatch. A validator can be async; the native driver must not block the async executor while waiting for terminal input. The proposed dependency/ownership boundary appears in [adapters](slice-2-design-adapters.md).

## Tested adapter and terminal behaviors

| Test source | Scenarios and disposition |
| --- | --- |
| Ingest `whatwg.Adapter.test.ts` | S2 complete route lifecycle; method/MIME/query/cookie/header construction; urlencoded load and already-cached no-op; string/status/headers, object JSON, typed-array raw bytes; errors/cookie revisions. Node streams → S2-T; existing native Web Response passthrough → N in buffered adapter; source slash normalization → deliberate changed contract. |
| Ingest `helpers.test.ts` | S2 JSON/urlencoded/multipart dispatch (already lib parsers), URL/query with/without params, binary/empty byte buffers. Unknown MIME empty-object fallback → N. Node↔Web stream empty/multiple/binary chunk conversion → S2-T. HTTP/TLS and forwarded-protocol scalar/array/comma/precedence/fallback behavior → S2-T trusted transport policy; do not copy blind trust. |
| Stackpress `Terminal.test.ts` | S2 typed settings/brand/cwd equivalent, config/listen/route order, command and terminal MIME, caught-command errors through shared error, verbose unknown-command message. Self-registration and false/dispatch-status inference → deliberate replacement. |
| Stackpress `plugin.test.ts` | S2 phase callback registers capabilities during listen; serve/develop/emit catalog is mapped above, not falsely implemented by matching export names. |
| Stackpress `serve.test.ts` | S2-T default/configured host and ready-only log; no socket fixture in this task. |
| Stackpress `develop.test.ts` | S3 direct CLI start and ready wait, backend debounce/frontend ignore, bind-error rejection, exactly one queued replacement during restart. |
| Lib `Terminal.test.ts` | S2 command args/brand/control method set, parsed data, alias/default lookup, successful run result and error envelope. |
| Lib `control.test.ts` | S2 plain brand, each level's ANSI color, empty-brand trim, sequential `%s` replacement, required prompt and default on empty input. Output channel split is a deliberate CLI correction. |
| Lib `input.test.ts` | S2 typed/default/required values; pattern reject; sync/async validation; Keep/Clear invalid answers; cancel before/during input; Editable/Tab prefills; backspace; Ctrl-C; clear-on-done; piped errors without escapes; transformed final display; control-to-input wiring and raw-mode cleanup. Add EOF, validator failure, Unicode, output error and guard-drop cases beyond upstream. |

## Slice 1 deferral closure

| Deferred item | Slice 2 disposition |
| --- | --- |
| `Terminal`, `control`, `input`, all six terminal input type families | Implement in tools/cli, including prompt state/validation/cancellation/cleanup. |
| `RouteMap`, `RouterContext`, `RouterArgs`, `RouterMap`, `RouterAction` enrichment | Host invocation context plus explicit transport envelopes; compatibility tuple/conditional types intentionally not ported. |
| Host/service context; generic CaseStudy service access | Typed registry behind explicit `Arc`, available through Invocation; no global state or registry locks. |
| Route-to-request capture injection | Explicit host route wrapper, named captures overwrite literal top-level aggregate-data keys per listener; positional captures remain Invocation-only. Raw lib behavior remains unchanged. |
| Native request/response/resource wrapping | Buffered HTTP native pair and CLI-owned sinks; foundation values remain non-generic and non-Clone. |
| Method/path resolve convenience | Host `handle_route`/`resolve_route`, uppercase checked method plus actual pathname. Raw resolve remains raw. |
| Body/FileStream/native stream ownership | Explicitly defer to S2-T for backpressure/stream cancellation and S3 for filesystem streams. Buffered byte output is implemented here; no stream support claim. |
| Native load/delivery callback result | Adapter returns an owned native HTTP response or CLI run result while using the existing one-attempt Response contract; no sockets or hidden transport resources. |

Scope decisions for the HTTP subset, prompt driver and S2-T deferrals remain in the [decision checkpoint](slice-2-design-decisions.md). All relevant export/type/test families now have a disposition, including source behavior intentionally changed rather than copied.
