//! Portable source presentation, distinct from Typed's application-policy projection.
use crate::UiModel;
use serde::{Deserialize, Serialize};

/// Source-authored UI roles only; no codec seed, provider arguments, SQL or native resources.
/// Roles describe presentation, never authorize an operation or install a global guard.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SourceUiCatalog {
    /// Source model order, including application-added columns in declaration order.
    pub models: Vec<UiModel>,
    /// Portable embedded shapes; these do not imply hidden tables.
    pub fieldsets: Vec<UiModel>,
}
