# Host and supervision API

NetworkOptions selects production deployment policy; HttpServer prepares it;
BoundHttpServer owns the explicit TCP listener. Source:
[deployment](../../src/deployment.rs), [host](../../src/lib.rs), and
[supervisor](../../src/supervisor.rs).

## NetworkOptions

Construct with `NetworkOptions::http(address, public_origin)` for deliberate
plaintext, or `NetworkOptions::https(address, public_origin, files)` with the
`tls` feature. Construction and public field edits perform no IO. Both return
Self; validation occurs in from_options/bind.

| Public field | Meaning and default |
| --- | --- |
| address: SocketAddr | Explicit bind address; supplied-IO seams do not bind it. |
| http: HttpOptions | Trusted origin plus buffered conversion/ParseLimits policy. Constructor sets base_url; remaining fields use HttpOptions defaults. |
| limits: NetworkLimits | Finite native defaults, separate from buffered policy. |
| allowed_authorities: Vec<String> | Additional literal authorities; empty by default. Public-origin authority is always allowed. |
| proxy: Option<ProxyPolicy> | None by default; forwarding cannot change trusted metadata. |
| observer: Option<Arc<dyn ExchangeObserver>> | None by default; no logging/sender task. |
| tls: Option<TlsFiles> | Available only with tls; None for http, Some(files) for https. |

`with_proxy(policy)` and `with_observer(observer)` consume Self and return Self
for chaining. See [proxy](proxy.md), [TLS](tls.md), and [limits](limits.md).
Public origin must be an absolute HTTP(S) root origin without credentials,
query or fragment. Literal authority checks apply to Host and absolute targets.

## Prepare or bind

The constructors share the same built `Arc<stackpress_server::Server>`. Asset
routes must already be loaded and registered before that Server builds.

| Operation | Inputs | Returns and effects |
| --- | --- | --- |
| HttpServer::from_options | Arc<Server>, NetworkOptions | Result<HttpServer>; validates deployment policy and synchronously reads bounded TLS files, without binding. |
| HttpServer::bind (async) | Arc<Server>, NetworkOptions | Result<BoundHttpServer>; performs the same validation before one TCP bind; does not start serving. |
| BoundHttpServer::local_addr | &self | Result<SocketAddr>; actual bound address, retaining native query errors. |
| BoundHttpServer::serve_until (async) | self, Send future returning () | DrainReport after stop/source failure and complete cleanup; consumes the bound listener. |

Configuration, file and bind errors preserve contextual typed causes. No
insecure fallback is attempted. The [production main](../getting-started.md)
shows the full sequence and explicit service-close point.

## Supplied IO and compatibility

`HttpServer::new(app, HttpOptions) -> Result<Self>` preserves the earlier
buffered adapter compatibility policy. It has finite native defaults but does
not activate NetworkOptions' strict deployment authority/proxy checks. Prefer
from_options/bind for deployments.

`with_limits(NetworkLimits)` and `with_shutdown_grace(Duration)` consume a host
and return Result<Self> after validation. `limits()` borrows its NetworkLimits;
`active()` returns `(connections, handshakes)` currently admitted.

`serve_io(&self, io, peer, stop).await -> ConnectionReport` admits and owns one
IO without spawning a host task. IO must implement Tokio AsyncRead + AsyncWrite
+ Unpin + Send + 'static; stop is a Send future returning unit. Prepared HTTPS
policy applies its selected TLS; legacy new is plaintext. Optional
`serve_tls_io(&self, io, peer, TlsConfig, stop)` explicitly selects TLS and shares
the same admission bounds. Supplying IO attests its peer; it does not
authenticate that address. Drop cancels owned work.

## Generic acceptor

Acceptor is Send with an associated owned Io satisfying the same IO bounds.
`accept(&mut self)` returns a Send future of
`Result<Option<AcceptedIo<Self::Io>>>`: Some for an IO, None for source end,
Err for a fatal source error. The accept future must be cancellation-safe and
must not detach work. Backlog bounds remain the source owner's responsibility.

Construct `AcceptedIo::plain(io, peer)` or optional
`AcceptedIo::tls(io, peer, config)` to select transport explicitly.
`Arc<HttpServer>::serve_acceptor(acceptor, Shutdown).await` and
`serve_acceptor_until(acceptor, stop).await` both return DrainReport.
They acquire permits before spawn/handshake, stop admission before drain, and
abort/reap overdue tasks. The thin TCP adapter uses this same supervision path.

Shutdown is Clone/Default. `new()` constructs one; `stop()` is sticky and
idempotent; `is_stopped()` reads state; `cancelled().await` waits for stop.
Stop and last admission share one boundary. See [shutdown guide](../guides/shutdown.md).

## Request context

`inv.adapter_context::<HttpRequestContext>()` through ServerInvocationExt
returns optional trusted adapter metadata; direct in-process calls need not
carry it. HttpRequestContext is Clone/Debug:

| Field | Meaning |
| --- | --- |
| peer: SocketAddr | Immediate TCP peer, or caller-attested supplied-IO peer. |
| scheme: &'static str | Actual connection transport; HTTPS only after selected TLS handshake. |
| public_origin: String | Configured or validated proxy origin. |
| cancellation: Cancellation | Cloneable read-only cooperative view; is_cancelled() reads it. |
| forwarded_client: Option<IpAddr> | Checked chain's original client; informational, never authorization. |

A forwarding claim cannot select connection transport. IO/body/future drop
and deadlines set cancellation; computation-only disconnect detection waits
for IO polling or lifecycle expiry. See [concepts](../concepts.md).
