# SSE and WebSocket hosting

Enable `sse` and/or `websocket`, then select `with_sse_limits` and/or
`with_websocket_limits` on the host before `bind_prepared` or `serve_*`.
Selecting a Cargo feature alone does not admit sessions. Endpoints are ordinary
GET routes: request, ordered route and response hooks run before one publication.
A response hook may replace or deny a WebSocket intent. Bare 101 remains invalid.

The [runnable example](../../examples/realtime.rs) sends an SSE event and echoes
WebSocket data. Development execution uses the managed listener command:

```sh
devmetrics start --summary "Stackpress realtime example" -- 'cargo run -p stackpress-server-http --features sse,websocket --example realtime -- {port}'
```

## Finite policies

| Resource | Default / bound |
| --- | --- |
| SSE / WebSocket slots | 64 each, pending handoff included; sum must leave ordinary HTTP capacity |
| Session age | At most 1 hour from publication, including upgrade handoff |
| SSE encoded event / cursor / event name | 64 KiB / 1 KiB / 256 UTF-8 bytes |
| SSE aggregate bytes | 64 MiB, including comments; EOF rotates connection |
| SSE heartbeat / actual write idle | 15 s / 30 s; heartbeat must be less than idle |
| SSE optional producer channel | At most 16 events and 1 MiB encoded bytes; immediate full error |
| WS reassembled message / incoming frame | 64 KiB each, UTF-8 and fragmentation validated |
| WS inbound / outbound queue | Each at most 16 messages and 1 MiB, reserved before enqueue |
| WS read / write target / maximum write buffer | 16 KiB / 16 KiB / 256 KiB |
| WS Ping / matching Pong / inbound idle | 20 s / 10 s / 60 s |
| WS send wait / close | 10 s / 5 s, close also capped by remaining host grace |

Builders reject zero/unrepresentable bounds and unsafe dependency buffer headroom.
SSE requests can select smaller limits; effective policy intersects host ceilings.
Ordinary HTTP retains its existing deadlines. Real native writes reset SSE idle;
producer readiness and heartbeat scheduling alone do not count as progress.

`WsConnection` is owned and nonclonable. `send` waits for finite queue capacity;
`recv` yields complete text/binary messages, then one terminal error if applicable,
then fused EOF. Ping/Pong/Close processing continues even when the app does not call
`recv`. `close_info` reports peer metadata and close cleanliness. Closing is
independent of data queue capacity. Callback captures must be owned, Send and static.
Snapshots contain URL/method and only explicitly selected headers/textual session
values, with a total 16 KiB bound; Debug redacts values. A snapshot is not live auth.

Stop rejects new admissions, finishes only the current SSE event and then EOF,
and sends WS Close(1001). Every source, callback and IO owner remains under the
serving future; grace aborts and reaps remaining work. `realtime_stats` exposes
active slots and cumulative rejection, local byte, close and forced-close counts.
Sent/byte counts mean local handoff, never remote acknowledgement.

## Application and deployment responsibilities

Choose authentication, Origin, CSRF, session expiry/revocation and per-message
authorization explicitly. No guard is installed. Browsers can send cookies in a
cross-site WebSocket handshake: accepting it without checking Origin can permit
cross-site WebSocket hijacking. CORS does not protect WebSockets; ordinary GET
CSRF middleware may do nothing. `websocket::origin_allowed` offers bounded exact
scheme/host/port comparison with explicit missing/null policies.

Apps own SSE cursor meaning, replay retention, duplicate handling and subscriber
lifetime. EventSource cannot attach arbitrary Authorization headers. Cross-origin
credentialed SSE needs app CORS/cookie policy. Authorize sensitive messages when
sent/received, rather than trusting an old handshake indefinitely.

HTTPS uses the existing Rustls host transport. Proxies must forward Upgrade and
Connection for WebSockets, avoid buffering SSE, and permit the selected idle
intervals. No compression, HTTP/2 extended CONNECT, rooms, broadcast, durable replay,
automatic message dispatch or arbitrary socket access is provided.
