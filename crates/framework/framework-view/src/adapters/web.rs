//! Shared cookie identity and flags for issuing and deleting presentation cookies.
use crate::BasePath;
use stackpress_framework_language::PreferenceError;

/// Fixed cookie key; its value is only a registered canonical locale token.
pub const LOCALE_COOKIE: &str = "stackpress-locale";
/// Bounded action input before parsing or policy evaluation.
pub const MAX_PREFERENCE_BYTES: usize = 1024;
/// Total cookie header budget, including unrelated scoped presentation cookies.
pub const MAX_COOKIE_BYTES: usize = 4096;

/// Validated origin and exact app cookie scope; no Domain attribute is emitted.
#[derive(Clone, Debug)]
pub struct CookieScope {
    /// Preference actions retain the exact origin; flash only needs validated scope/flags.
    #[cfg(feature = "preferences")]
    pub(crate) origin: String,
    /// Same checked mount governs both return paths and cookie Path.
    pub(crate) base: BasePath,
    /// HTTPS cookies always require secure delivery.
    secure: bool,
}
impl CookieScope {
    /// Check a canonical HTTP(S) origin and an already checked app mount.
    ///
    /// # Errors
    /// Rejects credentials, paths, noncanonical origins and non-HTTP schemes.
    pub fn new(origin: String, base: BasePath) -> Result<Self, PreferenceError> {
        let parsed = url::Url::parse(&origin).map_err(|_| PreferenceError::Configuration)?;
        // Exact serialization excludes path/query/fragment and credentials tricks.
        if !matches!(parsed.scheme(), "https" | "http")
            || parsed.origin().ascii_serialization() != origin
            || !parsed.username().is_empty()
            || parsed.password().is_some()
            || parsed.path() != "/"
            || parsed.query().is_some()
            || parsed.fragment().is_some()
        {
            return Err(PreferenceError::Configuration);
        }
        Ok(Self {
            secure: parsed.scheme() == "https",
            #[cfg(feature = "preferences")]
            origin,
            base,
        })
    }

    /// Issue a checked internal cookie or clear that same identity with max-age zero.
    pub(crate) fn header(&self, key: &str, value: &str, age: u32) -> String {
        let path = if self.base.as_str().is_empty() {
            "/"
        } else {
            self.base.as_str()
        };
        let secure = if self.secure { "; Secure" } else { "" };
        // Keys/values are internal constants, enum tokens or hex, never raw input.
        format!("{key}={value}; Path={path}; Max-Age={age}; HttpOnly; SameSite=Lax{secure}")
    }
}

/// Read exactly one value; duplicates (including malformed bare keys) are ambiguous.
pub(crate) fn unique_cookie<'a>(
    header: &'a str,
    key: &str,
) -> Result<Option<&'a str>, PreferenceError> {
    if header.len() > MAX_COOKIE_BYTES {
        return Err(PreferenceError::Limit);
    }
    let mut value = None;
    for entry in header.split(';').map(str::trim) {
        let (name, candidate) = entry.split_once('=').unwrap_or((entry, ""));
        if name == key {
            // Refuse duplicates instead of trusting ordering chosen by a caller.
            if value.is_some() {
                return Err(PreferenceError::Request);
            }
            value = Some(candidate);
        }
    }
    Ok(value)
}

#[cfg(any(feature = "web-preferences", feature = "web-flash"))]
pub(crate) mod host;
