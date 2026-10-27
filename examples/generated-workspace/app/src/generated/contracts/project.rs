//! Owned Project contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "title")]
    pub title: String,
    /// Declared stored value.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    /// Declared stored value.
    #[serde(rename = "state")]
    pub state: super::project_state::ProjectState,
    /// Declared stored value.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
}
impl Project {
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
            let value = &self.title;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 || value.is_empty() {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        {
            let value = &self.owner_id;
            // Text is unchanged UTF-8; size and declared presence are checked.
            if value.len() > 1024 * 1024 {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Invalid,
                    "text validation failed",
                ));
            }
        }
        {
            let value = &self.state;
            let _ = value;
        }
        {
            let value = &self.tags;
            // Lists have an explicit finite element budget.
            if value.len() > 1024 {
                return Err(stackpress_framework_schema::SchemaError::new(
                    stackpress_framework_schema::SchemaErrorKind::Limit,
                    "list element budget exceeded",
                ));
            }
            for value in value {
                // Text is unchanged UTF-8; size and declared presence are checked.
                if value.len() > 1024 * 1024 {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "text validation failed",
                    ));
                }
            }
        }
        Ok(())
    }
}
/// Create input preserving omitted, supplied and null values.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProject {
    /// Explicit create value or omission.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub title: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub owner_id: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "state",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub state: stackpress_framework_schema::Input<super::project_state::ProjectState>,
    /// Explicit create value or omission.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub tags: stackpress_framework_schema::Input<Vec<String>>,
}
impl CreateProject {
    /// Prepare literal defaults once; reject missing/null required fields.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Project> {
        use stackpress_framework_schema::{Input, SchemaError, SchemaErrorKind};
        let value = Project {
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
            title: match self.title {
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
            owner_id: match self.owner_id {
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
            state: match self.state {
                Input::Missing => serde_json::from_str("\"open\"").map_err(|cause| {
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
            tags: match self.tags {
                Input::Missing => serde_json::from_str("[]").map_err(|cause| {
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
        };
        // Validate nested values and finite collections before any IO.
        value.validate()?;
        Ok(value)
    }
}
