//! Library-local HTML and inert JSON escaping.
/// Escape inert script JSON without changing roundtrip values.
pub fn escape_script_json(json: &str) -> String {
    let mut safe = String::with_capacity(json.len());
    for character in json.chars() {
        // HTML parsers recognize script terminators even in non-executable script tags.
        match character {
            '<' => safe.push_str("\\u003c"),
            '>' => safe.push_str("\\u003e"),
            '&' => safe.push_str("\\u0026"),
            '\u{2028}' => safe.push_str("\\u2028"),
            '\u{2029}' => safe.push_str("\\u2029"),
            other => safe.push(other),
        }
    }
    safe
}

/// Escape ordinary HTML text and attributes.
pub fn escape_html(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => encoded.push_str("&amp;"),
            '<' => encoded.push_str("&lt;"),
            '>' => encoded.push_str("&gt;"),
            '"' => encoded.push_str("&quot;"),
            '\'' => encoded.push_str("&#39;"),
            other => encoded.push(other),
        }
    }
    encoded
}
