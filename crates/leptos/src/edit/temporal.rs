//! ISO text editing with explicit precision and exact supplied-offset ordering.
use super::{DisplayLocale, EditError, EditPolicy, check_bounds};
use crate::format::{CalendarDate, OffsetDateTime, TimeOfDay};
use std::ops::Bound;

/// Closed text grammar; native date pickers/ambient timezone conversion are not involved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalKind {
    /// Gregorian ISO calendar date, 0001..9999.
    Date,
    /// Local clock time, with no date or offset.
    Time,
    /// Calendar and clock with an explicit numeric UTC offset.
    Instant,
}
/// Validated ISO text plus exact comparison key; no floating point or ambient clock.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IsoValue {
    kind: TemporalKind,
    wire: String,
    order: i128,
}
impl IsoValue {
    /// Admit a complete typed bound/storage value before constructing an editor.
    /// # Errors
    /// Rejects impossible fields, leap seconds and missing/unknown/out-of-range offsets.
    pub fn parse(kind: TemporalKind, text: &str) -> Result<Self, EditError> {
        if text.len() > 35 {
            return Err(EditError::Limit);
        }
        let offset = match kind {
            TemporalKind::Date => {
                CalendarDate::parse(text).map_err(|_| EditError::Invalid)?;
                0
            }
            TemporalKind::Time => {
                TimeOfDay::parse(text).map_err(|_| EditError::Invalid)?;
                0
            }
            TemporalKind::Instant => OffsetDateTime::parse(text)
                .map_err(|_| EditError::Invalid)?
                .offset_seconds(),
        };
        // The strict upstream admission above establishes all ASCII slicing invariants.
        let number = |s: &str| s.parse::<i128>().expect("admitted ISO digits");
        let mut order = 0;
        if kind != TemporalKind::Time {
            let y = number(&text[..4]);
            let m = number(&text[5..7]);
            let d = number(&text[8..10]);
            let previous = y - 1;
            let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
            let days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
            order = (365 * previous + previous / 4 - previous / 100
                + previous / 400
                + days[(m - 1) as usize]
                + i128::from(leap && m > 2)
                + d
                - 1)
                * 86_400
                * 1_000_000_000;
        }
        if kind != TemporalKind::Date {
            let time = if kind == TemporalKind::Time {
                text
            } else {
                &text[11..]
            };
            let time = if kind == TemporalKind::Instant {
                if let Some(t) = time.strip_suffix('Z') {
                    t
                } else {
                    &time[..time.len() - 6]
                }
            } else {
                time
            };
            let hour = number(&time[..2]);
            let minute = number(&time[3..5]);
            let second = if time.len() >= 8 {
                number(&time[6..8])
            } else {
                0
            };
            let fraction = if time.len() > 8 {
                let fraction = &time[9..];
                number(fraction) * 10_i128.pow(9 - fraction.len() as u32)
            } else {
                0
            };
            order += ((hour * 3600 + minute * 60 + second) - i128::from(offset)) * 1_000_000_000
                + fraction;
        }
        Ok(Self {
            kind,
            wire: text.to_owned(),
            order,
        })
    }
    /// Canonical checked storage text; no localized date string is persisted.
    pub fn as_str(&self) -> &str {
        &self.wire
    }
}
/// Typed min/max are compared as dates, local times or offset-correct instants.
#[derive(Clone)]
pub struct TemporalPolicy {
    kind: TemporalKind,
    precision: u8,
    min: Bound<IsoValue>,
    max: Bound<IsoValue>,
}
impl TemporalPolicy {
    /// Choose ISO text editing, 0..=9 fraction digits and explicit typed bounds.
    /// Date precision must be zero. Time commits always include seconds.
    /// # Errors
    /// Rejects precision, mixed temporal kinds and inverted/empty bounds.
    pub fn new(
        kind: TemporalKind,
        precision: u8,
        min: Bound<IsoValue>,
        max: Bound<IsoValue>,
    ) -> Result<Self, EditError> {
        if precision > 9 || kind == TemporalKind::Date && precision != 0 {
            return Err(EditError::Precision);
        }
        for bound in [&min, &max] {
            if let Bound::Included(v) | Bound::Excluded(v) = bound
                && v.kind != kind
            {
                return Err(EditError::Bounds);
            }
        }
        if let (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) =
            (&min, &max)
            && (a.order > b.order
                || a.order == b.order
                    && (matches!(min, Bound::Excluded(_)) || matches!(max, Bound::Excluded(_))))
        {
            return Err(EditError::Bounds);
        }
        Ok(Self {
            kind,
            precision,
            min,
            max,
        })
    }
}
impl EditPolicy for TemporalPolicy {
    type Value = IsoValue;
    /// Calendar/time/instant maximal wire lengths, also bounding partial drafts.
    fn max_bytes(&self) -> usize {
        match self.kind {
            TemporalKind::Date => 10,
            TemporalKind::Time => 18,
            TemporalKind::Instant => 35,
        }
    }
    /// Validate fully, pad only admitted precision and normalize explicit zero offsets.
    fn parse(&self, draft: &str, locale: DisplayLocale) -> Result<IsoValue, EditError> {
        self.admit(draft, locale)?;
        let value = IsoValue::parse(self.kind, draft).map_err(|_| {
            if draft.len()
                < match self.kind {
                    TemporalKind::Date => 10,
                    TemporalKind::Time => 5,
                    TemporalKind::Instant => 17,
                }
            {
                EditError::Incomplete
            } else {
                EditError::Invalid
            }
        })?;
        let wire = if self.kind == TemporalKind::Date {
            draft.to_owned()
        } else {
            let (date, time) = if self.kind == TemporalKind::Instant {
                (&draft[..11], &draft[11..])
            } else {
                ("", draft)
            };
            let (time, suffix) = if self.kind == TemporalKind::Instant {
                if let Some(t) = time.strip_suffix('Z') {
                    (t, "+00:00")
                } else {
                    time.split_at(time.len() - 6)
                }
            } else {
                (time, "")
            };
            let (whole, fraction) = time.split_once('.').unwrap_or((time, ""));
            if fraction.len() > usize::from(self.precision) {
                return Err(EditError::Precision);
            }
            let whole = if whole.len() == 5 {
                format!("{whole}:00")
            } else {
                whole.to_owned()
            };
            let fraction = if self.precision == 0 {
                String::new()
            } else {
                format!(".{fraction:0<width$}", width = usize::from(self.precision))
            };
            format!("{date}{whole}{fraction}{suffix}")
        };
        self.check(&value)?;
        IsoValue::parse(self.kind, &wire)
    }
    /// Preserve incomplete fixed-field prefixes; invalid full calendars fail at commit.
    fn admit(&self, draft: &str, _locale: DisplayLocale) -> Result<(), EditError> {
        if draft.len() > self.max_bytes() {
            return Err(EditError::Limit);
        }
        if !draft.is_ascii() {
            return Err(EditError::Grammar);
        }
        let pattern = match self.kind {
            TemporalKind::Date => "dddd-dd-dd",
            TemporalKind::Time => "dd:dd:dd.ddddddddd",
            TemporalKind::Instant => "dddd-dd-ddTdd:dd:dd.ddddddddd",
        };
        let mut offset = None;
        for (i, ch) in draft.bytes().enumerate() {
            if self.kind == TemporalKind::Instant && i >= 16 && matches!(ch, b'+' | b'-' | b'Z') {
                offset = Some(i);
                break;
            }
            // A fixed prefix slot is either one ASCII digit or the literal ISO separator.
            let Some(expected) = pattern.as_bytes().get(i) else {
                return Err(EditError::Grammar);
            };
            let valid = if *expected == b'd' {
                ch.is_ascii_digit()
            } else {
                *expected == ch
            };
            if !valid {
                return Err(EditError::Grammar);
            }
        }
        if let Some(index) = offset {
            let suffix = &draft[index..];
            if suffix.starts_with('Z') && suffix != "Z" {
                return Err(EditError::Grammar);
            }
            if suffix != "Z" {
                let bytes = suffix.as_bytes();
                if bytes.len() > 6 {
                    return Err(EditError::Grammar);
                }
                for (i, ch) in bytes.iter().enumerate().skip(1) {
                    if i == 3 {
                        if *ch != b':' {
                            return Err(EditError::Grammar);
                        }
                    } else if !ch.is_ascii_digit() {
                        return Err(EditError::Grammar);
                    }
                }
            }
        }
        Ok(())
    }
    /// Compare exact validated keys, including explicitly supplied offset identity.
    fn check(&self, value: &IsoValue) -> Result<(), EditError> {
        if value.kind != self.kind {
            return Err(EditError::Bounds);
        }
        // Initial/reconfigured typed values obey the same precision rule as commits.
        if let Some((_, fraction)) = value.wire.split_once('.') {
            let digits = fraction.bytes().take_while(u8::is_ascii_digit).count();
            if digits > usize::from(self.precision) {
                return Err(EditError::Precision);
            }
        }
        check_bounds(value, &self.min, &self.max, |a, b| a.order.cmp(&b.order))
    }
    /// ISO field text remains explicit and stable at both supported display locales.
    fn display(&self, value: &IsoValue, _locale: DisplayLocale) -> String {
        value.wire.clone()
    }
    /// Submit validated ISO only, with an explicit offset for instants.
    fn canonical(&self, value: &IsoValue) -> String {
        value.wire.clone()
    }
    /// ISO separators identify fields rather than optional grouping.
    fn semantic(&self, _ch: char, _locale: DisplayLocale) -> bool {
        true
    }
}
