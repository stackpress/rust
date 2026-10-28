# HTTP ownership and trust

The network boundary turns untrusted bytes into one application invocation,
then drives one response. Keep these three owners distinct: Hyper parses and
frames HTTP, Server runs the lifecycle, and this crate controls admission,
IO deadlines, cancellation, and final reporting.

## Startup and request work

Load verified assets and register their exact routes before ServerBuilder
builds the application. Build services once. Prepare NetworkOptions and bind
only after startup succeeds. Certificate files and all policy are checked
before the bind syscall; asset verification happens even earlier at bundle load.

A connection runs requests sequentially. A pipelined successor cannot start
its lifecycle until the preceding producer and native flush finish. Both
request count and connection age are finite. Parser buffering plus one
admitted producer chunk is bounded; user allocations, TLS, kernel buffers,
and aggregate process memory are separate budgets.

## Publication and completion

Response hooks finish before stream polling begins. The final response is
preflighted once, then the adapter moves one native value to the transport.
Framework Sent means that publication happened; it does not mean bytes reached
a client. HttpDelivery::Completed requires producer handoff and native flush,
which still does not establish a remote acknowledgement.

A producer, timeout, or IO failure after publication closes output and retains
a typed diagnostic. The host never retries output, repeats response/error
hooks, or appends an error envelope to stream bytes. A later malformed request
cannot retroactively fail a previously settled exchange. See
[server streaming](../../server/docs/guides/streaming-http.md).

## Framing and connection reuse

The accepted G31 policy uses Hyper's authoritative parse. There is no raw-head
preflight, parser fork, or second framing parser. Hyper can merge equal
Content-Length occurrences and remove a later Content-Length when it has
already seen Transfer-Encoding, so service-level headers cannot reconstruct
every original framing field.

| Parsed input | Boundary behavior |
| --- | --- |
| Equal duplicate Content-Length | Hyper accepts one length, as permitted by RFC 9110 §8.6. |
| Invalid or differing Content-Length | Hyper rejects; no route invocation. |
| Transfer-Encoding and Content-Length both still visible | Empty 400 and close before callbacks. |
| Transfer-Encoding after Hyper has removed Content-Length | Transfer-Encoding controls framing. |
| Any accepted Transfer-Encoding request, including ordinary chunked | Respond, then close; dispatch no pipelined successor. |
| Content-Length request eligible for keep-alive | Reuse sequentially within all finite bounds. |

Closing after every Transfer-Encoding response covers the CL+TE closure rule
in RFC 9112 §6.1 even when CL was removed. Chunked bodies work; their decoded
bytes are body data, never another head. Non-final chunked coding, HTTP/1.0
Transfer-Encoding, unsupported codings/trailers, bad chunks, folded headers,
and control characters retain parser/boundary rejection before routing.

The RFC rationale is [RFC 9110 §8.6](https://www.rfc-editor.org/rfc/rfc9110.html#section-8.6)
and [RFC 9112 §6.1](https://www.rfc-editor.org/rfc/rfc9112.html#section-6.1).
The [M1 duplex proofs](../tests/m1.rs) cover ordering, bodies, and closure.

## Transport and proxy metadata

Deployment constructors require one Host and an allowed literal authority.
A forwarded field never turns a plaintext connection into TLS. In
HttpRequestContext, `peer` is the actual TCP peer (caller-attested for supplied
IO), and `scheme` is the actual connection transport. `public_origin` and the
normalized request URL can describe a checked external proxy origin.
`forwarded_client` is informational, never authorization.

Forwarding fields are ignored for trusted metadata by default and from
untrusted peers. A [proxy policy](guides/proxy.md) must explicitly identify the
immediate peer, all intermediaries, header family, and exact hop count.

## Cancellation and shutdown

Dropping a connection, body, or handling future cancels its owned work and sets
the read-only cancellation view. Disconnect recognition during computation is
bounded by IO polling and the lifecycle deadline; a second raw reader does not
race Hyper. Application work must cooperate with async cancellation.

Stop prevents admission first and drops the source/listener/backlog before
connection drain. One shared grace interval covers the entire group, followed
by abort and reap. Await the [drain report](api/reports.md) before closing
process services. Dropping the serving future aborts owned tasks, but awaiting
stop also establishes that every task was reaped before return.

## Supported boundaries

HTTP/1 is the only protocol family; there is no HTTP/2, upgrade, WebSocket,
CONNECT tunnel, or request-trailer support. HTTPS supports explicit Rustls
TLS 1.2/1.3 with HTTP/1.1 ALPN. No mTLS, certificate issuance, discovery,
password interface for encrypted keys, or certificate reload is supplied.

Assets support exact ASCII allowlist routes, GET/HEAD and ETag validation;
there is no range response, compression, directory index, or SPA fallback.
Startup files must be operator-owned and unchanged while loading: portable
metadata/open checks do not prevent hostile concurrent replacement.

No runtime, signal policy, detached observer sender, automatic secret logging,
or process-abort recovery is created. Synchronous startup reads and a blocking
or CPU-bound user poll cannot be preempted by an async deadline. The current
native proofs and managed smoke do not establish every OS or browser/WASM
runtime; keep the host feature outside browser dependency graphs.
