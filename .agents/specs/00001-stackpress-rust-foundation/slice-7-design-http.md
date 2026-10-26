# Slice 7 HTTP ownership and hosting contract

Proposed contract under [decisions](slice-7-design-decisions.md). Read before
implementing either the server extension or the separate network host.

## Crate boundaries and lifecycle

`stackpress-server` remains executor/listener-free by default. Its existing
`http` feature remains the buffered value adapter, with regression-compatible
normalization, header/cookie conversion, suppression and G12/G17 behavior.
An additive `http-stream` feature supplies protocol-neutral stream output and
reuses shared normalization/preflight code. `stackpress-server-http` owns
Tokio/Hyper connections, optional TLS, admission and shutdown; it depends on
server, never the reverse. No network edge enters lib, Idea or browser defaults.

Network handling has these boundaries:

1. Admit an owned connection before TLS/HTTP processing. Reject excess IO by
   closing it; do not allocate an unbounded queue or spawn before admission.
2. Hyper parses a bounded head. If it cannot produce HTTP request parts, no
   framework Request exists. This is before the existing native-value HTTP
   normalization boundary: allocate a connection/exchange ID and emit a
   transport rejection observation, no
   request/error/response callbacks. Let Hyper emit its safe parser error or close.
3. Validate head/URI and normalize through the shared server HTTP converter.
   Allocate host correlation/report inside that boundary. G17 failures return
   sanitized 400 with a host correlation/Input report, without a fabricated
   Request or callbacks. Never reclassify this normalized-adapter failure as an
   unreported connection error.
4. Read bounded body frames inside the normalized Input loader. Body timeout,
   parse failure or overrun follows G12's Input error offer. Request is loaded
   once. Trailer policy is reject request trailers initially; no discarded
   trailer fields can select a parser, auth policy or route.
5. Enter request → selected method/path operation → response once. Nested
   `Invocation::delegate` remains raw. Abort skips remaining hooks as today.
6. Inspect the final Response, preflight status/headers/cookies/body descriptor,
   consume one output authority, then publish a native body. No bytes/stream polls
   before the lifecycle succeeds or deliberately leaves Open output eligible.

`HandleOptions.adapter_context` carries trusted `HttpRequestContext`: peer
address, listener scheme, validated public authority and a cancellation view.
No raw socket, writer, password or request-derived event override is exposed.
The new HTTP entry must accept HandleOptions; current buffered `handle` defaults
remain unchanged. Proposed internal refactoring stays within G23 sequencing.

## Stream ownership and disposition

Proposed `HttpResponseExt::stream(mime, stream, limits)` installs a private
`HttpStreamBody` through the already explicit `Body::Value(Value::Shared(...))`
seam. The slot contains `Mutex<Option<OwnedStream>>`, where OwnedStream is an
owned, pinned `Stream<Item = Result<Vec<u8>>> + Send + 'static`. The mutex is
held only to transfer the Option, never during polling or user callbacks.
The wrapper is Sync because synchronization protects its Send payload. No unsafe
impl, borrowed request/Leptos owner, or cloneable stream handle is public.

- The stream belongs to the selected Response body. Replacing that Response or
  calling another body setter drops its association. An error creates a fresh
  response and cannot leak the previous success stream, headers or cookies.
- A cloned ResponseView explicitly shares only SharedNative identity; taking the
  stream twice fails. It cannot grant another output guard. Extra retained views
  may extend allocation lifetime, not duplicate stream ownership or polling.
- Header/body policy is checked before take. HEAD and 204/205/304 drop the stream
  without polling; omit unknown Content-Length and let Hyper own framing. HEAD
  does not implicitly dispatch GET. 205 uses zero length. No arbitrary trailers.
- Reject handler-supplied transfer-encoding, strip/recompute content-length,
  connection and nominated hop-by-hop fields. Repeated Set-Cookie stays separate.
- Existing buffered HTTP/CLI cannot serialize this private Shared body. They
  return their normal sanitized unsupported-output failure. They never collect
  the stream silently or expose its Debug/source to the caller.
- A non-stream SharedNative stays unsupported. Streaming must be installed with
  the typed extension, not inferred from any native value or MIME string.

## Publication versus network completion

The existing OutputGuard consumes authority before its callback and cannot be
reset by Response replacement. The HTTP publication callback moves the preflighted
native response/body into the connection owner, as the buffered adapter does
today. A framework Sent state records that completed adapter publication; it
does not prove bytes reached the client. Do not rewrite the foundation state
machine to pretend an asynchronously returned Hyper body is already drained.

The new host owns an `HttpExchangeReport { host: Option<HandleReport>, delivery }`.
Delivery states are rejected-before-host, suppressed, published, completed,
failed or cancelled, with byte count and local exchange ID. A supervised body
wrapper records EOF/error/drop; a connection completion can add native IO
failure. The supervisor emits one final exchange observation after draining or
drop. No detached sender task or unbounded completion channel is needed.
Failure before publication may use the existing sanitized output fallback only
while authority remains unused. Failure after publication closes output and
records diagnostics; it never retries, runs error hooks again or appends JSON
to a partial HTML document. EOF means handed to the transport, not remote ACK.

