//! Strict ISO admission and Gregorian display with explicit UTC-offset identity.
//!
//! Offsets are supplied by the authoritative domain value. This display layer
//! neither resolves IANA zones nor guesses DST gaps/folds. The offset is always
//! visible on datetime output, distinguishing both instances of a repeated hour.

use super::{DisplayLocale, FormatError, FormatErrorKind, admit_ascii};
use icu_calendar::{Date, Gregorian};
use icu_datetime::input::Time;
use icu_datetime::options::SubsecondDigits;
use icu_datetime::{FixedCalendarDateTimeFormatter, NoCalendarFormatter, fieldsets};

/// An ISO calendar date in the inclusive Gregorian range 0001..9999.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CalendarDate(Date<Gregorian>);

/// A local clock value; it has no date, offset or ambient timezone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TimeOfDay(Time);

/// Explicit time display precision. Fractional digits are truncated, never rounded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimePrecision {
    /// Hours and minutes; seconds remain in the value but are not displayed.
    Minutes,
    /// Hours, minutes and seconds.
    Seconds,
    /// Exactly three fractional digits, including trailing zeroes.
    Milliseconds,
    /// Exactly six fractional digits, including trailing zeroes.
    Microseconds,
    /// Exactly nine fractional digits, including trailing zeroes.
    Nanoseconds,
}

impl TimePrecision {
    /// Static fieldset keeps unrelated calendar/zone-format data out of the bundle.
    fn fieldset(self) -> fieldsets::T {
        match self {
            Self::Minutes => fieldsets::T::hm(),
            Self::Seconds => fieldsets::T::hms(),
            Self::Milliseconds => fieldsets::T::hmss(SubsecondDigits::S3),
            Self::Microseconds => fieldsets::T::hmss(SubsecondDigits::S6),
            Self::Nanoseconds => fieldsets::T::hmss(SubsecondDigits::S9),
        }
    }
}

impl CalendarDate {
    /// Admit exactly `YYYY-MM-DD`, including Gregorian leap-day validation.
    ///
    /// # Errors
    /// Rejects oversized, noncanonical or impossible dates; never normalizes overflow.
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        admit_ascii(input, 10, FormatErrorKind::Date)?;
        // ASCII admission makes the fixed byte offsets safe UTF-8 boundaries.
        if input.len() != 10 || &input[4..5] != "-" || &input[7..8] != "-" {
            return Err(FormatError::new(FormatErrorKind::Date));
        }
        let year = digits(&input[..4], FormatErrorKind::Date)?;
        let month = digits(&input[5..7], FormatErrorKind::Date)?;
        let day = digits(&input[8..], FormatErrorKind::Date)?;
        if year == 0 {
            return Err(FormatError::new(FormatErrorKind::Date));
        }
        Date::try_new_gregorian(year as i32, month as u8, day as u8)
            .map(Self)
            .map_err(|error| FormatError::caused(FormatErrorKind::Date, error))
    }

    /// Display the medium Gregorian date using only the selected pinned locale.
    ///
    /// # Errors
    /// Retains the library cause if pinned data cannot be constructed.
    pub fn display(&self, locale: DisplayLocale) -> Result<String, FormatError> {
        let formatter = FixedCalendarDateTimeFormatter::try_new(
            locale.locale().into(),
            fieldsets::YMD::medium(),
        )
        .map_err(|error| FormatError::caused(FormatErrorKind::Data, error))?;
        Ok(formatter.format(&self.0).to_string())
    }
}

impl TimeOfDay {
    /// Admit `HH:MM`, `HH:MM:SS` or `HH:MM:SS.fraction` (1..=9 digits).
    ///
    /// # Errors
    /// Rejects noncanonical/oversized input, 24:00 and leap seconds. Partial drafts
    /// are not values. Missing seconds mean exactly zero, never an invalid fallback.
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        admit_ascii(input, 18, FormatErrorKind::Time)?;
        if input.len() < 5 || &input[2..3] != ":" {
            return Err(FormatError::new(FormatErrorKind::Time));
        }
        let hour = digits(&input[..2], FormatErrorKind::Time)?;
        let minute = digits(&input[3..5], FormatErrorKind::Time)?;
        let (second, nanos) = if input.len() == 5 {
            (0, 0)
        } else {
            // Optional seconds require their complete fixed field before a fraction.
            if input.len() < 8 || &input[5..6] != ":" {
                return Err(FormatError::new(FormatErrorKind::Time));
            }
            let second = digits(&input[6..8], FormatErrorKind::Time)?;
            let nanos = if input.len() == 8 {
                0
            } else {
                if &input[8..9] != "." || input.len() == 9 {
                    return Err(FormatError::new(FormatErrorKind::Time));
                }
                let fraction = &input[9..];
                // At most nine admitted digits makes multiplication fit u32.
                digits(fraction, FormatErrorKind::Time)? * 10_u32.pow(9 - fraction.len() as u32)
            };
            (second, nanos)
        };
        if second > 59 {
            return Err(FormatError::new(FormatErrorKind::Time));
        }
        Time::try_new(hour as u8, minute as u8, second as u8, nanos)
            .map(Self)
            .map_err(|error| FormatError::caused(FormatErrorKind::Time, error))
    }

    /// Display the local time at explicit precision and the locale's hour cycle.
    ///
    /// # Errors
    /// Retains the library cause if pinned data cannot be constructed.
    pub fn display(
        &self,
        locale: DisplayLocale,
        precision: TimePrecision,
    ) -> Result<String, FormatError> {
        let formatter = NoCalendarFormatter::try_new(locale.locale().into(), precision.fieldset())
            .map_err(|error| FormatError::caused(FormatErrorKind::Data, error))?;
        Ok(formatter.format(&self.0).to_string())
    }
}

