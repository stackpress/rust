# Runtime and portability contracts

Authority: the linked Rust implementation and accepted decisions G06–G62. [Project direction](project-direction.md) states the composition and source-port policy; [source map](source-map.md) locates its producers and proofs.

## Ordered capability execution

[Foundation event dispatch](../../crates/lib/src/event.rs) freezes builder registrations into an immutable emitter. All matching listeners run sequentially in descending priority and registration-order ties. Matching retains exact/pattern captures in invocation-local metadata; Rust regex syntax is the matcher contract, without ECMAScript lookaround/backreference parity. Never replace all-matching events with a single-winner transport router.

Named async handlers borrow `Request`, `Response` and `Invocation` and produce `Send` futures returning `Result<Flow>`. `Flow::Abort` cooperatively stops the dispatch. `DispatchOutcome::{Completed,Aborted,NoListener}` is distinct from response status, HTTP status and CLI exit status. Registrations are not mutated under a global lock across handler awaits. Nested/concurrent invocations carry their own metadata and immutable host context.

Ordinary `Value`/`Nest` data clones deeply; sharing native payloads is explicit as `Value::Shared(SharedNative)`. Request/Response are non-Clone so attempt/finalization state is not silently duplicated. Loading and delivery are one-attempt state machines with no implicit retry. Native resources remain optional; callers can construct and mutate requests/responses without sockets.

## Startup, configuration and plugins

[ServerBuilder](../../crates/server/server/src/builder.rs), [plugin preflight](../../crates/server/server/src/plugin.rs) and [typed config](../../crates/server/server/src/config.rs) own host construction. Selected compiled registration functions contribute typed config sections, services and Config/Listen/Route startup callbacks. Unique plugin IDs, missing prerequisites, cycles, wrong order and singleton conflicts are checked before immutable host publication; listener/route duplicates remain allowed (G42).

Config layers append in precedence order and unknown top-level sections are rejected. The application convention (G47) is plugin defaults → shared `src/config.rs` → CLI `-b config/develop.json` or `config/build.json` → explicit app overrides. Shared computed/environment-derived values belong in application code. [CLI primitives](../../tools/cli/src/lib.rs) supply `main_with` for an application-owned binary, with operator flags before `emit`; tokens after the event are capability data. There is no stock executable.

## Top-level lifecycle and errors

[Server lifecycle](../../crates/server/server/src/lifecycle.rs) distinguishes `handle` / `handle_with_options` / `handle_route` / `call` from raw `emit` / `resolve` and nested dispatch. A top-level entry performs input normalization/load, `request`, the selected operation, and `response`, followed by adapter output. General hooks are not recursively wrapped around every nested event.

- Input failure before a normalized Request produces a fresh sanitized 400 and a correlation report, with no fabricated request or lifecycle callbacks (G17). A load/body failure after normalization follows the ordinary error path.
- Request Abort stops later stages. Request failure skips the operation even if `error` recovers the output; response may run if recovery leaves delivery open and does not abort.
- Operation Abort skips response. Open ordinary completion prepares output, including the missing-output 404 rule, then offers response once.
- An error discards partial body/headers/cookies and prepares a fresh public error response before offering `error`. Error Abort skips response; an error-listener failure retains both failures, never recurses and falls back to a sanitized generic 500. A response-stage failure is offered once without restarting response.
- Delivery finalization prevents further response mutation/hooks; output is a separate guarded adapter boundary. A response hook is not a finally block.

Reports retain error causes and explicit plugin/listener/event/stage/registration and nested invocation information where available. Correlation is per host, without a distributed-tracing promise. Public error views omit developer diagnostics. Do not automatically log raw request/session contents or claim a thread backtrace is a complete asynchronous call history.

## HTTP boundary

[Buffered HTTP](../../crates/server/server/src/http.rs) performs in-process normalization/conversion without a listener. The [owned HTTP/1 host](../../crates/server/http/src/lib.rs) uses Hyper over supplied IO/acceptors, with bounded keep-alive, sequential pipelining, admission/deadlines, cancellation/drain and immutable startup-checked static assets. Binding, runtime and process signals are caller-owned. TLS is opt-in Rustls; explicit deployment/proxy trust controls forwarding. HTTP/2 is outside the implemented host contract.

