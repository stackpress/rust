//! Shared application values; environment data is supplied separately through CLI -b.
//!
//! Keep computed/environment-derived values and typed code bindings in this application module.
//! This example needs only a data layer; no config file or environment variable is read here.

use stackpress_lib::Result;
use stackpress_server::ConfigLayers;

/// Supply common application policy above plugin defaults and below environment-file data.
///
/// # Errors
/// Propagates the existing ConfigLayers JSON/input-budget validation error.
pub(super) fn shared() -> Result<ConfigLayers> {
    let mut layers = ConfigLayers::new();
    // The environments restore the canonical three-row policy; without a file, use two.
    layers.json("shared", r#"{"profiles":{"limit":2}}"#)?;
    Ok(layers)
}
