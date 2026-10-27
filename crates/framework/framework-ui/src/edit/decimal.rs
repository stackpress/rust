//! Explicit en/fr draft grammar, exact decimal bounds and half-even commit.
use super::{DisplayLocale, EditError, EditPolicy, check_bounds};
use crate::format::{DecimalValue, NumberFormat};
use std::{cmp::Ordering, ops::Bound};

/// Checked decimal policy; raw edits are bounded independently of fixed commit scale.
#[derive(Clone)]
pub struct DecimalPolicy {
    scale: u8,
    digits: u8,
    min: Bound<DecimalValue>,
    max: Bound<DecimalValue>,
}
impl DecimalPolicy {
    /// Configure 0..=18 scale, 1..=96 total draft digits and typed open/closed bounds.
    /// # Errors
    /// Rejects unsupported precision/digit budgets and inverted/empty bounds.
    pub fn new(
        scale: u8,
        digits: u8,
        min: Bound<DecimalValue>,
        max: Bound<DecimalValue>,
    ) -> Result<Self, EditError> {
        if scale > 18 {
            return Err(EditError::Precision);
        }
        if digits == 0 || digits > 96 || digits <= scale {
            return Err(EditError::Limit);
        }
        if let (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) =
            (&min, &max)
        {
            let order = compare(a, b);
            if order == Ordering::Greater
                || order == Ordering::Equal
                    && (matches!(min, Bound::Excluded(_)) || matches!(max, Bound::Excluded(_)))
            {
                return Err(EditError::Bounds);
            }
        }
        Ok(Self {
            scale,
            digits,
            min,
            max,
        })
    }
    /// Normalize admitted separators only; no whitespace trimming or float coercion.
    fn canonical_draft(
        &self,
        draft: &str,
        locale: DisplayLocale,
        complete: bool,
        strict_groups: bool,
    ) -> Result<String, EditError> {
        self.budget(draft)?;
        let unsigned = draft.strip_prefix(['-', '+']).unwrap_or(draft);
        let decimal = match locale {
            DisplayLocale::English => '.',
            DisplayLocale::French => ',',
        };
        let mut parts = unsigned.split(decimal);
        let integer = parts.next().unwrap_or_default();
        let fraction = parts.next();
        if parts.next().is_some()
            || fraction.is_some_and(|text| !text.bytes().all(|b| b.is_ascii_digit()))
        {
            return Err(EditError::Grammar);
        }
        let mut group = None;
        for ch in integer.chars() {
            if ch.is_ascii_digit() {
                continue;
            }
            let allowed = match locale {
                DisplayLocale::English => ch == ',',
                DisplayLocale::French => matches!(ch, '\u{202f}' | '\u{00a0}'),
            };
            if !allowed || group.is_some_and(|g| g != ch) {
                return Err(EditError::Grammar);
            }
            group = Some(ch);
        }
        if let Some(group) = group {
            let groups: Vec<_> = integer.split(group).collect();
            // Native edits may move old grouping boundaries; pasted groups are strict.
            if groups.iter().any(|part| part.is_empty())
                || strict_groups
                    && (groups[0].len() > 3 || groups.iter().skip(1).any(|part| part.len() != 3))
            {
                return Err(EditError::Grammar);
            }
        }
        if complete
            && (unsigned.is_empty()
                || fraction == Some("")
                || integer.is_empty() && fraction.is_none())
        {
            return Err(EditError::Incomplete);
        }
        let mut canonical = String::with_capacity(draft.len());
        if draft.starts_with('-') {
            canonical.push('-');
        }
        if integer.is_empty() {
            canonical.push('0');
        } else {
            canonical.extend(integer.chars().filter(char::is_ascii_digit));
        }
        if let Some(fraction) = fraction {
            canonical.push('.');
            canonical.push_str(fraction);
        }
        Ok(canonical)
    }
}
impl EditPolicy for DecimalPolicy {
    type Value = DecimalValue;
    /// Unicode grouping bytes still fit a fixed pre-copy budget.
    fn max_bytes(&self) -> usize {
        384
    }
    /// Digit count is separate from UTF-8 grouping bytes.
    fn max_digits(&self) -> usize {
        usize::from(self.digits)
    }
    /// Count ASCII draft digits before any controller-owned allocation.
    fn budget(&self, draft: &str) -> Result<(), EditError> {
        if draft.len() > self.max_bytes()
            || draft.bytes().filter(u8::is_ascii_digit).count() > self.max_digits()
        {
            Err(EditError::Limit)
        } else {
            Ok(())
        }
    }
    /// Half-even rounding precedes typed bound checks, preserving signed zero.
    fn parse(&self, draft: &str, locale: DisplayLocale) -> Result<DecimalValue, EditError> {
        let wire = self.canonical_draft(draft, locale, true, false)?;
        let value = DecimalValue::parse(&wire).map_err(|_| EditError::Grammar)?;
        let rounded = value
            .display(
                DisplayLocale::English,
                NumberFormat::new(self.scale, false).map_err(|_| EditError::Precision)?,
            )
            .map_err(|_| EditError::Invalid)?;
        let value = DecimalValue::parse(&rounded).map_err(|_| EditError::Invalid)?;
        self.check(&value)?;
        Ok(value)
    }
    /// Partial signs/points/group tails remain drafts; mixed separators are rejected.
    fn admit(&self, draft: &str, locale: DisplayLocale) -> Result<(), EditError> {
        self.canonical_draft(draft, locale, false, true).map(|_| ())
    }
    /// Old grouping may shift during native middle edits; commit regroups exact digits.
    fn admit_input(&self, draft: &str, locale: DisplayLocale) -> Result<(), EditError> {
        self.canonical_draft(draft, locale, false, false)
            .map(|_| ())
    }
    /// Fixed scale and exact signed decimal bounds never use binary floating point.
    fn check(&self, value: &DecimalValue) -> Result<(), EditError> {
        let wire = value.canonical();
        if wire.bytes().filter(u8::is_ascii_digit).count() > self.max_digits() {
            return Err(EditError::Limit);
        }
        if wire
            .split_once('.')
            .is_some_and(|(_, fraction)| fraction.len() > usize::from(self.scale))
        {
            return Err(EditError::Precision);
        }
        check_bounds(value, &self.min, &self.max, compare)
    }
    /// Group only on commit/clean locale changes, leaving native edit/undo intact.
    fn display(&self, value: &DecimalValue, locale: DisplayLocale) -> String {
        value
            .display(
                locale,
                NumberFormat::new(self.scale, true).expect("checked scale"),
            )
            .expect("compiled en/fr data")
    }
    /// Persist the admitted ASCII decimal with its committed scale/sign.
    fn canonical(&self, value: &DecimalValue) -> String {
        value.canonical()
    }
    /// Grouping is presentation; signs/digits/decimal marks preserve semantic positions.
    fn semantic(&self, ch: char, locale: DisplayLocale) -> bool {
        ch.is_ascii_digit()
            || ch == '-'
            || ch
                == match locale {
                    DisplayLocale::English => '.',
                    DisplayLocale::French => ',',
                }
    }
}
/// Compare bounded canonical decimals by sign, integer magnitude and padded fractions.
fn compare(a: &DecimalValue, b: &DecimalValue) -> Ordering {
    let a = a.canonical();
    let b = b.canonical();
    let parts = |s: &str| {
        let (integer, fraction) = s
            .trim_start_matches('-')
            .split_once('.')
            .unwrap_or((s.trim_start_matches('-'), ""));
        (
            integer.trim_start_matches('0').to_owned(),
            fraction.trim_end_matches('0').to_owned(),
        )
    };
    let (ai, af) = parts(&a);
    let (bi, bf) = parts(&b);
    let an = a.starts_with('-') && (!ai.is_empty() || !af.is_empty());
    let bn = b.starts_with('-') && (!bi.is_empty() || !bf.is_empty());
    if an != bn {
        return if an {
            Ordering::Less
        } else {
            Ordering::Greater
        };
    }
    let order = ai
        .len()
        .cmp(&bi.len())
        .then_with(|| ai.cmp(&bi))
        .then_with(|| {
            af.bytes()
                .chain(std::iter::repeat(b'0'))
                .take(af.len().max(bf.len()))
                .cmp(
                    bf.bytes()
                        .chain(std::iter::repeat(b'0'))
                        .take(af.len().max(bf.len())),
                )
        });
    if an { order.reverse() } else { order }
}
