//! Small bounded import decoders; export remains UTF-8 and never uses these tables.
use super::{Limits, invalid};
use stackpress_framework_schema::SchemaResult;
use std::borrow::Cow;

/// Decode only after raw admission, and charge decoded UTF-8 bytes before each append.
pub(super) fn decode(bytes: &[u8], limits: Limits) -> SchemaResult<Cow<'_, str>> {
    if bytes.len() > limits.bytes {
        return Err(invalid("CSV file byte limit exceeded"));
    }
    // A BOM is authoritative: malformed marked input cannot fall back to a different encoding.
    if let Some(bytes) = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]) {
        return std::str::from_utf8(bytes)
            .map(Cow::Borrowed)
            .map_err(|cause| invalid("invalid BOM-marked CSV UTF-8").with_cause(cause));
    }
    if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        let little = bytes.starts_with(&[0xff, 0xfe]);
        let payload = &bytes[2..];
        if !payload.len().is_multiple_of(2) {
            return Err(invalid("invalid CSV UTF-16 byte length"));
        }
        let units = payload.chunks_exact(2).map(|pair| {
            let pair = [pair[0], pair[1]];
            if little {
                u16::from_le_bytes(pair)
            } else {
                u16::from_be_bytes(pair)
            }
        });
        let mut text = String::new();
        for scalar in char::decode_utf16(units) {
            let scalar = scalar
                .map_err(|cause| invalid("invalid CSV UTF-16 surrogate").with_cause(cause))?;
            append(&mut text, scalar, limits)?;
        }
        return Ok(Cow::Owned(text));
    }
    if let Ok(text) = std::str::from_utf8(bytes) {
        return Ok(Cow::Borrowed(text));
    }
    // Two adjacent Arabic letters supply evidence beyond a single ambiguous legacy byte.
    let table = if reads_as_arabic(bytes) {
        &WINDOWS_1256
    } else {
        &WINDOWS_1252
    };
    let mut text = String::new();
    for &byte in bytes {
        let scalar = if byte < 128 {
            char::from(byte)
        } else {
            table[usize::from(byte - 128)]
        };
        append(&mut text, scalar, limits)?;
    }
    Ok(Cow::Owned(text))
}

/// A deterministic heuristic, not a language guarantee: prefer connected Arabic letter evidence.
fn reads_as_arabic(bytes: &[u8]) -> bool {
    let (mut letters, mut latin, mut run, mut longest) = (0, 0, 0, 0);
    for &byte in bytes {
        let scalar = if byte < 128 {
            char::from(byte)
        } else {
            WINDOWS_1256[usize::from(byte - 128)]
        };
        if matches!(scalar, '\u{621}'..='\u{63a}' | '\u{641}'..='\u{64a}' | '\u{67e}' | '\u{686}' | '\u{698}' | '\u{6a9}' | '\u{6af}' | '\u{6ba}' | '\u{6be}' | '\u{6c1}' | '\u{6d2}')
        {
            letters += 1;
            run += 1;
            longest = longest.max(run);
        } else {
            // Diacritics and joining controls can occur inside Arabic words.
            if !matches!(
                scalar,
                '\u{640}' | '\u{64b}'..='\u{652}' | '\u{200c}' | '\u{200d}'
            ) {
                run = 0;
            }
            if matches!(scalar, '\u{c0}'..='\u{ff}') && scalar.is_alphabetic() {
                latin += 1;
            }
        }
    }
    longest >= 2 && letters > latin
}

/// Bound expansion before appending a scalar; no decoded buffer can exceed the file ceiling.
fn append(text: &mut String, scalar: char, limits: Limits) -> SchemaResult<()> {
    if scalar.len_utf8() > limits.bytes.saturating_sub(text.len()) {
        return Err(invalid("CSV decoded byte limit exceeded"));
    }
    text.push(scalar);
    Ok(())
}

