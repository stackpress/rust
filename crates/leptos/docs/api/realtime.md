# Browser realtime helpers

Enable `client-sse` or `client-websocket` independently of native server crates.
Inside a Leptos owner call `realtime::use_event_source(url, options)` or
`realtime::use_websocket(url, options)`. Each returns a fallible owned-lifetime
handle with `state()`, `latest()` and `error()` read signals. Native SSR stays
Disconnected and touches no Window. Effects connect after mount, so hydration
preserves server HTML and opens one connection per helper instance.

EventSource options select credentials and up to 32 named events. Latest data is
bounded to 64 KiB, event names to 256 bytes and last-event IDs to 1 KiB. Native
EventSource owns reconnect and Last-Event-ID. `close()` explicitly stops it.

WebSocket latest text/binary messages are at most 64 KiB. `send` checks the
browser's `bufferedAmount` against the configured ceiling (default 256 KiB) and
returns Backpressure immediately; it hides no Rust outbound queue. `reconnect()`
is explicit. Automatic reconnect is disabled by default; `with_reconnect` permits
at most five attempts at 1, 2, 4, 8 and 16 seconds. One retry timer exists at a time,
and no retry resends old messages. Explicit close allows browser codes 1000 or
3000–4999 and a reason of at most 123 UTF-8 bytes.

Owner disposal closes connections, removes exact listeners and cancels retry
timers. Handles used after disposal return Disposed. Changing URL means replacing
the helper under a new owner, disposing the previous owner first. Helpers retain
only the latest message; applications must bound any history they accumulate.
Browser/TLS/kernel buffers and browser parsing are outside that retention budget.

Applications own authentication, Origin/CORS checks, cookie policy, token expiry,
replay and deduplication. EventSource cannot set arbitrary Authorization headers.
WebSocket CORS is not an Origin guard. Use a deliberate server allowlist to prevent
cross-site WebSocket hijacking when cookies authorize the handshake.

The independent consumer's `realtime` feature supplies SSR, hydrate and CSR fixture
entries. Its finite Chromium proof mounts the actual helpers against a test-owned
native host; no development server is used by the proof.
