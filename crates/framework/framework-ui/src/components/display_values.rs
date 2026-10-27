//! Checked values for ordinary safe display; no ambient locale or browser parser.
//!
//! ```
//! use stackpress_framework_ui::components::{ProgressValue, PhoneNumber, TextLimit, overflow_text};
//! assert!(ProgressValue::determinate(0.0, 10.0).is_ok());
//! assert!(ProgressValue::determinate(f64::NAN, 10.0).is_err());
//! assert_eq!(PhoneNumber::new("+12025550123")?.href(), "tel:+12025550123");
//! assert_eq!(overflow_text("雪😀abc", TextLimit::Characters(2), true), "雪😀…");
//! # Ok::<(), stackpress_framework_ui::components::DisplayError>(())
//! ```
use stackpress_framework_view::PublicLocation;
use std::collections::BTreeSet;

/// Invalid/excessive display data; never interpolates untrusted input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DisplayError {
    /// Invalid or excessive public display data.
    InvalidValue,
    /// Preserve the concrete checked-location cause without reflecting its text.
    Location(stackpress_framework_view::ViewError),
}
impl std::fmt::Display for DisplayError {
    /// Keep public failures independent of private input.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid display value")
    }
}
impl std::error::Error for DisplayError {
    /// Checked URL failures retain their concrete cause for native diagnostics.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidValue => None,
            Self::Location(cause) => Some(cause),
        }
    }
}

/// Finite native progress state; indeterminate is distinct from zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ProgressValue {
    /// Work has no known total yet.
    Indeterminate,
    /// A checked numerator and strictly positive total.
    Determinate(ProgressAmount),
}
/// Checked progress bounds, never clamped from an invalid value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ProgressAmount {
    /// Finite admitted numerator or exact numeric rating.
    pub(super) value: f64,
    /// Strictly positive finite progress total.
    pub(super) max: f64,
}
impl ProgressValue {
    /// Construct finite inclusive bounds; zero is a valid completed-work count.
    /// # Errors
    /// Rejects NaN/infinity, nonpositive total or values outside 0..=max.
    pub fn determinate(value: f64, max: f64) -> Result<Self, DisplayError> {
        if !value.is_finite() || !max.is_finite() || max <= 0.0 || value < 0.0 || value > max {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self::Determinate(ProgressAmount { value, max }))
    }
    /// Native value attribute; None retains indeterminate semantics.
    pub(super) fn value(self) -> Option<f64> {
        match self {
            Self::Indeterminate => None,
            Self::Determinate(value) => Some(value.value),
        }
    }
    /// Native max attribute, including a harmless indeterminate default.
    pub(super) fn max(self) -> f64 {
        match self {
            Self::Indeterminate => 1.0,
            Self::Determinate(value) => value.max,
        }
    }
}

/// Rounding policy for a read-only star display, never an editing input.
#[derive(Clone, Copy, Debug)]
pub enum RatingRound {
    /// Nearest integer, halves away from zero.
    Nearest,
    /// Round upward.
    Ceil,
    /// Round downward.
    Floor,
}
/// Checked rating and bounded star count.
#[derive(Clone, Copy, Debug)]
pub struct RatingValue {
    /// Finite admitted numerator or exact numeric rating.
    pub(super) value: f64,
    /// Admitted star ceiling, in 1..=100.
    pub(super) max: u8,
}
impl RatingValue {
    /// Admit finite ratings with 1..=100 stars and an inclusive 0..=max value.
    /// # Errors
    /// Invalid values stay invalid; no invalid-to-zero fallback exists.
    pub fn new(value: f64, max: u8) -> Result<Self, DisplayError> {
        if max == 0 || max > 100 || !value.is_finite() || value < 0.0 || value > f64::from(max) {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self { value, max })
    }
    /// Render the selected bounded rounding policy and optional unfilled stars.
    pub fn stars(self, round: RatingRound, remainder: bool) -> String {
        let filled = match round {
            RatingRound::Nearest => self.value.round(),
            RatingRound::Ceil => self.value.ceil(),
            RatingRound::Floor => self.value.floor(),
        } as usize;
        let empty = if remainder {
            usize::from(self.max) - filled
        } else {
            0
        };
        format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
    }
}

