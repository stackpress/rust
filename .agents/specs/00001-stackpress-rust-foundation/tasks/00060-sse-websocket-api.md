# Task 00060: proposed APIs and execution

Companion to the [design checkpoint](00060-sse-websocket-design.md). This file describes proposed contracts, not existing or compiled APIs. Proposed defaults and owner choices live in the checkpoint.

## SSE public surface

Place runtime-neutral encoding/response descriptors in `stackpress_server::http::sse` behind `http-sse = ["http-stream"]`. Native host feature `sse` selects it and owns automatic heartbeat timing. Add facade `server-http-sse` and `server-http-host-sse`; neither changes default features. No new release crate or mandatory Tokio dependency in `stackpress-server`.

```rust
// Sketch only; new types have private fields and fallible builders.
pub struct SseEvent { /* owned bounded data and optional fields */ }
impl SseEvent {
    pub fn data(value: impl AsRef<str>) -> Result<Self>;
    pub fn id(self, value: impl AsRef<str>) -> Result<Self>;
    pub fn event(self, value: impl AsRef<str>) -> Result<Self>;
    pub fn retry(self, milliseconds: u32) -> Result<Self>;
    pub fn comment(value: impl AsRef<str>) -> Result<Self>;
    pub fn encode(&self, limits: &SseEncodeLimits) -> Result<Vec<u8>>;
}
pub trait SseResponseExt {
    fn sse<S>(&mut self, events: S, options: SseOptions) -> Result<&mut Self>
    where S: Stream<Item = Result<SseEvent>> + Send + 'static;
}
pub fn last_event_id(request: &Request, max_bytes: usize) -> Result<Option<&str>>;
```

`data` normalizes CRLF and bare CR to LF, then emits `data: ` for **every** line, preserving empty/trailing lines; empty data emits `data: \n\n`. Terminate each event with a blank line. `id` and `event` reject CR/LF, and `id` also rejects NUL; empty id is valid and resets the browser cursor. Reject an empty named event (omit the field for default `message`). `retry` emits only decimal milliseconds, including zero. Comments prefix each normalized line with `: `, ending with a blank line; a private comment-only variant rejects id/event/retry setters, so comments never create a message or update an id.

Field builders enforce default 1 KiB id, 256-byte event name and 64 KiB aggregate encoded-event limits before internal copies/growth. Encoding checks the complete byte cost, including repeated prefixes/newlines/UTF-8 and retry digits, with checked arithmetic before allocation. Caller-created Strings may already occupy memory; do not claim the encoder bounds caller allocations. Do not accept arbitrary raw field lines. Multiline data/comments are encoded safely; CR/LF in single-line fields is an error.

`last_event_id` reads the existing normalized `req.headers`, distinguishes missing from empty, rejects duplicate fields, CR/LF/NUL and >1 KiB by default. It is untrusted opaque application data, not authorization or an automatic database cursor. Applications own storage, replay windows, id meaning and duplicate suppression; the library does not retain event history.

`res.sse` installs a private one-shot owned descriptor in the existing `Response` native-value seam. It sets status 200, `Content-Type: text/event-stream; charset=utf-8`, `Cache-Control: no-cache`, and `X-Accel-Buffering: no` as a deployment hint. Unknown length omits Content-Length; Hyper owns transfer framing. Do not send a global `Connection: keep-alive` header. No compression feature is added, and reverse proxies must be configured to stream; the hint cannot guarantee proxy behavior.

```rust
// Sketch: existing builder/lifecycle style, application owns replay and security.
async fn updates(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let cursor = last_event_id(req, 1024)?.map(str::to_owned);
    let events = inv.service::<Updates>()?.subscribe(cursor).await?;
    res.sse(events, SseOptions::default())?;
    Ok(Flow::Continue) // Offer normal response hooks before publication.
}
// Inside a plugin's ordinary registration function:
builder.get("/updates", updates)?;
```

Request/response hooks may deny or replace the body exactly as today. Private descriptor identity, not just MIME, selects the long-lived policy; an arbitrary `text/event-stream` byte stream stays an ordinary finite stream. Add metadata accessors to `HttpBody` and a separate extension trait, preserving `HttpResponseExt` and `StreamLimits` layouts/contracts. SSE metadata carries finite byte/chunk/age/idle controls and heartbeat interval.

The native host wraps the selected body in a demand-driven heartbeat driver, initially yielding a comment to commit headers, then `: heartbeat\n\n` only after an idle interval. No ticker task or unlimited channel. Start timers after publication; prefer a ready application event to a simultaneous heartbeat, reset after actual event output, and do not emit catch-up bursts. An encoded event is one bounded chunk, so comments cannot split an event. Host byte accounting includes comments; actual IO progress, rather than producer polls, resets write-idle.

