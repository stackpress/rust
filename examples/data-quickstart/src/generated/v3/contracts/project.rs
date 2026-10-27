//! Owned Project contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "heading")]
    pub heading: String,
    /// Declared stored value.
    #[serde(rename = "summary")]
    pub summary: Option<String>,
    /// Declared stored value.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
}
impl Project {
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"heading\",\"rust_name\":\"heading\",\"column\":\"heading\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"summary\",\"rust_name\":\"summary\",\"column\":\"summary\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"owner\",\"rust_name\":\"owner\",\"column\":\"owner\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tasks\",\"rust_name\":\"tasks\",\"column\":\"tasks\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Task\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
    }
    /// Validate stored values without rerunning time-relative creation rules.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        self.validate_with(&stackpress_framework_schema::ValueContext::default(), true)
    }
    /// Validate against one explicit clock/provider context.
    pub fn validate_with(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
        stored: bool,
    ) -> stackpress_framework_schema::SchemaResult<()> {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::record_schema()?
            .validate(&value, context, false, stored)?
            .into_result()
    }
}
/// Create input preserving omitted, null and supplied values at every embedded depth.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CreateProject {
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "heading",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub heading: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "summary",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub summary: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub owner_id: stackpress_framework_schema::Input<String>,
}
impl CreateProject {
    /// Prepare using no ambient clock or provider.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Project> {
        self.prepare_with(&stackpress_framework_schema::ValueContext::default())
    }
    /// Fill defaults exactly once and validate all nested business rules once before effects.
    pub fn prepare_with(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<Project> {
        let mut budget = stackpress_framework_schema::PreparationBudget::default();
        let value = self.prepare_bounded(context, &mut budget)?;
        value.validate_with(context, false)?;
        Ok(value)
    }
    /// Internal cross-module composition shares the caller's aggregate budget; validation follows once.
    pub(crate) fn prepare_bounded(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
        budget: &mut stackpress_framework_schema::PreparationBudget,
    ) -> stackpress_framework_schema::SchemaResult<Project> {
        let _ = context;
        Ok(Project {
            // Explicit null never falls through to a literal or host default.
            id: match self.id {
                stackpress_framework_schema::Input::Missing => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is missing",
                    ));
                }
                stackpress_framework_schema::Input::Null => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                stackpress_framework_schema::Input::Value(value) => {
                    budget.charge("id", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            heading: match self.heading {
                stackpress_framework_schema::Input::Missing => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is missing",
                    ));
                }
                stackpress_framework_schema::Input::Null => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                stackpress_framework_schema::Input::Value(value) => {
                    budget.charge("heading", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            summary: match self.summary {
                stackpress_framework_schema::Input::Missing => None,
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("summary", &value)?;
                    value
                }),
            },
            // Explicit null never falls through to a literal or host default.
            owner_id: match self.owner_id {
                stackpress_framework_schema::Input::Missing => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is missing",
                    ));
                }
                stackpress_framework_schema::Input::Null => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                stackpress_framework_schema::Input::Value(value) => {
                    budget.charge("ownerId", &value)?;
                    value
                }
            },
        })
    }
}
