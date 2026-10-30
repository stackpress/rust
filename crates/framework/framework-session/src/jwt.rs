//! Source JWT wire/claim adaptation over the approved library's signing/verification.
//!
//! Signature primitives remain in jsonwebtoken. Parsing the header as an object avoids
//! imposing that crate's typed optional-header restrictions on jose's accepted inputs.

use crate::{SessionData, SessionError, SessionResult};
use base64::{
    Engine, alphabet,
    engine::{
        DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig, general_purpose::URL_SAFE_NO_PAD,
    },
};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, crypto, encode};
use serde_json::{Number, Value};

/// Sign HS256 with alg-only header, overwritten iat and optional absolute exp.
pub(crate) fn sign(
    mut data: SessionData,
    seed: &str,
    expires: f64,
    now: f64,
) -> SessionResult<String> {
    // WebCrypto, used by the source jose, rejects a zero-length HMAC key.
    // Keep that provider boundary without imposing a stronger minimum on the source seed.
    if seed.is_empty() {
        return Err(SessionError("JWT signing failed"));
    }
    data.insert("iat".into(), numeric_date(now)?);
    if expires != 0.0 {
        data.insert("exp".into(), numeric_date(expires)?);
    }
    let mut header = Header::new(Algorithm::HS256);
    header.typ = None;
    encode(&header, &data, &EncodingKey::from_secret(seed.as_bytes()))
        .map_err(|_| SessionError("JWT signing failed"))
}
/// Preserve JS JSON.stringify's integral seconds without losing fractional NumericDates.
fn numeric_date(value: f64) -> SessionResult<Value> {
    let number = if value.fract() == 0.0 && value >= i64::MIN as f64 && value < (i64::MAX as f64) {
        Number::from(value as i64)
    } else {
        Number::from_f64(value).ok_or(SessionError("invalid JWT NumericDate"))?
    };
    Ok(Value::Number(number))
}
/// Verify signatures with explicit source temporal policy, not jsonwebtoken's default claims.
pub(crate) fn verify(token: &str, seed: &str, now: f64) -> Option<SessionData> {
    // Source compact-signing input is ASCII, even when decoded JSON contains Unicode.
    if seed.is_empty() || !token.is_ascii() {
        return None;
    }
    let mut parts = token.split('.');
    let (protected, payload, signature) = (parts.next()?, parts.next()?, parts.next()?);
    if parts.next().is_some() {
        return None;
    }
    let header: SessionData = decode_object(protected)?;
    let algorithm = match header.get("alg")?.as_str()? {
        "HS256" => Algorithm::HS256,
        "HS384" => Algorithm::HS384,
        "HS512" => Algorithm::HS512,
        _ => return None,
    };
    // jose recognizes only the protected b64 extension by default. It ignores typ/jku/etc.
    // Parsing those through jsonwebtoken Header would impose types jose never requires.
    if let Some(critical) = header.get("crit") {
        let critical = critical.as_array()?;
        if critical.is_empty()
            || critical.iter().any(|v| v.as_str() != Some("b64"))
            || header.get("b64") != Some(&Value::Bool(true))
        {
            return None;
        }
    }
    // The source accepts padded URL-safe components. Canonicalize only the decoded signature
    // for the JWT library decoder; keep the original header/payload bytes in the signed input.
    let signature = URL_SAFE_NO_PAD.encode(decode_component(signature)?);
    // Keep cryptography in the approved JWT library; verify the exact original encoded bytes.
    if !crypto::verify(
        &signature,
        format!("{protected}.{payload}").as_bytes(),
        &DecodingKey::from_secret(seed.as_bytes()),
        algorithm,
    )
    .ok()?
    {
        return None;
    }
    let claims: SessionData = decode_object(payload)?;
    for claim in ["iat", "nbf", "exp"] {
        if let Some(value) = claims.get(claim) {
            let value = value.as_f64()?;
            if (claim == "exp" && value <= now) || (claim == "nbf" && value > now) {
                return None;
            }
        }
    }
    Some(claims)
}

/// Match the source TextDecoder replacement behaviour before JSON object parsing.
fn decode_object(encoded: &str) -> Option<SessionData> {
    let bytes = decode_component(encoded)?;
    serde_json::from_str(&String::from_utf8_lossy(&bytes)).ok()
}

/// Match jose's loose base64url decoding: optional padding and insignificant ASCII whitespace.
fn decode_component(encoded: &str) -> Option<Vec<u8>> {
    const DECODER: GeneralPurpose = GeneralPurpose::new(
        &alphabet::URL_SAFE,
        GeneralPurposeConfig::new()
            .with_decode_padding_mode(DecodePaddingMode::Indifferent)
            .with_decode_allow_trailing_bits(true),
    );
    let compact: Vec<u8> = encoded
        .bytes()
        .filter(|b| !b" \t\n\r\x0c".contains(b))
        .collect();
    DECODER.decode(compact).ok()
}
