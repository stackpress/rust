//! Checked public metadata and finite policies shared by native and client adapters.

use crate::{MAX_HTML_BYTES, MAX_SNAPSHOT_BYTES, ViewError};
use serde::{Deserialize, Serialize};

/// Finite byte ceilings; application policy may lower the framework defaults.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageLimits {
    /// Logical JSON bytes before script escaping.
    pub snapshot_bytes: usize,
    /// Complete HTML bytes after escaped seed, Head and asset insertion.
    pub html_bytes: usize,
}

impl Default for PageLimits {
    /// Use the accepted one-MiB snapshot and two-MiB document ceilings.
    fn default() -> Self {
        Self {
            snapshot_bytes: MAX_SNAPSHOT_BYTES,
            html_bytes: MAX_HTML_BYTES,
        }
    }
}

impl PageLimits {
    /// Reject policies which increase the framework ceilings; zero deliberately denies output.
    ///
    /// # Errors
    /// Returns an invalid policy for ceilings above the framework maximum.
    pub fn validate(self) -> Result<(), ViewError> {
        if self.snapshot_bytes > MAX_SNAPSHOT_BYTES || self.html_bytes > MAX_HTML_BYTES {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(())
    }
}

/// A checked same-origin public pathname/query, never a complete URL or fragment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PublicLocation(String);

impl PublicLocation {
    /// Validate an already percent-encoded public location without normalizing its meaning.
    ///
    /// # Errors
    /// Rejects credentials/origins, fragments, backslashes, malformed escapes and raw whitespace.
    pub fn new(value: impl Into<String>) -> Result<Self, ViewError> {
        let value = value.into();
        if !value.starts_with('/')
            || value.starts_with("//")
            || value.len() > 4096
            || !value.bytes().all(|b| {
                b.is_ascii_graphic() && !matches!(b, b'\\' | b'#' | b'<' | b'>' | b'"' | b'`')
            })
        {
            return Err(ViewError::InvalidSnapshot);
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
                    return Err(ViewError::InvalidSnapshot);
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
            return Err(ViewError::InvalidSnapshot);
        }
        Ok(Self(value))
    }

    /// Borrow the exact encoded location used at both routing boundaries.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PublicLocation {
    type Error = ViewError;
    /// Apply the same validation to wire decoding as to native construction.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<PublicLocation> for String {
    /// Transfer checked public text without decoding or normalizing it.
    fn from(value: PublicLocation) -> Self {
        value.0
    }
}

/// Trusted application mount prefix, canonicalized to no trailing slash.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BasePath(String);

impl BasePath {
    /// Create the root mount or an ASCII, segment-checked application prefix.
    ///
    /// # Examples
    /// ```
    /// use stackpress_framework_view::BasePath;
    /// let mount = BasePath::new("/demo")?;
    /// assert_eq!(mount.join("/ui/profiles")?.as_str(), "/demo/ui/profiles");
    /// # Ok::<(), stackpress_framework_view::ViewError>(())
    /// ```
    ///
    /// # Errors
    /// Rejects query/fragment data, empty/dot segments and ambiguous separators.
    pub fn new(value: impl Into<String>) -> Result<Self, ViewError> {
        let value = value.into();
        if value.is_empty() || value == "/" {
            return Ok(Self::default());
        }
        if value.len() > 1024
            || !value.starts_with('/')
            || value.ends_with('/')
            || value[1..].split('/').any(|part| {
                part.is_empty()
                    || matches!(part, "." | "..")
                    || !part.bytes().all(|b| {
                        b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~')
                    })
            })
        {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(Self(value))
    }

    /// Borrow the canonical mount prefix; root is the empty string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Join a checked application-relative absolute path to this trusted prefix.
    ///
    /// # Errors
    /// Rejects an unsafe suffix or a combined location exceeding its byte ceiling.
    pub fn join(&self, suffix: &str) -> Result<PublicLocation, ViewError> {
        PublicLocation::new(suffix)?;
        PublicLocation::new(format!("{}{suffix}", self.0))
    }

    /// Remove this exact mount only on a segment boundary.
    pub fn strip<'a>(&self, location: &'a str) -> Option<&'a str> {
        let rest = location.strip_prefix(&self.0)?;
        // A prefix named /app never owns /application or an origin-like // path.
        if rest.starts_with('/') && !rest.starts_with("//") {
            Some(rest)
        } else {
            None
        }
    }
}
impl TryFrom<String> for BasePath {
    type Error = ViewError;
    /// Validate deserialized trusted asset metadata before applying it.
    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl From<BasePath> for String {
    /// Transfer the canonical prefix into its data-only wire representation.
    fn from(value: BasePath) -> Self {
        value.0
    }
}

/// Deliberately disclosed page status; native error causes never enter this type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "StatusWire", into = "StatusWire")]
pub struct PublicStatus {
    /// A legal final page/HTTP status, distinct from dispatch outcome.
    code: u16,
    /// Optional explicitly public plain-text error, escaped by the render boundary.
    error: Option<String>,
}

/// Serde's unchecked intermediate is private and always passes the constructor.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StatusWire {
    code: u16,
    error: Option<String>,
}

