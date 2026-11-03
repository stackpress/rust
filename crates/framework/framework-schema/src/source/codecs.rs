//! Native source codecs and explicit host Date.toString conversion.
use super::DateError;
type Result<T> = std::result::Result<T, DateError>;
pub use crate::providers::{decrypt, encrypt, hash};
use time::{OffsetDateTime, UtcOffset};

/// Original @encrypted/@hashed field serializer, retaining its configured database seed.
pub struct SourceCodecs {
    /// Database codec seed retained natively, separate from the identity JWT seed.
    seed: String,
}
impl SourceCodecs {
    /// Use the exact caller-selected database seed (including the direct empty seed).
    pub fn new(seed: impl Into<String>) -> Self {
        Self { seed: seed.into() }
    }
    /// Encode a string token for both storage and equality lookup, with deterministic output.
    pub fn serialize_token(&self, token: &str) -> String {
        encrypt(token, &self.seed)
    }
    /// Decode the stored token without changing its original source text.
    /// # Errors
    /// Propagates block/padding errors through the shared source decoder.
    pub fn unserialize_token(&self, token: &str) -> crate::SchemaResult<String> {
        decrypt(token, &self.seed)
    }
    /// Preserve the source deterministic SHAKE256 hex credential/challenge representation.
    pub fn serialize_secret(&self, secret: &str) -> String {
        hash(secret)
    }
}
/// Format the source English Date.toString shape for a known local offset and zone name.
/// Subseconds are omitted exactly as JavaScript does; the original stored instant is unchanged.
/// # Errors
/// Returns unsupported native calendar/offset ranges instead of substituting ISO strings.
pub fn date_string(milliseconds: i64, offset_minutes: i32, zone_name: &str) -> Result<String> {
    let nanos = i128::from(milliseconds) * 1_000_000;
    let date = OffsetDateTime::from_unix_timestamp_nanos(nanos)
        .map_err(|_| DateError("source date out of native range"))?;
    let seconds = offset_minutes
        .checked_mul(60)
        .ok_or(DateError("source offset out of range"))?;
    let offset = UtcOffset::from_whole_seconds(seconds)
        .map_err(|_| DateError("source offset out of range"))?;
    let local = date.to_offset(offset);
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    let days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let absolute = offset_minutes.unsigned_abs();
    // JS uses a signed six-digit year before year zero, a four-digit minimum otherwise.
    let year = if local.year() < 0 {
        format!("-{:06}", local.year().unsigned_abs())
    } else {
        format!("{:04}", local.year())
    };
    Ok(format!(
        "{} {} {:02} {} {:02}:{:02}:{:02} GMT{sign}{:02}{:02} ({zone_name})",
        days[local.weekday().number_days_from_monday() as usize],
        months[local.month() as usize - 1],
        local.day(),
        year,
        local.hour(),
        local.minute(),
        local.second(),
        absolute / 60,
        absolute % 60
    ))
}