The final M0 design must prove report ownership through Hyper Body Drop and
connection cancellation. A String slot probe alone does not establish this.

## Finite proposed defaults

All values are trusted startup config, validated before bind. Zero is invalid
for positive duration/count controls, never an unlimited sentinel. An aggregate
deployment still needs an explicitly selected connection/memory budget.

| Policy | Proposed default / enforcement |
| --- | --- |
| Connections / handshakes | 256 total, 32 TLS handshakes; acquire before spawn; excess closed immediately. |
| HTTP/1 in-flight work | One active lifecycle/response per connection; no background request dispatch. Process pipelined input in order after prior output finishes. |
| Header/URI | 64 header occurrences, 32KiB parser buffer, 16KiB aggregate decoded header bytes, 8KiB target. Configure Hyper max buffer explicitly, then exact framework head checks. |
| Body | 1MiB actual decoded bytes and 2MiB aggregate normalized request. Reject known excessive length early; still count chunks. No collect-before-limit. |
| Deadlines | TLS 5s, complete request head 10s, body total 30s and idle progress 5s, lifecycle 30s, stream total 60s and write progress 10s. |
| Connection reuse | Keep-alive enabled; idle 15s, maximum 100 requests or 5min lifetime, then graceful close. Pipelining is supported sequentially, not disabled by `pipeline_flush(false)`. |
| Stream | Maximum chunk 64KiB, retained Hyper buffer configured 32KiB, total 64MiB; no application channel by default. If a bridge is needed: at most two charged chunks, fixed-size permits. |
| Shutdown | Stop accepts/admission, mark connections graceful, drain 30s, cancel overdue work, abort and reap owned tasks. Close database/services after HTTP tasks release leases. |

Hyper's buffer may overshoot by a single admitted chunk. The probe observed seven
64KiB chunks polled with its default buffer; explicitly configuring 8KiB reduced
that fixture to one. Budget retained bytes as configured buffer plus admitted
chunk and any explicit bridge slots, not the buffer setting alone. Kernel/TLS
buffers and user callback allocations are separate. Never promise a hard process
memory ceiling from this table.

## Cancellation, disconnect and draining

IO EOF/reset, Body drop, request/connection deadline and shutdown set an
idempotent cancellation signal and drop the active owned future/stream. A small
trusted context view lets handlers stop cooperative spawned work; the host never
silently spawns domain work. Async cancellation cannot preempt a blocking or
CPU-bound user poll. Tests for those cases require an external process watchdog.

HTTP/1 disconnect detection is bounded by IO polling and deadlines. While a
handler computes without writes, EOF detection is not guaranteed instantaneous.
Do not add a second raw socket reader racing Hyper. A client half-close is not
accepted as an unlimited extra lifetime. Local producer tasks, if used by SSR,
must be tracked, cancelled and joined under the same exchange owner.

Graceful stop is idempotent. Startup failure publishes no ready notification.
Accept errors stop admission and drain existing tasks before returning the
original typed error. Request/task panics are recorded as host failures; unwind
cleanup releases resources, without claiming process-abort recovery.

## TLS, proxy trust and assets

- HTTPS is opt-in via Rustls + tokio-rustls. Validate certificate/key before
  bind; HTTP/1.1 ALPN only initially. Bound handshake concurrency/time. No
  automatic issuance, live reload, mTLS authorization or HTTP/2 claim.
- Proxy mode supplies an explicit public origin. Default ignores Forwarded and
  X-Forwarded-* completely. Optional trusted-peer CIDR policy accepts one
  configured header family, validates the immediate peer and exact hop count,
  protocol and host allowlist. Reject ambiguous/duplicated chains; never pick
  arbitrary first/last values. Client IP is metadata, not authorization.
- Host/absolute target must match configured authority allowlist; reject a
  mismatching authority before route callbacks. Preserve path spelling.
- Static assets are registered ordinary exact routes from a startup-verified
  immutable manifest. Check regular files, component symlinks, bytes and hashes
  before publishing the bundle. Require an operator-owned immutable build tree;
  portable metadata-then-open is not safe against concurrent hostile writers.
- Proposed bundle caps: manifest 64KiB, 1024 entries, file 64MiB, total 128MiB.
  MIME explicit; nosniff; content hash ETag; immutable cache only for hashed
  paths. GET/HEAD plus conditional 304. No directory index, SPA fallback,
  ranges, compression or caller-derived filesystem open in this slice.
- Observers receive bounded IDs, status, timing, bytes, outcome and error kind.
  No bodies/cookies/auth headers/SQL/binds by default; no distributed ID adoption
  from arbitrary headers. Observer callbacks are synchronous/nonblocking and
  called outside locks. Observer panic follows normal unwind policy.
