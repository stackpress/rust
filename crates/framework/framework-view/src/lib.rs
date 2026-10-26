//! Public page contracts independent of hosts and render engines.
//!
//! Applications explicitly project their public data into [`PageSnapshot`]. No
//! request, session, service or diagnostic report is implicitly serialized.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;

/// Maximum serialized snapshot bytes before script escaping.
pub const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;
/// Maximum complete buffered document bytes, including escaped seed and assets.
pub const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;

/// Checked page identity, independent of source module paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PageId(String);

impl TryFrom<String> for PageId {
    type Error = ViewError;

    /// Reject paths, empty names and unbounded identifiers at either wire boundary.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        // Names are catalog keys, so dots, separators and dynamic imports have no meaning.
        if value.is_empty()
            || value.len() > 64
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        {
            return Err(ViewError::InvalidSnapshot);
        }
        Ok(Self(value))
    }
}

impl From<PageId> for String {
    /// Transfer the validated identity into the serde representation.
    fn from(value: PageId) -> Self {
        value.0
    }
}

/// A loader's cooperative Abort skips rendering and preserves response state.
#[derive(Debug)]
pub enum PageLoad<T> {
    /// Owned public data, safe to carry beyond the invocation borrow.
    Ready(T),
    /// The child capability stopped processing.
    Abort,
}

/// The deliberately small public envelope; unknown wire fields are rejected.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageSnapshot<T> {
    /// Version of this application's public wire schema.
    pub revision: u32,
    /// Exact asset build associated with the markup.
    pub build: String,
    /// Compiled page catalog identity.
    pub page: PageId,
    /// Public absolute path/query, never a URL containing credentials.
    pub location: String,
    /// Initial locale, supplied explicitly on both targets.
    pub locale: String,
    /// Explicit initial theme; no initial browser/environment guess.
    pub theme: String,
    /// Application-owned public projection.
    pub data: T,
}

impl<T> PageSnapshot<T> {
    /// Check asset compatibility and the limited public location/settings contract.
    ///
    /// # Errors
    /// Returns a refresh-required mismatch or malformed public metadata.
    pub fn validate(&self, revision: u32, build: &str) -> Result<(), ViewError> {
        // Reject drift before any component can consume incompatible data.
        if self.revision != revision || self.build != build {
            return Err(ViewError::ContractMismatch);
        }
        // Only bounded same-origin paths and explicit demo settings are accepted.
        if !self.location.starts_with('/')
            || self.location.starts_with("//")
            || self.location.len() > 4096
            || self.location.chars().any(char::is_control)
            || self.locale != "en"
            || self.theme != "light"
        {
            return Err(ViewError::InvalidSnapshot);
        }
        Ok(())
    }
}

impl<T: Serialize> PageSnapshot<T> {
    /// Serialize data as inert script text while preserving an exact JSON roundtrip.
    ///
    /// # Errors
    /// Rejects serialization failures and the pre-escape snapshot byte ceiling.
    pub fn script_json(&self) -> Result<String, ViewError> {
        let json = serde_json::to_string(self).map_err(|_| ViewError::InvalidSnapshot)?;
        // Check original bytes separately; the complete HTML limit covers escape expansion.
        if json.len() > MAX_SNAPSHOT_BYTES {
            return Err(ViewError::SnapshotLimit);
        }
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
        Ok(safe)
    }
}

impl<T: DeserializeOwned + Serialize> PageSnapshot<T> {
    /// Decode bounded textContent and verify compatibility before hydration.
    ///
    /// # Errors
    /// Rejects malformed/oversized JSON or incompatible metadata.
    pub fn decode(json: &str, revision: u32, build: &str) -> Result<Self, ViewError> {
        // Escaping can expand each byte to six characters; retain a finite decoder ceiling.
        if json.len() > MAX_SNAPSHOT_BYTES * 6 {
            return Err(ViewError::SnapshotLimit);
        }
        let snapshot: Self = serde_json::from_str(json).map_err(|_| ViewError::InvalidSnapshot)?;
        // The escape allowance must not admit an oversized unescaped logical snapshot.
        if serde_json::to_vec(&snapshot)
            .map_err(|_| ViewError::InvalidSnapshot)?
            .len()
            > MAX_SNAPSHOT_BYTES
        {
            return Err(ViewError::SnapshotLimit);
        }
        snapshot.validate(revision, build)?;
        Ok(snapshot)
    }
}

/// Complete buffered output; construction cannot publish partial HTML.
#[derive(Debug)]
pub struct RenderedPage(String);

impl RenderedPage {
    /// Apply the final ceiling after metadata and escaped seed have been added.
    ///
    /// # Errors
    /// Returns [`ViewError::HtmlLimit`] for an oversized complete document.
    pub fn new(html: String) -> Result<Self, ViewError> {
        if html.len() > MAX_HTML_BYTES {
            return Err(ViewError::HtmlLimit);
        }
        Ok(Self(html))
    }

    /// Transfer staged bytes to the host's existing HTML response path.
    pub fn into_html(self) -> String {
        self.0
    }
}

/// Safe public page failures; developer reports stay outside this envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewError {
    /// Native SSR requires initialize_ssr() inside an active Tokio runtime.
    ExecutorUnavailable,
    /// Malformed or unapproved public input.
    InvalidSnapshot,
    /// Client and host contract/assets differ; require a refresh.
    ContractMismatch,
    /// Public seed exceeded its finite ceiling.
    SnapshotLimit,
    /// Complete document exceeded its finite ceiling.
    HtmlLimit,
}

impl fmt::Display for ViewError {
    /// Provide stable safe messages without echoing caller input or native diagnostics.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ExecutorUnavailable => {
                "Native SSR requires initialize_ssr() inside an active Tokio runtime"
            }
            Self::InvalidSnapshot => "Invalid public page data",
            Self::ContractMismatch => "Page version changed. Refresh to continue.",
            Self::SnapshotLimit => "Page data is too large",
            Self::HtmlLimit => "Rendered page is too large",
        })
    }
}

impl std::error::Error for ViewError {}
