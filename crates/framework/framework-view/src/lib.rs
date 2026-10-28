//! Public page contracts independent of hosts and render engines.
//!
//! Applications explicitly project their public data into [`PageSnapshot`]. No
//! request, session, service or diagnostic report is implicitly serialized.
//! Presentation capabilities are opt-in modules; see the migration guide in `docs/migration-issue15.md`.

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;

mod policy;
mod routes;
pub use policy::{BasePath, PageAssets, PageContract, PageLimits, PublicLocation, PublicStatus};
pub use routes::{PageRoute, RouteCatalog, RouteSegment, encode_uri_component};

/// Maximum serialized snapshot bytes before script escaping.
pub const MAX_SNAPSHOT_BYTES: usize = 1024 * 1024;
/// Maximum complete buffered document bytes, including escaped seed and assets.
pub const MAX_HTML_BYTES: usize = 2 * 1024 * 1024;

/// Checked page identity, independent of source module paths.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PageId(String);

impl PageId {
    /// Borrow the catalog identity without treating it as a module or file path.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

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
///
/// # Examples
/// ```
/// use stackpress_framework_view::{PageId, PageSnapshot, PublicStatus};
/// let page = PageSnapshot {
///     revision: 1, build: "app-build".into(),
///     page: PageId::try_from("profile".to_owned())?, location: "/profile".into(),
///     locale: "en".into(), theme: None, status: PublicStatus::ok(),
///     data: "</script><script>untrusted()</script>".to_owned(),
/// };
/// let inert = page.script_json()?;
/// assert!(!inert.contains('<'));
/// let decoded = PageSnapshot::<String>::decode(&inert, 1, "app-build")?;
/// assert_eq!(decoded.data, page.data);
/// # Ok::<(), stackpress_framework_view::ViewError>(())
/// ```
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
    /// Saved Light/Dark choice; missing/null is unset, separate from its effective color.
    #[serde(default)]
    pub theme: Option<String>,
    /// Deliberately disclosed application status, separate from lifecycle outcome.
    #[serde(default)]
    pub status: PublicStatus,
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
        PublicLocation::new(self.location.clone())?;
        PageContract {
            revision: self.revision,
            build: self.build.clone(),
            locale: self.locale.clone(),
            theme: self.theme.clone(),
            limits: PageLimits::default(),
        }
        .validate()?;
        Ok(())
    }
}

impl<T: Serialize> PageSnapshot<T> {
    /// Serialize data as inert script text while preserving an exact JSON roundtrip.
    ///
    /// # Errors
    /// Rejects serialization failures and the pre-escape snapshot byte ceiling.
    pub fn script_json(&self) -> Result<String, ViewError> {
        self.script_json_with(PageLimits::default())
    }

    /// Serialize under a checked application policy, preserving the standard escape contract.
    ///
    /// # Errors
    /// Rejects invalid policy, serialization failure and a logical byte-limit excess.
    pub fn script_json_with(&self, limits: PageLimits) -> Result<String, ViewError> {
        limits.validate()?;
        self.validate(self.revision, &self.build)?;
        let json = serde_json::to_string(self).map_err(ViewError::from_source)?;
        // Check original bytes separately; the complete HTML limit covers escape expansion.
        if json.len() > limits.snapshot_bytes {
            return Err(ViewError::SnapshotLimit);
        }
        Ok(escape_script_json(&json))
    }
}

impl<T: DeserializeOwned + Serialize> PageSnapshot<T> {
    /// Decode bounded textContent and verify compatibility before hydration.
    ///
    /// # Errors
    /// Rejects malformed/oversized JSON or incompatible metadata.
    pub fn decode(json: &str, revision: u32, build: &str) -> Result<Self, ViewError> {
        Self::decode_with(json, revision, build, PageLimits::default())
    }

