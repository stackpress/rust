//! The compatibility accessor returns the generated declaration without a second inventory.
use super::RecordSchema;
impl RecordSchema {
    /// Complete source model metadata; no seed, executor or credential enters this value.
    pub fn model(&self) -> stackpress_framework_schema::Model {
        self.describe().clone()
    }
}
