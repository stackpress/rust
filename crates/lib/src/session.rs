//! Cookie-backed values and scoped outgoing revisions, not authenticated identity.
//!
//! ```
//! use stackpress_lib::session::{CookieOptions, Session};
//! let mut session = Session::new();
//! session.set(
//!     "theme",
//!     "dark",
//!     Some(CookieOptions {
//!         path: Some("/preferences".into()),
//!         ..Default::default()
//!     }),
//! )?;
//! // Deletion retains the cookie's path so the browser removes the same scoped value.
//! session.remove("theme")?;
//! assert!(session.cookie_headers()?[0].contains("Path=/preferences"));
//! # Ok::<(), stackpress_lib::Error>(())
//! ```

use crate::{DataMap, Error, ParseLimits, Result};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Typed SameSite options; a boolean true upstream corresponds to Strict.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SameSite {
    /// Send under strict same-site rules.
    Strict,
    /// Send under lax same-site rules.
    Lax,
    /// Explicitly select None; browser acceptance/security policy belongs to callers.
    None,
}
/// Cookie eviction priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Priority {
    /// Low eviction priority.
    Low,
    /// Medium eviction priority.
    Medium,
    /// High eviction priority.
    High,
}
/// Per-key cookie settings; output validation never silently repairs invalid data.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CookieOptions {
    /// Optional DNS domain, including an intentional leading dot.
    pub domain: Option<String>,
    /// Optional cookie scope path.
    pub path: Option<String>,
    /// Supported HTTP-date instant in 1970..=9999.
    pub expires: Option<SystemTime>,
    /// Signed integer lifetime in seconds.
    pub max_age: Option<i64>,
    /// Hide from script access when serialized.
    pub http_only: bool,
    /// Select secure transport policy when serialized.
    pub secure: bool,
    /// Select partitioned-cookie behavior when serialized.
    pub partitioned: bool,
    /// Optional eviction priority.
    pub priority: Option<Priority>,
    /// Optional SameSite behavior.
    pub same_site: Option<SameSite>,
}
impl CookieOptions {
    /// Validate scope/date attributes before accepting a revision.
    ///
    /// # Errors
    /// Rejects invalid domains/paths or out-of-range HTTP dates.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::CookieOptions;
    /// CookieOptions {
    ///     domain: Some("example.com".into()),
    ///     path: Some("/profiles".into()),
    ///     ..Default::default()
    /// }
    /// .validate()?;
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn validate(&self) -> Result<()> {
        // Validate scope and date before retaining options or invoking an infallible HTTP-date
        // formatter.
        if let Some(domain) = &self.domain {
            // Permit one intentional leading dot while validating the actual DNS labels.
            let domain = domain.strip_prefix('.').unwrap_or(domain);
            if domain.is_empty()
                || domain.len() > 253
                || !domain.split('.').all(|label| {
                    !label.is_empty()
                        && label.len() <= 63
                        && label.as_bytes()[0].is_ascii_alphanumeric()
                        && label.as_bytes()[label.len() - 1].is_ascii_alphanumeric()
                        && label
                            .bytes()
                            .all(|c| c.is_ascii_alphanumeric() || c == b'-')
                })
            {
                return Err(Error::invalid("invalid cookie domain"));
            }
        }
        // Reject cookie attribute separators/control bytes before they could enter a header.
        if self.path.as_ref().is_some_and(|path| {
            path.len() > 8192
                || path
                    .bytes()
                    .any(|b| !(0x20..=0x7e).contains(&b) || b == b';')
        }) {
            return Err(Error::invalid("invalid cookie path"));
        }
        // Only supported nonnegative HTTP calendar instants reach httpdate formatting.
        if let Some(expires) = self.expires {
            let seconds = expires
                .duration_since(UNIX_EPOCH)
                .map_err(|_| Error::invalid("cookie date precedes supported epoch"))?
                .as_secs();
            // Guard httpdate's infallible formatter's documented calendar range first.
            if seconds >= 253_402_300_800 {
                return Err(Error::invalid("cookie date exceeds supported HTTP range"));
            }
        }
        Ok(())
    }
}