    /// Decode under the same checked policy used by the native page registration.
    ///
    /// # Errors
    /// Rejects unsafe metadata, drift, malformed JSON and either encoded/logical size excess.
    pub fn decode_with(
        json: &str,
        revision: u32,
        build: &str,
        limits: PageLimits,
    ) -> Result<Self, ViewError> {
        limits.validate()?;
        // Escaping can expand each byte to six characters; retain a finite decoder ceiling.
        if json.len() > limits.snapshot_bytes * 6 {
            return Err(ViewError::SnapshotLimit);
        }
        let snapshot: Self = serde_json::from_str(json).map_err(|_| ViewError::InvalidSnapshot)?;
        // The escape allowance must not admit an oversized unescaped logical snapshot.
        if serde_json::to_vec(&snapshot)
            .map_err(ViewError::from_source)?
            .len()
            > limits.snapshot_bytes
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
        Self::with_limits(html, PageLimits::default())
    }

    /// Stage a complete document under a lowerable application policy.
    ///
    /// # Errors
    /// Rejects invalid policy or an oversized completed document.
    pub fn with_limits(html: String, limits: PageLimits) -> Result<Self, ViewError> {
        limits.validate()?;
        if html.len() > limits.html_bytes {
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
#[derive(Debug, Clone)]
pub enum ViewError {
    /// Original serialization/render cause retained only for native diagnostics.
    Failure(std::sync::Arc<dyn std::error::Error + Send + Sync>),
    /// Trusted page/asset configuration violates its checked policy.
    InvalidPolicy,
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
            Self::Failure(_) => "Could not render page",
            Self::InvalidPolicy => "Invalid page configuration",
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

impl ViewError {
    /// Preserve a developer cause while keeping its text out of public Display.
    pub fn from_source(cause: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Failure(std::sync::Arc::new(cause))
    }
}
impl PartialEq for ViewError {
    /// Stable policy failures compare by kind; native causes compare by shared identity.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Failure(a), Self::Failure(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}
impl Eq for ViewError {}
impl std::error::Error for ViewError {
    /// Explicit diagnostic traversal can inspect the original retained failure.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Failure(cause) => Some(cause.as_ref()),
            _ => None,
        }
    }
}

/// Owned buffered metadata assembly future; no live reactive state may cross it.
pub type RenderFuture<'a> = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<RenderedPage, ViewError>> + Send + 'a>,
>;

/// Native renderer boundary over owned explicitly public page data.
pub trait PageRenderer<T>: Send + Sync + 'static {
    /// Immutable version/settings/limit contract checked before native registration.
    fn contract(&self) -> &PageContract;
    /// Stage and finish complete output without native output authority.
    fn render(&self, page: PageSnapshot<T>) -> RenderFuture<'_>;
}

/// Complete asynchronous rendering may own a reactive root through body/replay/Head.
///
/// This distinct boundary leaves [`RenderFuture`]/[`PageRenderer`]'s synchronous
/// staging contract intact. It is a buffered result, never a streaming response.
pub type AsyncRenderFuture<'a> = std::pin::Pin<
    Box<dyn std::future::Future<Output = Result<RenderedPage, ViewError>> + Send + 'a>,
>;

/// Explicit opt-in renderer for component-owned asynchronous work.
pub trait AsyncPageRenderer<T>: Send + Sync + 'static {
    /// Immutable public settings and bounded page/asset contract.
    fn contract(&self) -> &PageContract;
    /// Own the complete render lifetime; cancellation/disposal is renderer-specific.
    fn render_async(&self, page: PageSnapshot<T>) -> AsyncRenderFuture<'_>;
}

#[cfg(all(feature = "server", target_arch = "wasm32"))]
compile_error!("framework-view/server is a native adapter; use default contracts on wasm32");
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
mod server;
#[cfg(all(feature = "server", not(target_arch = "wasm32")))]
pub use server::{PageLoader, PageSpec, ServerBuilderViewExt};

/// Escape JSON text for an inert script element without altering roundtrip data.
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

/// Fail-closed host admission and safe action display hints.
pub mod access;
/// Explicit web codecs and opt-in native persistence.
pub mod adapters;
/// Optional bounded public notices.
#[cfg(feature = "flash")]
pub mod flash;
/// Optional public settings and save-state contracts.
#[cfg(feature = "preferences")]
pub mod preferences;
#[cfg(all(feature = "native-file", target_arch = "wasm32"))]
compile_error!("framework-view/native-file is a Unix native adapter");
#[cfg(all(test, feature = "preferences", feature = "flash"))]
mod presentation_tests;

/// Initial-theme asset and checked head metadata.
pub mod theme;
