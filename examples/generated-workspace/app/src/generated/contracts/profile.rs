//! Owned Profile contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "name")]
    pub name: String,
    /// Declared stored value.
    #[serde(rename = "active")]
    pub active: bool,
    /// Declared stored value.
    #[serde(rename = "address")]
    pub address: Option<super::address::Address>,
}
impl Profile {
    /// Validate finite owned values before storage or projection.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        {
            let value = &self.id;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        {
            let value = &self.name;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 || value.is_empty() {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        {
            let value = &self.active;
            let _ = value;
        }
        if let Some(value) = &self.address {
            // Embedded validation uses the checked acyclic schema graph.
            value.validate()?;
        }
        Ok(())
    }
}
/// Create input preserving omitted, supplied and null values.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProfile {
    /// Explicit create value or omission.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub name: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub active: stackpress_framework_schema::Input<bool>,
    /// Explicit create value or omission.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub address: stackpress_framework_schema::Input<super::address::Address>,
}
impl CreateProfile {
    /// Prepare literal defaults once; reject missing/null required fields.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Profile> {
        use stackpress_framework_schema::{Input, SchemaError, SchemaErrorKind};
        let value = Profile {
            // Omission consults only the declared literal default.
            id: match self.id {
                Input::Missing => {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "required create field is missing",
                    ));
                }
                Input::Null => {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                Input::Value(value) => value,
            },
            // Omission consults only the declared literal default.
            name: match self.name {
                Input::Missing => {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "required create field is missing",
                    ));
                }
                Input::Null => {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                Input::Value(value) => value,
            },
            // Omission consults only the declared literal default.
            active: match self.active {
                Input::Missing => serde_json::from_str("true").map_err(|cause| {
                    SchemaError::new(SchemaErrorKind::Codec, "default decoding failed")
                        .with_cause(cause)
                })?,
                Input::Null => {
                    return Err(SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                Input::Value(value) => value,
            },
            // Omission consults only the declared literal default.
            address: match self.address {
                Input::Missing => None,
                Input::Null => None,
                Input::Value(value) => Some(value),
            },
        };
        // Validate nested values and finite collections before any IO.
        value.validate()?;
        Ok(value)
    }
}
