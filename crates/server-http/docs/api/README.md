# HTTP host API reference

Use these pages to look up the public network surface. Start with the
[walkthrough](../getting-started.md) for a complete main; application lifecycle
and response contracts live in [Server](../../../server/docs/api/README.md).

| Surface | Reference |
| --- | --- |
| NetworkOptions, HttpServer, BoundHttpServer, supplied IO and Acceptor | [Host and supervision](host.md) |
| NetworkLimits, deadlines, status behavior | [Limits](limits.md) |
| TlsFiles, TlsConfig and file formats | [TLS](tls.md) |
| ProxyPolicy, ProxyHeaders, IpCidr | [Proxy](proxy.md) |
| AssetBundle, AssetEntry, AssetLimits, AssetCache | [Assets](assets.md) |
| Connection/exchange/drain reports and ExchangeObserver | [Reports and observation](reports.md) |

Imports use `stackpress_server_http`. Optional facade features are
`server-http-host` and `server-http-host-tls`, exposing
`stackpress::server_http_host`. The crate's own `tls` feature defaults off.
