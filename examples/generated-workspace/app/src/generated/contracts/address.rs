//! Owned Address contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Address {
    /// Declared stored value.
    #[serde(rename = "city")]
    pub city: String,
    /// Declared stored value.
    #[serde(rename = "country")]
    pub country: String,
}
impl Address {
    /// Validate finite owned values before storage or projection.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        {
            let value = &self.city;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 || value.is_empty() {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        {
            let value = &self.country;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        Ok(())
    }
}
