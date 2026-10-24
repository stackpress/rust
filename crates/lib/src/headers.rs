//! Validated case-insensitive multi-value headers without native HTTP resources.
//!
//! ```
//! use stackpress_lib::Headers;
//! let mut headers = Headers::new();
//! headers
//!     .append("Set-Cookie", "first=1")?
//!     .append("set-cookie", "second=2")?;
//! assert_eq!(headers.get_all("SET-COOKIE").unwrap().len(), 2);
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use crate::{DataMap, Error, ParseLimits, Result};
use std::fmt;

/// Owned header values. Repeated values remain separate, including Set-Cookie.
#[derive(Clone, Default)]
pub struct Headers(DataMap<String, Vec<String>>);
impl Headers {
    /// Construct an empty validated map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert!(Headers::new().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // Each instance owns its field lists; there is no implicit native header map.
        Self::default()
    }
    /// Validate supplied header pairs, retaining repeated values in input order.
    ///
    /// # Errors
    /// Rejects invalid names/values or aggregate budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert_eq!(headers.get("x-id"), Some("7"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_pairs<K: AsRef<str>, V: AsRef<str>>(
        pairs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<Self> {
        // Validate each supplied occurrence while preserving repeated-value order.
        let mut headers = Self::new();
        for (name, value) in pairs {
            headers.append(name.as_ref(), value.as_ref())?;
        }
        Ok(headers)
    }
    /// Borrow the first value of a case-insensitive header name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert_eq!(headers.get("X-ID"), Some("7"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn get(&self, name: &str) -> Option<&str> {
        // Normalize only the lookup name and borrow the first occurrence without comma folding.
        self.0
            .get(&name.to_ascii_lowercase())?
            .first()
            .map(String::as_str)
    }
    /// Borrow repeated values without comma folding.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// headers.append("X-Id", "8")?;
    /// assert_eq!(headers.get_all("x-id").unwrap().len(), 2);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn get_all(&self, name: &str) -> Option<&[String]> {
        // Keep repeated Set-Cookie and similar fields as independent borrowed values.
        self.0.get(&name.to_ascii_lowercase()).map(Vec::as_slice)
    }
    /// Check case-insensitive presence.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert!(headers.has("X-ID"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn has(&self, name: &str) -> bool {
        // Presence is case-insensitive even when a stored value is empty.
        self.get_all(name).is_some()
    }
    /// Count distinct names rather than repeated values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert_eq!(headers.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        // The public length counts normalized names rather than wire occurrences.
        self.0.len()
    }
    /// Report no names.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert!(!headers.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        // An absent map differs from a present name whose value happens to be empty.
        self.0.is_empty()
    }
    /// Inspect normalized names and separate values in insertion order.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert_eq!(headers.entries().next().unwrap().0, "x-id");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vec<String>)> {
        // Borrow normalized names and their original occurrence order without exposing mutation.
        self.0.iter()
    }
    /// Replace a name's complete value list.
    ///
    /// # Errors
    /// Rejects invalid token/value or aggregate budget without partial mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// headers.set("X-Id", "8")?;
    /// assert_eq!(headers.get("x-id"), Some("8"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        // Replacement and append share validation, but replacement discards all previous
        // occurrences.
        self.write(name, value, false)
    }
    /// Append a repeated value without collapsing its separate identity.
    ///
    /// # Errors
    /// Rejects invalid input/budgets without partial mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// headers
    ///     .append("Set-Cookie", "a=1")?
    ///     .append("Set-Cookie", "b=2")?;
    /// assert_eq!(headers.get_all("set-cookie").unwrap().len(), 2);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn append(&mut self, name: &str, value: &str) -> Result<&mut Self> {
        // Append preserves repeated header identity rather than inventing a combined wire value.
        self.write(name, value, true)
    }
    /// Remove every value under a case-insensitive name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Headers;
    /// let mut headers = Headers::from_pairs([("X-Id", "7")])?;
    /// assert!(headers.remove("X-ID").is_some());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn remove(&mut self, name: &str) -> Option<Vec<String>> {
        // Remove every occurrence under the normalized name while retaining other insertion order.
        self.0.shift_remove(&name.to_ascii_lowercase())
    }
    /// Validate a candidate before replacing this map.
    fn write(&mut self, name: &str, value: &str, append: bool) -> Result<&mut Self> {
        // Reject framing characters before creating a candidate so failed updates preserve the
        // original.
        if !token(name) || value.bytes().any(|b| (b < 0x20 && b != b'\t') || b == 0x7f) {
            return Err(Error::invalid("invalid header name/value"));
        }
        // Reject one oversized value before allocating its candidate copy.
        crate::data::check_input(
            name.len().saturating_add(value.len()),
            &ParseLimits::default(),
        )?;
        // Headers are bounded strings only; stage this small candidate to preserve atomic
        // replacement.
        let mut candidate = self.clone();
        let entry = candidate.0.entry(name.to_ascii_lowercase()).or_default();
        // Replacement resets the old value list; appending leaves every earlier occurrence intact.
        if !append {
            entry.clear();
        }
        entry.push(value.into());
        // Check the combined name/value budget before publishing the candidate.
        candidate.validate(&ParseLimits::default())?;
        *self = candidate;
        Ok(self)
    }
    /// Enforce aggregate name/value count and bytes for a chosen invocation.
    pub(crate) fn validate(&self, limits: &ParseLimits) -> Result<()> {
        // Count each value occurrence but each name once, matching this map's storage budget.
        let mut bytes = 0usize;
        let mut count = 0usize;
        for (name, values) in &self.0 {
            bytes = bytes.saturating_add(name.len());
            for value in values {
                bytes = bytes.saturating_add(value.len());
                count = count.saturating_add(1);
            }
        }
        // Aggregate limits prevent many individually valid occurrences from bypassing the
        // invocation budget.
        if bytes > limits.max_bytes || count > limits.max_entries {
            return Err(Error::limit("header byte/entry limit exceeded"));
        }
        Ok(())
    }
}
impl fmt::Debug for Headers {
    /// Redact values that may contain cookies/authorization credentials.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Header values can contain credentials; only normalized names are safe in wrapper
        // diagnostics.
        f.debug_struct("Headers")
            .field("names", &self.0.keys().collect::<Vec<_>>())
            .finish_non_exhaustive()
    }
}
/// Recognize a nonempty bounded ASCII token, shared by method/header/cookie checks.
pub(crate) fn token(text: &str) -> bool {
    // A bounded nonempty ASCII token is shared by names, cookies and MIME checks.
    !text.is_empty()
        && text.len() <= 8192
        && text
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"!#$%&'*+-.^_|~".contains(&c) || c == 0x60)
}
