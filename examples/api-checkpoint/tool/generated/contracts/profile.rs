//! Owned Profile contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: String,
    /// Declared stored value.
    #[serde(rename = "image")]
    pub image: Option<String>,
    /// Declared stored value.
    #[serde(rename = "name")]
    pub name: String,
    /// Declared stored value.
    #[serde(rename = "type")]
    pub type_: String,
    /// Declared stored value.
    #[serde(rename = "roles")]
    pub roles: Vec<String>,
    /// Declared stored value.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
    /// Declared stored value.
    #[serde(rename = "references")]
    pub references: Option<stackpress_framework_schema::JsonObject>,
    /// Declared stored value.
    #[serde(rename = "active")]
    pub active: bool,
    /// Declared stored value.
    #[serde(rename = "created")]
    pub created: stackpress_framework_schema::DatetimeValue,
    /// Declared stored value.
    #[serde(rename = "updated")]
    pub updated: stackpress_framework_schema::DatetimeValue,
}
impl Profile {
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"image\",\"rust_name\":\"image\",\"column\":\"image\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"type\",\"rust_name\":\"type_\",\"column\":\"type_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"roles\",\"rust_name\":\"roles\",\"column\":\"roles\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"references\",\"rust_name\":\"references\",\"column\":\"references\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
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
        rename = "image",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub image: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub name: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "type",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub type_: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "roles",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub roles: stackpress_framework_schema::Input<Vec<String>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub tags: stackpress_framework_schema::Input<Vec<String>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "references",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub references: stackpress_framework_schema::Input<stackpress_framework_schema::JsonObject>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub active: stackpress_framework_schema::Input<bool>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "created",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub created: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
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
            image: match self.image {
                stackpress_framework_schema::Input::Missing => None,
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("image", &value)?;
                    value
                }),
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
            type_: match self.type_ {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("\"person\"").map_err(
                        |cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "literal default decoding failed",
                            )
                            .with_cause(cause)
                        },
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
                            budget.charge("type", &value)?;
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
                    budget.charge("type", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            roles: match self.roles {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("[\"GUEST\"]").map_err(
                        |cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "literal default decoding failed",
                            )
                            .with_cause(cause)
                        },
                    )?;
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
                            budget.charge("roles", &value)?;
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
                    budget.charge("roles", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            tags: match self.tags {
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
                            budget.charge("tags", &value)?;
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
                    budget.charge("tags", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            references: match self.references {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("{}").map_err(|cause| {
                        stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Codec,
                            "literal default decoding failed",
                        )
                        .with_cause(cause)
                    })?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        None
                    } else {
                        let value: stackpress_framework_schema::JsonObject =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        Some({
                            budget.charge("references", &value)?;
                            value
                        })
                    }
                }
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("references", &value)?;
                    value
                }),
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