/// An ISO local datetime with explicit `Z` or numeric `±HH:MM` offset.
/// This preserves the supplied wall time and offset; it performs no zone conversion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OffsetDateTime {
    /// Calendar portion validated before publication.
    date: CalendarDate,
    /// Local wall time, including admitted fractional seconds.
    time: TimeOfDay,
    /// Validated seconds east of UTC, bounded to ±14:00.
    offset_seconds: i32,
    /// Canonical visible UTC suffix; zero offsets normalize to UTC+00:00.
    label: String,
}

impl OffsetDateTime {
    /// Admit `YYYY-MM-DDTHH:MM[:SS[.fraction]](Z|±HH:MM)`.
    ///
    /// # Errors
    /// Rejects missing/unknown offsets, `-00:00`, offsets beyond ±14:00, annotations,
    /// whitespace, invalid calendar/time fields and input over 35 bytes.
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        admit_ascii(input, 35, FormatErrorKind::Offset)?;
        if input.len() < 17 || &input[10..11] != "T" {
            return Err(FormatError::new(FormatErrorKind::Offset));
        }
        let (local, suffix) = if let Some(local) = input.strip_suffix('Z') {
            (local, "+00:00")
        } else if input.len() >= 22 {
            input.split_at(input.len() - 6)
        } else {
            return Err(FormatError::new(FormatErrorKind::Offset));
        };
        // A local value is checked independently; offset syntax never repairs it.
        let date = CalendarDate::parse(&local[..10])?;
        let time = TimeOfDay::parse(&local[11..])?;
        if suffix.len() != 6
            || !matches!(&suffix[..1], "+" | "-")
            || &suffix[3..4] != ":"
            || suffix == "-00:00"
        {
            return Err(FormatError::new(FormatErrorKind::Offset));
        }
        let hours = digits(&suffix[1..3], FormatErrorKind::Offset)?;
        let minutes = digits(&suffix[4..6], FormatErrorKind::Offset)?;
        if hours > 14 || minutes > 59 || (hours == 14 && minutes != 0) {
            return Err(FormatError::new(FormatErrorKind::Offset));
        }
        let sign = if suffix.starts_with('-') { -1 } else { 1 };
        let seconds = sign * (hours * 3600 + minutes * 60) as i32;
        Ok(Self {
            date,
            time,
            offset_seconds: seconds,
            label: format!("UTC{suffix}"),
        })
    }

    /// Display localized date/time plus the always-visible supplied UTC offset.
    /// The fixed middle dot separates two independently localized static fieldsets.
    ///
    /// # Errors
    /// Retains the library cause if pinned locale data cannot be constructed.
    pub fn display(
        &self,
        locale: DisplayLocale,
        precision: TimePrecision,
    ) -> Result<String, FormatError> {
        Ok(format!(
            "{} · {} {}",
            self.date.display(locale)?,
            self.time.display(locale, precision)?,
            self.label
        ))
    }

    /// Inspect the explicit offset, in seconds east of UTC, without consulting a clock.
    pub fn offset_seconds(&self) -> i32 {
        self.offset_seconds
    }
}