[HTTP JSON projection](../../crates/server/server/src/http/json.rs) writes native Dates as UTC ISO-8601 text at HTTP output only and retains redirect bodies (G57). Core value/envelope serialization stays strict; other native non-JSON values still return errors. [HTTP options](../../crates/server/server/src/http.rs) default cookie `Path=/`. Explicit revision options win as a whole, even an explicit empty options object; removal uses the same selected scope and the foundation tombstone lifetime. Do not change CookieOptions globally to implement host defaults.

Hyper owns parsing/framing (G28/G31). Equal duplicate Content-Length may be normalized by Hyper; still-visible Transfer-Encoding plus Content-Length is rejected, and connections carrying Transfer-Encoding close after their response. Do not promise inspection of raw headers already normalized by Hyper. Streaming is separately selected through server `http-stream`; response publication follows lifecycle completion, and post-commit failure cannot replace already sent headers/body.

## Opt-in realtime sessions (G63)

[Runtime-neutral SSE](../../crates/server/server/src/http/sse.rs) uses `http-sse`; the native host separately selects `sse`/`websocket` and validated host limits. Slots include pending publication/handoff and remain held under the global connection permit until final cleanup. Combined protocol caps must leave ordinary HTTP capacity. Valid enabled WebSocket endpoints now answer 101; feature-off upgrade requests retain 400 rejection. Existing exhaustive output enums and required response-trait methods remain unchanged; native publication uses additive sibling types.

[Realtime policy](../../crates/server/http/docs/guides/realtime.md) defaults to 64 SSE plus 64 WS slots, a separate finite one-hour session clock, 64 KiB events/messages, SSE heartbeat/write idle 15/30 seconds and 64 MiB aggregate bytes. WS queues are bounded by count and bytes, with independent Ping/Pong/Close control and finite send/close deadlines. Stop finishes the current SSE event then EOF; WS sends Close(1001), with both bounded by host grace. Application callbacks, producers and upgraded IO remain supervised by the serving future. Counters describe local handoff, never peer acknowledgement.

Endpoints retain normal G42/G43 lifecycle and ordered routes. Applications own Origin/authentication/CSRF, session expiry and per-message authorization; no implicit guard is added. Cookie-bearing cross-site WS requires deliberate Origin policy to prevent hijacking. SSE cursor/replay/deduplication and proxy buffering/timeouts are application concerns.

[Leptos realtime helpers](../../crates/leptos/src/realtime/mod.rs) are independent `client-sse`/`client-websocket` features. Native SSR is inert; browser effects connect after mount. Owner disposal closes transports, removes listeners and cancels timers. EventSource owns SSE reconnect/Last-Event-ID; WS retries are opt-in, finite and never resend messages. Latest values and browser send admission are bounded; browser-internal parsing/buffers are outside Rust retention accounting.

## Generation and application-owned families

Idea parser/AST/composition remain separate from framework interpretation. Native app-owned generators compile independently before the application. [Idea transformer](../../crates/idea/idea-transformer/src/lib.rs) runs selected function transforms in declared order with input snapshots, disjoint plugin partitions and one aggregate output owner. [Output installation](../../crates/idea/idea-transformer/src/output.rs) checks ownership/drift, stages changes and synchronizes staged files before rename, with best-effort directory synchronization; it promises no crash atomicity. Generated source is inspectable; `--check` detects missing/edited/stale output. Remove only explicitly owned files, never handwritten application files.

Framework source imports are compiled catalog entries, resolved through [PackageLoader](../../crates/idea/idea-transformer/src/source/package.rs) and [the aggregate schema loader](../../crates/framework/framework/src/native/commands/schema.rs):

| Import | Source owner and export |
| --- | --- |
| `stackpress-session/schema.idea` | [session schema](../../crates/framework/framework-session/schema.idea), `stackpress_framework_session::SCHEMA_SOURCE` |
| `stackpress-api/schema.idea` | [API schema](../../crates/framework/framework-api/schema.idea), `stackpress_framework_api::SCHEMA_SOURCE` |
| `stackpress/stackpress.idea` | [aggregate schema](../../crates/framework/framework/stackpress.idea), `stackpress_framework::SCHEMA_SOURCE` |

Apps extend these sources and generate their own contracts/metadata, schema/columns/codecs, stores/actions/events, relations and UI/admin catalogs. The native [SourceModel / SessionModels / ApiModels contracts](../../crates/framework/framework-sql/src/source/mod.rs) bind the generated family. Session-only generation keeps API relations external without silently generating API tables; API/aggregate families supply all four framework models. Stock signup keeps source inputs; accepting extra signup fields requires app form/event/hook work (G50).

