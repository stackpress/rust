//! Owned MatrixAddress contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatrixAddress {
    /// Declared stored value.
    #[serde(rename = "city")]
    pub city: String,
    /// Declared stored value.
    #[serde(rename = "zip")]
    pub zip: Option<String>,
    /// Declared stored value.
    #[serde(rename = "country")]
    pub country: String,
    /// Declared stored value.
    #[serde(rename = "nonce")]
    pub nonce: String,
}
impl MatrixAddress {
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
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
pub struct CreateMatrixAddress {
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "city",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub city: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "zip",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub zip: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "country",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub country: stackpress_framework_schema::Input<String>,
}
impl CreateMatrixAddress {
    /// Prepare using no ambient clock or provider.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<MatrixAddress> {
        self.prepare_with(&stackpress_framework_schema::ValueContext::default())
    }
    /// Fill defaults exactly once and validate all nested business rules once before effects.
    pub fn prepare_with(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<MatrixAddress> {
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
    ) -> stackpress_framework_schema::SchemaResult<MatrixAddress> {
        let _ = context;
        Ok(MatrixAddress {
            // Explicit null never falls through to a literal or host default.
            city: match self.city {
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
                    budget.charge("city", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            zip: match self.zip {
                stackpress_framework_schema::Input::Missing => None,
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("zip", &value)?;
                    value
                }),
            },
            // Explicit null never falls through to a literal or host default.
            country: match self.country {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("\"PH\"").map_err(|cause| {
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
                        let value: String = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("country", &value)?;
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
                    budget.charge("country", &value)?;
                    value
                }
            },
            // Generated values cannot be supplied by the caller.
            nonce: {
                let raw = serde_json::from_str::<stackpress_framework_schema::ValueRule>(
                    "{\"provider\":\"nanoid\",\"args\":[8],\"revision\":\"app-fixture-1\"}",
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
                        budget.charge("nonce", &value)?;
                        value
                    }
                }
            },
        })
    }
}
