# Trust an explicit reverse proxy

Behind a proxy, the immediate socket peer and the external client are different
identities. Configure a narrow peer allowlist and exact forwarding chain before
using external origin metadata. By default forwarding fields cannot influence
trusted metadata, including when they come from an untrusted peer.

## Select one family and exact hop count

This example expects one Forwarded entry from a loopback proxy. It prepares
policy only; it does not bind a listener:

```rust
use stackpress_server_http::{NetworkOptions, ProxyHeaders, ProxyPolicy};

/// Declare the proxy that may describe the external HTTPS origin.
fn options() -> Result<NetworkOptions, Box<dyn std::error::Error>> {
    let proxy = ProxyPolicy {
        peers: vec!["127.0.0.1/32".parse()?, "::1/128".parse()?],
        hops: 1,
        headers: ProxyHeaders::Forwarded,
    };
    Ok(
        NetworkOptions::http("127.0.0.1:8080".parse()?, "https://profiles.example/")
            .with_proxy(proxy),
    )
}
```

`http` deliberately selects a plaintext connection from the proxy; the public
origin remains HTTPS. For this policy the proxy must emit exactly one Forwarded
line containing `for=192.0.2.10;proto=https;host=profiles.example`. The first
address is the informational client. The native Host must also pass deployment
authority checks; configure the proxy's origin request accordingly.

## Validate every declared hop

`hops` counts declared entries including the original client, from 1 to 32.
The actual immediate peer and every declared intermediary after that client
must match one of 1–128 explicit CIDRs. IPv4 and IPv6 membership are separate;
IPv4-mapped IPv6 does not inherit IPv4 trust. Prefix zero is supported, so use
it only deliberately. `forwarded_client` never grants authorization.

Forwarded requires exactly `for`, `proto`, and `host` once per entry. Quoted
bracketed IPv6 with optional port is supported. Extensions (including `by`),
escapes, unknown/obfuscated identities, partial entries, and duplicate parameters
are rejected for trusted chains.

With `ProxyHeaders::XForwarded`, require one line each of X-Forwarded-For,
X-Forwarded-Proto and X-Forwarded-Host. Every comma list must have the exact hop
count; For supports bare IPv6. Configure the proxy to emit one chosen family.
Mixed families, duplicate lines, extra/missing hops, untrusted intermediaries,
inconsistent origins, unsupported protocols, and authority mismatch produce
empty 400 and close before callbacks.

## Separate authority from transport

Deployment policy requires exactly one Host. The configured public-origin
authority is allowed automatically; `allowed_authorities` adds literal
host/port values, without wildcards. Host and an absolute request target must
match both the allowlist and each other. Checked forwarding origins must agree
with this policy. DNS case, default ports, and equivalent IPv6 spelling compare
canonically; only HTTP(S) schemes are supported.

Read HttpRequestContext via ServerInvocationExt::adapter_context. `peer` and
`scheme` describe the actual transport; `public_origin` and normalized request
URL can reflect the validated external origin. Native path/query still select
the route. See [proxy reference](../api/proxy.md) and
[metadata reference](../api/host.md#request-context).