impl Default for PublicStatus {
    /// Missing legacy status metadata means the ordinary successful page status.
    fn default() -> Self {
        Self::ok()
    }
}
impl PublicStatus {
    /// Ordinary successful page status.
    pub fn ok() -> Self {
        Self {
            code: 200,
            error: None,
        }
    }

    /// Construct a final public status with bounded, deliberately disclosed text.
    ///
    /// # Errors
    /// Rejects informational/non-HTTP status, an oversized message or a success/error mismatch.
    pub fn new(code: u16, error: Option<String>) -> Result<Self, ViewError> {
        if !(200..=599).contains(&code)
            || error.as_ref().is_some_and(|value| value.len() > 8192)
            || (code < 400 && error.is_some())
        {
            return Err(ViewError::InvalidSnapshot);
        }
        Ok(Self { code, error })
    }

    /// Native response status, independent of completed/aborted dispatch.
    pub fn code(&self) -> u16 {
        self.code
    }
    /// Borrow only the deliberately public message.
    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
}
impl TryFrom<StatusWire> for PublicStatus {
    type Error = ViewError;
    /// Prevent malformed wire status from bypassing public construction checks.
    fn try_from(value: StatusWire) -> Result<Self, Self::Error> {
        Self::new(value.code, value.error)
    }
}
impl From<PublicStatus> for StatusWire {
    /// Serialize only the allowed public status fields.
    fn from(value: PublicStatus) -> Self {
        Self {
            code: value.code,
            error: value.error,
        }
    }
}

/// One trusted data/asset version and explicit initial display settings.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageContract {
    /// Public wire revision expected by this compiled application.
    pub revision: u32,
    /// Asset build expected by both HTML and the browser bootstrap.
    pub build: String,
    /// Explicit initial language; never inferred from request/session/browser state.
    pub locale: String,
    /// Explicit initial theme, kept compatible with the M0 seed's string representation.
    pub theme: String,
    /// Lowerable serialization and complete-output ceilings.
    pub limits: PageLimits,
}

impl PageContract {
    /// Create checked version metadata with en/light defaults and finite standard ceilings.
    ///
    /// # Errors
    /// Rejects a zero revision or an empty/unbounded/unsafe build identity.
    pub fn new(revision: u32, build: impl Into<String>) -> Result<Self, ViewError> {
        let contract = Self {
            revision,
            build: build.into(),
            locale: "en".into(),
            theme: "light".into(),
            limits: PageLimits::default(),
        };
        contract.validate()?;
        Ok(contract)
    }

    /// Validate trusted configuration before registration or rendering can publish output.
    ///
    /// # Errors
    /// Rejects unsafe version/settings metadata or an expanded byte policy.
    pub fn validate(&self) -> Result<(), ViewError> {
        self.limits.validate()?;
        if self.revision == 0
            || self.build.is_empty()
            || self.build.len() > 128
            || !self
                .build
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.'))
            || !valid_locale(&self.locale)
            || !matches!(self.theme.as_str(), "light" | "dark")
        {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(())
    }
}

/// Bound public locale syntax without implementing locale negotiation or persistence.
pub(crate) fn valid_locale(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 35
        && value
            .split('-')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// Trusted build metadata for one application mount and bootstrap module.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageAssets {
    /// Exact public build ID, checked against the snapshot contract.
    pub build: String,
    /// Canonical public mount prefix used by routes and resources.
    pub base: BasePath,
    /// Application-relative module path; joining base is explicit and checked.
    pub module: PublicLocation,
    /// Optional application-relative stylesheet path.
    pub stylesheet: Option<PublicLocation>,
}

impl PageAssets {
    /// Validate module paths and the exact asset/data build agreement.
    ///
    /// # Errors
    /// Rejects version drift, query-bearing resource paths and mount/path ambiguity.
    pub fn validate(&self, contract: &PageContract) -> Result<(), ViewError> {
        contract.validate()?;
        if self.build != contract.build {
            return Err(ViewError::ContractMismatch);
        }
        for path in std::iter::once(&self.module).chain(self.stylesheet.iter()) {
            if path.as_str().contains('?') {
                return Err(ViewError::InvalidPolicy);
            }
            self.base.join(path.as_str())?;
        }
        Ok(())
    }
}

impl PageAssets {
    /// Serialize inert trusted bootstrap metadata under its independent finite ceiling.
    ///
    /// # Errors
    /// Rejects invalid assets or oversized metadata before adding it to the document.
    pub fn script_json(&self) -> Result<String, ViewError> {
        self.validate(&PageContract::new(1, self.build.clone())?)?;
        let json = serde_json::to_string(self).map_err(|_| ViewError::InvalidPolicy)?;
        if json.len() > 16 * 1024 {
            return Err(ViewError::InvalidPolicy);
        }
        Ok(crate::escape_script_json(&json))
    }

    /// Decode asset textContent and refuse version drift before routing/hydration starts.
    ///
    /// # Errors
    /// Rejects malformed/oversized metadata, unsafe paths and mismatched asset builds.
    pub fn decode(json: &str, build: &str) -> Result<Self, ViewError> {
        if json.len() > 16 * 1024 {
            return Err(ViewError::InvalidPolicy);
        }
        let assets: Self = serde_json::from_str(json).map_err(|_| ViewError::InvalidPolicy)?;
        assets.validate(&PageContract::new(1, build)?)?;
        Ok(assets)
    }
}
