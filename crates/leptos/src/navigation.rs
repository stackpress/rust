//! Strict app-local location values used by reusable image presentation.
/// Invalid local location; never reflects caller input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocationError;
impl std::fmt::Display for LocationError {
    /// Return a fixed safe diagnostic.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid local location")
    }
}
impl std::error::Error for LocationError {}
/// Exact encoded same-origin pathname/query; no origin, fragment or normalization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocalLocation(String);
impl LocalLocation {
    /// Admit only the strict local-location grammar used by image sources.
    /// # Errors
    /// Rejects malformed percent escapes, dot aliases, raw Unicode/whitespace and excessive bytes.
    pub fn new(value: impl Into<String>) -> Result<Self, LocationError> {
        let value = value.into();
        if !value.starts_with('/')
            || value.starts_with("//")
            || value.len() > 4096
            || !value.bytes().all(|b| {
                b.is_ascii_graphic() && !matches!(b, b'\\' | b'#' | b'<' | b'>' | b'"' | b'`')
            })
        {
            return Err(LocationError);
        }
        // Browsers must not repair a partial percent escape differently from native routing.
        let bytes = value.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            if bytes[index] == b'%' {
                if index + 2 >= bytes.len()
                    || !bytes[index + 1].is_ascii_hexdigit()
                    || !bytes[index + 2].is_ascii_hexdigit()
                {
                    return Err(LocationError);
                }
                index += 3;
            } else {
                index += 1;
            }
        }
        // URL parsers remove literal and percent-encoded dot segments before matching.
        // Refuse those aliases so a checked canonical location has one pathname.
        if value
            .split('?')
            .next()
            .unwrap_or_default()
            .split('/')
            .any(|segment| {
                let dots = segment.to_ascii_lowercase().replace("%2e", ".");
                matches!(dots.as_str(), "." | "..")
            })
        {
            return Err(LocationError);
        }
        Ok(Self(value))
    }

    /// Borrow the exact admitted encoded bytes.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
