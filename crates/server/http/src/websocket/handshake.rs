//! Strict bounded validation around tungstenite's standard response generator.
use base64::{Engine, engine::general_purpose::STANDARD};
use http::{
    HeaderMap,
    header::{CONNECTION, CONTENT_LENGTH, TRANSFER_ENCODING, UPGRADE},
};
use stackpress_lib::{Error, Result};

/// Trusted handshake metadata retained outside normalized application input.
pub(crate) struct Handshake {
    pub(crate) key: String,
    pub(crate) accept: http::HeaderValue,
    pub(crate) protocols: Vec<String>,
}
/// Require a single bounded textual field, preserving duplicate rejection.
fn singleton<'a>(headers: &'a HeaderMap, name: &str, max: usize) -> Result<&'a str> {
    let mut values = headers.get_all(name).iter();
    let value = values
        .next()
        .ok_or_else(|| Error::invalid("missing WebSocket handshake field").with_code(400))?;
    if values.next().is_some() || value.as_bytes().len() > max {
        return Err(
            Error::invalid("duplicate or oversized WebSocket handshake field").with_code(400),
        );
    }
    value
        .to_str()
        .map_err(|_| Error::invalid("invalid WebSocket header text").with_code(400))
}
/// Validate an HTTP token without accepting commas, controls or quoted subprotocols.
pub(crate) fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}
/// Inspect only parser-produced fields; no first-party HTTP parser or accept-key algorithm.
pub(crate) fn validate<B>(request: &http::Request<B>) -> Result<Handshake> {
    let headers = request.headers();
    if request.version() != http::Version::HTTP_11
        || request.method() != http::Method::GET
        || headers.contains_key(TRANSFER_ENCODING)
        || headers.contains_key("trailer")
    {
        return Err(Error::invalid("unsupported WebSocket request framing").with_code(400));
    }
    if headers.contains_key(CONTENT_LENGTH) && singleton(headers, "content-length", 20)? != "0" {
        return Err(Error::invalid("WebSocket handshake body forbidden").with_code(400));
    }
    if !singleton(headers, "upgrade", 32)?.eq_ignore_ascii_case("websocket") {
        return Err(Error::invalid("unsupported upgrade").with_code(400));
    }
    let mut upgrade = false;
    for value in headers.get_all(CONNECTION) {
        let value = value
            .to_str()
            .map_err(|_| Error::invalid("invalid Connection tokens").with_code(400))?;
        for part in value.split(',') {
            let part = part.trim();
            if !token(part) {
                return Err(Error::invalid("invalid Connection token").with_code(400));
            }
            upgrade |= part.eq_ignore_ascii_case("upgrade");
        }
    }
    if !upgrade {
        return Err(Error::invalid("missing Connection upgrade token").with_code(400));
    }
    if singleton(headers, "sec-websocket-version", 32)? != "13" {
        return Err(Error::invalid("unsupported WebSocket version").with_code(426));
    }
    let key = singleton(headers, "sec-websocket-key", 24)?;
    let mut decoded = [0u8; 18];
    if STANDARD.decode_slice(key, &mut decoded).ok() != Some(16) {
        return Err(Error::invalid("invalid WebSocket key").with_code(400));
    }
    let mut protocols = Vec::new();
    let mut bytes = 0usize;
    for offered in headers.get_all("sec-websocket-protocol") {
        let offered = offered
            .to_str()
            .map_err(|_| Error::invalid("invalid subprotocol text").with_code(400))?;
        for protocol in offered.split(',').map(str::trim) {
            bytes = bytes.saturating_add(protocol.len());
            if !token(protocol)
                || protocols.len() >= 32
                || bytes > 8192
                || protocols.iter().any(|p| p == protocol)
            {
                return Err(Error::invalid("invalid subprotocol offers").with_code(400));
            }
            protocols.push(protocol.to_owned());
        }
    }
    // The generator sees normalized singleton headers after every original occurrence was checked.
    let normalized = http::Request::builder()
        .method("GET")
        .version(http::Version::HTTP_11)
        .uri("/")
        .header(CONNECTION, "Upgrade")
        .header(UPGRADE, "websocket")
        .header("sec-websocket-version", "13")
        .header("sec-websocket-key", key)
        .body(())
        .map_err(|e| crate::external("WebSocket request conversion failed", e))?;
    let response = tungstenite::handshake::server::create_response(&normalized)
        .map_err(|e| crate::external("WebSocket handshake generation failed", e))?;
    let accept = response
        .headers()
        .get("sec-websocket-accept")
        .cloned()
        .ok_or_else(|| Error::new("WebSocket accept unavailable"))?;
    Ok(Handshake {
        key: key.into(),
        accept,
        protocols,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn request() -> http::Request<()> {
        http::Request::builder()
            .uri("/")
            .header("upgrade", "WebSocket")
            .header("connection", "keep-alive")
            .header("connection", "UpGrAdE")
            .header("sec-websocket-key", "dGhlIHNhbXBsZSBub25jZQ==")
            .header("sec-websocket-version", "13")
            .body(())
            .unwrap()
    }
    #[test]
    fn known_vector_and_strict_decision_table() {
        assert_eq!(
            validate(&request()).unwrap().accept,
            "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="
        );
        for name in ["upgrade", "sec-websocket-key", "sec-websocket-version"] {
            let mut r = request();
            let value = r.headers()[name].clone();
            r.headers_mut().append(
                http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value,
            );
            assert!(validate(&r).is_err());
        }
        for key in [
            "",
            "not base64",
            "AAAAAAAAAAAAAAAAAAAA",
            "AAAAAAAAAAAAAAAAAAAAAA==",
            "AAAAAAAAAAAAAAAAAAAAAAA=",
            "AAAAAAAAAAAAAAAAAAAAAA=A",
        ] {
            let mut r = request();
            r.headers_mut()
                .insert("sec-websocket-key", key.parse().unwrap());
            assert_eq!(validate(&r).is_ok(), key == "AAAAAAAAAAAAAAAAAAAAAA==");
        }
        let mut r = request();
        r.headers_mut()
            .insert("sec-websocket-version", "12".parse().unwrap());
        assert_eq!(validate(&r).err().unwrap().code(), 426);
        let mut r = request();
        *r.version_mut() = http::Version::HTTP_10;
        assert!(validate(&r).is_err());
        let mut r = request();
        *r.method_mut() = http::Method::POST;
        assert!(validate(&r).is_err());
        for (name, value) in [
            ("upgrade", "websocket,other"),
            ("connection", "keep-alive"),
            ("content-length", "1"),
            ("transfer-encoding", "chunked"),
            ("sec-websocket-protocol", "x,x"),
        ] {
            let mut r = request();
            r.headers_mut().insert(
                http::HeaderName::from_bytes(name.as_bytes()).unwrap(),
                value.parse().unwrap(),
            );
            assert!(validate(&r).is_err());
        }
    }
}
