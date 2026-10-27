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
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"projects\",\"rust_name\":\"projects\",\"column\":\"projects\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Project\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
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
pub struct CreateProfile {
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub name: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub active: stackpress_framework_schema::Input<bool>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub address: stackpress_framework_schema::Input<super::address::CreateAddress>,
}
impl CreateProfile {
    /// Prepare using no ambient clock or provider.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Profile> {
        self.prepare_with(&stackpress_framework_schema::ValueContext::default())
    }
    /// Fill defaults exactly once and validate all nested business rules once before effects.
    pub fn prepare_with(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<Profile> {
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
    ) -> stackpress_framework_schema::SchemaResult<Profile> {
        let _ = context;
        Ok(Profile {
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
            name: match self.name {
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
                    budget.charge("name", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            active: match self.active {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("true").map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "literal default decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        return Err(stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Invalid,
                            "required default is null",
                        ));
                    } else {
                        let value: bool = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("active", &value)?;
                            value
                        }
                    }
                }
                stackpress_framework_schema::Input::Null => {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required create field is null",
                    ));
                }
                stackpress_framework_schema::Input::Value(value) => {
                    budget.charge("active", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            address: match self.address {
                stackpress_framework_schema::Input::Missing => None,
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.container("address")?;
                    value.prepare_bounded(context, budget)?
                }),
            },
        })
    }
}
