//! Source base32/HMAC-SHA1 helpers, using the already approved ring implementation.
//!
//! The source ignores leftover base32 bits and accepts lowercase/trailing padding.
//! Comparisons are ordinary equality, as upstream; no replay/expiry store is added.

use ring::hmac;
use stackpress_lib::{Error, Result};

/// Decode source base32, discarding trailing incomplete bytes.
/// # Errors
/// Rejects any nonalphabet character after removing trailing `=` and uppercasing.
pub fn base32_decode(secret: &str) -> Result<Vec<u8>> {
    let mut buffer = 0_u16;
    let mut bits = 0_u8;
    let mut bytes = Vec::new();
    for character in secret.trim_end_matches('=').to_uppercase().chars() {
        let value = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567"
            .find(character)
            .ok_or_else(|| Error::invalid("Invalid base32 character"))? as u16;
        buffer = (buffer << 5) | value;
        bits += 5;
        if bits >= 8 {
            bits -= 8;
            bytes.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }
    Ok(bytes)
}
/// Generate a source six-digit code at epoch seconds, with signed window and time step.
/// # Errors
/// Rejects invalid base32, zero step or a counter outside unsigned 64-bit bounds.
pub fn generate_at(secret: &str, seconds: u64, window: i64, step: u64) -> Result<String> {
    let key = base32_decode(secret)?;
    let counter = seconds
        .checked_div(step)
        .and_then(|counter| i128::from(counter).checked_add(i128::from(window)))
        .and_then(|counter| u64::try_from(counter).ok())
        .ok_or_else(|| Error::invalid("TOTP counter out of range"))?;
    let key = hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &key);
    let digest = hmac::sign(&key, &counter.to_be_bytes());
    let bytes = digest.as_ref();
    let offset = usize::from(bytes[bytes.len() - 1] & 15);
    let code = u32::from_be_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("SHA1 dynamic offset fits"),
    ) & 0x7fff_ffff;
    Ok(format!("{:06}", code % 1_000_000))
}
/// Compare from -window through +window, stopping at the first equal source code.
/// # Errors
/// Preserves decoder/counter errors rather than silently converting malformed secrets to false.
pub fn verify_at(token: &str, secret: &str, seconds: u64, window: i64) -> Result<bool> {
    let first = window
        .checked_neg()
        .ok_or_else(|| Error::invalid("TOTP window out of range"))?;
    for delta in first..=window {
        if generate_at(secret, seconds, delta, 30)? == token {
            return Ok(true);
        }
    }
    Ok(false)
}
