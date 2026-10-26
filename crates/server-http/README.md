# Stackpress HTTP host: M1

`stackpress-server-http` drives HTTP/1.1 over caller-supplied IO and a generic
acceptor. Hyper is the sole HTTP parser. The host owns finite admission,
deadlines, sequential reuse, cancellation, graceful drain and final reporting.
No TCP listener, process signal handler or runtime is created. Binding, proxy
policy, assets, observers and application deployment remain M2.

Create `HttpServer::new(Arc<Server>, HttpOptions)` after application startup.
`with_limits(NetworkLimits)` validates the native policy before sharing the
host. Existing buffered `HttpOptions` remain separate and retain their G12/G17
conversion behavior. `serve_io(io, peer, stop)` admits supplied plaintext IO
immediately and owns its lifetime without spawning a task. `serve_tls_io` also
acquires a handshake permit before any TLS work. Excess IO closes promptly;
its report records 503 without fabricating HTTP bytes on an unparsed/TLS peer.

`Arc<HttpServer>::serve_acceptor(acceptor, Shutdown)` owns admission and connection
tasks over any `Acceptor`. The source supplies `AcceptedIo::plain` or, with TLS,
`AcceptedIo::tls`. It owns its backlog and must be cancellation-safe: dropping
an `accept` future must not detach work. The host creates no unbounded queue.
It acquires connection/handshake permits before spawning, with the same mutex
linearizing `Shutdown::stop()` and the last permitted spawn. Stop is sticky and
idempotent; a ready stop wins over a ready accept. Source end/error also drains.

All owned connections begin graceful shutdown together. One absolute grace
covers the entire group, then overdue tasks are aborted and every handle reaped.
Await the returned `DrainReport` before closing database/process services.
Dropping the supervisor aborts its owned tasks via JoinSet; awaiting shutdown
additionally guarantees reaping before return. A blocking or CPU-bound user
poll cannot be preempted by async cancellation; tests have process watchdogs.

`DrainReport` counts all admitted/rejected/completed/aborted tasks. Its recent
connection reports are bounded by `max_connections`, with a discarded-history
counter. Each connection retains at most its bounded request count. Native
causes remain typed under explicit diagnostic fields; default report Debug
redacts them. The library does not automatically log request payloads or secrets.

Native defaults are 256 connections, 32 TLS handshakes, 64 header occurrences,
32KiB Hyper buffer, 16KiB decoded header bytes, 8KiB target, 1MiB decoded body,
2MiB aggregate normalized input and 100 requests per connection. All counts,
byte ceilings and durations must be positive/representable; zero is never an
unlimited sentinel. Hyper requires a parser buffer of at least 8192 bytes.

Head parsing has a 10s absolute and 5s progress budget. A timer supplied to Hyper
also enforces these at Hyper's own parsing boundaries, including partial heads
already in its read-ahead buffer; it never inspects or reparses raw bytes.
Body loading has 30s total and 5s decoded-progress budgets inside Input. Handler
lifecycle work has a separate absolute 30s budget. Output has 60s total and
10s actual-write progress budgets, further tightened by a stream's own limits.
Quiet keep-alive expires after 15s; connection age expires after 5min and starts
bounded drain. One-byte/tiny-chunk progress never resets an absolute deadline.

Head count/byte/target rejections use 431. Known and actual body/aggregate
excess uses 413; body timeouts use 408 through the once-only G12 Input error
offer. Malformed normalization retains G17's sanitized 400 and host Input report
without callbacks. Lifecycle deadlines cancel unfinished work and may publish
one empty 408 before output. Producer/write failures after publication close
output and retain diagnostics, never retry or append JSON. Head timeouts emit
408 when this host still owns an unpublished writer; Hyper may instead close
at its own parser timeout. Admission before parsing closes IO with report-only
503. Native parser failures retain Hyper's safe error/close behavior.

Persistent requests run sequentially, including pipelined requests. The next
lifecycle waits for the previous producer and native flush. HEAD/204/205/304
still drop producers without polling. `HttpResponseExt::stream` accepts only an
owned `Send + 'static` producer. Byte/chunk limits are enforced before handoff;
its total/idle time controls are now enforced by the connection supervisor.
Actual IO progress, not producer polling, resets the output progress budget.
Hyper may retain its configured buffer plus one admitted producer chunk; this
is not a whole-process memory ceiling or a bound on TLS/kernel/user allocations.

The accepted framing policy uses Hyper's normalized interpretation:

- For CL-framed requests, equal duplicate Content-Length fields become one
  length; differing/invalid lengths are rejected by Hyper. This follows
  [RFC 9110 section 8.6](https://www.rfc-editor.org/rfc/rfc9110.html#section-8.6).
- When both CL and TE remain visible, the host rejects with 400 and closes before
  callbacks. When Hyper has dropped CL because TE appeared first, TE wins.
- **Every Transfer-Encoding request closes after its response**, including a
  valid chunked request. No pipelined successor on that IO is dispatched. This
  implements the closure requirement in [RFC 9112 section 6.1](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.1)
  even when the original CL is no longer visible. Body bytes remain opaque data.
- Unsupported transfer codings, trailers, upgrades and CONNECT tunnels are
  rejected. Hyper retains its invalid chunk/fold/control/header rejections.

There is no direct httparse dependency, raw-head preflight, parser fork or
first-party body-framing state machine. Service checks alone could not reject
all original duplicate/CL+TE fields because Hyper normalizes or removes them.

Framework Sent means publication; Completed means local producer handoff and
native flush, never remote acknowledgement. A later request failure does not
retroactively fail a previously settled exchange. Dropping IO/body/handling
sets trusted cancellation metadata; disconnect detection during a computation
is bounded by IO polling and lifecycle deadlines rather than a second reader.

The facade preserves buffered-only `server-http`. Separate `server-http-host`
and `server-http-host-tls` expose this crate as `stackpress::server_http_host`.
Default and browser graphs gain no network edge. TLS uses explicit ring,
TLS 1.2/1.3 and HTTP/1.1 ALPN, with finite handshake admission/time; certificate
file loading and deployment policy remain M2. Public test-only fixtures must
never be used for deployment. All tests use duplex IO, never listening sockets.
