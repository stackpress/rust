//! TEMPORARY: replace this import with framework_view::PageAssets at M1 merge.
//!
//! Only the bootstrap-needed field and decoder are mirrored. No asset discovery,
//! alternate module import, network fetch or missing-element fallback is added.

use stackpress_example_profile_ui_shared::m1_compat::BasePath;
use stackpress_framework_view::ViewError;

/// Checked asset bootstrap subset consumed by this client.
pub(crate) struct PageAssets {
    /// Canonical trusted application mount.
    pub(crate) base: BasePath,
}

impl PageAssets {
    /// Decode bounded inert metadata and reject malformed paths or build drift.
    pub(crate) fn decode(json: &str, build: &str) -> Result<Self, ViewError> {
        if json.len() > 16 * 1024 {
            return Err(ViewError::InvalidSnapshot);
        }
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|_| ViewError::InvalidSnapshot)?;
        let data = value.as_object().ok_or(ViewError::InvalidSnapshot)?;
        if data.len() != 4
            || !data
                .keys()
                .all(|key| matches!(key.as_str(), "build" | "base" | "module" | "stylesheet"))
        {
            return Err(ViewError::InvalidSnapshot);
        }
        if data.get("build").and_then(serde_json::Value::as_str) != Some(build) {
            return Err(ViewError::ContractMismatch);
        }
        let base = BasePath::new(
            data.get("base")
                .and_then(serde_json::Value::as_str)
                .ok_or(ViewError::InvalidSnapshot)?,
        )
        .map_err(|_| ViewError::InvalidSnapshot)?;
        let module = data
            .get("module")
            .and_then(serde_json::Value::as_str)
            .ok_or(ViewError::InvalidSnapshot)?;
        let stylesheet = match data.get("stylesheet") {
            Some(serde_json::Value::Null) => None,
            Some(serde_json::Value::String(path)) => Some(path.as_str()),
            _ => return Err(ViewError::InvalidSnapshot),
        };
        for path in std::iter::once(module).chain(stylesheet) {
            // These are resource paths, so query strings and executable schemes are excluded.
            if path.contains('?') || path.contains('#') || path.chars().any(char::is_whitespace) {
                return Err(ViewError::InvalidSnapshot);
            }
            base.join(path).map_err(|_| ViewError::InvalidSnapshot)?;
        }
        Ok(Self { base })
    }
}
