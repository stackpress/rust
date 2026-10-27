//! Opt-in deterministic display with pinned ICU data and phone metadata.
//!
//! These types admit canonical values, never partial input drafts. No formatter
//! reads the clock, OS/browser locale or ambient timezone. Dates use the Gregorian
//! calendar; instants carry an explicit numeric UTC offset. Named-zone conversion,
//! monetary arithmetic, reachability checks and specialized editing are outside
//! this module. Returned strings are text and must never be inserted as HTML.
//!
//! ```
//! use stackpress_framework_ui::format::{DecimalValue, DisplayLocale, NumberFormat};
//! let number = DecimalValue::parse("1234.50")?;
//! assert_eq!(number.display(DisplayLocale::French, NumberFormat::new(2, true)?)?,
//!            "1\u{202f}234,50");
//! # Ok::<(), stackpress_framework_ui::format::FormatError>(())
//! ```

use std::{error::Error, fmt};

#[path = "format_number.rs"]
mod number;
#[path = "format_phone.rs"]
mod phone;
#[path = "format_time.rs"]
mod time;

pub use number::{DecimalValue, MAX_DECIMAL_BYTES, NumberFormat};
pub use phone::E164Phone;
pub use time::{CalendarDate, OffsetDateTime, TimeOfDay, TimePrecision};

/// The deliberately closed initial locale set; aliases are resolved by the host.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DisplayLocale {
    /// ICU's pinned English data and default English hour cycle.
    English,
    /// ICU's pinned French data, including Unicode separators.
    French,
}

impl DisplayLocale {
    /// Convert the explicit selection without browser negotiation or fallback.
    fn locale(self) -> icu_locale::Locale {
        match self {
            Self::English => icu_locale::locale!("en"),
            Self::French => icu_locale::locale!("fr"),
        }
    }
}

/// Public safe category; diagnostics never include the input value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatErrorKind {
    /// Admission rejected the byte count before parsing or allocation.
    InputLimit,
    /// A decimal was empty, partial or outside the canonical ASCII grammar.
    Decimal,
    /// Requested decimal fraction digits exceeded the supported 0..=18 range.
    Precision,
    /// An ISO calendar date was malformed or impossible.
    Date,
    /// An ISO time was malformed, out of range or a leap second.
    Time,
    /// A datetime lacked its explicit offset or used an unsupported offset.
    Offset,
    /// A phone was not canonical E.164 or failed the pinned metadata validation.
    Phone,
    /// Pinned formatter data could not be constructed.
    Data,
}

/// Safe display diagnostic with the original library error retained as a cause.
/// No input strings are attached by this module; hosts keep causes out of public DTOs.
#[derive(Debug)]
pub struct FormatError {
    /// Stable caller-facing classification.
    kind: FormatErrorKind,
    /// Original parsing/data failure, never flattened to text or serialized.
    cause: Option<Box<dyn Error + Send + Sync>>,
}

impl FormatError {
    /// Construct an admission error before invoking a dependency.
    fn new(kind: FormatErrorKind) -> Self {
        Self { kind, cause: None }
    }

    /// Preserve an actual library failure behind the safe public category.
    fn caused(kind: FormatErrorKind, cause: impl Error + Send + Sync + 'static) -> Self {
        Self {
            kind,
            cause: Some(Box::new(cause)),
        }
    }

    /// Inspect the safe category without exposing diagnostic causes.
    pub fn kind(&self) -> FormatErrorKind {
        self.kind
    }
}

impl fmt::Display for FormatError {
    /// Render only a fixed message; malformed content cannot become a diagnostic.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self.kind {
            FormatErrorKind::InputLimit => "Formatting input too large",
            FormatErrorKind::Decimal => "Invalid decimal value",
            FormatErrorKind::Precision => "Invalid formatting precision",
            FormatErrorKind::Date => "Invalid calendar date",
            FormatErrorKind::Time => "Invalid time value",
            FormatErrorKind::Offset => "Invalid explicit UTC offset",
            FormatErrorKind::Phone => "Invalid E.164 phone number",
            FormatErrorKind::Data => "Formatting data unavailable",
        })
    }
}

impl Error for FormatError {
    /// Retain typed upstream causes for native diagnostics without exposing them publicly.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.cause.as_deref().map(|cause| cause as &dyn Error)
    }
}

/// Admit a fixed ASCII wire grammar before callers perform byte indexing.
fn admit_ascii(input: &str, max: usize, kind: FormatErrorKind) -> Result<(), FormatError> {
    // Byte limits precede character scans and dependency parsing.
    if input.len() > max {
        return Err(FormatError::new(FormatErrorKind::InputLimit));
    }
    if !input.is_ascii() {
        return Err(FormatError::new(kind));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Public diagnostics stay fixed while the native cause remains downcastable.
    #[test]
    fn error_categories_and_retained_cause() {
        for (kind, message) in [
            (FormatErrorKind::InputLimit, "Formatting input too large"),
            (FormatErrorKind::Decimal, "Invalid decimal value"),
            (FormatErrorKind::Precision, "Invalid formatting precision"),
            (FormatErrorKind::Date, "Invalid calendar date"),
            (FormatErrorKind::Time, "Invalid time value"),
            (FormatErrorKind::Offset, "Invalid explicit UTC offset"),
            (FormatErrorKind::Phone, "Invalid E.164 phone number"),
            (FormatErrorKind::Data, "Formatting data unavailable"),
        ] {
            let error = FormatError::new(kind);
            assert_eq!(error.kind(), kind);
            assert_eq!(error.to_string(), message);
            assert!(error.source().is_none());
        }
        let error = FormatError::caused(
            FormatErrorKind::Data,
            std::io::Error::other("private diagnostic"),
        );
        assert_eq!(error.to_string(), "Formatting data unavailable");
        assert!(
            error
                .source()
                .unwrap()
                .downcast_ref::<std::io::Error>()
                .is_some()
        );
    }

    /// Every field boundary rejects injected Unicode/control bytes before slicing.
    #[test]
    fn strict_wire_boundaries_preserve_utf8_safety() {
        for (kind, input) in [
            (FormatErrorKind::Date, "2024-02-29"),
            (FormatErrorKind::Time, "23:59:59.123456789"),
            (
                FormatErrorKind::Offset,
                "2024-02-29T23:59:59.123456789+05:45",
            ),
        ] {
            for index in 0..=input.len() {
                for character in ['\0', '\t', '\n', '\r', ' ', '\u{7f}', 'é', '𝟘'] {
                    let mut hostile = input.to_owned();
                    hostile.insert(index, character);
                    let error = match kind {
                        FormatErrorKind::Date => CalendarDate::parse(&hostile).unwrap_err(),
                        FormatErrorKind::Time => TimeOfDay::parse(&hostile).unwrap_err(),
                        FormatErrorKind::Offset => OffsetDateTime::parse(&hostile).unwrap_err(),
                        _ => unreachable!("fixed fixture categories"),
                    };
                    assert!(matches!(
                        error.kind(),
                        FormatErrorKind::InputLimit
                            | FormatErrorKind::Date
                            | FormatErrorKind::Time
                            | FormatErrorKind::Offset
                    ));
                }
            }
        }
    }
}