/// Cookie input/output codecs are pure boundary operations.
pub mod cookie {
    use super::{CookieOptions, Priority, SameSite};
    use crate::{DataMap, Error, ParseLimits, Result};
    /// Parse cookies; first decoded value wins, malformed fragments are ignored.
    ///
    /// # Errors
    /// Rejects input/entry budgets; undecodable values remain raw text.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, session::cookie};
    /// let values = cookie::parse("theme=dark; theme=second")?;
    /// assert_eq!(values["theme"], "dark");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn parse(text: &str) -> Result<DataMap<String, String>> {
        // Use percent decoding once with raw fallback; a literal plus is not form-query space.
        parse_with(text, |value| {
            Some(crate::data::decode(value, false).unwrap_or_else(|_| value.into()))
        })
    }
    /// Supply a deliberate decoder; None omits the value rather than manufacturing it.
    ///
    /// # Errors
    /// Rejects byte/count budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, session::cookie};
    /// let values = cookie::parse_with("theme=dark", |value| Some(value.to_uppercase()))?;
    /// assert_eq!(values["theme"], "DARK");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn parse_with(
        text: &str,
        decode: impl Fn(&str) -> Option<String>,
    ) -> Result<DataMap<String, String>> {
        // Bound raw input and decoded output separately because a custom decoder can expand values.
        let limits = ParseLimits::default();
        if text.len() > limits.max_bytes {
            return Err(Error::limit("cookie input byte limit exceeded"));
        }
        let mut values = DataMap::new();
        let mut bytes = 0usize;
        // Count every fragment, including malformed/duplicate ones, to bound parsing work.
        for (i, part) in text.split(';').enumerate() {
            if i >= limits.max_entries {
                return Err(Error::limit("cookie entry limit exceeded"));
            }
            // Ignore fragments without a value boundary, matching the deliberate permissive cookie
            // input policy.
            let Some((name, value)) = part.split_once('=') else {
                continue;
            };
            let name = name.trim_matches([' ', '\t']);
            // Normalized input order chooses the first valid decoded cookie; later duplicates
            // cannot override it.
            if !crate::headers::token(name) || values.contains_key(name) {
                continue;
            }
            // A custom decoder may intentionally omit a cookie with None; retain an empty decoded
            // string when present.
            if let Some(value) = decode(value.trim_matches([' ', '\t'])) {
                // Charge the decoded representation, not just the possibly much smaller encoded
                // source.
                bytes = bytes.saturating_add(name.len()).saturating_add(value.len());
                if bytes > limits.max_bytes {
                    return Err(Error::limit("decoded cookie byte limit exceeded"));
                }
                values.insert(name.into(), value);
            }
        }
        Ok(values)
    }
    /// Serialize a scalar cookie with checked attributes and percent encoding.
    ///
    /// # Errors
    /// Rejects invalid name/value/options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, session::cookie};
    /// let header = cookie::serialize("theme", "dark mode", &CookieOptions::default())?;
    /// assert_eq!(header, "theme=dark%20mode");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn serialize(name: &str, value: &str, options: &CookieOptions) -> Result<String> {
        // Default output percent-encodes a scalar before the shared injection/attribute checks.
        serialize_with(name, value, options, crate::data::encode)
    }
    /// Supply an encoder but revalidate its output against cookie injection.
    ///
    /// # Errors
    /// Rejects invalid names, attributes, encoded separators/control characters or size.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, session::cookie};
    /// let header =
    ///     cookie::serialize_with("theme", "dark", &CookieOptions::default(), str::to_owned)?;
    /// assert_eq!(header, "theme=dark");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn serialize_with(
        name: &str,
        value: &str,
        options: &CookieOptions,
        encode: impl Fn(&str) -> String,
    ) -> Result<String> {
        // Validate attributes/name before invoking the caller encoder; then revalidate its produced
        // bytes.
        options.validate()?;
        if !crate::headers::token(name) {
            return Err(Error::invalid("invalid cookie name"));
        }
        // Encoder output is untrusted framing data even when the original scalar was valid.
        let value = encode(value);
        if value.len() > ParseLimits::default().max_bytes
            || value.bytes().any(|b| {
                !(0x21..=0x7e).contains(&b) || b == b';' || b == b',' || b == b'"' || b == b'\\'
            })
        {
            return Err(Error::invalid("invalid encoded cookie value"));
        }
        // The name/value pair is safe; append only validated typed attributes afterward.
        let mut serial = format!("{name}={value}");
        append_options(&mut serial, options);
        Ok(serial)
    }
    /// Append validated attributes in deterministic wire order after value encoding.
    fn append_options(serial: &mut String, options: &CookieOptions) {
        // These attributes were validated before encoding; append only selected options in stable
        // wire order.

        // An explicit zero/negative age is retained for expiration semantics, not treated as
        // absent.
        if let Some(age) = options.max_age {
            serial.push_str(&format!("; Max-Age={age}"));
        }
        // Retain exact selected domain scope so removals target the same browser cookie.
        if let Some(domain) = &options.domain {
            serial.push_str(&format!("; Domain={domain}"));
        }
        // Retain the explicit path rather than substituting an adapter default.
        if let Some(path) = &options.path {
            serial.push_str(&format!("; Path={path}"));
        }
        if let Some(expires) = options.expires {
            serial.push_str(&format!("; Expires={}", httpdate::fmt_http_date(expires)));
        }
        // Flag attributes are emitted only when selected; false adds no wire field.
        if options.http_only {
            serial.push_str("; HttpOnly");
        }
        // Secure is an explicit application policy; serialization never infers transport trust.
        if options.secure {
            serial.push_str("; Secure");
        }
        // Partitioning is a typed output option, with browser acceptance left to its host policy.
        if options.partitioned {
            serial.push_str("; Partitioned");
        }
        // Exhaustive typed priority avoids accepting arbitrary attribute text.
        if let Some(priority) = options.priority {
            serial.push_str(match priority {
                Priority::Low => "; Priority=Low",
                Priority::Medium => "; Priority=Medium",
                Priority::High => "; Priority=High",
            });
        }
        // Emit the explicit SameSite selection, including None, without boolean/string ambiguity.
        if let Some(same_site) = options.same_site {
            serial.push_str(match same_site {
                SameSite::Strict => "; SameSite=Strict",
                SameSite::Lax => "; SameSite=Lax",
                SameSite::None => "; SameSite=None",
            });
        }
    }
}