Use session `auth::register_with::<M>` / `composite::register_with::<M>`, session/API SQL `register_with::<M>` and API process `register_with::<M>` where applicable. [Aggregate register_with](../../crates/framework/framework/src/native/mod.rs) takes the app's static `CompiledClient`, including `FrameworkBindings` for selected session/API stages and ordered model listen/admin callbacks. Selecting these stages without required generated bindings is a construction error. No default model aliases or default-family package remain (G59). Crate-local generated test families use official regeneration examples and freshness checks.

Schema revision/plan artifacts record history, distinct from applied database state. Generation/check does not create history or migrate a database. [Framework SQL](../../crates/framework/framework-sql/src/lib.rs) borrows an Engine/exclusive transaction; operation success does not implicitly commit. SQLite has an explicit migration runner. [Source setup](../../crates/framework/framework-sql/src/setup.rs) executes MySQL DDL in source order with explicit non-atomic progress, versus atomic PostgreSQL/SQLite setup plans (G48). This does not add a generated MySQL migration journal/repair runner. MySQL purge omits CASCADE and uses child-first DELETE for referenced tables; PostgreSQL retains CASCADE and SQLite uses DELETE.

## Shared UI and execution hosts

[stackpress-leptos](../../crates/leptos/src/lib.rs) supplies reusable components/root-scoped state and synchronous, buffered async and opt-in streaming rendering. [framework-view](../../crates/framework/framework-view/src/lib.rs) supplies explicit public snapshots, page selection and optional Leptos/host adapters. Build SSR, hydrate and CSR separately. Stackpress selects server pages; leptos_router/leptos_meta provide client routing and Head. Initialize native SSR inside the caller's Tokio runtime. Snapshot projection/escaped document transport are deliberate disclosure boundaries.

Installed Tauri applications use packaged client assets and remote or permitted native-host adapters. Keep filesystem, permissions, notifications, signing and process/network differences at explicit adapters; browser WASM has no direct native/server-secret access. The blog is web-only with desktop deferred and MCP unsupported (G52). Existing native proof code, WASM builds and Chromium tests are distinct from installed platform acceptance. No general serverless, offline sync or all-platform support is inferred.

## Accepted source-port exceptions

G42/G43 preserve source routes, config activation and each route/event's security model. Internal dispatch and client permission display do not authorize callers. Typed generated-admin surfaces retain their explicit host policy/CSRF requirements; do not generalize that into a new global framework guard. Earlier accepted foundation/parser corrections (G08/G12/G18/G19) remain scoped contracts: strict bounded parsing, cooperative operation abort, scoped cookie removal, repaired Idea grammar and upstream immutable-import replacement semantics.

| Decision | Resulting exception and source |
| --- | --- |
| G51 | [Source validation](../../crates/framework/framework-schema/src/source/checks.rs) makes broken/omitted assertions work as named, retaining attribute names/arguments/messages: eq/neq comparisons, regex, word count, type checks and action-enforced uniqueness; cgt/clt are strict exclusive character bounds. |
| G53 | [Source predicates](../../crates/framework/framework-sql/src/source/predicates.rs) require **all** values in multi-value `has` on MySQL/PostgreSQL/SQLite, addressing the same JSON/list column. `eq` remains single-value; array misuse retains source indexed behavior. |
| G54 | The same predicates make scalar `hasnt` exclude that member; multiple values require none of them, joined with AND. |
| G55 | [Rust source components](../../crates/leptos/src/components/source) use pulldown-cmark Markdown; monospaced code/JSON display without syntax coloring; tab-aware text areas; common Rust mask patterns; losslessly copied Frui country/currency data. Formula is bounded arithmetic (`+ - * / %`, parentheses and template filling), returning 0 on error; arbitrary JavaScript is unsupported. No bundled CodeMirror/Shiki/Inputmask/markdown-to-jsx libraries. |
| G62 | [CSV import](../../crates/framework/framework-admin/src/transfer) admits raw bytes, detects UTF-8/UTF-16 BOMs, then UTF-8 or a Windows-1252 / Arabic-evidence Windows-1256 fallback, and sniffs comma/semicolon/tab. Encoding inference is heuristic; malformed BOM-marked input fails. Export remains UTF-8 comma; existing transfer validation, ceilings, create-only batches and guard decoding remain. |

[Historical direction and runtime](../references/00042-preimplementation-direction-runtime.md) retains the previous unresolved questions and proposals; load only for provenance, not as pending work.
