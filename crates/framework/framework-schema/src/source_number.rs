//! Shared source Number(string)||0 semantics, without native values or crypto/provider features.

/// Source string-number conversion, including ECMAScript trim, radix rounding and infinities.
/// Invalid/NaN input returns zero, as the generated column and overflow falsy fallback require.
pub fn number(value: &str) -> f64 {
    let text = trim(value);
    if text.is_empty() {
        0.0
    } else if let Some(value) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
        radix_number(value, 16)
    } else if let Some(value) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
        radix_number(value, 2)
    } else if let Some(value) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
        radix_number(value, 8)
    } else if matches!(text, "Infinity" | "+Infinity" | "-Infinity")
        || text
            .bytes()
            .all(|b| b.is_ascii_digit() || b"+-eE.".contains(&b))
    {
        text.parse::<f64>()
            .ok()
            .filter(|value| !value.is_nan())
            .unwrap_or(0.0)
    } else {
        0.0
    }
}

/// ECMAScript whitespace differs from Rust's Unicode whitespace at U+0085 and FEFF.
pub(crate) fn trim(value: &str) -> &str {
    value.trim_matches(|c|matches!(c,'\u{0009}'..='\u{000d}'|'\u{0020}'|'\u{00a0}'|'\u{1680}'|'\u{2000}'..='\u{200a}'|'\u{2028}'|'\u{2029}'|'\u{202f}'|'\u{205f}'|'\u{3000}'|'\u{feff}'))
}

/// Parse power-of-two radix literals with one IEEE-754 ties-to-even rounding step.
/// A u64 parser would silently turn valid larger literals into zero; repeated f64
/// multiplication would instead round at each digit. Keep 53 bits plus guard/sticky bits.
fn radix_number(text: &str, radix: u32) -> f64 {
    if text.is_empty() {
        return 0.0;
    }
    let width = radix.trailing_zeros();
    let mut count = 0usize;
    let mut top = 0u64;
    let mut guard = false;
    let mut sticky = false;
    for c in text.chars() {
        let Some(digit) = c.to_digit(radix) else {
            return 0.0;
        };
        for shift in (0..width).rev() {
            let bit = (digit >> shift) & 1;
            if count == 0 && bit == 0 {
                continue;
            }
            if count < 53 {
                top = (top << 1) | u64::from(bit)
            } else if count == 53 {
                guard = bit != 0
            } else {
                sticky |= bit != 0
            }
            count += 1;
        }
    }
    if guard && (sticky || top & 1 != 0) {
        top += 1;
    }
    top as f64 * 2_f64.powi(count.saturating_sub(53).min(i32::MAX as usize) as i32)
}

/// JavaScript Number.toString spelling for the native finite/non-finite number family.
/// Integral floats have no decimal suffix; exponent thresholds and positive signs match source.
pub fn number_text(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value.is_infinite() {
        return if value.is_sign_negative() {
            "-Infinity"
        } else {
            "Infinity"
        }
        .into();
    }
    if value == 0.0 {
        return "0".into();
    }
    if value.abs() >= 1e21 || value.abs() < 1e-6 {
        let scientific = format!("{value:e}");
        if let Some((mantissa, exponent)) = scientific.split_once('e')
            && !exponent.starts_with('-')
        {
            return format!("{mantissa}e+{exponent}");
        }
        return scientific;
    }
    value.to_string()
}
