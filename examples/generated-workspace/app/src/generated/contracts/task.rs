//! Owned Task contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// Declared stored value.
    #[serde(rename = "title")]
    pub title: String,
    /// Declared stored value.
    #[serde(rename = "done")]
    pub done: bool,
}
impl Task {
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
            let value = &self.project_id;
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
            let value = &self.done;
            let _ = value;
        }
        Ok(())
    }
}
/// Create input preserving omitted, supplied and null values.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateTask {
    /// Explicit create value or omission.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "projectId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub project_id: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub title: stackpress_framework_schema::Input<String>,
    /// Explicit create value or omission.
    #[serde(
        rename = "done",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub done: stackpress_framework_schema::Input<bool>,
}
impl CreateTask {
    /// Prepare literal defaults once; reject missing/null required fields.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Task> {
        use stackpress_framework_schema::{Input, SchemaError, SchemaErrorKind};
        let value = Task {
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
            project_id: match self.project_id {
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
            done: match self.done {
                Input::Missing => serde_json::from_str("false").map_err(|cause| {
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
