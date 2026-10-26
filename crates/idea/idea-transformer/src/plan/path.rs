//! Portable relative-path and collision rules shared by plans and owned-root inspection.

use crate::{Result, TransformError};

/// Validate a portable output path before any filesystem operation.
pub(crate) fn valid_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.len() > 240
        || !path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-./".contains(&byte))
        || path.starts_with('.')
        || path.split('/').count() > 16
        || path.split('/').any(|part| {
            let stem = part.split('.').next().unwrap_or("").to_ascii_uppercase();
            part.is_empty()
                || part == "."
                || part == ".."
                || part.ends_with('.')
                || matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                || (stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && stem.as_bytes()[3].is_ascii_digit())
        })
    {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Ownership,
            "invalid relative generated path",
        ));
    }
    Ok(())
}

/// Reject file, ancestor and directory-spelling collisions between already validated paths.
pub(crate) fn paths_collide(left: &str, right: &str) -> bool {
    for (left, right) in left.split('/').zip(right.split('/')) {
        // Different folded components branch into unrelated files/directories.
        if !left.eq_ignore_ascii_case(right) {
            return false;
        }
        // A shared directory must use one spelling, even when its descendant leaves differ.
        if left != right {
            return true;
        }
    }
    // Identical common components mean the same file, or a file used as an ancestor.
    true
}
