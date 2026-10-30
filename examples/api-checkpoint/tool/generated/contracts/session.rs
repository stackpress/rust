//! Owned Session contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "applicationId")]
    pub application_id: String,
    /// Declared stored value.
    #[serde(rename = "profileId")]
    pub profile_id: String,
    /// Declared stored value.
    #[serde(rename = "secret")]
    pub secret: String,
    /// Declared stored value.
    #[serde(rename = "scopes")]
    pub scopes: Vec<String>,
    /// Declared stored value.
    #[serde(rename = "active")]
    pub active: bool,
    /// Declared stored value.
    #[serde(rename = "expires")]
    pub expires: stackpress_framework_schema::DatetimeValue,
    /// Declared stored value.
    #[serde(rename = "created")]
    pub created: stackpress_framework_schema::DatetimeValue,
    /// Declared stored value.
    #[serde(rename = "updated")]
    pub updated: stackpress_framework_schema::DatetimeValue,
}
impl Session {
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Session\",\"rust_name\":\"Session\",\"module\":\"session\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"applicationId\",\"rust_name\":\"application_id\",\"column\":\"application_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"App is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"profileId\",\"rust_name\":\"profile_id\",\"column\":\"profile_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"scopes\",\"rust_name\":\"scopes\",\"column\":\"scopes\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"expires\",\"rust_name\":\"expires\",\"column\":\"expires\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"application\",\"rust_name\":\"application\",\"column\":\"application\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Application\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"applicationId\",\"foreign\":\"id\",\"inverse\":\"session\"},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"profile\",\"rust_name\":\"profile\",\"column\":\"profile\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
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
pub struct CreateSession {
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "applicationId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub application_id: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "profileId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub profile_id: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "secret",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub secret: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "scopes",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub scopes: stackpress_framework_schema::Input<Vec<String>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub active: stackpress_framework_schema::Input<bool>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "expires",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub expires: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "created",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub created: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
}
impl CreateSession {
    /// Prepare using no ambient clock or provider.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Session> {
        self.prepare_with(&stackpress_framework_schema::ValueContext::default())
    }
    /// Fill defaults exactly once and validate all nested business rules once before effects.
    pub fn prepare_with(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<Session> {
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
    ) -> stackpress_framework_schema::SchemaResult<Session> {
        let _ = context;
        Ok(Session {
            // Explicit null never falls through to a literal or host default.
            id: match self.id {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<stackpress_framework_schema::ValueRule>(
                        "{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"}",
                    )
                    .map_err(|cause| {
                        stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Codec,
                            "value rule decoding failed",
                        )
                        .with_cause(cause)
                    })?
                    .resolve_for(
                        &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                            "{\"kind\":\"String\"}",
                        )
                        .map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "provider type decoding failed",
                            )
                            .with_cause(cause)
                        })?,
                        context,
                    )?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        return Err(stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Invalid,
                            "required default is null",
                        ));
                    } else {
                        let value: String = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("id", &value)?;
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
                    budget.charge("id", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            application_id: match self.application_id {
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
                    budget.charge("applicationId", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            profile_id: match self.profile_id {
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
                    budget.charge("profileId", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            secret: match self.secret {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<stackpress_framework_schema::ValueRule>(
                        "{\"provider\":\"cuid\",\"args\":[],\"revision\":\"cuid2-3.3.0\"}",
                    )
                    .map_err(|cause| {
                        stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Codec,
                            "value rule decoding failed",
                        )
                        .with_cause(cause)
                    })?
                    .resolve_for(
                        &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                            "{\"kind\":\"String\"}",
                        )
                        .map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "provider type decoding failed",
                            )
                            .with_cause(cause)
                        })?,
                        context,
                    )?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        return Err(stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Invalid,
                            "required default is null",
                        ));
                    } else {
                        let value: String = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("secret", &value)?;
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
                    budget.charge("secret", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            scopes: match self.scopes {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("[]").map_err(|cause| {
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
                        let value: Vec<String> = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("scopes", &value)?;
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
                    budget.charge("scopes", &value)?;
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
            expires: match self.expires {
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
                    budget.charge("expires", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            created: match self.created {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<stackpress_framework_schema::ValueRule>(
                        "{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"}",
                    )
                    .map_err(|cause| {
                        stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Codec,
                            "value rule decoding failed",
                        )
                        .with_cause(cause)
                    })?
                    .resolve_for(
                        &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                            "{\"kind\":\"Datetime\"}",
                        )
                        .map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "provider type decoding failed",
                            )
                            .with_cause(cause)
                        })?,
                        context,
                    )?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        return Err(stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Invalid,
                            "required default is null",
                        ));
                    } else {
                        let value: stackpress_framework_schema::DatetimeValue =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("created", &value)?;
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
                    budget.charge("created", &value)?;
                    value
                }
            },
            // Generated values cannot be supplied by the caller.
            updated: {
                let raw = serde_json::from_str::<stackpress_framework_schema::ValueRule>(
                    "{\"provider\":\"now\",\"args\":[],\"revision\":\"1\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "value rule decoding failed",
                    )
                    .with_cause(cause)
                })?
                .resolve_for(
                    &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                        "{\"kind\":\"Datetime\"}",
                    )
                    .map_err(|cause| {
                        stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Codec,
                            "provider type decoding failed",
                        )
                        .with_cause(cause)
                    })?,
                    context,
                )?;
                stackpress_framework_schema::check_json(&raw)?;
                if raw.is_null() {
                    return Err(stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Invalid,
                        "required default is null",
                    ));
                } else {
                    let value: stackpress_framework_schema::DatetimeValue =
                        serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                    {
                        budget.charge("updated", &value)?;
                        value
                    }
                }
            },
        })
    }
}
