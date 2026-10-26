//! Shared deterministic candidate order for native and in-memory adapters.

use std::path::{Component, Path, PathBuf};

use super::{LoadError, SourceId, SourceKind};

/// Resolve aliases against the explicit root and ordinary imports against their owning file.
pub(super) fn base_path(root: &Path, importer: Option<&SourceId>, specifier: &str) -> PathBuf {
    // Root aliases override importer-relative lookup; absolute paths survive Path::join.
    if let Some(relative) = specifier.strip_prefix("@/") {
        return root.join(relative);
    }
    let base = importer
        .and_then(|id| id.as_path().parent())
        .unwrap_or(root);
    base.join(specifier)
}

/// Try the exact file, extension candidates, then directory indexes in this fixed order.
pub(super) fn candidates(path: &Path) -> Vec<PathBuf> {
    let mut paths = vec![path.to_owned()];
    // An explicit extension must not accidentally fall back to a different input grammar.
    if path.extension().is_none() {
        paths.push(path.with_extension("idea"));
        paths.push(path.with_extension("json"));
    }
    paths.push(path.join("index.idea"));
    paths.push(path.join("index.json"));
    paths
}

/// Select only data formats; extensionless exact files are Idea sources.
pub(super) fn kind(path: &Path) -> Result<SourceKind, LoadError> {
    match path.extension().and_then(|extension| extension.to_str()) {
        None | Some("idea") => Ok(SourceKind::Idea),
        Some("json") => Ok(SourceKind::Json),
        _ => Err(LoadError::classified(
            crate::TransformErrorKind::Parse,
            "unsupported schema source extension",
        )),
    }
}

/// Normalize virtual absolute paths without asking the host filesystem or current directory.
pub(super) fn normalize(path: &Path) -> PathBuf {
    let mut result = PathBuf::new();
    // Virtual roots are absolute, so parent traversal at the root stays at the root.
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                result.pop();
            }
            component => result.push(component.as_os_str()),
        }
    }
    result
}

/// Prefer a portable project-relative display name without changing canonical identity.
pub(super) fn logical_name(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .into_owned()
}
