# Stackpress HTTP host: M0

`stackpress-server-http` drives an HTTP/1.1 connection over caller-supplied
`AsyncRead + AsyncWrite` IO. It owns no listener, runtime, process signals or
background tasks. This milestone establishes the stream and report ownership
boundary; it is not the completed production server.

Use `HttpServer::new(Arc<Server>, HttpOptions)` after application startup,
then await `serve_io(io, peer, stop)`. Stop initiates a graceful drain; expiry
of the positive shutdown grace drops the connection and all active work before
returning `ConnectionReport`. Dropping that future also drops active work.
CPU-bound or blocking user polls cannot be preempted by an async timer.

A handler imports `stackpress_server::http::stream::HttpResponseExt` and
calls `response.stream(mime, producer, StreamLimits)`. The producer must be
owned, `Send + 'static` and yield `Result<Vec<u8>>`. It is taken once from the
final response only after lifecycle hooks and HTTP preflight finish. Response
views share allocation identity, never another producer or output permit.
HEAD and 204/205/304 discard the producer without polling it. Existing buffered
adapters reject this native stream value instead of collecting it silently.

Framework `Sent` means native response publication. A final exchange report
separately records producer EOF, error, suppression or cancellation, admitted
bytes and original typed diagnostic causes. `Completed` means local EOF handed
to Hyper, not remote acknowledgement. A native IO error may downgrade that
handoff to `Failed`. No response retry or error-envelope append follows a
producer failure. Reports are returned after Hyper/body/IO owners are dropped.
No payload, cookie or authorization header is automatically logged.

Enable `tls` for `TlsConfig::new(DER chain, matching key)` and
`serve_tls_io`. TLS uses an explicit ring provider, TLS 1.2/1.3 and HTTP/1.1 ALPN,
with a fixed five-second handshake timeout. It never falls back to plaintext.
Certificate file loading, handshake admission and deployment policy are later
milestones. The duplex tests verify a valid connection and hostname mismatch.

The facade preserves its existing buffered-only `server-http` feature.
`server-http-host` separately exposes this crate as `stackpress::server_http_host`;
`server-http-host-tls` also selects TLS. Default and browser graphs do not select
these features.

M0 deliberately serves one request per connection. Fixed parser controls and
incremental body counting are present, but full body/lifecycle/stream/write
and reuse deadlines, admission, status matrix, proxy/authority policy,
immutable assets, observers and listener/app integration remain M1/M2.
`StreamLimits` byte/chunk limits are enforced now; its time controls are retained
for the IO-host deadline work in M1 and are **not yet enforced by this host**.
No production readiness, HTTP/2, browser hosting or deployment claim is made.

The watchdog test launches its own ignored child scenario explicitly, kills and
reaps it on timeout, and treats timeout as failure. All IO is Tokio duplex;
no socket is bound. Test certificates and their generation recipe live under
`tests/fixtures` and must never be used for deployment.
