//! Explicit-region national drafts and international prefixes; no extension loss.
use super::{DisplayLocale, EditError, EditPolicy};
use crate::format::E164Phone;

/// An explicitly selected pinned phone metadata region; never browser-negotiated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PhoneRegion(phonenumber::country::Id);
impl PhoneRegion {
    /// Admit exactly a supported uppercase two-letter metadata region.
    /// # Errors
    /// Rejects unknown/noncanonical region identifiers before library parsing.
    pub fn parse(region: &str) -> Result<Self, EditError> {
        if region.len() != 2 || !region.bytes().all(|ch| ch.is_ascii_uppercase()) {
            return Err(EditError::Grammar);
        }
        region.parse().map(Self).map_err(|_| EditError::Invalid)
    }
}
/// Phone editing requires an explicit national-region policy even for `+` drafts.
#[derive(Clone)]
pub struct PhonePolicy {
    region: PhoneRegion,
}
impl PhonePolicy {
    /// Select national parsing context. Dirty region changes are explicitly rejected.
    pub fn new(region: PhoneRegion) -> Self {
        Self { region }
    }
}
impl EditPolicy for PhonePolicy {
    type Value = E164Phone;
    /// Spacing/punctuation cannot turn phone editing into an unbounded document.
    fn max_bytes(&self) -> usize {
        96
    }
    /// Allow national/international prefixes while bounding dialing digits.
    fn max_digits(&self) -> usize {
        20
    }
    /// Bound dialing digits as well as formatting bytes before draft copying.
    fn budget(&self, draft: &str) -> Result<(), EditError> {
        if draft.len() > self.max_bytes()
            || draft.bytes().filter(u8::is_ascii_digit).count() > self.max_digits()
        {
            Err(EditError::Limit)
        } else {
            Ok(())
        }
    }
    /// Validate complete metadata-backed E.164, retaining no extension or dial prefix.
    fn parse(&self, draft: &str, locale: DisplayLocale) -> Result<E164Phone, EditError> {
        self.admit(draft, locale)?;
        let digits = draft.bytes().filter(u8::is_ascii_digit).count();
        if digits < 4 {
            return Err(EditError::Incomplete);
        }
        // 00 is an explicit international paste spelling; national IDD follows region metadata.
        let trimmed = draft.replace([' ', '-', '(', ')', '\u{00a0}', '\u{202f}'], "");
        let international = trimmed.strip_prefix("00").map(|text| format!("+{text}"));
        let input = international.as_deref().unwrap_or(&trimmed);
        let region = if input.starts_with('+') {
            None
        } else {
            Some(self.region.0)
        };
        let parsed = phonenumber::parse(region, input).map_err(|_| EditError::Invalid)?;
        if !phonenumber::is_valid(&parsed) || parsed.extension().is_some() {
            return Err(EditError::Invalid);
        }
        let canonical = parsed.format().mode(phonenumber::Mode::E164).to_string();
        E164Phone::parse(&canonical).map_err(|_| EditError::Invalid)
    }
    /// Allow incomplete international/national drafts; reject all extension spellings.
    fn admit(&self, draft: &str, _locale: DisplayLocale) -> Result<(), EditError> {
        self.budget(draft)?;
        for (index, ch) in draft.char_indices() {
            if !(ch.is_ascii_digit()
                || matches!(ch, ' ' | '-' | '(' | ')' | '\u{00a0}' | '\u{202f}')
                || ch == '+' && index == 0)
            {
                return Err(EditError::Grammar);
            }
        }
        Ok(())
    }
    /// The private phone value is already admitted canonical E.164.
    fn check(&self, _value: &E164Phone) -> Result<(), EditError> {
        Ok(())
    }
    /// International metadata formatting is deterministic across en/fr and regions.
    fn display(&self, value: &E164Phone, _locale: DisplayLocale) -> String {
        value.display()
    }
    /// Submission contains only complete E.164, never national draft text.
    fn canonical(&self, value: &E164Phone) -> String {
        value.as_str().to_owned()
    }
    /// Ignore presentation punctuation; preserve leading international plus identity.
    fn semantic(&self, ch: char, _locale: DisplayLocale) -> bool {
        ch.is_ascii_digit() || ch == '+'
    }
}