/// A scalar or explicitly multi-valued cookie-backed session value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionValue {
    /// One scalar string.
    Text(String),
    /// A list requiring an intentional wire codec.
    List(Vec<String>),
}
impl SessionValue {
    /// Borrow a scalar without silently collapsing a list.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::SessionValue;
    /// assert_eq!(SessionValue::from("dark").as_str(), Some("dark"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn as_str(&self) -> Option<&str> {
        // Only a scalar value has a borrowed string view; list encoding must be an explicit caller
        // choice.
        if let Self::Text(value) = self {
            Some(value)
        } else {
            None
        }
    }
}
impl From<&str> for SessionValue {
    /// Own a scalar string.
    fn from(value: &str) -> Self {
        // Own borrowed text so a session Clone never aliases a caller buffer.
        Self::Text(value.into())
    }
}
impl From<String> for SessionValue {
    /// Move a scalar string.
    fn from(value: String) -> Self {
        // Move scalar ownership without interpreting it as identity or authentication.
        Self::Text(value)
    }
}
impl From<Vec<String>> for SessionValue {
    /// Retain an explicit string list.
    fn from(value: Vec<String>) -> Self {
        // Keep lists typed rather than silently joining them into a cookie wire value.
        Self::List(value)
    }
}

/// Latest outgoing change for one key, retaining scope after live-option deletion.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionRevision {
    /// Set this owned value with its keyed options.
    Set {
        /// New scalar/list value.
        value: SessionValue,
        /// Current keyed cookie settings.
        options: Option<CookieOptions>,
    },
    /// Expire the original cookie scope, not an unrelated default path.
    Remove {
        /// Prior keyed settings retained in the tombstone.
        options: Option<CookieOptions>,
    },
}