Back-pressure comes from Hyper body demand: at most one bounded event plus transport buffers, no unbounded prefetch. Application subscriptions must also be bounded; provide an optional bounded-channel adapter with explicit full/closed errors. A disconnected/expired/replaced stream drops the source and subscription. A source error after commit closes delivery and records its cause; it cannot send another HTTP status, append JSON, or replay lifecycle hooks.

## In-process and buffered contracts

`StreamingHttpAdapter` can still return SSE's encoded `HttpBody` for finite in-process polling. It spawns nothing and promises no wall-clock enforcement. Add an explicit finite `SseDriver` taking an injected clock/tick source for heartbeat/age tests; native HTTP uses the same driver with Tokio. EOF cancels the timer and fuses the driver. Default in-process polling without that driver yields application events only; document this distinction.

Tests use finite sources and wrap each poll/collection in a timeout plus count/byte budget; never implicitly collect an infinite SSE source. Buffered `HttpAdapter` and CLI/value adapters reject the typed native descriptor by their existing unsupported-native output path (sanitized output failure/500 for HTTP), without polling. A reusable encoder may separately generate finite text bytes for a buffered test response. WS intent through a non-upgrade adapter likewise fails finitely before callback invocation; direct calls may inspect a descriptor but cannot open IO.

SSE works unchanged over the existing opt-in Rustls HTTPS transport. Sessions and CookieOptions retain their existing HTTP behavior. `EventSource` has browser-controlled cookies/credentials and CORS rules; the helper cannot attach arbitrary Authorization headers. Document same-origin use and application-configured cross-origin credentials/CORS, with no global policy change.

## WebSocket declaration and handshake

Expose `stackpress_server_http::websocket` behind native `websocket`; facade `server-http-host-websocket` forwards it. Endpoints are ordinary GET routes registered by functions/plugins, with no separate router or route uniqueness registry.

```rust
// Sketch only; WsConnect owns a checked request snapshot and selected service Arcs.
pub trait WebSocketResponseExt {
    fn websocket<F, Fut>(&mut self, request: &Request, options: WebSocketOptions,
                        connect: F) -> Result<&mut Self>
    where F: FnOnce(WsConnection) -> Fut + Send + 'static,
          Fut: Future<Output = Result<()>> + Send + 'static;
}
impl WsConnection {
    pub async fn send(&mut self, message: WsMessage) -> Result<()>;
    pub async fn recv(&mut self) -> Result<Option<WsMessage>>;
    pub async fn close(&mut self, close: WsClose) -> Result<()>;
    pub fn request(&self) -> &WsRequestSnapshot;
    pub fn close_info(&self) -> Option<&WsCloseInfo>;
}
async fn chat(req: &mut Request, res: &mut Response, inv: Invocation<'_>) -> Result<Flow> {
    let service = inv.shared_service::<Chat>()?;
    let policy = inv.config::<ChatPolicy>()?.clone();
    // App-owned check covers authentication, Origin and this endpoint's policy.
    authorize_chat(req, &policy)?;
    res.websocket(req, WebSocketOptions::default(), move |connection| async move {
        service.run(connection, policy).await
    })?;
    Ok(Flow::Continue)
}
builder.get("/chat", chat)?;
```

At the native boundary, retain existing head/target/framing/deployment limits. When enabled, classify only a plausible WebSocket candidate; reject CONNECT, other upgrades, trailers, Transfer-Encoding and nonzero Content-Length for this handshake. Require HTTP/1.1 GET, Upgrade exactly `websocket`, a Connection token `upgrade` across all header occurrences, single Version `13`, and a single Key whose strict base64 decoding is exactly 16 bytes. Reject duplicate singleton fields and malformed/comma-mixed upgrades. Unsupported version may return 426 with `Sec-WebSocket-Version: 13`; other malformed candidates return sanitized 400, before route callbacks.

The cached tungstenite `handshake::server::create_response` validates several fields and derives the accept key, but only checks that Key exists: first-party code must validate its encoding/length and singleton headers. Its Connection check reads only the first occurrence; pass a small typed request with the validated key and normalized Connection/Upgrade fields after checking all original occurrences. Reuse that response generator; do not write an HTTP parser or duplicate the SHA-1/GUID algorithm. No `accept_async` after Hyper's handshake: that would attempt a second handshake.

