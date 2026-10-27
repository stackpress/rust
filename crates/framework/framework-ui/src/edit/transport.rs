//! Pre-copy UTF-16 admission for JavaScript-owned native/clipboard strings.
use super::EditError;

/// Count exact UTF-8 bytes and ASCII digits without allocating a replacement string.
/// Malformed surrogate sequences are rejected instead of silently becoming U+FFFD.
pub(super) fn admit_utf16(
    units: impl Iterator<Item = u16>,
    bytes_limit: usize,
    digits_limit: usize,
) -> Result<(), EditError> {
    let mut units = units;
    let mut bytes = 0usize;
    let mut digits = 0usize;
    while let Some(unit) = units.next() {
        // JavaScript lengths are UTF-16 units; a valid surrogate pair encodes four bytes.
        let width = if (0xd800..=0xdbff).contains(&unit) {
            if !units
                .next()
                .is_some_and(|low| (0xdc00..=0xdfff).contains(&low))
            {
                return Err(EditError::Grammar);
            }
            4
        } else if (0xdc00..=0xdfff).contains(&unit) {
            return Err(EditError::Grammar);
        } else if unit < 0x80 {
            1
        } else if unit < 0x800 {
            2
        } else {
            3
        };
        bytes = bytes.checked_add(width).ok_or(EditError::Limit)?;
        if (u16::from(b'0')..=u16::from(b'9')).contains(&unit) {
            digits += 1;
        }
        if bytes > bytes_limit || digits > digits_limit {
            return Err(EditError::Limit);
        }
    }
    Ok(())
}
