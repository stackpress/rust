//! Explicit immediate-peer and exact-hop trust over already parsed native headers.
//!
//! Forwarded parsing is a restricted policy grammar, never another HTTP/framing parser.
//! Untrusted peers and the default policy ignore forwarding headers entirely.
use crate::external;
use http::{HeaderMap, Uri, uri::Authority};
use stackpress_lib::{Error, Result};
use std::{
    net::{IpAddr, SocketAddr},
    str::FromStr,
};

/// One concrete IP network; IPv4 and IPv6 never implicitly match each other.
#[derive(Clone, Debug)]
pub struct IpCidr {
    /// Masked network in one concrete address family.
    network: IpAddr,
    /// Validated width (0..=32 or 0..=128); masking never shifts by the full width.
    prefix: u8,
}
impl FromStr for IpCidr {
    type Err = Error;
    /// Parse an explicit CIDR, masking host bits; mapped IPv6 remains IPv6.
    fn from_str(value: &str) -> Result<Self> {
        let (ip, prefix) = value
            .split_once('/')
            .ok_or_else(|| Error::invalid("proxy CIDR requires prefix"))?;
        let ip: IpAddr = ip
            .parse()
            .map_err(|cause| external("proxy CIDR address invalid", cause))?;
        let prefix: u8 = prefix
            .parse()
            .map_err(|cause| external("proxy CIDR prefix invalid", cause))?;
        if prefix > if ip.is_ipv4() { 32 } else { 128 } {
            return Err(Error::invalid("proxy CIDR prefix out of range"));
        }
        let network = match ip {
            IpAddr::V4(ip) => IpAddr::V4((u32::from(ip) & mask4(prefix)).into()),
            IpAddr::V6(ip) => IpAddr::V6((u128::from(ip) & mask6(prefix)).into()),
        };
        Ok(Self { network, prefix })
    }
}
impl IpCidr {
    /// Test concrete-family membership; /0 is supported deliberately.
    pub fn contains(&self, address: IpAddr) -> bool {
        match (self.network, address) {
            (IpAddr::V4(net), IpAddr::V4(ip)) => {
                u32::from(net) == u32::from(ip) & mask4(self.prefix)
            }
            (IpAddr::V6(net), IpAddr::V6(ip)) => {
                u128::from(net) == u128::from(ip) & mask6(self.prefix)
            }
            _ => false,
        }
    }
}
/// Avoid a full-width shift for an intentionally trusted /0.
fn mask4(prefix: u8) -> u32 {
    if prefix == 0 {
        0
    } else {
        u32::MAX << (32 - prefix)
    }
}
/// IPv6 CIDR masking stays in its own address family.
fn mask6(prefix: u8) -> u128 {
    if prefix == 0 {
        0
    } else {
        u128::MAX << (128 - prefix)
    }
}
/// Exactly one configured forwarding family may influence trusted metadata.
#[derive(Clone, Copy, Debug)]
pub enum ProxyHeaders {
    /// Each comma-separated hop must contain for, proto and host exactly once.
    Forwarded,
    /// X-Forwarded-For/Proto/Host must each occur once with exactly the configured hop count.
    XForwarded,
}
/// Trusted startup configuration, not inferred from a remote header or address range.
#[derive(Clone, Debug)]
pub struct ProxyPolicy {
    /// Immediate peer and every declared intermediary must be in this explicit allowlist.
    pub peers: Vec<IpCidr>,
    /// Exact number of declared hops, including the original client, from 1 through 32.
    pub hops: usize,
    /// Mixed families, duplicate lines/parameters and unsupported obfuscated identities fail closed.
    pub headers: ProxyHeaders,
}
impl ProxyPolicy {
    /// Check finite chain and peer policy before bind.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.peers.is_empty() || self.peers.len() > 128 || !(1..=32).contains(&self.hops) {
            return Err(Error::invalid("proxy trust policy invalid"));
        }
        Ok(())
    }
    /// Family-strict membership prevents IPv4-mapped values acquiring IPv4 trust accidentally.
    fn trusts(&self, ip: IpAddr) -> bool {
        self.peers.iter().any(|net| net.contains(ip))
    }
}
/// Canonical authority has a concrete port, lowercased hostname and no user information.
#[derive(Clone, Debug, PartialEq, Eq)]
struct AuthorityKey {
    host: String,
    port: u16,
}
/// Validated deployment policy; legacy HttpServer::new retains its buffered compatibility behavior.
pub(crate) struct RequestPolicy {
    /// Canonical fallback origin; untrusted forwarding fields cannot replace it.
    origin: String,
    /// Default port interpretation for origin-form Host and configured extra authorities.
    scheme: String,
    /// Literal canonical identities, bounded at startup; no wildcard domain policy.
    authorities: Vec<AuthorityKey>,
    /// None leaves all forwarding claims informational request data only.
    proxy: Option<ProxyPolicy>,
}
/// Validated per-request metadata; client IP is informational, never authorization.
pub(crate) struct EffectiveRequest {
    /// Checked origin used only as informational URL metadata; native path stays authoritative.
    pub(crate) origin: String,
    /// First declared client after exact-hop/intermediary checks; never an authorization input.
    pub(crate) client: Option<IpAddr>,
}
impl RequestPolicy {
    /// Validate the public origin using the shared HTTP adapter, and bound authority configuration.
    pub(crate) fn new(
        origin: &str,
        allowed: &[String],
        proxy: Option<ProxyPolicy>,
    ) -> Result<Self> {
        let options = stackpress_server::http::HttpOptions {
            base_url: origin.to_owned(),
            ..Default::default()
        };
        stackpress_server::http::HttpAdapter::new(options)?;
        let url = stackpress_lib::Request::builder().url(origin).build()?.url;
        let scheme = url.scheme().to_owned();
        let origin = url.as_str().to_owned();
        let authority = http::Uri::from_str(&origin)
            .map_err(|cause| external("public origin URI invalid", cause))?
            .authority()
            .map(ToString::to_string)
            .ok_or_else(|| Error::invalid("public authority absent"))?;
        if allowed.len() > 128 {
            return Err(Error::limit("authority allowlist too large"));
        }
        let mut authorities = vec![authority_key(&authority, &scheme)?];
        for value in allowed {
            let key = authority_key(value, &scheme)?;
            if !authorities.contains(&key) {
                authorities.push(key);
            }
        }
        if let Some(policy) = &proxy {
            policy.validate()?;
        }
        Ok(Self {
            origin,
            scheme,
            authorities,
            proxy,
        })
    }
    /// Enforce native authority first, then optionally use one trusted complete forwarding chain.
    pub(crate) fn check(
        &self,
        uri: &Uri,
        headers: &HeaderMap,
        peer: SocketAddr,
    ) -> Result<EffectiveRequest> {
        let host = one(headers, "host")?;
        // Native authority is mandatory even for peers whose forwarding claims will be ignored.
        let native_scheme = uri.scheme_str().unwrap_or(&self.scheme);
        let host_key = authority_key(host, native_scheme)?;
        if !self.authorities.contains(&host_key) {
            return Err(Error::invalid("HTTP authority not allowed"));
        }
        if let Some(target) = uri.authority()
            && (!matches!(uri.scheme_str(), Some("http" | "https"))
                || authority_key(target.as_str(), native_scheme)? != host_key)
        {
            return Err(Error::invalid("HTTP target and Host authority mismatch"));
        }
        let Some(proxy) = &self.proxy else {
            return Ok(EffectiveRequest {
                origin: self.origin.clone(),
                client: None,
            });
        };
        // A spoofed family/chain from an untrusted peer cannot alter any trusted metadata.
        if !proxy.trusts(peer.ip()) {
            return Ok(EffectiveRequest {
                origin: self.origin.clone(),
                client: None,
            });
        }
        let chain = match proxy.headers {
            ProxyHeaders::Forwarded => {
                if headers
                    .keys()
                    .any(|key| key.as_str().starts_with("x-forwarded-"))
                {
                    return Err(Error::invalid("mixed proxy header families"));
                }
                {
                    let value = one(headers, "forwarded")?;
                    check_hops(value, proxy.hops)?;
                    forwarded(value)?
                }
            }
            ProxyHeaders::XForwarded => {
                if headers.contains_key("forwarded") {
                    return Err(Error::invalid("mixed proxy header families"));
                }
                let ips = one(headers, "x-forwarded-for")?;
                let protos = one(headers, "x-forwarded-proto")?;
                let hosts = one(headers, "x-forwarded-host")?;
                for value in [ips, protos, hosts] {
                    check_hops(value, proxy.hops)?;
                }
                let ips = values(ips);
                let protos = values(protos);
                let hosts = values(hosts);
                if ips.len() != protos.len() || ips.len() != hosts.len() {
                    return Err(Error::invalid("proxy chain lengths differ"));
                }
                ips.into_iter()
                    .zip(protos)
                    .zip(hosts)
                    .map(|((ip, proto), host)| {
                        Ok(Hop {
                            ip: parse_ip(ip, false)?,
                            proto: proto.to_ascii_lowercase(),
                            host: host.to_owned(),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?
            }
        };
        // A trusted proxy must emit exactly the configured chain, never a picked first/last token.
        if chain.len() != proxy.hops {
            return Err(Error::invalid("proxy hop count mismatch"));
        }
        let first = chain
            .first()
            .ok_or_else(|| Error::invalid("proxy chain empty"))?;
        // Canonicalize once, then compare every intermediary to the same approved external origin.
        let origin_key = authority_key(&first.host, &first.proto)?;
        if !self.authorities.contains(&origin_key) {
            return Err(Error::invalid("proxy origin not allowed"));
        }
        for hop in chain.iter().skip(1) {
            if hop.proto != first.proto
                || authority_key(&hop.host, &hop.proto)? != origin_key
                || !proxy.trusts(hop.ip)
            {
                return Err(Error::invalid("proxy chain not trusted"));
            }
        }
        if let Some(target) = uri.authority()
            && (uri.scheme_str() != Some(first.proto.as_str())
                || authority_key(target.as_str(), &first.proto)? != origin_key)
        {
            return Err(Error::invalid("absolute target and proxy origin mismatch"));
        }
        // Canonical authority metadata agrees with the URL adapter, while native path/query stay exact.
        let authority = if origin_key.port == if first.proto == "https" { 443 } else { 80 } {
            origin_key.host
        } else {
            format!("{}:{}", origin_key.host, origin_key.port)
        };
        Ok(EffectiveRequest {
            origin: format!("{}://{authority}/", first.proto),
            client: Some(first.ip),
        })
    }
}
/// Exactly one textual header occurrence prevents picking an arbitrary duplicate.
fn one<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str> {
    let mut fields = headers.get_all(name).iter();
    let value = fields
        .next()
        .ok_or_else(|| Error::invalid("required authority/proxy header absent"))?;
    if fields.next().is_some() {
        return Err(Error::invalid("duplicate authority/proxy header"));
    }
    value
        .to_str()
        .map_err(|cause| external("authority/proxy header text invalid", cause))
}
/// Split only the explicit list grammar; empty items fail subsequent IP/authority checks.
fn values(value: &str) -> Vec<&str> {
    value.split(',').map(str::trim).collect()
}
/// A full proxy hop has no request-derived free-form observer payload.
struct Hop {
    ip: IpAddr,
    proto: String,
    host: String,
}
/// Strict Forwarded subset supports quoted IPv6 but refuses escapes/extensions/partial hops.
fn forwarded(value: &str) -> Result<Vec<Hop>> {
    values(value)
        .into_iter()
        .map(|entry| {
            let mut ip = None;
            let mut proto = None;
            let mut host = None;
            for field in entry.split(';') {
                let (name, value) = field
                    .trim()
                    .split_once('=')
                    .ok_or_else(|| Error::invalid("Forwarded parameter invalid"))?;
                let value = unquote(value)?;
                match name.to_ascii_lowercase().as_str() {
                    "for" if ip.is_none() => ip = Some(parse_ip(value, true)?),
                    "proto" if proto.is_none() => proto = Some(value.to_ascii_lowercase()),
                    "host" if host.is_none() => host = Some(value.to_owned()),
                    _ => return Err(Error::invalid("duplicate/unsupported Forwarded parameter")),
                }
            }
            Ok(Hop {
                ip: ip.ok_or_else(|| Error::invalid("Forwarded for absent"))?,
                proto: proto.ok_or_else(|| Error::invalid("Forwarded proto absent"))?,
                host: host.ok_or_else(|| Error::invalid("Forwarded host absent"))?,
            })
        })
        .collect()
}
/// Only a balanced optional quoted value is accepted; backslash escapes are deliberately unsupported.
fn unquote(value: &str) -> Result<&str> {
    let value = if let Some(value) = value.strip_prefix('"') {
        value
            .strip_suffix('"')
            .ok_or_else(|| Error::invalid("proxy quote invalid"))?
    } else {
        value
    };
    if value.is_empty()
        || value
            .bytes()
            .any(|b| b == b'"' || b == b'\\' || b.is_ascii_control())
    {
        return Err(Error::invalid("proxy value invalid"));
    }
    Ok(value)
}
/// Concrete addresses only: no unknown/obfuscated tokens, hostnames or implicit address-family maps.
fn parse_ip(value: &str, forwarded: bool) -> Result<IpAddr> {
    if value.starts_with('[') {
        if let Some(ip) = value.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            return ip
                .parse()
                .map_err(|cause| external("proxy IPv6 invalid", cause));
        }
        return value
            .parse::<SocketAddr>()
            .map(|s| s.ip())
            .map_err(|cause| external("proxy address invalid", cause));
    }
    if let Ok(ip) = value.parse::<IpAddr>() {
        if forwarded && ip.is_ipv6() {
            return Err(Error::invalid("Forwarded IPv6 requires brackets"));
        }
        return Ok(ip);
    }
    value
        .parse::<SocketAddr>()
        .map(|s| s.ip())
        .map_err(|cause| external("proxy address invalid", cause))
}
/// Validate a literal authority with concrete default ports and no userinfo, wildcard or escapes.
fn authority_key(value: &str, scheme: &str) -> Result<AuthorityKey> {
    if !matches!(scheme, "http" | "https") || value.len() > 1024 || value.contains(['@', '%', '\\'])
    {
        return Err(Error::invalid("authority invalid"));
    }
    let authority: Authority = value
        .parse()
        .map_err(|cause| external("authority parsing failed", cause))?;
    let host = authority.host();
    if host.is_empty()
        || !host
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b':' | b'[' | b']'))
    {
        return Err(Error::invalid("authority hostname invalid"));
    }
    // Authority::port() silently returns None for malformed ports; inspect the explicit suffix.
    let suffix = value
        .strip_prefix(host)
        .ok_or_else(|| Error::invalid("authority hostname invalid"))?;
    let port = if suffix.is_empty() {
        if scheme == "https" { 443 } else { 80 }
    } else {
        let number = suffix
            .strip_prefix(':')
            .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
            .ok_or_else(|| Error::invalid("authority port invalid"))?;
        number
            .parse::<u16>()
            .map_err(|cause| external("authority port invalid", cause))?
    };
    let host = if let Some(literal) = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        // Compare IPv6 values rather than their many equivalent textual spellings.
        let ip: std::net::Ipv6Addr = literal
            .parse()
            .map_err(|cause| external("authority IPv6 invalid", cause))?;
        format!("[{ip}]")
    } else {
        if host.contains(':') {
            return Err(Error::invalid("IPv6 authority requires brackets"));
        }
        host.to_ascii_lowercase()
    };
    Ok(AuthorityKey { host, port })
}

/// Bound chain allocation before splitting or cloning any per-hop fields.
fn check_hops(value: &str, hops: usize) -> Result<()> {
    if value.split(',').count() != hops {
        return Err(Error::invalid("proxy hop count mismatch"));
    }
    Ok(())
}
