# Stackpress HTTP host

`stackpress-server-http` drives HTTP/1.1 over caller-supplied IO and a generic
acceptor. Hyper is the sole HTTP parser. The host owns finite admission,
deadlines, sequential reuse, cancellation, graceful drain and final reporting.
`HttpServer::bind` explicitly creates one TCP listener after startup validation.
The library creates no runtime or process signal handler. Behavioral tests use
supplied duplex IO; the coordinator owns the separately managed listener smoke.

Create `HttpServer::new(Arc<Server>, HttpOptions)` after application startup.
`with_limits(NetworkLimits)` validates the native policy before sharing the
host. This legacy constructor preserves the existing adapter compatibility
policy. Production deployments use `NetworkOptions` with `from_options` or `bind`
to enable strict native authority checks and optional proxy trust. Existing buffered `HttpOptions` remain separate and retain their G12/G17
conversion behavior. `serve_io(io, peer, stop)` admits supplied IO using the
prepared deployment transport (plaintext for the legacy constructor) immediately and owns its lifetime without spawning a task. `serve_tls_io` also
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
file loading is explicit and bounded. Public test-only fixtures must never be
used for deployment. All automated behavioral tests use duplex IO, never sockets.


## Deployment API

`NetworkOptions::http(address, public_origin)` deliberately selects plaintext;
`https(address, public_origin, TlsFiles::pem(cert, key))` requires the `tls`
feature. Edit the public startup fields or use `with_proxy`/`with_observer`, then
call `HttpServer::from_options(app, options)` for supplied IO or
`HttpServer::bind(app, options).await` for TCP. Validation precedes bind. Missing,
invalid or mismatched TLS files fail startup; HTTPS never falls back to HTTP.
PEM chains and one private key, or explicit DER, are supported. Certificate and
key files must be regular with no symlink components, capped at 64KiB/16KiB.
They must remain operator-owned and unchanged during preparation, as with asset
startup reads; portable metadata/open is not atomic against hostile replacement.
No certificate discovery, issuance, reload, mTLS or HTTP/2 is implied.

`BoundHttpServer::serve_until(stop).await` delegates to the same
`Arc<HttpServer>::serve_acceptor_until(acceptor, stop)` path used by duplex
proofs. Stop is a caller-owned `Future<Output = ()>`. Source/listener/backlog drops
before connection drain. The returned `DrainReport::failure` retains a fatal
accept error **after** cleanup. Await it before closing database/process services.
An acceptor is trusted and cancellation-safe; its `AcceptedIo` explicitly selects
plain or TLS transport. No forwarded field can select a transport.

Deployment authority checks require exactly one Host value. Host and any absolute
target must match the configured authority allowlist and each other. DNS case,
default ports and equivalent IPv6 spellings compare canonically; IPv4 and IPv6
CIDRs never map implicitly. The public origin's authority is included by default;
`allowed_authorities` adds literal authorities, never wildcard domains.

## Explicit proxy trust

Default and untrusted peers cannot influence trusted metadata through forwarding
fields. With `ProxyPolicy`, the immediate peer and every declared intermediary
must match `peers`; `hops` is exact (1–32). The original client IP is informational
`HttpRequestContext::forwarded_client`, never authorization. `scheme` remains the
actual listener transport; the checked public origin and request URL describe
the external origin. The native path/query are preserved.

Choose one `ProxyHeaders` family. Trusted peers must send one complete chain:

- `Forwarded`: one header, each hop has `for`, `proto`, `host` once. Quoted
  bracketed IPv6 with optional port works. Extensions, escapes, partial hops,
  unknown/obfuscated identities and duplicate parameters are rejected.
- `XForwarded`: one X-Forwarded-For, -Proto and -Host line, each with exactly
  `hops` comma-separated values. Bare IPv6 addresses are supported in For.

Mixed families, duplicate lines, wrong hop count, untrusted intermediaries,
disagreeing origins, protocols other than HTTP(S), unlisted host/port and
absolute-target mismatches return empty 400 and close before callbacks. This is
an intentionally strict forwarding subset; configure the proxy to emit it.

## Immutable assets

Load `AssetBundle::load(root, manifest, AssetLimits)` and call
`bundle.register(&mut builder)` **before** `builder.build()`. This keeps GET/HEAD
as ordinary exact routes, including request/error/response hooks and cookies.
The already-built server cannot accept new route registration during bind.
Startup validates the entire bundle before any listener is published.

The strict JSON root is `{ "entries": [...] }`; each entry requires `route`,
`file`, `size`, `sha256`, `mime`, `cache`. Unknown/duplicate fields fail parsing.
See the checked [example manifest](examples/assets/manifest.json). Route/file
components are literal ASCII alphanumeric, `-`, `_`, `.`; no empty/dot/parent
segments, percent escapes, Unicode, patterns, backslash, query or fragment.
File names are relative to the operator-owned immutable build tree. No symlink
component, FIFO, directory or device is opened as an asset. Portable
metadata-then-open is not safe against concurrent hostile writers; exclusive
operator ownership during loading is required.

Defaults cap serialized manifest bytes at 64KiB, entries at 1024, one file at
64MiB and retained bytes at 128MiB. Size and SHA-256 are verified before retaining
each representation. MIME is explicitly allowlisted and sent with nosniff.
Bytes are preloaded; filesystem mutation/removal after startup cannot change
responses. Requests never choose a filesystem path. Unknown routes remain 404;
there is no directory index, SPA fallback, range or compression feature.

GET and HEAD expose the verified length and strong hash ETag; weak/strong
If-None-Match or `*` can select bodyless 304. Revalidate cache is public with
max-age=0/must-revalidate; immutable cache requires the full hash in the URL
basename and permits one year. Application hooks retain normal response authority.
`HttpResponseExt::immutable_bytes(mime, Arc<[u8]>, StreamLimits)` publishes this
known immutable representation once, copying at most one bounded 64KiB chunk
on demand. HEAD/bodyless output takes/drops without polling or byte copying.
The buffered value adapter continues to reject native descriptors.

## Observations and managed smoke

`with_observer(Arc<dyn ExchangeObserver>)` receives final fixed-size
`ExchangeObservation` values: checked local IDs, status, elapsed, byte count,
delivery, error kind and host-report presence. No path, headers, cookies, body,
client IP, SQL or native source message enters this type. No observation queue
or detached sender is created. Snapshots are emitted when connection ownership
ends, bounded by request/admission limits, outside locks and after IO/task-local
resources drop. A synchronous callback must not block; panic unwinds and is not
retried. A task poll panic emits redacted failure metadata after dropping IO,
then resumes its original panic payload for normal supervisor reporting. Drop
that is already unwinding skips callbacks to avoid a second panic.

The [deployment example](examples/deployment.rs) takes `--port`, provides
`/health`, `/assets/hello.txt`, `/stream`, and drops application services after
drain. It is compile-checked here. Only the coordinator runs the smoke:

```sh
devmetrics start --wait 120 --summary "Stackpress Slice 7 HTTP M2 smoke" -- 'CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 cargo run --locked -p stackpress-server-http --example deployment -- --port {port}'
```

Run from this worktree. Devmetrics selects/registers the port and owns expiry;
use its reported URL. No automated proof in this milestone binds any socket.