/// Parse an already bounded nonempty digit field, retaining only safe failures.
fn digits(input: &str, kind: FormatErrorKind) -> Result<u32, FormatError> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(FormatError::new(kind));
    }
    input
        .parse()
        .map_err(|error| FormatError::caused(kind, error))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// Strict calendar values do not overflow into another month or year.
    #[test]
    fn dates_and_errors() {
        let leap = CalendarDate::parse("2024-02-29").unwrap();
        assert_eq!(
            leap.display(DisplayLocale::English).unwrap(),
            "Feb 29, 2024"
        );
        assert_eq!(
            leap.display(DisplayLocale::French).unwrap(),
            "29 févr. 2024"
        );
        for input in [
            "",
            "2023-02-29",
            "1900-02-29",
            "2024-04-31",
            "0000-01-01",
            "2024-00-01",
            "2024-13-01",
            "2024-01-00",
            "2024/01/01",
            "2024-1-01",
            "２０２４-01-01",
        ] {
            assert!(CalendarDate::parse(input).is_err(), "{input}");
        }
        assert!(CalendarDate::parse("0001-01-01").is_ok());
        assert!(CalendarDate::parse("9999-12-31").is_ok());
        assert!(CalendarDate::parse("2000-02-29").is_ok());
        let error = CalendarDate::parse("2023-02-29").unwrap_err();
        assert!(error.source().is_some());
        assert_eq!(error.to_string(), "Invalid calendar date");
    }

    /// Precision truncates without carrying across a day; localized separators are exact.
    #[test]
    fn time_precision() {
        let time = TimeOfDay::parse("23:59:59.987654321").unwrap();
        for (precision, expected) in [
            (TimePrecision::Minutes, "23:59"),
            (TimePrecision::Seconds, "23:59:59"),
            (TimePrecision::Milliseconds, "23:59:59,987"),
            (TimePrecision::Microseconds, "23:59:59,987654"),
            (TimePrecision::Nanoseconds, "23:59:59,987654321"),
        ] {
            assert_eq!(
                time.display(DisplayLocale::French, precision).unwrap(),
                expected
            );
        }
        assert_eq!(
            time.display(DisplayLocale::English, TimePrecision::Seconds)
                .unwrap(),
            "11:59:59\u{202f}PM"
        );
        assert_eq!(
            TimeOfDay::parse("00:00")
                .unwrap()
                .display(DisplayLocale::French, TimePrecision::Milliseconds)
                .unwrap(),
            "00:00:00,000"
        );
        assert!(TimeOfDay::parse("12:34:56.1").is_ok());
        for input in [
            "",
            "1:23",
            "12:",
            "12:34:",
            "24:00",
            "12:60",
            "12:34:60",
            "12:34:56.",
            "12:34:56.1234567890",
            "12-34",
            "12:34:5x",
            "12:34:56.x",
            "12:34Z",
            "１２:34",
        ] {
            assert!(TimeOfDay::parse(input).is_err(), "{input}");
        }
    }

    /// Explicit offsets preserve both sides of spring changes and repeated autumn hours.
    #[test]
    fn dst_boundaries_and_offsets() {
        for input in [
            "2026-03-08T01:59:59-05:00",
            "2026-03-08T03:00:00-04:00",
            "2026-11-01T01:30:00-04:00",
            "2026-11-01T01:30:00-05:00",
            "2026-03-29T01:59:59+01:00",
            "2026-03-29T03:00:00+02:00",
            "2026-10-25T02:30:00+02:00",
            "2026-10-25T02:30:00+01:00",
        ] {
            let value = OffsetDateTime::parse(input).unwrap();
            for locale in [DisplayLocale::English, DisplayLocale::French] {
                assert!(
                    value
                        .display(locale, TimePrecision::Seconds)
                        .unwrap()
                        .ends_with(&format!("UTC{}", &input[input.len() - 6..]))
                );
            }
        }
        let first = OffsetDateTime::parse("2026-11-01T01:30:00-04:00").unwrap();
        let second = OffsetDateTime::parse("2026-11-01T01:30:00-05:00").unwrap();
        assert_eq!(first.offset_seconds(), -14400);
        assert_eq!(second.offset_seconds(), -18000);
        assert_ne!(
            first
                .display(DisplayLocale::English, TimePrecision::Seconds)
                .unwrap(),
            second
                .display(DisplayLocale::English, TimePrecision::Seconds)
                .unwrap()
        );
        for (input, seconds) in [
            ("2024-02-29T00:00Z", 0),
            ("2024-02-29T00:00+14:00", 50400),
            ("2024-02-29T00:00-14:00", -50400),
            ("2024-02-29T00:00+05:45", 20700),
            ("2024-02-29T00:00:00.123456789+00:00", 0),
        ] {
            assert_eq!(
                OffsetDateTime::parse(input).unwrap().offset_seconds(),
                seconds
            );
        }
        for input in [
            "",
            "2024-02-29T12:00",
            "2024-02-29T12:00-00:00",
            "2024-02-29T12:00+14:01",
            "2024-02-29T12:00+00:60",
            "2024-02-29T12:00+15:00",
            "2024-02-29 12:00Z",
            "2024-02-30T12:00Z",
            "2024-02-29T24:00Z",
            "2024-02-29T12:00+aa:00",
            "2024-02-29T12:00*01:00",
            "2024-02-29T12:00+01x00",
            "2024-02-29T12:00Z[Europe/Paris]",
            "2024-02-29T12:00z",
        ] {
            assert!(OffsetDateTime::parse(input).is_err(), "{input}");
        }
    }
}
