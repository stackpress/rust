//! First-paint metadata and one-shot browser acquisition, independent of rendering.
use crate::{BasePath, PublicLocation, ViewError};
/// Cacheable immutable asset; its filename changes with its content.
pub const BOOTSTRAP_FILE: &str = "initial-theme-152fe9618311a2eb.js";
/// Trusted tiny external bytes; these perform no storage or registration.
pub const BOOTSTRAP_SOURCE: &str = include_str!("initial-theme.js");
/// Return the fixed application-relative path for serving the matching asset.
pub fn bootstrap_location() -> Result<PublicLocation, ViewError> {
    PublicLocation::new(format!("/assets/{BOOTSTRAP_FILE}"))
}
/// Format an unset-only external classic head tag under checked mount/nonce policy.
///
/// # Errors
/// Rejects an unsafe nonce or a resource outside the checked mount/path contract.
pub fn bootstrap_tag(
    base: &BasePath,
    path: Option<&PublicLocation>,
    nonce: Option<&str>,
) -> Result<String, ViewError> {
    let Some(path) = path else {
        return Ok(String::new());
    };
    if path.as_str().contains('?') {
        return Err(ViewError::InvalidPolicy);
    }
    let nonce = match nonce {
        None => String::new(),
        Some(value)
            if !value.is_empty()
                && value.len() <= 256
                && value.bytes().all(|b| {
                    b.is_ascii_alphanumeric() || matches!(b, b'+' | b'/' | b'=' | b'-' | b'_')
                }) =>
        {
            format!(" nonce=\"{value}\"")
        }
        Some(_) => return Err(ViewError::InvalidPolicy),
    };
    let href = base.join(path.as_str())?;
    // Paths are still escaped as attributes, independently from inert JSON escaping.
    let href = href
        .as_str()
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    Ok(format!("<script{nonce} src=\"{href}\"></script>"))
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Strict external nonces and checked paths cannot inject a second attribute/script.
    #[test]
    fn checked_external_tag_and_nonce() {
        let base = BasePath::new("/demo").unwrap();
        let path = bootstrap_location().unwrap();
        assert_eq!(bootstrap_tag(&base, None, Some("bad\"nonce")).unwrap(), "");
        assert_eq!(
            bootstrap_tag(&base, Some(&path), Some("abc+/=_-")).unwrap(),
            format!("<script nonce=\"abc+/=_-\" src=\"/demo/assets/{BOOTSTRAP_FILE}\"></script>")
        );
        for nonce in ["", "bad\"", "a b", "<script>"] {
            assert!(bootstrap_tag(&base, Some(&path), Some(nonce)).is_err());
        }
        assert!(bootstrap_tag(&base, Some(&path), Some(&"x".repeat(257))).is_err());
        let query = PublicLocation::new("/assets/theme.js?x=1").unwrap();
        assert!(bootstrap_tag(&base, Some(&query), None).is_err());
    }
}
