//! Public cookie notices with best-effort consumption and bounded per-root dedupe.
use crate::cookie::{CookieScope, unique_cookie};
use serde::{Deserialize, Serialize};
use stackpress_framework_language::PreferenceError;
use std::collections::BTreeSet;

/// Fixed presentation cookie, never an authentication credential.
const FLASH_COOKIE: &str = "stackpress-flash";

/// A checked public plain-text notice. Hostile-looking text remains ordinary text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "NoticeWire")]
pub struct FlashNotice {
    /// Stable public identifier, not a caller/session token.
    id: String,
    /// Explicitly public text; consumers render through escaped text nodes.
    message: String,
}
/// Checked serde intermediate prevents seeds bypassing limits.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoticeWire {
    id: String,
    message: String,
}
impl TryFrom<NoticeWire> for FlashNotice {
    type Error = PreferenceError;
    /// Enforce the same bounds for cookie and public seed decoding.
    fn try_from(wire: NoticeWire) -> Result<Self, Self::Error> {
        Self::new(wire.id, wire.message)
    }
}
impl FlashNotice {
    /// Check a 1–64 byte ASCII id and 1–512 byte public message.
    ///
    /// # Errors
    /// Rejects empty/excess text and ids containing header/path/control syntax.
    pub fn new(id: String, message: String) -> Result<Self, PreferenceError> {
        if id.is_empty()
            || id.len() > 64
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
            || message.is_empty()
            || message.len() > 512
        {
            return Err(PreferenceError::Request);
        }
        Ok(Self { id, message })
    }
    /// Borrow the root dedupe key; it conveys no authority.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// Borrow plain text; never insert it as raw HTML.
    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Result of reading the cookie; deletion applies even when its data is invalid.
#[derive(Clone, Debug)]
pub struct FlashConsumption {
    /// Valid public content only; malformed and duplicate cookies disclose nothing.
    pub notice: Option<FlashNotice>,
    /// Delete at the exact issuing Path/flags, before HTML is committed.
    pub clear_cookie: Option<String>,
}

/// Immutable issuer/consumer. Concurrent tabs can both read before deletion arrives.
/// This is best effort, never exactly-once or durable notification delivery.
#[derive(Clone, Debug)]
pub struct FlashCookie(CookieScope);
impl FlashCookie {
    /// Use the same checked scope as other presentation cookies.
    pub fn new(scope: CookieScope) -> Self {
        Self(scope)
    }
    /// Encode bounded public data as header-safe lowercase hex with five-minute TTL.
    ///
    /// # Errors
    /// Returns a safe failure if JSON/hex/header expansion exceeds the wire ceiling.
    pub fn issue(&self, notice: &FlashNotice) -> Result<String, PreferenceError> {
        let json = serde_json::to_vec(notice).map_err(|_| PreferenceError::Request)?;
        // Escaped control characters can expand sixfold. Bound the encoded form
        // too, so every issued cookie can be admitted again by consume.
        if json.len() > 1800 {
            return Err(PreferenceError::Limit);
        }
        let mut hex = String::with_capacity(json.len() * 2);
        for byte in json {
            // Hex has no delimiters, quotes or control characters in a header.
            use std::fmt::Write;
            write!(&mut hex, "{byte:02x}").map_err(|_| PreferenceError::Request)?;
        }
        let header = self.0.header(FLASH_COOKIE, &hex, 300);
        if header.len() > 4096 {
            return Err(PreferenceError::Limit);
        }
        Ok(header)
    }
    /// Consume once into this response snapshot and clear malformed/duplicate values.
    ///
    /// # Errors
    /// Rejects an oversized total cookie header before parsing. A host must send
    /// no-store HTML and preserve this deletion header when returning the snapshot.
    pub fn consume(&self, cookie: &str) -> Result<FlashConsumption, PreferenceError> {
        if cookie.len() > stackpress_framework_language::MAX_COOKIE_BYTES {
            return Err(PreferenceError::Limit);
        }
        let value = unique_cookie(cookie, FLASH_COOKIE);
        let present = !matches!(value, Ok(None));
        let notice = value.ok().flatten().and_then(decode);
        Ok(FlashConsumption {
            notice,
            clear_cookie: present.then(|| self.0.header(FLASH_COOKIE, "", 0)),
        })
    }
}

/// Reject malformed or excessive hex before allocating/decoding untrusted JSON.
fn decode(hex: &str) -> Option<FlashNotice> {
    if hex.len() > 3600 || !hex.len().is_multiple_of(2) {
        return None;
    }
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = (pair[0] as char).to_digit(16)?;
            let low = (pair[1] as char).to_digit(16)?;
            Some((high * 16 + low) as u8)
        })
        .collect::<Option<Vec<_>>>()?;
    serde_json::from_slice(&bytes).ok()
}

/// Dedupe for one root lifetime, including dismissed notices. No global caller state.
#[derive(Debug, Default)]
pub struct FlashRoot {
    /// At most 64 ids; saturation denies new notices rather than growing forever.
    seen: BTreeSet<String>,
}
impl FlashRoot {
    /// Admit a notice at most once per root; dismissal does not forget its id.
    pub fn admit(&mut self, notice: FlashNotice) -> Option<FlashNotice> {
        if self.seen.len() >= 64 || !self.seen.insert(notice.id.clone()) {
            return None;
        }
        Some(notice)
    }
}