A valid candidate then runs the normal normalized load, `request`, all matching route handlers, and applicable `response`/`error` stages once. Configured session/CSRF/origin handlers run according to their normal ordering/activation. An HTTP denial or replacement drops the pending callback; it does not upgrade. No endpoint intent yields the normal HTTP route response (including 404); a GET to a WS-only handler without a valid candidate yields 400. A hand-written status 101 without the typed intent is invalid.

G42 applies unchanged: later handlers may replace the response/intent and drop the prior callback; only the final descriptor is consumed. Return Continue to allow response hooks; Abort preserves today's skip-later-stage semantics, not a new mandatory WS guard. Framework code never locks a route to its first handler. Applications must compose denial/abort correctly; setting a denial status and then continuing to an accepting handler is not an authorization barrier.

### One-shot native output seam

Add an opt-in server `http-upgrade` feature with an additive generic native-output adapter entry, sharing normalization, `handle_input` and guarded publication with `handle_body`. Its host-supplied preflight examines the **final** Response and returns either ordinary output or a typed pending native response/action. Use new reply/output types rather than extend existing exhaustive enums; `handle_body` keeps rejecting 101.

The HTTP host owns the concrete private WS intent and callback. Server only provides the generic output seam, not tungstenite or a back-reference to server-http. Preflight validates foundation response budgets, cookies and headers before consuming its descriptor, then the existing OutputGuard grants exactly one publication. Only a WS intent matched to trusted candidate metadata may choose 101. SharedNative/body/ResponseView clones convey no new permit. No request-global pending callback table.

The host obtains `hyper::upgrade::on` from the original incoming request, retains it as trusted host-only ownership outside the normalized Request, and drives `serve_connection(...).with_upgrades()`. It publishes a bodyless 101 with Connection/Upgrade/Sec-WebSocket-Accept and an optional single application-selected offered subprotocol. Reuse ordinary cookie/header conversion (including repeated Set-Cookie), but regenerate protected handshake/framing headers after hooks; strip length/transfer coding and do not negotiate extensions/compression. Reject an unoffered or invalid subprotocol before 101.

After Hyper completes the handoff, wrap `Upgraded` in `TokioIo` and call `tokio_tungstenite::WebSocketStream::from_raw_socket(..., Role::Server, validated_config)`. Reading through `Upgraded` preserves Hyper read-ahead bytes; never recover a bare TCP socket and discard its unread buffer. Bound handoff/101 flush by the existing output deadline. Upgrade failure cannot send a second HTTP response or invoke open.

M0 must prove the generic output seam can preserve output/error reports and private ownership without a circular dependency or breaking API. A 101 is not an arbitrary native-response passthrough: endpoint and protocol validation remain host-owned. Stop for a revised design if this boundary cannot be kept small and testable.

### Connection lifecycle and application state

One supervised connection driver owns protocol IO, finite buffers, timers, callback and queue permits. `WsConnection` is the application-facing send/receive handle, not a clonable raw socket. Its bounded outbound queue lets send complete without requiring an application recv call; the driver keeps reading Pong/Close and flushing control frames while the callback waits. No mutex is held across user code or await. Dropping the handle asks the driver to close; host cancellation aborts/reaps both.

Open means callback invocation after successful handoff. Message means one complete Text/Binary from `recv`, in wire order. Peer Close finishes `recv` with None and exposes code/reason/cleanliness; a transport/protocol error yields one Err then fused EOF, and is retained in a new connection report/observer. Close is reported once on every exit, even if open failed; callbacks are not recursively reentered on errors. Callback normal return requests 1000; callback error requests sanitized 1011 and records the cause. Reserve terminal outcome storage independently of a full data queue.

Do not automatically dispatch frames through `Server::handle`, which would falsely repeat HTTP request/session/response stages. Applications may explicitly invoke named internal events; that API retains all-matching priority/FIFO and Abort. Automatic `open`/`message`/`close`/`error` event names are deferred under O5, not silently exposed to clients.

`WsRequestSnapshot` defaults to bounded URL/method; header names and session keys require explicit allowlists in WebSocketOptions, at most 32 of each and 16 KiB total snapshot bytes. Check selected values before copying when the handler selects the intent, omitting the HTTP body; later hooks do not refresh that snapshot. Request/Invocation are never borrowed across handoff. Capture typed config copies and `inv.shared_service` Arcs deliberately. Later mutation cannot set HTTP cookies or imply session persistence, revocation or fresh authorization. Apps use their session services and recheck privileged messages as required. Never log these snapshots by default.

