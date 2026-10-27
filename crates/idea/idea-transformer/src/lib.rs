//! Build-host transforms over Idea config, with exclusive cooperative output ownership.
//!
//! Native and in-memory loaders compose bounded importer-relative Idea/JSON sources, retaining
//! typed reference sites and declaration provenance for explicit parser finalization. Selected
//! function transforms run sequentially after dependency/contract preflight, with declared input
//! snapshots and disjoint output partitions. The native writer checks drift and stages owned
//! output with cooperative locking and explicit recovery; it promises no crash atomicity.
//! Nothing here starts an async runtime or listener. Trusted Rust plugins and source resolution
//! roots are not filesystem sandboxes. General template rendering remains a separate task.

mod compose;
mod output;
mod plan;
mod registry;
mod source;

pub use compose::{ComposedSchema, Composer, CompositionLimits, ImportEdge};
pub use source::{
    LoadError, LoadedSource, MemoryLoader, ProjectLoader, SourceId, SourceKind, SourceLoader,
};
pub use stackpress_idea_parser::Reference;

use std::fmt;
use std::io::Read;
use std::path::Path;

pub use output::GenerationReport;
pub use output::{CheckReport, OutputWorkspace};
pub use plan::ModuleContribution;
pub use registry::{ContextTransform, TransformContext, TransformRegistry, TransformSelection};
pub use registry::{GeneratedFile, OutputPlan, Registry, Transform};
pub use registry::{TransformInputs, TransformSpec};

/// Stable phase classification, independent of diagnostic wording and private cause text.
///
/// Additional categories may be added; downstream matches must allow unknown variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TransformErrorKind {
    /// Caller-created error or an internal invariant without a narrower category.
    Other,
    /// Native acquisition, encoding or process IO failed; inspect the private source.
    Io,
    /// Idea/JSON parsing, normalization or import topology is invalid.
    Parse,
    /// Another cooperative writer owns the output lock.
    LockExists,
    /// Retained installation state requires explicit recovery.
    RecoveryRequired,
    /// Output paths, partitions, manifests or existing bytes violate ownership.
    Ownership,
    /// The compiled selection, prerequisites or runtime contracts are invalid.
    Selection,
    /// An acquisition, planning, metadata or subprocess resource budget was exceeded.
    Limit,
    /// A selected callback failed; contextual callbacks retain the error as their source.
    /// Legacy callbacks preserve their existing Display and source chain.
    Callback,
}

/// A build-tool failure retaining IO causes without automatically dumping source contents.
#[derive(Debug)]
pub struct TransformError {
    /// Stable category chosen at the failing operation, never inferred from its message.
    kind: TransformErrorKind,
    /// Safe phase or ownership explanation.
    message: String,
    /// Original filesystem, parser or callback diagnostic, disclosed only by explicit source traversal.
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl TransformError {
    /// Add bounded, distinct callback context while retaining a concrete private cause.
    ///
    /// # Errors
    /// Rejects empty, control-containing or over-1024-byte context. The cause's
    /// Display is never copied into the context or exposed by this constructor.
    pub fn with_source(
        kind: TransformErrorKind,
        context: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Result<Self> {
        let context = context.into();
        // Validate the independently authored explanation before retaining its cause.
        if context.trim().is_empty()
            || context.len() > 1024
            || context.chars().any(char::is_control)
        {
            return Err(Self::classified(
                TransformErrorKind::Other,
                "invalid error context",
            ));
        }
        Ok(Self::caused(kind, context, cause))
    }

    /// Describe a caller-created failure with kind Other and no source.
    pub fn new(message: impl Into<String>) -> Self {
        Self::classified(TransformErrorKind::Other, message)
    }

    /// Inspect the stable category without disclosing the private source chain.
    pub fn kind(&self) -> TransformErrorKind {
        self.kind
    }

    /// Create an owned failure at the operation that knows its category.
    fn classified(kind: TransformErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            cause: None,
        }
    }

    /// Retain a private source while adding a distinct safe phase explanation.
    fn caused(
        kind: TransformErrorKind,
        message: impl Into<String>,
        cause: impl std::error::Error + Send + Sync + 'static,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl fmt::Display for TransformError {
    /// Display the safe phase explanation; callers can inspect the separate cause.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for TransformError {
    /// Keep native IO context available to explicit developer diagnostics.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|error| error.as_ref() as _)
    }
}

impl From<std::io::Error> for TransformError {
    /// Preserve the OS error while keeping its potentially private path out of Display.
    fn from(cause: std::io::Error) -> Self {
        Self {
            kind: TransformErrorKind::Io,
            message: "generation filesystem operation failed".into(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl From<stackpress_idea_parser::IdeaError> for TransformError {
    /// Keep logical source coordinates and the original typed diagnostic.
    fn from(cause: stackpress_idea_parser::IdeaError) -> Self {
        Self {
            kind: if cause.code == stackpress_idea_parser::ErrorCode::Limit {
                TransformErrorKind::Limit
            } else {
                TransformErrorKind::Parse
            },
            message: "Idea parsing failed".into(),
            cause: Some(Box::new(cause)),
        }
    }
}

/// Fallible build-host operation.
pub type Result<T> = std::result::Result<T, TransformError>;

/// Read at most the declared byte budget, distinguishing invalid UTF-8 from missing files.
///
/// Source symlinks are followed. Pre/open/post checks assume cooperative path writers;
/// they are not race-free against concurrent replacement and do not bound OS IO latency.
///
/// # Errors
/// Returns original IO causes, a budget error, or invalid UTF-8 without lossy conversion.
pub fn read_source(path: &Path, max_bytes: usize) -> Result<String> {
    let mut bytes = Vec::new();
    // Read one detection byte beyond the bound; never allocate for the full untrusted file.
    source::open_regular(path)?
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(TransformError::classified(
            crate::TransformErrorKind::Limit,
            "source byte budget exceeded",
        ));
    }
    String::from_utf8(bytes).map_err(|cause| {
        TransformError::caused(TransformErrorKind::Io, "source is not UTF-8", cause)
    })
}

/// Merge one imported type into the accumulated local declaration in source import order.
///
/// G19 deliberately matches upstream code: an immutable local is **replaced by the child**.
/// Mutable locals retain their own attributes and entire colliding columns; missing child
/// columns prepend in child order. Neither branch mutates the imported child.
pub fn merge_type(
    local: &mut stackpress_idea_parser::TypeConfig,
    child: &stackpress_idea_parser::TypeConfig,
) {
    // Immutable means replacement in this project's accepted upstream composition contract.
    if !local.mutable {
        *local = child.clone();
        return;
    }
    let mut attributes = child.attributes.clone();
    attributes.extend(local.attributes.clone());
    local.attributes = attributes;
    // Borrowed filtering avoids upstream's in-place reverse of cached child columns.
    let mut columns: Vec<_> = child
        .columns
        .iter()
        .filter(|column| !local.columns.iter().any(|own| own.name == column.name))
        .cloned()
        .collect();
    columns.append(&mut local.columns);
    local.columns = columns;
}

#[cfg(test)]
#[path = "../tests/support/watchdog.rs"]
mod watchdog;