/// Restricted hex color for an inert swatch; arbitrary CSS is never accepted.
#[derive(Clone, Debug)]
pub struct HexColor(String);
impl HexColor {
    /// Admit #RGB or #RRGGBB, retaining exact source spelling for text output.
    /// # Errors
    /// Rejects other CSS tokens, functions and style syntax.
    pub fn new(value: impl Into<String>) -> Result<Self, DisplayError> {
        let value = value.into();
        if !matches!(value.len(), 4 | 7)
            || !value.starts_with('#')
            || !value.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self(value))
    }
    /// Borrow only the checked token.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// ASCII mailbox with a closed local-part/domain policy; no headers/query syntax.
#[derive(Clone, Debug)]
pub struct EmailAddress(String);
impl EmailAddress {
    /// Admit a simple ASCII mailbox; international mailbox routing needs a separate adapter.
    /// # Errors
    /// Rejects controls, display names, quoted local parts and malformed domain labels.
    pub fn new(value: impl Into<String>) -> Result<Self, DisplayError> {
        let value = value.into();
        let Some((local, domain)) = value.split_once('@') else {
            return Err(DisplayError::InvalidValue);
        };
        let atom = |b: u8| b.is_ascii_alphanumeric() || b".!#$%&'*+-/=?^_`{|}~".contains(&b);
        if value.len() > 254
            || local.is_empty()
            || local.len() > 64
            || !local.bytes().all(atom)
            || local.starts_with('.')
            || local.ends_with('.')
            || local.contains("..")
            || domain.is_empty()
            || domain.split('.').any(|part| {
                part.is_empty()
                    || part.len() > 63
                    || part.starts_with('-')
                    || part.ends_with('-')
                    || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
            })
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self(value))
    }
    /// Exact safe visible mailbox text.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Percent encode all URI-sensitive local-part characters; no header injection.
    pub fn href(&self) -> String {
        let mut out = "mailto:".to_owned();
        for byte in self.0.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~@".contains(&byte) {
                out.push(char::from(byte));
            } else {
                out.push_str(&format!("%{byte:02X}"));
            }
        }
        out
    }
}
/// Canonical international phone token; formatting/region inference is M3.
#[derive(Clone, Debug)]
pub struct PhoneNumber(String);
impl PhoneNumber {
    /// Admit + followed by 2..=15 digits, beginning with a nonzero digit.
    /// # Errors
    /// Rejects extensions, punctuation, query syntax and implicit country guessing.
    pub fn new(value: impl Into<String>) -> Result<Self, DisplayError> {
        let value = value.into();
        if !(3..=16).contains(&value.len())
            || !value.starts_with('+')
            || value.as_bytes()[1] == b'0'
            || !value.as_bytes()[1..].iter().all(u8::is_ascii_digit)
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self(value))
    }
    /// Borrow the canonical visible value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// A dedicated tel URL never broadens generic safe_href.
    pub fn href(&self) -> String {
        format!("tel:{}", self.0)
    }
}

/// A host-declared same-origin raster asset, with bounded declared bytes and dimensions.
///
/// The host must verify real MIME/bytes against its asset manifest. An img element
/// cannot enforce a download-byte budget; this type validates trusted metadata.
#[derive(Clone, Debug)]
pub struct ImageSource {
    /// Checked app-local raster path from trusted asset metadata.
    pub(super) path: String,
    /// Admitted width in 1..=4096 pixels.
    pub(super) width: u32,
    /// Admitted height in 1..=4096 pixels.
    pub(super) height: u32,
}
impl ImageSource {
    /// Admit a local PNG/JPEG/WebP/AVIF path, <=1 MiB and 1..=4096 pixels per side.
    /// # Errors
    /// Rejects external/data/blob/SVG sources, query/fragment tricks and excess budgets.
    pub fn new(
        path: impl Into<String>,
        bytes: usize,
        width: u32,
        height: u32,
    ) -> Result<Self, DisplayError> {
        let path = path.into();
        PublicLocation::new(&path).map_err(DisplayError::Location)?;
        if path.contains(['?', '#'])
            || ![".png", ".jpg", ".jpeg", ".webp", ".avif"]
                .iter()
                .any(|suffix| path.ends_with(suffix))
            || bytes == 0
            || bytes > 1024 * 1024
            || !(1..=4096).contains(&width)
            || !(1..=4096).contains(&height)
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self {
            path,
            width,
            height,
        })
    }
}

