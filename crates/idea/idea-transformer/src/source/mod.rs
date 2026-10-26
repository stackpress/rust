//! Bounded, data-only source adapters for native projects and deterministic memory fixtures.
//!
//! The explicit project root is a resolution base, not a filesystem sandbox. Absolute imports
//! and parent-relative imports are supported. No operation changes the process directory.

mod memory;
mod native;
pub(crate) use native::open_regular;
mod paths;

use std::fmt;
use std::path::{Path, PathBuf};

pub use memory::MemoryLoader;
pub use native::ProjectLoader;

/// Canonical loader identity used for caching, graph edges and active-stack cycle detection.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(PathBuf);

impl SourceId {
    /// Construct an identity in a custom loader's canonical namespace.
    ///
    /// A loader must return the same identity for every alias of one source.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self(path.into())
    }

    /// Borrow the canonical path; native paths may contain non-Unicode components.
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

/// Syntax selected by a source's resolved extension; executable modules are never loaded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Idea source text, compiled using the explicitly supplied parser inputs.
    Idea,
    /// Normalized schema JSON, structurally validated by the composer before merging.
    Json,
}

/// One immutable UTF-8 snapshot, owned by its caller rather than a mutable global cache.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSource {
    /// Canonical identity for cache and import graph lookup.
    pub id: SourceId,
    /// Display name, relative to the project root where possible.
    pub name: String,
    /// Data grammar determined by the adapter.
    pub kind: SourceKind,
    /// Bounded source bytes, also retained for explicit developer diagnostics.
    pub text: String,
}

/// Actionable source failure that retains the original IO/UTF-8 cause separately.
#[derive(Debug)]
pub struct LoadError {
    /// Internal category carried into composition without parsing message text.
    pub(crate) kind: crate::TransformErrorKind,
    /// Safe adapter explanation without source contents.
    message: String,
    /// Underlying native error, inspectable through `Error::source`.
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl LoadError {
    /// Describe a resolution, format, or byte-budget failure.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            kind: crate::TransformErrorKind::Other,
            message: message.into(),
            cause: None,
        }
    }

    /// Carry a known loader policy into the transformer's composition boundary.
    pub(crate) fn classified(kind: crate::TransformErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            cause: None,
        }
    }

    /// Preserve an underlying error without embedding its private details in Display.
    pub fn with_cause(
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind: crate::TransformErrorKind::Io,
            message: message.into(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl fmt::Display for LoadError {
    /// Render only the adapter's safe explanation.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for LoadError {
    /// Retain IO/encoding causes for explicit developer diagnostics.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|cause| cause.as_ref() as _)
    }
}

/// Resolve importer-relative data sources and bound acquisition before allocating their text.
///
/// `max_bytes` is the remaining per-call acquisition budget. Adapters must respect it before
/// reading/cloning input; the composer also checks results from custom adapters defensively.
/// Aliases must return one canonical identity. A run reads no process environment.
pub trait SourceLoader {
    /// Load one source relative to its importer, or the explicit root for an entry source.
    ///
    /// # Errors
    /// Returns resolution, IO, UTF-8, unsupported-format or byte-budget errors.
    fn load(
        &self,
        importer: Option<&SourceId>,
        specifier: &str,
        max_bytes: usize,
    ) -> std::result::Result<LoadedSource, LoadError>;
}

impl<T: SourceLoader + ?Sized> SourceLoader for &T {
    /// Forward a borrowed adapter without cloning it or sharing mutable composer state.
    fn load(
        &self,
        importer: Option<&SourceId>,
        specifier: &str,
        max_bytes: usize,
    ) -> std::result::Result<LoadedSource, LoadError> {
        (**self).load(importer, specifier, max_bytes)
    }
}