// Standard byte mappings: Unicode MAPPINGS/VENDORS/MICSFT/WINDOWS/CP1252.TXT and CP1256.TXT.
// Undefined Windows-1252 bytes retain their C1 control code rather than losing source bytes.
/// Upper-half byte mapping for Windows-1252; ASCII is shared unchanged.
const WINDOWS_1252: [char; 128] = [
    '\u{20ac}', '\u{81}', '\u{201a}', '\u{192}', '\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{2c6}', '\u{2030}', '\u{160}', '\u{2039}', '\u{152}', '\u{8d}', '\u{17d}', '\u{8f}',
    '\u{90}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{2dc}', '\u{2122}', '\u{161}', '\u{203a}', '\u{153}', '\u{9d}', '\u{17e}', '\u{178}',
    '\u{a0}', '\u{a1}', '\u{a2}', '\u{a3}', '\u{a4}', '\u{a5}', '\u{a6}', '\u{a7}', '\u{a8}',
    '\u{a9}', '\u{aa}', '\u{ab}', '\u{ac}', '\u{ad}', '\u{ae}', '\u{af}', '\u{b0}', '\u{b1}',
    '\u{b2}', '\u{b3}', '\u{b4}', '\u{b5}', '\u{b6}', '\u{b7}', '\u{b8}', '\u{b9}', '\u{ba}',
    '\u{bb}', '\u{bc}', '\u{bd}', '\u{be}', '\u{bf}', '\u{c0}', '\u{c1}', '\u{c2}', '\u{c3}',
    '\u{c4}', '\u{c5}', '\u{c6}', '\u{c7}', '\u{c8}', '\u{c9}', '\u{ca}', '\u{cb}', '\u{cc}',
    '\u{cd}', '\u{ce}', '\u{cf}', '\u{d0}', '\u{d1}', '\u{d2}', '\u{d3}', '\u{d4}', '\u{d5}',
    '\u{d6}', '\u{d7}', '\u{d8}', '\u{d9}', '\u{da}', '\u{db}', '\u{dc}', '\u{dd}', '\u{de}',
    '\u{df}', '\u{e0}', '\u{e1}', '\u{e2}', '\u{e3}', '\u{e4}', '\u{e5}', '\u{e6}', '\u{e7}',
    '\u{e8}', '\u{e9}', '\u{ea}', '\u{eb}', '\u{ec}', '\u{ed}', '\u{ee}', '\u{ef}', '\u{f0}',
    '\u{f1}', '\u{f2}', '\u{f3}', '\u{f4}', '\u{f5}', '\u{f6}', '\u{f7}', '\u{f8}', '\u{f9}',
    '\u{fa}', '\u{fb}', '\u{fc}', '\u{fd}', '\u{fe}', '\u{ff}',
];
/// Upper-half byte mapping for Windows-1256; ASCII is shared unchanged.
const WINDOWS_1256: [char; 128] = [
    '\u{20ac}', '\u{67e}', '\u{201a}', '\u{192}', '\u{201e}', '\u{2026}', '\u{2020}', '\u{2021}',
    '\u{2c6}', '\u{2030}', '\u{679}', '\u{2039}', '\u{152}', '\u{686}', '\u{698}', '\u{688}',
    '\u{6af}', '\u{2018}', '\u{2019}', '\u{201c}', '\u{201d}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{6a9}', '\u{2122}', '\u{691}', '\u{203a}', '\u{153}', '\u{200c}', '\u{200d}', '\u{6ba}',
    '\u{a0}', '\u{60c}', '\u{a2}', '\u{a3}', '\u{a4}', '\u{a5}', '\u{a6}', '\u{a7}', '\u{a8}',
    '\u{a9}', '\u{6be}', '\u{ab}', '\u{ac}', '\u{ad}', '\u{ae}', '\u{af}', '\u{b0}', '\u{b1}',
    '\u{b2}', '\u{b3}', '\u{b4}', '\u{b5}', '\u{b6}', '\u{b7}', '\u{b8}', '\u{b9}', '\u{61b}',
    '\u{bb}', '\u{bc}', '\u{bd}', '\u{be}', '\u{61f}', '\u{6c1}', '\u{621}', '\u{622}', '\u{623}',
    '\u{624}', '\u{625}', '\u{626}', '\u{627}', '\u{628}', '\u{629}', '\u{62a}', '\u{62b}',
    '\u{62c}', '\u{62d}', '\u{62e}', '\u{62f}', '\u{630}', '\u{631}', '\u{632}', '\u{633}',
    '\u{634}', '\u{635}', '\u{636}', '\u{d7}', '\u{637}', '\u{638}', '\u{639}', '\u{63a}',
    '\u{640}', '\u{641}', '\u{642}', '\u{643}', '\u{e0}', '\u{644}', '\u{e2}', '\u{645}',
    '\u{646}', '\u{647}', '\u{648}', '\u{e7}', '\u{e8}', '\u{e9}', '\u{ea}', '\u{eb}', '\u{649}',
    '\u{64a}', '\u{ee}', '\u{ef}', '\u{64b}', '\u{64c}', '\u{64d}', '\u{64e}', '\u{f4}', '\u{64f}',
    '\u{650}', '\u{f7}', '\u{651}', '\u{f9}', '\u{652}', '\u{fb}', '\u{fc}', '\u{200e}',
    '\u{200f}', '\u{6d2}',
];