/// At most 100 bounded text values; static display identity has no editable state.
#[derive(Clone, Debug)]
pub struct DisplayValues(Vec<String>);
impl DisplayValues {
    /// Admit strings without truthiness conversion or active markup interpretation.
    /// # Errors
    /// Rejects more than 100 values or individual values over 4096 bytes.
    pub fn new(values: Vec<String>) -> Result<Self, DisplayError> {
        if values.len() > 100 || values.iter().any(|value| value.len() > 4096) {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self(values))
    }
    /// Borrow exact values, preserving empty, zero text and duplicates.
    pub fn entries(&self) -> &[String] {
        &self.0
    }
}
/// One immutable row with a stable application identity and ordered cells.
#[derive(Clone, Debug)]
pub struct DisplayRow {
    /// Unique row identity.
    pub key: String,
    /// Exact ordered cell text, including empty/zero.
    pub cells: Vec<String>,
}
/// Validated read-only tabular/metadata data; no data operations or generated CRUD.
#[derive(Clone, Debug)]
pub struct DisplayTable {
    /// Unique bounded column labels in presentation order.
    pub(super) columns: Vec<String>,
    /// Unique bounded application rows with exact column widths.
    pub(super) rows: Vec<DisplayRow>,
}
impl DisplayTable {
    /// Admit <=20 unique columns, <=100 unique rows, exact widths and bounded text.
    /// # Errors
    /// Rejects inconsistent widths/identities and excessive data before rendering.
    pub fn new(columns: Vec<String>, rows: Vec<DisplayRow>) -> Result<Self, DisplayError> {
        let mut keys = BTreeSet::new();
        let mut labels = BTreeSet::new();
        if columns.is_empty()
            || columns.len() > 20
            || rows.len() > 100
            || columns
                .iter()
                .any(|value| value.is_empty() || value.len() > 1024 || !labels.insert(value))
            || rows.iter().any(|row| {
                row.key.is_empty()
                    || row.key.len() > 1024
                    || !keys.insert(&row.key)
                    || row.cells.len() != columns.len()
                    || row.cells.iter().any(|value| value.len() > 4096)
            })
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self { columns, rows })
    }
}

/// Explicit overflow measurement; characters mean Unicode scalars, not grapheme clusters.
#[derive(Clone, Copy, Debug)]
pub enum TextLimit {
    /// Keep at most this many Unicode scalar values.
    Characters(usize),
    /// Keep this many whitespace-separated words, preserving interior whitespace.
    Words(usize),
}
/// Truncate only at UTF-8 boundaries. Zero deliberately means an empty prefix.
pub fn overflow_text(text: &str, limit: TextLimit, ellipsis: bool) -> String {
    let cutoff = match limit {
        TextLimit::Characters(count) => text.char_indices().nth(count).map(|(index, _)| index),
        TextLimit::Words(0) if !text.is_empty() => Some(0),
        TextLimit::Words(count) => {
            let mut words = 0;
            let mut in_word = false;
            let mut cutoff = None;
            for (index, character) in text.char_indices() {
                if character.is_whitespace() {
                    in_word = false;
                } else if !in_word {
                    if words == count {
                        cutoff = Some(index);
                        break;
                    }
                    words += 1;
                    in_word = true;
                }
            }
            cutoff
        }
    };
    match cutoff {
        Some(index) => {
            // Character limits preserve exact whitespace; word limits discard
            // only the separator immediately before the first omitted word.
            let prefix = match limit {
                TextLimit::Characters(_) => &text[..index],
                TextLimit::Words(_) => text[..index].trim_end_matches(char::is_whitespace),
            };
            format!("{prefix}{}", if ellipsis { "…" } else { "" })
        }
        None => text.to_owned(),
    }
}

/// Bounded named values for a semantic definition list.
#[derive(Clone, Debug)]
pub struct MetadataValues(pub(super) Vec<(String, String)>);
impl MetadataValues {
    /// Admit <=100 unique nonempty labels and bounded exact text values.
    /// # Errors
    /// Rejects ambiguous labels or excessive data; empty and zero values remain valid.
    pub fn new(values: Vec<(String, String)>) -> Result<Self, DisplayError> {
        let mut labels = BTreeSet::new();
        if values.len() > 100
            || values.iter().any(|(label, value)| {
                label.is_empty()
                    || label.len() > 1024
                    || value.len() > 4096
                    || !labels.insert(label)
            })
        {
            return Err(DisplayError::InvalidValue);
        }
        Ok(Self(values))
    }
}
