//! UTF-16 admission and semantic caret mapping; no byte/DOM offset conflation.
use super::EditError;

/// Native selection direction, including a collapsed/no-direction selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionDirection {
    /// Selection anchor precedes its focus.
    Forward,
    /// Selection anchor follows its focus.
    Backward,
    /// Collapsed or native direction unavailable.
    #[default]
    None,
}
/// Native start/end are ordered UTF-16 offsets; direction retains anchor/focus order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EditSelection {
    /// Inclusive UTF-16 start.
    pub start: u32,
    /// Exclusive UTF-16 end.
    pub end: u32,
    /// Native anchor/focus orientation.
    pub direction: SelectionDirection,
}
impl EditSelection {
    /// A collapsed selection at the end of a bounded string.
    pub fn end(text: &str) -> Self {
        let n = text.encode_utf16().count() as u32;
        Self {
            start: n,
            end: n,
            direction: SelectionDirection::None,
        }
    }
    /// Convert only actual scalar boundaries, rejecting split surrogate pairs.
    pub(super) fn bytes(self, text: &str) -> Result<(usize, usize), EditError> {
        if self.start > self.end {
            return Err(EditError::Selection);
        }
        let at = |wanted| {
            let mut offset = 0;
            for (byte, ch) in text.char_indices() {
                if offset == wanted {
                    return Some(byte);
                }
                offset += ch.len_utf16() as u32;
            }
            if offset == wanted {
                Some(text.len())
            } else {
                None
            }
        };
        at(self.start).zip(at(self.end)).ok_or(EditError::Selection)
    }
    /// Map semantic token counts, with right affinity across inserted grouping separators.
    /// End-of-text stays at the new end, including rounded/padded fraction digits.
    pub(super) fn remap(
        self,
        old: &str,
        new: &str,
        meaningful: impl Fn(char, bool) -> bool,
    ) -> Self {
        let map = |offset| {
            if offset == 0 {
                return 0;
            }
            if offset >= old.encode_utf16().count() as u32 {
                return new.encode_utf16().count() as u32;
            }
            let mut units = 0;
            let mut tokens = 0;
            for ch in old.chars() {
                if units >= offset {
                    break;
                }
                units += ch.len_utf16() as u32;
                if meaningful(ch, false) {
                    tokens += 1;
                }
            }
            let mut seen = 0;
            let mut position = 0;
            for ch in new.chars() {
                if meaningful(ch, true) {
                    if seen == tokens {
                        break;
                    }
                    seen += 1;
                }
                position += ch.len_utf16() as u32;
            }
            position
        };
        Self {
            start: map(self.start),
            end: map(self.end),
            direction: self.direction,
        }
    }
}