/// Deeply clonable cookie state/revisions, distinct from identity/authentication.
#[derive(Clone, Default)]
pub struct Session {
    /// Live cookie-backed values; initialization does not imply outgoing writes.
    values: DataMap<String, SessionValue>,
    /// Live per-key scope; deletion transfers it to the outgoing tombstone.
    options: DataMap<String, CookieOptions>,
    /// Latest outgoing change per key, retained separately from live presence.
    revisions: DataMap<String, SessionRevision>,
}
impl Session {
    /// Construct empty state with no outgoing revisions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert!(Session::new().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new() -> Self {
        // An independent session starts with no live values, scopes or pending revisions.
        Self::default()
    }
    /// Construct initial values without marking them changed.
    ///
    /// # Errors
    /// Rejects invalid names or value budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// let initial = Session::from_pairs([("theme", "dark")])?;
    /// assert!(initial.revisions().is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_pairs<K: Into<String>, V: Into<SessionValue>>(
        pairs: impl IntoIterator<Item = (K, V)>,
    ) -> Result<Self> {
        // Initial cookie input is validated through setters, then outgoing revisions are
        // deliberately cleared.
        let mut session = Self::new();
        for (key, value) in pairs {
            session.set(&key.into(), value, None)?;
        }
        session.revisions.clear();
        Ok(session)
    }
    /// Borrow one scalar/list value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert_eq!(
    ///     session.get("theme").and_then(SessionValue::as_str),
    ///     Some("dark")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn get(&self, name: &str) -> Option<&SessionValue> {
        // Borrow cookie-backed data only; possession of this value is not an authorization
        // decision.
        self.values.get(name)
    }
    /// Inspect current keyed options.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert_eq!(
    ///     session.options("theme").unwrap().path.as_deref(),
    ///     Some("/preferences")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn options(&self, name: &str) -> Option<&CookieOptions> {
        // Return only live scope options; removal scope lives in its outgoing tombstone instead.
        self.options.get(name)
    }
    /// Inspect immutable latest-per-key outgoing changes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert!(session.revisions().contains_key("theme"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn revisions(&self) -> &DataMap<String, SessionRevision> {
        // Borrow the latest revision per key without silently acknowledging or clearing native
        // delivery.
        &self.revisions
    }
    /// Inspect insertion-ordered live entries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert_eq!(session.entries().count(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn entries(&self) -> impl Iterator<Item = (&String, &SessionValue)> {
        // Expose live insertion order independently from pending outgoing revision order.
        self.values.iter()
    }
    /// Count live values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert_eq!(session.len(), 1);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn len(&self) -> usize {
        // Count live names, not tombstones awaiting output.
        self.values.len()
    }
    /// Report no live values, independently of deletion revisions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert!(!session.is_empty());
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn is_empty(&self) -> bool {
        // A session may have no live values yet still contain pending removal revisions.
        self.values.is_empty()
    }
    /// Set a value; omitted options retain that key's settings.
    ///
    /// # Errors
    /// Rejects invalid name/options or aggregate byte/entry limits before mutation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// // Omitting options retains this key's existing scope settings.
    /// session.set("theme", "light", None)?;
    /// assert_eq!(
    ///     session.options("theme").unwrap().path.as_deref(),
    ///     Some("/preferences")
    /// );
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn set(
        &mut self,
        name: &str,
        value: impl Into<SessionValue>,
        options: Option<CookieOptions>,
    ) -> Result<&mut Self> {
        // Validate name/options first, then stage live state and its matching latest outgoing
        // revision together.
        if !crate::headers::token(name) {
            return Err(Error::invalid("invalid session cookie name"));
        }
        if let Some(options) = &options {
            options.validate()?;
        }
        let value = value.into();
        let mut candidate = self.clone();
        if let Some(options) = options {
            candidate.options.insert(name.into(), options);
        }
        // Live state and the revision each own ordinary string/list data; the clone never aliases a
        // mutable value.
        candidate.values.insert(name.into(), value.clone());
        candidate.revisions.insert(
            name.into(),
            SessionRevision::Set {
                value,
                // Omitted options retain the prior scope, while an explicit options argument
                // replaces it.
                options: candidate.options.get(name).cloned(),
            },
        );
        // Validate live-plus-revision budgets before publishing any staged session change.
        candidate.validate()?;
        *self = candidate;
        Ok(self)
    }
    /// Delete live state but preserve its previous cookie scope in a tombstone.
    ///
    /// # Errors
    /// Rejects invalid names or revision budgets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert!(session.remove("theme")?);
    /// assert!(session.cookie_headers()?[0].contains("Path=/preferences"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn remove(&mut self, name: &str) -> Result<bool> {
        // A tombstone must retain the original scope even when the live key is already absent.
        if !crate::headers::token(name) {
            return Err(Error::invalid("invalid session cookie name"));
        }
        let mut candidate = self.clone();
        // Remove live presence first while retaining enough staged metadata to create its
        // expiration revision.
        let removed = candidate.values.shift_remove(name).is_some();
        let options =
            candidate
                .options
                .shift_remove(name)
                .or_else(|| match candidate.revisions.get(name) {
                    // Repeated deletion keeps the existing tombstone scope instead of expiring an
                    // unrelated default-path cookie.
                    Some(SessionRevision::Remove { options }) => options.clone(),
                    _ => None,
                });
        candidate
            .revisions
            .insert(name.into(), SessionRevision::Remove { options });
        candidate.validate()?;
        *self = candidate;
        Ok(removed)
    }
    /// Clear live state, recording scoped removals for every current key.
    ///
    /// # Errors
    /// Returns validation/budget errors without partially clearing the session.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// session.clear()?;
    /// assert!(session.is_empty());
    /// assert!(session.revisions().contains_key("theme"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn clear(&mut self) -> Result<()> {
        // Stage the complete batch so a failure cannot leave half the live cookies removed.
        let mut candidate = self.clone();
        for key in self.values.keys() {
            candidate.remove(key)?;
        }
        *self = candidate;
        Ok(())
    }
    /// Produce separate Set-Cookie strings for scalar revisions.
    ///
    /// # Errors
    /// Rejects lists unless a caller supplies an intentional codec.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// assert!(session.cookie_headers()?[0].starts_with("theme=dark"));
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn cookie_headers(&self) -> Result<Vec<String>> {
        // The default wire policy accepts scalar values only; lists need a deliberate codec.
        self.cookie_headers_with(|value| match value {
            SessionValue::Text(value) => Ok(value.clone()),
            SessionValue::List(_) => Err(Error::invalid(
                "session list requires an explicit cookie codec",
            )),
        })
    }
    /// Serialize changes using an explicit list/scalar representation policy.
    ///
    /// # Errors
    /// Returns codec/attribute validation failures.
    /// Revisions are encoded in insertion order and processing stops at the first
    /// failure; no partial header list is returned. Session values and revisions
    /// remain unchanged, but side effects performed by the supplied codec are not
    /// rolled back and may run again if the caller retries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::{CookieOptions, Session, SessionValue};
    /// let mut session = Session::new();
    /// session.set(
    ///     "theme",
    ///     "dark",
    ///     Some(CookieOptions {
    ///         path: Some("/preferences".into()),
    ///         ..Default::default()
    ///     }),
    /// )?;
    /// session.set("roles", vec!["ADMIN".into(), "USER".into()], None)?;
    /// let headers = session.cookie_headers_with(|value| match value {
    ///     SessionValue::Text(value) => Ok(value.clone()),
    ///     SessionValue::List(values) => Ok(serde_json::to_string(values)?),
    /// })?;
    /// assert_eq!(headers.len(), 2);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn cookie_headers_with(
        &self,
        codec: impl Fn(&SessionValue) -> Result<String>,
    ) -> Result<Vec<String>> {
        // Serialize revisions separately in order; never comma-fold multiple Set-Cookie values.
        self.revisions
            .iter()
            .map(|(name, revision)| match revision {
                SessionRevision::Set { value, options } => {
                    cookie::serialize(name, &codec(value)?, &options.clone().unwrap_or_default())
                }
                SessionRevision::Remove { options } => {
                    let mut options = options.clone().unwrap_or_default();
                    // Expiration retains scope but forces immediate lifetime and a deterministic
                    // epoch date.
                    options.max_age = Some(0);
                    options.expires = Some(UNIX_EPOCH);
                    cookie::serialize(name, "", &options)
                }
            })
            .collect()
    }
    /// Treat supplied cookie values/options as initial, without outgoing writes.
    pub(crate) fn into_initial(mut self) -> Self {
        // Supplied initial data is not an outgoing write; clear only revisions while retaining
        // values/options.
        self.revisions.clear();
        self
    }
    /// Check live and revision counts/bytes, including removed scoped metadata.
    fn validate(&self) -> Result<()> {
        // Bound both live storage and outgoing revisions, including scope retained by removed
        // cookies.
        let limits = ParseLimits::default();
        let mut bytes = 0usize;
        for (key, value) in &self.values {
            bytes = bytes.saturating_add(key.len());
            bytes = bytes.saturating_add(match value {
                SessionValue::Text(v) => v.len(),
                SessionValue::List(v) => v.iter().fold(0usize, |n, v| n.saturating_add(v.len())),
            });
        }
        // Live scope metadata has its own retained allocation and contributes to the total.
        for (key, options) in &self.options {
            bytes = bytes
                .saturating_add(key.len())
                .saturating_add(option_bytes(options));
        }
        // Removed scope and duplicated revision payloads remain stored, so both must count toward
        // limits.
        for (key, revision) in &self.revisions {
            bytes = bytes.saturating_add(key.len());
            let options = match revision {
                SessionRevision::Set { value, options } => {
                    bytes = bytes.saturating_add(match value {
                        SessionValue::Text(value) => value.len(),
                        SessionValue::List(values) => values
                            .iter()
                            .fold(0usize, |sum, v| sum.saturating_add(v.len())),
                    });
                    options
                }
                SessionRevision::Remove { options } => options,
            };
            if let Some(options) = options {
                bytes = bytes.saturating_add(option_bytes(options));
            }
        }
        if self.values.len() > limits.max_entries
            || self.revisions.len() > limits.max_entries
            || bytes > limits.max_bytes
        {
            return Err(Error::limit("session byte/entry limit exceeded"));
        }
        Ok(())
    }
}
/// Count variable-sized scope metadata retained in a revision.
fn option_bytes(options: &CookieOptions) -> usize {
    // Only variable-sized scope strings consume this metadata byte budget.
    options
        .domain
        .as_ref()
        .map_or(0, String::len)
        .saturating_add(options.path.as_ref().map_or(0, String::len))
}
impl fmt::Debug for Session {
    /// Redact live values and pending cookie revisions.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Both live cookies and revisions may contain secrets, so diagnostics list names/counts
        // only.
        f.debug_struct("Session")
            .field("keys", &self.values.keys().collect::<Vec<_>>())
            .field("pending", &self.revisions.len())
            .finish_non_exhaustive()
    }
}
