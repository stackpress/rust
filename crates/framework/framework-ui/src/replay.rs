//! Bounded inert Resource replay; no generated JavaScript or unchecked decoding.
use stackpress_framework_view::{PageLimits, ViewError, escape_script_json};
use std::collections::BTreeSet;

/// Maximum Resource values in one buffered M0 root.
pub const MAX_RESOURCES: usize = 64;
/// Bounded sparse resource identity for browser replay arrays.
pub const MAX_RESOURCE_ID: usize = 1024;

/// Validated encoded values, retained as JSON strings for Leptos's own codecs.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceReplay(Vec<(usize, String)>);

impl ResourceReplay {
    /// Validate count, identities, uniqueness and logical serialized byte ceiling.
    ///
    /// # Errors
    /// Rejects malformed identities/duplicates or count/byte/policy excess.
    pub fn new(values: Vec<(usize, String)>, byte_limit: usize) -> Result<Self, ViewError> {
        PageLimits {
            snapshot_bytes: byte_limit,
            ..PageLimits::default()
        }
        .validate()?;
        if values.len() > MAX_RESOURCES {
            return Err(ViewError::SnapshotLimit);
        }
        // A sparse browser array must never allocate an unbounded ID or overwrite
        // an earlier entry through duplicate resource identities.
        let mut ids = BTreeSet::new();
        for (id, _) in &values {
            if *id > MAX_RESOURCE_ID || !ids.insert(*id) {
                return Err(ViewError::InvalidSnapshot);
            }
        }
        // Measure logical serialized bytes before HTML escaping expands them.
        let json = serde_json::to_string(&values).map_err(ViewError::from_source)?;
        if json.len() > byte_limit {
            return Err(ViewError::SnapshotLimit);
        }
        Ok(Self(values))
    }

    /// Decode inert script text under the same logical/expanded byte limits.
    ///
    /// # Errors
    /// Rejects oversized/malformed JSON before exposing entries to a browser root.
    pub fn decode(text: &str, byte_limit: usize) -> Result<Self, ViewError> {
        PageLimits {
            snapshot_bytes: byte_limit,
            ..PageLimits::default()
        }
        .validate()?;
        // JSON HTML escapes expand a byte by at most six characters.
        if text.len() > byte_limit.saturating_mul(6) {
            return Err(ViewError::SnapshotLimit);
        }
        let values = serde_json::from_str(text).map_err(|_| ViewError::InvalidSnapshot)?;
        Self::new(values, byte_limit)
    }

    /// Serialize for an inert application/json script without HTML terminators.
    ///
    /// # Errors
    /// Retains the concrete serializer cause in a safe ViewError envelope.
    pub fn script_json(&self) -> Result<String, ViewError> {
        serde_json::to_string(&self.0)
            .map(|json| escape_script_json(&json))
            .map_err(ViewError::from_source)
    }

    /// Borrow validated pairs; no setter or unchecked deserializer is exposed.
    pub fn entries(&self) -> &[(usize, String)] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Replay preserves exact hostile Unicode data, rejecting ambiguity and resource excess.
    #[test]
    fn replay_roundtrip_and_limits() {
        let replay =
            ResourceReplay::new(vec![(0, "</script>&\u{2028}\u{2029}".into())], 1024).unwrap();
        let encoded = replay.script_json().unwrap();
        assert!(!encoded.contains('<'));
        assert_eq!(ResourceReplay::decode(&encoded, 1024).unwrap(), replay);
        assert_eq!(replay.entries().len(), 1);
        assert_eq!(
            ResourceReplay::new(vec![(0, "x".into()), (0, "y".into())], 100),
            Err(ViewError::InvalidSnapshot)
        );
        assert_eq!(
            ResourceReplay::new(vec![(MAX_RESOURCE_ID + 1, "x".into())], 100),
            Err(ViewError::InvalidSnapshot)
        );
        assert_eq!(
            ResourceReplay::new((0..65).map(|id| (id, String::new())).collect(), 1024),
            Err(ViewError::SnapshotLimit)
        );
        assert_eq!(
            ResourceReplay::new(vec![(0, "long".into())], 1),
            Err(ViewError::SnapshotLimit)
        );
        assert_eq!(
            ResourceReplay::decode("bad", 100),
            Err(ViewError::InvalidSnapshot)
        );
        assert_eq!(
            ResourceReplay::decode(&" ".repeat(61), 10),
            Err(ViewError::SnapshotLimit)
        );
        assert_eq!(
            ResourceReplay::decode("[]", 0),
            Err(ViewError::SnapshotLimit)
        );
    }
}
