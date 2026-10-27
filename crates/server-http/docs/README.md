# Stackpress HTTP host documentation

Expose an existing Stackpress application's routes without changing its
request/operation/response/error lifecycle. This crate adds the HTTP/1 network
boundary, optional HTTPS, verified asset routes, and final delivery reports.

## Start here

[Getting started](getting-started.md) builds a complete application main and
explains where to close process services. [Concepts](concepts.md) explains
publication, framing, trust, and cancellation after that first example.

All Rust fences are compile-checked in a disposable consumer. Listener examples
are compiled without execution; repository development uses devmetrics.

## Common tasks

Choose a guide for the deployment behavior you need. Each links to its public
API reference and the relevant limits.

| I want to… | Read |
| --- | --- |
| Serve HTTPS from certificate/key files | [TLS files](guides/tls.md) |
| Trust one explicit reverse-proxy chain | [Proxy policy](guides/proxy.md) |
| Serve preloaded, hash-verified static files | [Asset bundles](guides/assets.md) |
| Collect redacted final observations | [Observers](guides/observers.md) |
| Stop admission and await all connection cleanup | [Shutdown and drain](guides/shutdown.md) |
| Run the example on a managed development port | [Development recipe](guides/development.md) |
| Attach an owned response producer | [Server streaming guide](../../server/docs/guides/streaming-http.md) |

## Returning to the API

The [API index](api/README.md) covers constructors, options, IO seams, reports,
and defaults. [Server docs](../../server/docs/README.md) own application startup
and lifecycle; [Lib docs](../../lib/docs/README.md) own Request and Response.

The facade's `server-http-host` and `server-http-host-tls` features expose
`stackpress::server_http_host`. The older `server-http` remains buffered-only.
Source authority is [the crate](../src/lib.rs) and its
[duplex proofs](../tests/); no HTTP/2, mTLS, certificate reload, or implicit
forwarding trust is provided.
