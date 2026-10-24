//! Build-host transforms over Idea config, with exclusive cooperative output ownership.
//!
//! Nothing here starts an async runtime or listener. M0 proves sequential function transforms,
//! explicit output partitions, drift checking and recoverable staging. File composition,
//! dependency-rich plugin preflight and general auxiliary-input tracking remain later milestones.
//! Trusted plugin code is ordinary Rust, not a filesystem sandbox.

mod output;
mod plan;
mod registry;

use std::fmt;
use std::io::Read;
use std::path::Path;

pub use output::GenerationReport;
pub use output::{CheckReport, OutputWorkspace};
pub use plan::ModuleContribution;
pub use registry::{ContextTransform, TransformContext, TransformRegistry, TransformSelection};
pub use registry::{GeneratedFile, OutputPlan, Registry, Transform};
pub use registry::{TransformInputs, TransformSpec};

/// A build-tool failure retaining IO causes without automatically dumping source contents.
#[derive(Debug)]
pub struct TransformError {
    /// Safe phase or ownership explanation.
    message: String,
    /// Original IO failure, when the host filesystem failed.
    cause: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl TransformError {
    /// Describe a plan, selection or ownership conflict.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            cause: None,
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
            message: "generation filesystem operation failed".into(),
            cause: Some(Box::new(cause)),
        }
    }
}

impl From<stackpress_idea_parser::IdeaError> for TransformError {
    /// Keep logical source coordinates and the original typed diagnostic.
    fn from(cause: stackpress_idea_parser::IdeaError) -> Self {
        Self {
            message: cause.to_string(),
            cause: Some(Box::new(cause)),
        }
    }
}

/// Fallible build-host operation.
pub type Result<T> = std::result::Result<T, TransformError>;

/// Read at most the declared byte budget, distinguishing invalid UTF-8 from missing files.
///
/// # Errors
/// Returns original IO causes, a budget error, or invalid UTF-8 without lossy conversion.
pub fn read_source(path: &Path, max_bytes: usize) -> Result<String> {
    let mut bytes = Vec::new();
    // Read one detection byte beyond the bound; never allocate for the full untrusted file.
    std::fs::File::open(path)?
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(TransformError::new("source byte budget exceeded"));
    }
    String::from_utf8(bytes).map_err(|_| TransformError::new("source is not UTF-8"))
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
