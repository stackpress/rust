//! Canonical bounded decimal admission and explicit fixed-precision display.

use super::{DisplayLocale, FormatError, FormatErrorKind, admit_ascii};
use icu_decimal::{DecimalFormatter, input::Decimal, options::GroupingStrategy};

/// Includes sign and point; exponent expansion is deliberately unsupported.
pub const MAX_DECIMAL_BYTES: usize = 128;

/// An admitted decimal string represented exactly, with signed zero preserved.
#[derive(Clone, Debug, PartialEq)]
pub struct DecimalValue(Decimal);

/// Explicit display precision and grouping; rounding is always half to even.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NumberFormat {
    /// Fractional digits, bounded before negating into ICU's position index.
    fraction_digits: u8,
    /// Whether ICU locale grouping is enabled.
    grouping: bool,
}

impl NumberFormat {
    /// Select exactly 0..=18 fraction digits, padding or rounding as necessary.
    ///
    /// # Errors
    /// Rejects precision greater than 18. There is no implicit default precision.
    pub fn new(fraction_digits: u8, grouping: bool) -> Result<Self, FormatError> {
        if fraction_digits > 18 {
            return Err(FormatError::new(FormatErrorKind::Precision));
        }
        Ok(Self {
            fraction_digits,
            grouping,
        })
    }
}

impl DecimalValue {
    /// Borrow-free exact canonical ASCII storage text, including signed zero.
    pub fn canonical(&self) -> String {
        self.0.to_string()
    }
    /// Admit `-?digits(.digits)?`; reject whitespace, plus, exponent and separators.
    ///
    /// # Errors
    /// Rejects empty/partial/malformed or oversized input; never coerces it to zero.
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        admit_ascii(input, MAX_DECIMAL_BYTES, FormatErrorKind::Decimal)?;
        let unsigned = input.strip_prefix('-').unwrap_or(input);
        let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, "0"));
        // Requiring both digit runs also rejects a trailing dot or a second point.
        if integer.is_empty()
            || fraction.is_empty()
            || !integer.bytes().all(|byte| byte.is_ascii_digit())
            || !fraction.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(FormatError::new(FormatErrorKind::Decimal));
        }
        input
            .parse()
            .map(Self)
            .map_err(|error| FormatError::caused(FormatErrorKind::Decimal, error))
    }

    /// Format a copy with pinned locale data, half-even rounding and signed zero.
    ///
    /// # Errors
    /// Retains the library cause if pinned locale data is unavailable.
    pub fn display(
        &self,
        locale: DisplayLocale,
        options: NumberFormat,
    ) -> Result<String, FormatError> {
        let mut value = self.0.clone();
        let position = -i16::from(options.fraction_digits);
        // Round first so padding never preserves discarded fractional digits.
        value.round(position);
        // Canonical display omits input padding without changing signed zero.
        value.trim_start();
        value.pad_end(position);
        let mut icu_options = icu_decimal::options::DecimalFormatterOptions::default();
        icu_options.grouping_strategy = Some(if options.grouping {
            GroupingStrategy::Auto
        } else {
            GroupingStrategy::Never
        });
        let formatter = DecimalFormatter::try_new(locale.locale().into(), icu_options)
            .map_err(|error| FormatError::caused(FormatErrorKind::Data, error))?;
        Ok(formatter.format(&value).to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exact decimals never pass through binary floats or ambient locale rules.
    #[test]
    fn decimal_precision_and_unicode() {
        for (input, precision, en, fr) in [
            (
                "1234567.125",
                2,
                "1,234,567.12",
                "1\u{202f}234\u{202f}567,12",
            ),
            (
                "1234567.135",
                2,
                "1,234,567.14",
                "1\u{202f}234\u{202f}567,14",
            ),
            ("0", 2, "0.00", "0,00"),
            ("-0", 2, "-0.00", "-0,00"),
            ("-0.004", 2, "-0.00", "-0,00"),
            ("999.999", 2, "1,000.00", "1\u{202f}000,00"),
            (
                "9007199254740993",
                0,
                "9,007,199,254,740,993",
                "9\u{202f}007\u{202f}199\u{202f}254\u{202f}740\u{202f}993",
            ),
            ("1.5", 0, "2", "2"),
            ("2.5", 0, "2", "2"),
        ] {
            let value = DecimalValue::parse(input).unwrap();
            let options = NumberFormat::new(precision, true).unwrap();
            assert_eq!(value.display(DisplayLocale::English, options).unwrap(), en);
            assert_eq!(value.display(DisplayLocale::French, options).unwrap(), fr);
        }
        let value = DecimalValue::parse("001234.5").unwrap();
        assert_eq!(
            value
                .display(DisplayLocale::French, NumberFormat::new(3, false).unwrap())
                .unwrap(),
            "1234,500"
        );
        assert_eq!(
            value
                .display(
                    DisplayLocale::English,
                    NumberFormat::new(18, false).unwrap()
                )
                .unwrap(),
            "1234.500000000000000000"
        );
    }

    /// Partial, localized and malicious drafts remain errors rather than valid zeroes.
    #[test]
    fn decimal_admission_and_limits() {
        for input in [
            "", "-", ".", "1.", ".1", "+1", "1e2", "1,234", "1 2", " 0", "NaN", "Infinity", "0x10",
            "١", "1.2.3", "--1",
        ] {
            assert_eq!(
                DecimalValue::parse(input).unwrap_err().kind(),
                FormatErrorKind::Decimal,
                "{input}"
            );
        }
        assert!(DecimalValue::parse(&"9".repeat(MAX_DECIMAL_BYTES)).is_ok());
        assert_eq!(
            DecimalValue::parse(&"9".repeat(MAX_DECIMAL_BYTES + 1))
                .unwrap_err()
                .kind(),
            FormatErrorKind::InputLimit
        );
        assert_eq!(
            NumberFormat::new(19, false).unwrap_err().kind(),
            FormatErrorKind::Precision
        );
    }
}
