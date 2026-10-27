//! Explicit application projections. Trusted stored rows must use project() before public output.
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicMatrixAddress {
    /// Approved public field.
    #[serde(rename = "city")]
    pub city: String,
    /// Approved public field.
    #[serde(rename = "country")]
    pub country: String,
}
impl super::matrix_address::MatrixAddress {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicMatrixAddress> {
        self.validate()?;
        let public = PublicMatrixAddress {
            city: self.city.clone(),
            country: self.country.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateMatrixAddress {
    /// Writable public input.
    #[serde(
        rename = "city",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub city: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "country",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub country: stackpress_framework_schema::Input<String>,
}
impl PublicCreateMatrixAddress {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::matrix_address::CreateMatrixAddress {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::matrix_address::MatrixAddress> {
        self.prepare_with(context)
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPayload {
    /// Approved public field.
    #[serde(rename = "row")]
    pub row: String,
    /// Approved public field.
    #[serde(rename = "state")]
    pub state: String,
    /// Approved public field.
    #[serde(rename = "value")]
    pub value: i64,
}
impl super::payload::Payload {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicPayload> {
        self.validate()?;
        let public = PublicPayload {
            row: self.row.clone(),
            state: self.state.clone(),
            value: self.value,
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreatePayload {
    /// Writable public input.
    #[serde(
        rename = "row",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub row: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "state",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub state: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "value",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub value: stackpress_framework_schema::Input<i64>,
}
impl PublicCreatePayload {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::payload::CreatePayload {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::payload::Payload> {
        self.prepare_with(context)
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicMatrix {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: i64,
    /// Approved public field.
    #[serde(rename = "text")]
    pub text: String,
    /// Approved public field.
    #[serde(rename = "count")]
    pub count: i64,
    /// Approved public field.
    #[serde(rename = "real")]
    pub real: stackpress_framework_schema::FiniteNumber,
    /// Approved public field.
    #[serde(rename = "float")]
    pub float: Option<stackpress_framework_schema::FiniteNumber>,
    /// Approved public field.
    #[serde(rename = "flag")]
    pub flag: bool,
    /// Approved public field.
    #[serde(rename = "day")]
    pub day: stackpress_framework_schema::DateValue,
    /// Approved public field.
    #[serde(rename = "time")]
    pub time: stackpress_framework_schema::TimeValue,
    /// Approved public field.
    #[serde(rename = "stamp")]
    pub stamp: stackpress_framework_schema::DatetimeValue,
    /// Approved public field.
    #[serde(rename = "object")]
    pub object: stackpress_framework_schema::JsonObject,
    /// Approved public field.
    #[serde(rename = "choice")]
    pub choice: super::count::Count,
    /// Approved public field.
    #[serde(rename = "truth")]
    pub truth: super::truth::Truth,
    /// Approved public field.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
    /// Approved public field.
    #[serde(rename = "numbers")]
    pub numbers: Vec<stackpress_framework_schema::FiniteNumber>,
    /// Approved public field.
    #[serde(rename = "dates")]
    pub dates: Vec<stackpress_framework_schema::DateValue>,
    /// Approved public field.
    #[serde(rename = "times")]
    pub times: Vec<stackpress_framework_schema::TimeValue>,
    /// Approved public field.
    #[serde(rename = "instants")]
    pub instants: Vec<stackpress_framework_schema::DatetimeValue>,
    /// Approved public field.
    #[serde(rename = "objects")]
    pub objects: Vec<stackpress_framework_schema::JsonObject>,
    /// Approved public field.
    #[serde(rename = "address")]
    pub address: Option<PublicMatrixAddress>,
    /// Approved public field.
    #[serde(rename = "siblings")]
    pub siblings: Vec<PublicMatrixAddress>,
    /// Approved public field.
    #[serde(rename = "favorite")]
    pub favorite: PublicMatrixAddress,
    /// Approved public field.
    #[serde(rename = "created")]
    pub created: stackpress_framework_schema::DatetimeValue,
    /// Approved public field.
    #[serde(rename = "token")]
    pub token: String,
}
impl super::matrix::Matrix {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicMatrix> {
        self.validate()?;
        let public = PublicMatrix {
            id: self.id,
            text: self.text.clone(),
            count: self.count,
            real: self.real,
            float: self.float,
            flag: self.flag,
            day: self.day.clone(),
            time: self.time.clone(),
            stamp: self.stamp.clone(),
            object: self.object.clone(),
            choice: self.choice.clone(),
            truth: self.truth.clone(),
            tags: self.tags.clone(),
            numbers: self.numbers.clone(),
            dates: self.dates.clone(),
            times: self.times.clone(),
            instants: self.instants.clone(),
            objects: self.objects.clone(),
            address: self
                .address
                .as_ref()
                .map(|value| value.project())
                .transpose()?,
            siblings: self
                .siblings
                .iter()
                .map(|value| value.project())
                .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?,
            favorite: self.favorite.project()?,
            created: self.created.clone(),
            token: self.token.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateMatrix {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<i64>,
    /// Writable public input.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub text: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "count",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub count: stackpress_framework_schema::Input<i64>,
    /// Writable public input.
    #[serde(
        rename = "real",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub real: stackpress_framework_schema::Input<stackpress_framework_schema::FiniteNumber>,
    /// Writable public input.
    #[serde(
        rename = "float",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub float: stackpress_framework_schema::Input<stackpress_framework_schema::FiniteNumber>,
    /// Writable public input.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub flag: stackpress_framework_schema::Input<bool>,
    /// Writable public input.
    #[serde(
        rename = "day",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub day: stackpress_framework_schema::Input<stackpress_framework_schema::DateValue>,
    /// Writable public input.
    #[serde(
        rename = "time",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub time: stackpress_framework_schema::Input<stackpress_framework_schema::TimeValue>,
    /// Writable public input.
    #[serde(
        rename = "stamp",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub stamp: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
    /// Writable public input.
    #[serde(
        rename = "object",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub object: stackpress_framework_schema::Input<stackpress_framework_schema::JsonObject>,
    /// Writable public input.
    #[serde(
        rename = "choice",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub choice: stackpress_framework_schema::Input<super::count::Count>,
    /// Writable public input.
    #[serde(
        rename = "truth",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub truth: stackpress_framework_schema::Input<super::truth::Truth>,
    /// Writable public input.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub tags: stackpress_framework_schema::Input<Vec<String>>,
    /// Writable public input.
    #[serde(
        rename = "numbers",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub numbers: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::FiniteNumber>>,
    /// Writable public input.
    #[serde(
        rename = "dates",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub dates: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::DateValue>>,
    /// Writable public input.
    #[serde(
        rename = "times",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub times: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::TimeValue>>,
    /// Writable public input.
    #[serde(
        rename = "instants",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub instants:
        stackpress_framework_schema::Input<Vec<stackpress_framework_schema::DatetimeValue>>,
    /// Writable public input.
    #[serde(
        rename = "objects",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub objects: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::JsonObject>>,
    /// Writable public input.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub address: stackpress_framework_schema::Input<PublicCreateMatrixAddress>,
    /// Writable public input.
    #[serde(
        rename = "siblings",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub siblings: stackpress_framework_schema::Input<Vec<PublicCreateMatrixAddress>>,
    /// Writable public input.
    #[serde(
        rename = "favorite",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub favorite: stackpress_framework_schema::Input<PublicCreateMatrixAddress>,
}
impl PublicCreateMatrix {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"Café\",\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0.25,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-02-29\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"12:00:00.12\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-03-01T00:00:00Z\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"nested\":[null,true]},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"city\":\"Default\"},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::matrix::CreateMatrix {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"Café\",\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0.25,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-02-29\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"12:00:00.12\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-03-01T00:00:00Z\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"nested\":[null,true]},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"city\":\"Default\"},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::matrix::Matrix> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchMatrix {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub text: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "count",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub count: stackpress_framework_schema::Patch<i64>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "real",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub real: stackpress_framework_schema::Patch<stackpress_framework_schema::FiniteNumber>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "float",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub float: stackpress_framework_schema::Patch<stackpress_framework_schema::FiniteNumber>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub flag: stackpress_framework_schema::Patch<bool>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "day",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub day: stackpress_framework_schema::Patch<stackpress_framework_schema::DateValue>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "time",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub time: stackpress_framework_schema::Patch<stackpress_framework_schema::TimeValue>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "stamp",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub stamp: stackpress_framework_schema::Patch<stackpress_framework_schema::DatetimeValue>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "object",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub object: stackpress_framework_schema::Patch<stackpress_framework_schema::JsonObject>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "choice",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub choice: stackpress_framework_schema::Patch<super::count::Count>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "truth",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub truth: stackpress_framework_schema::Patch<super::truth::Truth>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub tags: stackpress_framework_schema::Patch<Vec<String>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "numbers",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub numbers: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::FiniteNumber>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "dates",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub dates: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::DateValue>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "times",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub times: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::TimeValue>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "instants",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub instants:
        stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::DatetimeValue>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "objects",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub objects: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::JsonObject>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub address: stackpress_framework_schema::Patch<PublicCreateMatrixAddress>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "siblings",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub siblings: stackpress_framework_schema::Patch<Vec<PublicCreateMatrixAddress>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "favorite",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub favorite: stackpress_framework_schema::Patch<PublicCreateMatrixAddress>,
}
impl PublicPatchMatrix {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
}
/// Ordered model key, with exact scalar types.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatrixKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: i64,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchMatrix {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub text: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "count",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub count: stackpress_framework_schema::Patch<i64>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "real",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub real: stackpress_framework_schema::Patch<stackpress_framework_schema::FiniteNumber>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "float",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub float: stackpress_framework_schema::Patch<stackpress_framework_schema::FiniteNumber>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub flag: stackpress_framework_schema::Patch<bool>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "day",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub day: stackpress_framework_schema::Patch<stackpress_framework_schema::DateValue>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "time",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub time: stackpress_framework_schema::Patch<stackpress_framework_schema::TimeValue>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "stamp",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub stamp: stackpress_framework_schema::Patch<stackpress_framework_schema::DatetimeValue>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "object",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub object: stackpress_framework_schema::Patch<stackpress_framework_schema::JsonObject>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "choice",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub choice: stackpress_framework_schema::Patch<super::count::Count>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "truth",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub truth: stackpress_framework_schema::Patch<super::truth::Truth>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub tags: stackpress_framework_schema::Patch<Vec<String>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "numbers",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub numbers: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::FiniteNumber>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "dates",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub dates: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::DateValue>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "times",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub times: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::TimeValue>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "instants",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub instants:
        stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::DatetimeValue>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "objects",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub objects: stackpress_framework_schema::Patch<Vec<stackpress_framework_schema::JsonObject>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub address: stackpress_framework_schema::Patch<super::matrix_address::MatrixAddress>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "siblings",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub siblings: stackpress_framework_schema::Patch<Vec<super::matrix_address::MatrixAddress>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "favorite",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub favorite: stackpress_framework_schema::Patch<super::matrix_address::MatrixAddress>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "secret",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub secret: stackpress_framework_schema::Patch<String>,
}
impl PatchMatrix {
    /// Reject empty updates and validate supplied fields without applying create defaults.
    pub fn validate(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<()> {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        if value.as_object().is_none_or(|v| v.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Invalid,
                "empty patch is invalid",
            ));
        }
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchMatrix {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
}
/// Approved field predicates; variants cannot name private paths.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum MatrixFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<i64>),
}
impl MatrixFilter {
    /// Check operator applicability and value bounds before future IO.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        match self {
            Self::Id(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Integer\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
        }
    }
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum MatrixSort {
    /// Approved scalar ordering.
    #[serde(rename = "text")]
    Text,
    /// Approved scalar ordering.
    #[serde(rename = "float")]
    Float,
    /// Approved scalar ordering.
    #[serde(rename = "stamp")]
    Stamp,
}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MatrixQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<MatrixFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<MatrixSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for MatrixQuery {
    fn default() -> Self {
        Self {
            q: None,
            filters: Vec::new(),
            sort: Vec::new(),
            includes: Vec::new(),
            skip: 0,
            take: 20,
            total: false,
        }
    }
}
impl MatrixQuery {
    /// Validate the complete allowlist/budget contract before any future query compilation.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        stackpress_framework_schema::query_budget(
            self.q.as_deref(),
            self.filters.len(),
            self.sort.len(),
            &self.includes,
            self.skip,
            self.take,
        )?;
        if self.q.as_ref().is_some_and(|value| !value.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Unsupported,
                "public text search is unavailable",
            ));
        }
        for filter in &self.filters {
            filter.validate()?;
        }
        if !self.includes.is_empty() {
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"19017e7b67932c0bb1b70af22dc5caa5b782b3a66379ca973e5f024b51aedbe8\",\"models\":[{\"id\":\"Matrix\",\"label\":\"Matrix\",\"fields\":[{\"id\":\"Matrix.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"Café\"},\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"relation\":null},{\"id\":\"Matrix.count\",\"wire_name\":\"count\",\"label\":\"count\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.real\",\"wire_name\":\"real\",\"label\":\"real\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0.25},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.float\",\"wire_name\":\"float\",\"label\":\"float\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-02-29\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"12:00:00.12\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.stamp\",\"wire_name\":\"stamp\",\"label\":\"stamp\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-03-01T00:00:00Z\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.object\",\"wire_name\":\"object\",\"label\":\"object\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"nested\":[null,true]}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.numbers\",\"wire_name\":\"numbers\",\"label\":\"numbers\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.dates\",\"wire_name\":\"dates\",\"label\":\"dates\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.times\",\"wire_name\":\"times\",\"label\":\"times\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.instants\",\"wire_name\":\"instants\",\"label\":\"instants\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.objects\",\"wire_name\":\"objects\",\"label\":\"objects\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.siblings\",\"wire_name\":\"siblings\",\"label\":\"siblings\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.favorite\",\"wire_name\":\"favorite\",\"label\":\"favorite\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"city\":\"Default\"}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.created\",\"wire_name\":\"created\",\"label\":\"created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.token\",\"wire_name\":\"token\",\"label\":\"token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.links\",\"wire_name\":\"links\",\"label\":\"links\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Link\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Link\",\"many\":true,\"mapping\":null,\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Link\",\"label\":\"Link\",\"fields\":[{\"id\":\"Link.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Matrix\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Matrix\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"links\"},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"owner.text\",\"owner.favorite.city\"],\"operations\":[]},{\"id\":\"Compound\",\"label\":\"Compound\",\"fields\":[{\"id\":\"Compound.region\",\"wire_name\":\"region\",\"label\":\"region\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Compound.code\",\"wire_name\":\"code\",\"label\":\"code\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.enabled\",\"wire_name\":\"enabled\",\"label\":\"enabled\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.clock\",\"wire_name\":\"clock\",\"label\":\"clock\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.instant\",\"wire_name\":\"instant\",\"label\":\"instant\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.ratio\",\"wire_name\":\"ratio\",\"label\":\"ratio\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null}],\"key\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"query_fields\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Query\",\"label\":\"Query\",\"fields\":[{\"id\":\"Query.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Query.self\",\"wire_name\":\"self\",\"label\":\"self\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"safe\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Query.payload\",\"wire_name\":\"payload\",\"label\":\"payload\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"payload.row\",\"payload.state\"],\"operations\":[]}],\"fieldsets\":[{\"id\":\"MatrixAddress\",\"label\":\"MatrixAddress\",\"fields\":[{\"id\":\"MatrixAddress.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"MatrixAddress.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"PH\"},\"validation\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]},{\"id\":\"Payload\",\"label\":\"Payload\",\"fields\":[{\"id\":\"Payload.row\",\"wire_name\":\"row\",\"label\":\"row\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Matrix", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicLink {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "ownerId")]
    pub owner_id: Option<i64>,
}
impl super::link::Link {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicLink> {
        self.validate()?;
        let public = PublicLink {
            id: self.id.clone(),
            owner_id: self.owner_id,
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateLink {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub owner_id: stackpress_framework_schema::Input<i64>,
}
impl PublicCreateLink {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::link::CreateLink {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::link::Link> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchLink {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub owner_id: stackpress_framework_schema::Patch<i64>,
}
impl PublicPatchLink {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
}
/// Ordered model key, with exact scalar types.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchLink {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub owner_id: stackpress_framework_schema::Patch<i64>,
}
impl PatchLink {
    /// Reject empty updates and validate supplied fields without applying create defaults.
    pub fn validate(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<()> {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        if value.as_object().is_none_or(|v| v.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Invalid,
                "empty patch is invalid",
            ));
        }
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchLink {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Link\",\"rust_name\":\"Link\",\"module\":\"link\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
}
/// Approved field predicates; variants cannot name private paths.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum LinkFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
}
impl LinkFilter {
    /// Check operator applicability and value bounds before future IO.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        match self {
            Self::Id(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"String\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
        }
    }
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LinkSort {}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct LinkQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<LinkFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<LinkSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for LinkQuery {
    fn default() -> Self {
        Self {
            q: None,
            filters: Vec::new(),
            sort: Vec::new(),
            includes: Vec::new(),
            skip: 0,
            take: 20,
            total: false,
        }
    }
}
impl LinkQuery {
    /// Validate the complete allowlist/budget contract before any future query compilation.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        stackpress_framework_schema::query_budget(
            self.q.as_deref(),
            self.filters.len(),
            self.sort.len(),
            &self.includes,
            self.skip,
            self.take,
        )?;
        if self.q.as_ref().is_some_and(|value| !value.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Unsupported,
                "public text search is unavailable",
            ));
        }
        for filter in &self.filters {
            filter.validate()?;
        }
        if !self.includes.is_empty() {
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"19017e7b67932c0bb1b70af22dc5caa5b782b3a66379ca973e5f024b51aedbe8\",\"models\":[{\"id\":\"Matrix\",\"label\":\"Matrix\",\"fields\":[{\"id\":\"Matrix.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"Café\"},\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"relation\":null},{\"id\":\"Matrix.count\",\"wire_name\":\"count\",\"label\":\"count\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.real\",\"wire_name\":\"real\",\"label\":\"real\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0.25},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.float\",\"wire_name\":\"float\",\"label\":\"float\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-02-29\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"12:00:00.12\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.stamp\",\"wire_name\":\"stamp\",\"label\":\"stamp\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-03-01T00:00:00Z\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.object\",\"wire_name\":\"object\",\"label\":\"object\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"nested\":[null,true]}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.numbers\",\"wire_name\":\"numbers\",\"label\":\"numbers\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.dates\",\"wire_name\":\"dates\",\"label\":\"dates\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.times\",\"wire_name\":\"times\",\"label\":\"times\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.instants\",\"wire_name\":\"instants\",\"label\":\"instants\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.objects\",\"wire_name\":\"objects\",\"label\":\"objects\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.siblings\",\"wire_name\":\"siblings\",\"label\":\"siblings\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.favorite\",\"wire_name\":\"favorite\",\"label\":\"favorite\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"city\":\"Default\"}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.created\",\"wire_name\":\"created\",\"label\":\"created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.token\",\"wire_name\":\"token\",\"label\":\"token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.links\",\"wire_name\":\"links\",\"label\":\"links\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Link\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Link\",\"many\":true,\"mapping\":null,\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Link\",\"label\":\"Link\",\"fields\":[{\"id\":\"Link.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Matrix\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Matrix\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"links\"},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"owner.text\",\"owner.favorite.city\"],\"operations\":[]},{\"id\":\"Compound\",\"label\":\"Compound\",\"fields\":[{\"id\":\"Compound.region\",\"wire_name\":\"region\",\"label\":\"region\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Compound.code\",\"wire_name\":\"code\",\"label\":\"code\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.enabled\",\"wire_name\":\"enabled\",\"label\":\"enabled\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.clock\",\"wire_name\":\"clock\",\"label\":\"clock\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.instant\",\"wire_name\":\"instant\",\"label\":\"instant\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.ratio\",\"wire_name\":\"ratio\",\"label\":\"ratio\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null}],\"key\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"query_fields\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Query\",\"label\":\"Query\",\"fields\":[{\"id\":\"Query.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Query.self\",\"wire_name\":\"self\",\"label\":\"self\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"safe\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Query.payload\",\"wire_name\":\"payload\",\"label\":\"payload\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"payload.row\",\"payload.state\"],\"operations\":[]}],\"fieldsets\":[{\"id\":\"MatrixAddress\",\"label\":\"MatrixAddress\",\"fields\":[{\"id\":\"MatrixAddress.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"MatrixAddress.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"PH\"},\"validation\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]},{\"id\":\"Payload\",\"label\":\"Payload\",\"fields\":[{\"id\":\"Payload.row\",\"wire_name\":\"row\",\"label\":\"row\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Link", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCompound {
    /// Approved public field.
    #[serde(rename = "region")]
    pub region: String,
    /// Approved public field.
    #[serde(rename = "code")]
    pub code: i64,
    /// Approved public field.
    #[serde(rename = "enabled")]
    pub enabled: bool,
    /// Approved public field.
    #[serde(rename = "day")]
    pub day: stackpress_framework_schema::DateValue,
    /// Approved public field.
    #[serde(rename = "clock")]
    pub clock: stackpress_framework_schema::TimeValue,
    /// Approved public field.
    #[serde(rename = "instant")]
    pub instant: stackpress_framework_schema::DatetimeValue,
    /// Approved public field.
    #[serde(rename = "ratio")]
    pub ratio: stackpress_framework_schema::FiniteNumber,
    /// Approved public field.
    #[serde(rename = "choice")]
    pub choice: super::count::Count,
    /// Approved public field.
    #[serde(rename = "truth")]
    pub truth: super::truth::Truth,
    /// Approved public field.
    #[serde(rename = "value")]
    pub value: String,
    /// Approved public field.
    #[serde(rename = "note")]
    pub note: Option<String>,
}
impl super::compound::Compound {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicCompound> {
        self.validate()?;
        let public = PublicCompound {
            region: self.region.clone(),
            code: self.code,
            enabled: self.enabled,
            day: self.day.clone(),
            clock: self.clock.clone(),
            instant: self.instant.clone(),
            ratio: self.ratio,
            choice: self.choice.clone(),
            truth: self.truth.clone(),
            value: self.value.clone(),
            note: self.note.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateCompound {
    /// Writable public input.
    #[serde(
        rename = "region",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub region: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "code",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub code: stackpress_framework_schema::Input<i64>,
    /// Writable public input.
    #[serde(
        rename = "enabled",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub enabled: stackpress_framework_schema::Input<bool>,
    /// Writable public input.
    #[serde(
        rename = "day",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub day: stackpress_framework_schema::Input<stackpress_framework_schema::DateValue>,
    /// Writable public input.
    #[serde(
        rename = "clock",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub clock: stackpress_framework_schema::Input<stackpress_framework_schema::TimeValue>,
    /// Writable public input.
    #[serde(
        rename = "instant",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub instant: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
    /// Writable public input.
    #[serde(
        rename = "ratio",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub ratio: stackpress_framework_schema::Input<stackpress_framework_schema::FiniteNumber>,
    /// Writable public input.
    #[serde(
        rename = "choice",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub choice: stackpress_framework_schema::Input<super::count::Count>,
    /// Writable public input.
    #[serde(
        rename = "truth",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub truth: stackpress_framework_schema::Input<super::truth::Truth>,
    /// Writable public input.
    #[serde(
        rename = "value",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub value: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "note",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub note: stackpress_framework_schema::Input<String>,
}
impl PublicCreateCompound {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"region\",\"rust_name\":\"region\",\"column\":\"region\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"code\",\"rust_name\":\"code\",\"column\":\"code\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"enabled\",\"rust_name\":\"enabled\",\"column\":\"enabled\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"clock\",\"rust_name\":\"clock\",\"column\":\"clock\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instant\",\"rust_name\":\"instant\",\"column\":\"instant\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ratio\",\"rust_name\":\"ratio\",\"column\":\"ratio\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::compound::CreateCompound {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"region\",\"rust_name\":\"region\",\"column\":\"region\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"code\",\"rust_name\":\"code\",\"column\":\"code\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"enabled\",\"rust_name\":\"enabled\",\"column\":\"enabled\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"clock\",\"rust_name\":\"clock\",\"column\":\"clock\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instant\",\"rust_name\":\"instant\",\"column\":\"instant\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ratio\",\"rust_name\":\"ratio\",\"column\":\"ratio\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::compound::Compound> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchCompound {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "value",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub value: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "note",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub note: stackpress_framework_schema::Patch<String>,
}
impl PublicPatchCompound {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
}
/// Ordered model key, with exact scalar types.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompoundKey {
    /// Key member.
    #[serde(rename = "region")]
    pub region: String,
    /// Key member.
    #[serde(rename = "code")]
    pub code: i64,
    /// Key member.
    #[serde(rename = "enabled")]
    pub enabled: bool,
    /// Key member.
    #[serde(rename = "day")]
    pub day: stackpress_framework_schema::DateValue,
    /// Key member.
    #[serde(rename = "clock")]
    pub clock: stackpress_framework_schema::TimeValue,
    /// Key member.
    #[serde(rename = "instant")]
    pub instant: stackpress_framework_schema::DatetimeValue,
    /// Key member.
    #[serde(rename = "ratio")]
    pub ratio: stackpress_framework_schema::FiniteNumber,
    /// Key member.
    #[serde(rename = "choice")]
    pub choice: super::count::Count,
    /// Key member.
    #[serde(rename = "truth")]
    pub truth: super::truth::Truth,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchCompound {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "value",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub value: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "note",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub note: stackpress_framework_schema::Patch<String>,
}
impl PatchCompound {
    /// Reject empty updates and validate supplied fields without applying create defaults.
    pub fn validate(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<()> {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        if value.as_object().is_none_or(|v| v.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Invalid,
                "empty patch is invalid",
            ));
        }
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchCompound {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Compound\",\"rust_name\":\"Compound\",\"module\":\"compound\",\"fields\":[{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"note\",\"rust_name\":\"note\",\"column\":\"note\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
}
/// Approved field predicates; variants cannot name private paths.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum CompoundFilter {
    /// Approved typed predicate.
    #[serde(rename = "region")]
    Region(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "code")]
    Code(stackpress_framework_schema::Predicate<i64>),
    /// Approved typed predicate.
    #[serde(rename = "enabled")]
    Enabled(stackpress_framework_schema::Predicate<bool>),
    /// Approved typed predicate.
    #[serde(rename = "day")]
    Day(stackpress_framework_schema::Predicate<stackpress_framework_schema::DateValue>),
    /// Approved typed predicate.
    #[serde(rename = "clock")]
    Clock(stackpress_framework_schema::Predicate<stackpress_framework_schema::TimeValue>),
    /// Approved typed predicate.
    #[serde(rename = "instant")]
    Instant(stackpress_framework_schema::Predicate<stackpress_framework_schema::DatetimeValue>),
    /// Approved typed predicate.
    #[serde(rename = "ratio")]
    Ratio(stackpress_framework_schema::Predicate<stackpress_framework_schema::FiniteNumber>),
    /// Approved typed predicate.
    #[serde(rename = "choice")]
    Choice(stackpress_framework_schema::Predicate<super::count::Count>),
    /// Approved typed predicate.
    #[serde(rename = "truth")]
    Truth(stackpress_framework_schema::Predicate<super::truth::Truth>),
}
impl CompoundFilter {
    /// Check operator applicability and value bounds before future IO.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        match self {
            Self::Region(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"String\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Code(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Integer\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Enabled(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Boolean\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Day(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Date\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Clock(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Time\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Instant(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Datetime\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Ratio(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Number\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Choice(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Integer\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
            Self::Truth(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"Boolean\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
        }
    }
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum CompoundSort {
    /// Approved scalar ordering.
    #[serde(rename = "region")]
    Region,
    /// Approved scalar ordering.
    #[serde(rename = "note")]
    Note,
}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CompoundQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<CompoundFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<CompoundSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for CompoundQuery {
    fn default() -> Self {
        Self {
            q: None,
            filters: Vec::new(),
            sort: Vec::new(),
            includes: Vec::new(),
            skip: 0,
            take: 20,
            total: false,
        }
    }
}
impl CompoundQuery {
    /// Validate the complete allowlist/budget contract before any future query compilation.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        stackpress_framework_schema::query_budget(
            self.q.as_deref(),
            self.filters.len(),
            self.sort.len(),
            &self.includes,
            self.skip,
            self.take,
        )?;
        for filter in &self.filters {
            filter.validate()?;
        }
        if !self.includes.is_empty() {
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"19017e7b67932c0bb1b70af22dc5caa5b782b3a66379ca973e5f024b51aedbe8\",\"models\":[{\"id\":\"Matrix\",\"label\":\"Matrix\",\"fields\":[{\"id\":\"Matrix.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"Café\"},\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"relation\":null},{\"id\":\"Matrix.count\",\"wire_name\":\"count\",\"label\":\"count\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.real\",\"wire_name\":\"real\",\"label\":\"real\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0.25},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.float\",\"wire_name\":\"float\",\"label\":\"float\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-02-29\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"12:00:00.12\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.stamp\",\"wire_name\":\"stamp\",\"label\":\"stamp\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-03-01T00:00:00Z\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.object\",\"wire_name\":\"object\",\"label\":\"object\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"nested\":[null,true]}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.numbers\",\"wire_name\":\"numbers\",\"label\":\"numbers\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.dates\",\"wire_name\":\"dates\",\"label\":\"dates\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.times\",\"wire_name\":\"times\",\"label\":\"times\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.instants\",\"wire_name\":\"instants\",\"label\":\"instants\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.objects\",\"wire_name\":\"objects\",\"label\":\"objects\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.siblings\",\"wire_name\":\"siblings\",\"label\":\"siblings\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.favorite\",\"wire_name\":\"favorite\",\"label\":\"favorite\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"city\":\"Default\"}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.created\",\"wire_name\":\"created\",\"label\":\"created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.token\",\"wire_name\":\"token\",\"label\":\"token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.links\",\"wire_name\":\"links\",\"label\":\"links\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Link\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Link\",\"many\":true,\"mapping\":null,\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Link\",\"label\":\"Link\",\"fields\":[{\"id\":\"Link.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Matrix\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Matrix\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"links\"},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"owner.text\",\"owner.favorite.city\"],\"operations\":[]},{\"id\":\"Compound\",\"label\":\"Compound\",\"fields\":[{\"id\":\"Compound.region\",\"wire_name\":\"region\",\"label\":\"region\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Compound.code\",\"wire_name\":\"code\",\"label\":\"code\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.enabled\",\"wire_name\":\"enabled\",\"label\":\"enabled\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.clock\",\"wire_name\":\"clock\",\"label\":\"clock\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.instant\",\"wire_name\":\"instant\",\"label\":\"instant\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.ratio\",\"wire_name\":\"ratio\",\"label\":\"ratio\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null}],\"key\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"query_fields\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Query\",\"label\":\"Query\",\"fields\":[{\"id\":\"Query.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Query.self\",\"wire_name\":\"self\",\"label\":\"self\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"safe\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Query.payload\",\"wire_name\":\"payload\",\"label\":\"payload\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"payload.row\",\"payload.state\"],\"operations\":[]}],\"fieldsets\":[{\"id\":\"MatrixAddress\",\"label\":\"MatrixAddress\",\"fields\":[{\"id\":\"MatrixAddress.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"MatrixAddress.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"PH\"},\"validation\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]},{\"id\":\"Payload\",\"label\":\"Payload\",\"fields\":[{\"id\":\"Payload.row\",\"wire_name\":\"row\",\"label\":\"row\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Compound", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicQuery {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "self")]
    pub self_: String,
    /// Approved public field.
    #[serde(rename = "payload")]
    pub payload: Option<PublicPayload>,
}
impl super::query::Query {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicQuery> {
        self.validate()?;
        let public = PublicQuery {
            id: self.id.clone(),
            self_: self.self_.clone(),
            payload: self
                .payload
                .as_ref()
                .map(|value| value.project())
                .transpose()?,
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateQuery {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "self",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub self_: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "payload",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub payload: stackpress_framework_schema::Input<PublicCreatePayload>,
}
impl PublicCreateQuery {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"safe\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
    /// Prepare approved literal defaults in public input only; host-generated values remain omitted.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<serde_json::Value> {
        let value = stackpress_framework_schema::to_bounded_value(&self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.prepare(value, context)
    }
}
impl super::query::CreateQuery {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"safe\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_create(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Resolve host defaults once and return the fully validated trusted value.
    pub fn prepare_create(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<super::query::Query> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchQuery {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "self",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub self_: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "payload",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub payload: stackpress_framework_schema::Patch<PublicCreatePayload>,
}
impl PublicPatchQuery {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, false)
    }
    /// Evaluate client-checkable rules only; host rules and authorization remain required.
    pub fn validate_client(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        stackpress_framework_schema::check_browser_json(&value)?;
        Self::input_schema()?.validate(&value, context, true)
    }
}
/// Ordered model key, with exact scalar types.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchQuery {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "self",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub self_: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "payload",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub payload: stackpress_framework_schema::Patch<super::payload::Payload>,
}
impl PatchQuery {
    /// Reject empty updates and validate supplied fields without applying create defaults.
    pub fn validate(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<()> {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        if value.as_object().is_none_or(|v| v.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Invalid,
                "empty patch is invalid",
            ));
        }
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchQuery {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Query\",\"rust_name\":\"Query\",\"module\":\"query\",\"fields\":[{\"name\":\"self\",\"rust_name\":\"self_\",\"column\":\"self_\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"payload\",\"rust_name\":\"payload\",\"column\":\"payload\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
    }
    /// Validate supplied inputs without generating values or executing database work.
    pub fn validate_patch(
        &self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::ValidationErrors>
    {
        let value = stackpress_framework_schema::to_bounded_value(self)?;
        Self::input_schema()?.validate(&value, context, false)
    }
}
/// Approved field predicates; variants cannot name private paths.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "field", content = "predicate", deny_unknown_fields)]
pub enum QueryFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
}
impl QueryFilter {
    /// Check operator applicability and value bounds before future IO.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        match self {
            Self::Id(predicate) => predicate.validate(
                &serde_json::from_str::<stackpress_framework_schema::ValueType>(
                    "{\"kind\":\"String\"}",
                )
                .map_err(|cause| {
                    stackpress_framework_schema::SchemaError::new(
                        stackpress_framework_schema::SchemaErrorKind::Codec,
                        "query type decoding failed",
                    )
                    .with_cause(cause)
                })?,
                false,
                false,
            ),
        }
    }
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum QuerySort {
    /// Approved scalar ordering.
    #[serde(rename = "self")]
    SelfValue,
}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct QueryQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<QueryFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<QuerySort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for QueryQuery {
    fn default() -> Self {
        Self {
            q: None,
            filters: Vec::new(),
            sort: Vec::new(),
            includes: Vec::new(),
            skip: 0,
            take: 20,
            total: false,
        }
    }
}
impl QueryQuery {
    /// Validate the complete allowlist/budget contract before any future query compilation.
    pub fn validate(&self) -> stackpress_framework_schema::SchemaResult<()> {
        stackpress_framework_schema::query_budget(
            self.q.as_deref(),
            self.filters.len(),
            self.sort.len(),
            &self.includes,
            self.skip,
            self.take,
        )?;
        if self.q.as_ref().is_some_and(|value| !value.is_empty()) {
            return Err(stackpress_framework_schema::SchemaError::new(
                stackpress_framework_schema::SchemaErrorKind::Unsupported,
                "public text search is unavailable",
            ));
        }
        for filter in &self.filters {
            filter.validate()?;
        }
        if !self.includes.is_empty() {
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"19017e7b67932c0bb1b70af22dc5caa5b782b3a66379ca973e5f024b51aedbe8\",\"models\":[{\"id\":\"Matrix\",\"label\":\"Matrix\",\"fields\":[{\"id\":\"Matrix.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"Café\"},\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"relation\":null},{\"id\":\"Matrix.count\",\"wire_name\":\"count\",\"label\":\"count\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.real\",\"wire_name\":\"real\",\"label\":\"real\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0.25},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.float\",\"wire_name\":\"float\",\"label\":\"float\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-02-29\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"12:00:00.12\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.stamp\",\"wire_name\":\"stamp\",\"label\":\"stamp\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-03-01T00:00:00Z\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.object\",\"wire_name\":\"object\",\"label\":\"object\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"nested\":[null,true]}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.numbers\",\"wire_name\":\"numbers\",\"label\":\"numbers\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.dates\",\"wire_name\":\"dates\",\"label\":\"dates\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.times\",\"wire_name\":\"times\",\"label\":\"times\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.instants\",\"wire_name\":\"instants\",\"label\":\"instants\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.objects\",\"wire_name\":\"objects\",\"label\":\"objects\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.siblings\",\"wire_name\":\"siblings\",\"label\":\"siblings\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.favorite\",\"wire_name\":\"favorite\",\"label\":\"favorite\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"city\":\"Default\"}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.created\",\"wire_name\":\"created\",\"label\":\"created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.token\",\"wire_name\":\"token\",\"label\":\"token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.links\",\"wire_name\":\"links\",\"label\":\"links\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Link\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Link\",\"many\":true,\"mapping\":null,\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Link\",\"label\":\"Link\",\"fields\":[{\"id\":\"Link.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Link.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Matrix\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Matrix\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"links\"},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"owner.text\",\"owner.favorite.city\"],\"operations\":[]},{\"id\":\"Compound\",\"label\":\"Compound\",\"fields\":[{\"id\":\"Compound.region\",\"wire_name\":\"region\",\"label\":\"region\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Compound.code\",\"wire_name\":\"code\",\"label\":\"code\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.enabled\",\"wire_name\":\"enabled\",\"label\":\"enabled\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.clock\",\"wire_name\":\"clock\",\"label\":\"clock\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.instant\",\"wire_name\":\"instant\",\"label\":\"instant\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.ratio\",\"wire_name\":\"ratio\",\"label\":\"ratio\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Compound.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null}],\"key\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"query_fields\":[\"region\",\"code\",\"enabled\",\"day\",\"clock\",\"instant\",\"ratio\",\"choice\",\"truth\"],\"default_selection\":[],\"operations\":[]},{\"id\":\"Query\",\"label\":\"Query\",\"fields\":[{\"id\":\"Query.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Query.self\",\"wire_name\":\"self\",\"label\":\"self\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"safe\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[],\"relation\":null},{\"id\":\"Query.payload\",\"wire_name\":\"payload\",\"label\":\"payload\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Payload\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"payload.row\",\"payload.state\"],\"operations\":[]}],\"fieldsets\":[{\"id\":\"MatrixAddress\",\"label\":\"MatrixAddress\",\"fields\":[{\"id\":\"MatrixAddress.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"MatrixAddress.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"PH\"},\"validation\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]},{\"id\":\"Payload\",\"label\":\"Payload\",\"fields\":[{\"id\":\"Payload.row\",\"wire_name\":\"row\",\"label\":\"row\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Payload.value\",\"wire_name\":\"value\",\"label\":\"value\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Query", path)?;
            }
        }
        Ok(())
    }
}
