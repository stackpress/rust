//! Explicit source Date conversion without process-wide timezone mutation.

use super::Value;
use crate::{SchemaError, SchemaErrorKind, SchemaResult};
use time::{
    OffsetDateTime, PrimitiveDateTime, UtcOffset,
    format_description::{self, well_known::Rfc3339},
};

/// Decode a source Date input to milliseconds; invalid text uses the source epoch fallback.
/// Numeric inputs are milliseconds. Naive strings use the host-supplied local UTC offset.
pub fn date_millis(value: &Value, offset: i32) -> i64 {
    if let Some(millis) = super::as_date(value) {
        return millis;
    }
    if let Some(number) = value.as_f64() {
        return number as i64;
    }
    let text = super::string(value);
    // V8 Date.parse treats short numeric strings as month/year spellings, unlike
    // numeric values, which are epoch milliseconds. Keep that source distinction.
    if !text.is_empty()
        && text.bytes().all(|byte| byte.is_ascii_digit())
        && let Ok(number) = text.parse::<i32>()
    {
        let (year, month) = match number {
            0 => (2000, 1),
            1..=12 => (2001, number as u8),
            32..=49 => (2000 + number, 1),
            50..=99 => (1900 + number, 1),
            100..=9999 => (number, 1),
            _ => return 0,
        };
        if let Ok(month) = time::Month::try_from(month)
            && let Ok(date) = time::Date::from_calendar_date(year, month, 1)
        {
            let seconds = offset.checked_mul(60).unwrap_or_default();
            return date
                .midnight()
                .assume_offset(UtcOffset::from_whole_seconds(seconds).unwrap_or(UtcOffset::UTC))
                .unix_timestamp()
                * 1000;
        }
    }
    if let Ok(date) = OffsetDateTime::parse(&text, &Rfc3339) {
        return (date.unix_timestamp_nanos() / 1_000_000) as i64;
    }
    // Source SQL text is naive local time. ISO date-only is UTC, matching Date.parse.
    for pattern in [
        "[year]-[month]-[day] [hour]:[minute]:[second]",
        "[year]-[month]-[day]T[hour]:[minute]:[second]",
    ] {
        if let Ok(format) = format_description::parse_borrowed::<2>(pattern)
            && let Ok(date) = PrimitiveDateTime::parse(&text, &format)
            && let Some(seconds) = offset.checked_mul(60)
            && let Ok(zone) = UtcOffset::from_whole_seconds(seconds)
        {
            return (date.assume_offset(zone).unix_timestamp_nanos() / 1_000_000) as i64;
        }
    }
    if let Ok(format) = format_description::parse_borrowed::<2>("[year]-[month]-[day]")
        && let Ok(date) = time::Date::parse(&text, &format)
    {
        return date.midnight().assume_utc().unix_timestamp() * 1000;
    }
    0
}

/// Match Date.toISOString's date combined with Date.toTimeString's local time.
pub fn date_text(millis: i64, offset: i32) -> SchemaResult<String> {
    let date = instant(millis)?;
    let seconds = offset.checked_mul(60).ok_or_else(range)?;
    let local = date.to_offset(UtcOffset::from_whole_seconds(seconds).map_err(|_| range())?);
    Ok(format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
        date.year(),
        u8::from(date.month()),
        date.day(),
        local.hour(),
        local.minute(),
        local.second()
    ))
}

/// Canonical native Date wire/storage representation, always including milliseconds.
pub fn iso(millis: i64) -> SchemaResult<String> {
    let date = instant(millis)?;
    Ok(format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        date.year(),
        u8::from(date.month()),
        date.day(),
        date.hour(),
        date.minute(),
        date.second(),
        date.millisecond()
    ))
}

/// Checked native time range rather than an overflowing conversion.
fn instant(millis: i64) -> SchemaResult<OffsetDateTime> {
    OffsetDateTime::from_unix_timestamp_nanos(i128::from(millis) * 1_000_000).map_err(|_| range())
}

/// Explain the Rust time representation boundary without including private values.
fn range() -> SchemaError {
    SchemaError::new(
        SchemaErrorKind::Unsupported,
        "date or local offset exceeds native representation",
    )
}
