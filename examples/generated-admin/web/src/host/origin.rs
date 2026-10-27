//! Local mutation-origin admission; uses the existing HTTP dependency, without cookie policy.
use stackpress_lib::{Error, Result};

/// Admit an exact HTTP(S) origin before registering any startup work.
/// Origins are at most 512 bytes, with a nonempty URI host and an optional decimal
/// u16 port. Bracketed hosts must be IPv6 literals. Userinfo, paths (even `/`),
/// queries and fragments are forbidden; the admitted string is never normalized.
pub(super) fn validate(origin: &str) -> Result<()> {
    let invalid = || Error::invalid("invalid admin mutation origin");
    // Bound parsing and accept only the two explicit lowercase scheme prefixes.
    if origin.len() > 512 {
        return Err(invalid());
    }
    let raw = origin
        .strip_prefix("http://")
        .or_else(|| origin.strip_prefix("https://"))
        .ok_or_else(invalid)?;
    // Authority parsing permits userinfo and URI parsing may normalize an empty path.
    // Reject every non-origin suffix and backslash before using either interpretation.
    if raw.contains(['@', '/', '\\', '?', '#']) {
        return Err(invalid());
    }
    let authority: http::uri::Authority =
        raw.parse().map_err(|cause| invalid().with_cause(cause))?;
    let host = authority.host();
    if host.is_empty() {
        return Err(invalid());
    }
    // Authority checks bracket structure; additionally require an actual IPv6 address.
    if let Some(address) = host
        .strip_prefix('[')
        .and_then(|host| host.strip_suffix(']'))
    {
        address
            .parse::<std::net::Ipv6Addr>()
            .map_err(|cause| invalid().with_cause(cause))?;
    } else if host.contains(['[', ']']) {
        return Err(invalid());
    }
    // Without userinfo the host is the authority prefix. Any remainder must be one
    // nonempty decimal port; Authority::port() alone would silently ignore a bad port.
    let remainder = &raw[host.len()..];
    if !remainder.is_empty() {
        let port = remainder.strip_prefix(':').ok_or_else(invalid)?;
        if port.is_empty()
            || !port.bytes().all(|byte| byte.is_ascii_digit())
            || port.parse::<u16>().is_err()
        {
            return Err(invalid());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate;

    /// Public fixture and loopback origins admit hosts, IP literals and numeric ports.
    #[test]
    fn accepted_origin_shapes() {
        for origin in [
            "https://admin.invalid",
            "http://localhost",
            "https://example.test:443",
            "http://127.0.0.1:3001",
            "http://localhost:0",
            "http://localhost:00080",
            "https://[2001:db8::1]:65535",
            "http://[::1]",
        ] {
            assert!(validate(origin).is_ok(), "{origin}");
        }
    }

    /// Ambiguous authority, non-origin URL parts and invalid/bounded input fail closed.
    #[test]
    fn rejected_origin_shapes() {
        for origin in [
            "",
            "admin.invalid",
            "//admin.invalid",
            "ftp://admin.invalid",
            "HTTP://admin.invalid",
            "http://",
            "https://:443",
            "http://user@host",
            "https://user:password@host",
            "http://host/",
            "http://host/path",
            "http://host?",
            "http://host?x=1",
            "http://host#",
            "http://host#fragment",
            "http://host\\path",
            "http://host:",
            "http://host:abc",
            "http://host:+80",
            "http://host:65536",
            "http://host:80:90",
            "http:// host",
            "http://host\n",
            "http://[::1",
            "http://[]",
            "http://[not-an-ip]",
            "http://host[]",
            "http://host%2fpath",
            "http://ho^st",
        ] {
            assert!(validate(origin).is_err(), "{origin:?}");
        }
        assert!(validate(&format!("https://{}", "a".repeat(512))).is_err());
    }
}
