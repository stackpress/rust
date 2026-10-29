//! Generic replay admission errors, independent of application pages.
/// Stable replay policy, data, budget and serializer failure categories.
#[derive(Clone, Debug)]
pub enum ReplayError {
    /// Logical replay policy exceeds its fixed maximum.
    InvalidPolicy,
    /// Malformed identity, duplicate or invalid JSON.
    InvalidSnapshot,
    /// Logical or expanded replay exceeds its byte/count budget.
    SnapshotLimit,
    /// A concrete serializer failure retained for diagnostics.
    Failure(std::sync::Arc<dyn std::error::Error + Send + Sync>),
}
impl ReplayError {
    /// Preserve a native serializer cause behind a safe error.
    pub fn from_source(e: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Failure(std::sync::Arc::new(e))
    }
}
impl std::fmt::Display for ReplayError {
    /// Never reflect replay content into error text.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid resource replay")
    }
}
impl std::error::Error for ReplayError {
    /// Only explicit diagnostic traversal receives the original cause.
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Failure(e) = self {
            Some(e.as_ref())
        } else {
            None
        }
    }
}
impl PartialEq for ReplayError {
    /// Compare stable kinds and retain concrete failure identity.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Failure(a), Self::Failure(b)) => std::sync::Arc::ptr_eq(a, b),
            _ => std::mem::discriminant(self) == std::mem::discriminant(other),
        }
    }
}
impl Eq for ReplayError {}
