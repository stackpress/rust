//! Bounded explicit data inputs for application bootstrap, without environment or file reads.

use stackpress_lib::{Error, Nest, ParseLimits, Result};

/// Ordered named object layers supplied by application code or a CLI data-file adapter.
///
/// M0 retains inputs for the compile-time composition proof. Section decoding,
/// recursive merge and unknown-section validation belong to the M1 builder.
#[derive(Default)]
pub struct ConfigLayers {
    /// JSON-only owned layers; these cannot select uncompiled plugin code.
    layers: Vec<(String, serde_json::Value)>,
    /// Aggregate encoded size prevents many individually valid layers growing without bound.
    bytes: usize,
}

impl ConfigLayers {
    /// Start with no external configuration; applications still own typed defaults.
    pub fn new() -> Self {
        // Empty configuration does not read process environment or the working directory.
        Self::default()
    }

    /// Validate and retain one explicitly supplied named JSON object.
    ///
    /// # Errors
    /// Rejects empty/control-character labels, oversized text, non-object JSON,
    /// or values exceeding the foundation's default depth/entry/byte budgets.
    pub fn json(&mut self, label: &str, text: &str) -> Result<&mut Self> {
        // Check the raw input budget before allocating an untrusted JSON document.
        if text.len() > ParseLimits::default().max_bytes {
            return Err(Error::limit("configuration input exceeds byte limit"));
        }
        let value = serde_json::from_str(text)?;
        self.value(label, value)
    }

    /// Retain a validated data-only layer, preserving its supplied order.
    ///
    /// # Errors
    /// Rejects invalid labels and non-object or over-budget values before insertion.
    pub fn value(&mut self, label: &str, value: serde_json::Value) -> Result<&mut Self> {
        // Labels are diagnostics, so they must not inject terminal control sequences.
        if label.is_empty() || label.len() > 256 || label.chars().any(char::is_control) {
            return Err(Error::invalid("invalid configuration layer label"));
        }
        let value = Nest::from_object(value)?.to_json()?;
        let bytes = self
            .bytes
            .checked_add(serde_json::to_vec(&value)?.len())
            .ok_or_else(|| Error::limit("configuration layer byte total overflow"))?;
        let limits = ParseLimits::default();
        if bytes > limits.max_bytes || self.layers.len() >= limits.max_entries {
            return Err(Error::limit("aggregate configuration layers exceed limits"));
        }
        // Publish only after validation; a returned error leaves all earlier layers intact.
        self.layers.push((label.to_owned(), value));
        self.bytes = bytes;
        Ok(self)
    }

    /// Inspect explicitly supplied layers without merging or logging their contents.
    pub fn entries(&self) -> impl Iterator<Item = (&str, &serde_json::Value)> {
        // Borrow values so the composition callback need not clone entire documents.
        self.layers
            .iter()
            .map(|(label, value)| (label.as_str(), value))
    }
}