Cookie-bearing browser handshakes can be sent cross-site; missing Origin checks can permit cross-site WebSocket hijacking. CORS is not WS authorization and a GET CSRF hook may intentionally do nothing. Provide an optional bounded Origin comparison helper and examples checking scheme/host/port, with explicit absent/`null` Origin policy for non-browser clients. G43 leaves selecting that guard, credentials and per-message policy to the app. Avoid credentials in query strings where logs can retain them.

The existing Rustls acceptor supplies WSS using the same post-TLS upgrade path; do not enable tungstenite client/TLS connectors. Proxy deployment must explicitly forward Upgrade/Connection and have suitable idle timeouts; retain existing trusted-forwarding rules. No HTTP/2 extended CONNECT, tunnels, broadcast rooms or reliable delivery guarantee is added.

## WebSocket flow control, close and no-panic rules

Await outbound capacity within the send deadline and charge both count and bytes. A full inbound data queue closes with 1013 instead of accumulating memory or indefinitely starving protocol control processing. Bound messages even when fragmented; enforce quotas per connection and release all queued byte permits on failure/drop. There is no unbounded broadcast queue or whole-message clone per listener.

Use 1000 normal, 1001 stop/age, 1002 protocol/masking, 1007 invalid UTF-8, 1009 oversized message/frame, 1013 overload and 1011 internal callback error. App-owned policy rejection may use 1008 after upgrade or ordinary HTTP denial before it. Validate outgoing close codes and <=123 UTF-8 reason bytes before queueing; never transmit reserved 1005/1006/1015. Preserve valid peer close metadata; abnormal disconnect is a local outcome, not an invented wire code. Browser-originated application closes are restricted to browser-supported codes.

G62: errors instead of unwrap/expect/panic/unreachable, checked duration/count arithmetic, fallible header/base64/status conversion, bounded allocations and explicit closed/poison/cancel states. Do not expose tungstenite's mutable config: its constructor asserts `max_write_buffer_size > write_buffer_size`. Validate that inequality and headroom for a maximum message, every finite duration and semaphore bound before calling it. This prevents a known dependency panic; it is not a claim that all dependency/runtime/OOM or user callback panics are impossible.

## Leptos client helpers

Add `stackpress_leptos::realtime::{use_event_source,use_websocket}` behind independent `client-sse` / `client-websocket` features, forwarded by facade `leptos-sse` / `leptos-websocket`. Available with separate hydrate/CSR products; SSR returns inert Disconnected state and never touches Window/network. Construct connections only after browser mount, so rendering does not open or duplicate them during hydration.

```rust
// Sketch: owner-scoped handles expose reactive state and explicit close.
pub fn use_event_source(url: String, options: EventSourceOptions) -> Result<EventSourceHandle>;
pub fn use_websocket(url: String, options: BrowserWebSocketOptions) -> Result<WebSocketHandle>;
// SSE handle: state, latest MessageEvent data/type/last_event_id, close().
// WS handle: state, latest text/binary, bounded send(), close(), reconnect().
```

SSE options include with_credentials=false and a bounded list of named event types. Keep one EventSource alive: the browser handles retry/Last-Event-ID reconnect, including server EOF rotation; do not run a competing reconnect timer. Explicit close or Leptos disposal removes listeners, drops retained closures and permanently closes that instance. A new URL/owner creates a fresh instance and does not promise persisted cursor replay; EventSource cannot set an initial Last-Event-ID header. Preserve MessageEvent.last_event_id in delivered state.

WS uses binaryType=ArrayBuffer and typed text/binary delivery with browser exceptions mapped to Result. Check max message and `bufferedAmount` before send; return a back-pressure error above 256 KiB, and queue nothing in Rust. Browser WebSocket itself lacks receive back-pressure: bound data retained/exposed by the helper (latest value or a bounded queue), document that browser internal allocation is outside that guarantee. Reconnect defaults off; explicit reconnect never resends messages. Optional bounded retry (1/2/4/8/16/30 seconds, <=5 attempts) is canceled on explicit close/disposal and resets only after a successful open; it still cannot promise exactly-once delivery.

Target-wasm-only web-sys additions: `EventSource`, `EventSourceInit`, `MessageEvent`, `Event`, `EventTarget` for SSE; `WebSocket`, `BinaryType`, `CloseEvent`, `ErrorEvent` plus MessageEvent/Event/EventTarget for WS. Reuse existing Window, js-sys ArrayBuffer/Uint8Array and wasm-bindgen Closure. Retain closures without `forget`, remove each installed listener on disposal/failure, clear retry timers and close the native browser object. Browser graphs must contain neither Hyper, Tokio native networking nor tungstenite.
