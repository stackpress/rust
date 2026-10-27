//! Canonical E.164 display with pinned metadata, without region guessing.

use super::{FormatError, FormatErrorKind, admit_ascii};

/// Canonical `+` and at most fifteen digits, validated against pinned metadata.
/// Validation does not establish assignment, ownership, reachability or consent.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct E164Phone {
    /// Canonical wire value retained unchanged for a separately checked tel link.
    canonical: String,
    /// Metadata-backed parsed value; extensions and local numbers never enter it.
    number: phonenumber::PhoneNumber,
}

impl E164Phone {
    /// Admit only canonical international E.164, never national input or extensions.
    ///
    /// # Errors
    /// Rejects non-ASCII, empty/partial, oversized or metadata-invalid numbers.
    pub fn parse(input: &str) -> Result<Self, FormatError> {
        admit_ascii(input, 16, FormatErrorKind::Phone)?;
        let digits = input.strip_prefix('+').unwrap_or("");
        // Reject normalization inputs so phone parsing cannot silently remove data.
        if digits.is_empty()
            || digits.starts_with('0')
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
        {
            return Err(FormatError::new(FormatErrorKind::Phone));
        }
        let number = phonenumber::parse(None, input)
            .map_err(|error| FormatError::caused(FormatErrorKind::Phone, error))?;
        if !phonenumber::is_valid(&number)
            || number.format().mode(phonenumber::Mode::E164).to_string() != input
        {
            return Err(FormatError::new(FormatErrorKind::Phone));
        }
        Ok(Self {
            canonical: input.to_owned(),
            number,
        })
    }

    /// Borrow the original canonical value; constructing a URL remains caller policy.
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Use the international metadata pattern, identical for English and French.
    /// No translated country names, ambient region, dialing prefix or network lookup.
    pub fn display(&self) -> String {
        self.number
            .format()
            .mode(phonenumber::Mode::International)
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error;

    /// International display is independent of the viewer's region or language.
    #[test]
    fn canonical_and_display() {
        for (input, output) in [
            ("+14155552671", "+1 415-555-2671"),
            ("+33123456789", "+33 1 23 45 67 89"),
            ("+442079460018", "+44 20 7946 0018"),
        ] {
            let phone = E164Phone::parse(input).unwrap();
            assert_eq!(phone.as_str(), input);
            assert_eq!(phone.display(), output);
        }
    }

    /// No lenient parse, extension loss, Unicode digit conversion or invalid-to-zero path.
    #[test]
    fn invalid_and_bounded() {
        for input in [
            "",
            "+",
            "0",
            "+0123",
            "14155552671",
            "+1 4155552671",
            "+14155552671x2",
            "+１４１５５５５２６７１",
            "+1",
            "+99912345678",
            "+10000000000",
        ] {
            assert!(E164Phone::parse(input).is_err(), "{input}");
        }
        assert_eq!(
            E164Phone::parse("+1234567890123456").unwrap_err().kind(),
            FormatErrorKind::InputLimit
        );
        let error = E164Phone::parse("+99912345678").unwrap_err();
        assert!(error.source().is_some());
        assert_eq!(error.to_string(), "Invalid E.164 phone number");
    }
}
