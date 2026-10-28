# Proxy policy API

ProxyPolicy is Clone/Debug and is ignored unless explicitly placed in
NetworkOptions. See the [proxy guide](../guides/proxy.md) for a complete
configuration. Source: [proxy.rs](../../src/proxy.rs).

## Types

| Surface | Contract |
| --- | --- |
| ProxyPolicy::peers: Vec<IpCidr> | 1–128 CIDRs permitting the immediate peer and every declared intermediary. |
| ProxyPolicy::hops: usize | Exact chain entries, including original client; 1–32. |
| ProxyPolicy::headers: ProxyHeaders | One selected family; Forwarded or XForwarded. |
| IpCidr: FromStr<Err = Error> | Requires address/prefix, masks host bits, validates IPv4 0–32 or IPv6 0–128. |
| IpCidr::contains(&self, IpAddr) -> bool | Concrete-family membership; mapped IPv6 remains IPv6. |

Construction/edits perform no IO; host preparation validates the policy.
Untrusted peers and absent policy cannot change trusted metadata via forwarding
fields. For trusted peers, one complete strict family is required. Bad chains
return empty 400 and close before callbacks. Protocol/host/for checks never
select a connection transport or authorize the original client.

## Authority and chain rules

Public origin plus additional allowed_authorities form a bounded literal
allowlist, without wildcard domains. Deployment requests require exactly one
Host, consistent with any absolute-target authority. Canonical DNS case,
default ports and IPv6 spelling compare; malformed ports, credentials, escapes,
and mismatches reject.

Forwarded requires one header with for/proto/host once per entry. XForwarded
requires one line each of For, Proto, Host and equal exact list counts. Supported
syntax and rejected duplicates, mixed families, extensions, obfuscated values,
and intermediary checks are detailed in the [guide](../guides/proxy.md).
HttpRequestContext retains actual peer/transport separately from the checked
external origin and informational client. See [request context](host.md#request-context).
