//! Source number spelling shared by generated columns, diagnostics and native SQL interpolation.

/// JavaScript source number spelling shared with the portable UI runtime.
pub fn number_text(value: f64) -> String {
    crate::source_number::number_text(value)
}

/// ECMAScript trim characters; Rust's broader Unicode whitespace set differs at U+0085.
pub fn trim(value: &str) -> &str {
    crate::source_number::trim(value)
}
