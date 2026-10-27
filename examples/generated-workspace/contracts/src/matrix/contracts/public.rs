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
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"Café\",\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0.25,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-02-29\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"12:00:00.12\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-03-01T00:00:00Z\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"nested\":[null,true]},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"city\":\"Default\"},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"Café\",\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0.25,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-02-29\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"12:00:00.12\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"2024-03-01T00:00:00Z\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"nested\":[null,true]},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":{\"city\":\"Default\"},\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"PH\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchMatrix {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":3,\"schema_fingerprint\":\"0371db951ceae78c1e592ef5737d828ea5067ace7e9fcc31220e4e0d2503780c\",\"models\":[{\"id\":\"Matrix\",\"label\":\"Matrix\",\"fields\":[{\"id\":\"Matrix.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"Café\"},\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":false},{\"role\":\"view\",\"component\":\"stackpress.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":false}],\"relation\":null},{\"id\":\"Matrix.count\",\"wire_name\":\"count\",\"label\":\"count\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.real\",\"wire_name\":\"real\",\"label\":\"real\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0.25},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.float\",\"wire_name\":\"float\",\"label\":\"float\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.day\",\"wire_name\":\"day\",\"label\":\"day\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-02-29\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"12:00:00.12\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.stamp\",\"wire_name\":\"stamp\",\"label\":\"stamp\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"2024-03-01T00:00:00Z\"},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.object\",\"wire_name\":\"object\",\"label\":\"object\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"nested\":[null,true]}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.choice\",\"wire_name\":\"choice\",\"label\":\"choice\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"multiple\":false,\"choices\":[0,1]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.truth\",\"wire_name\":\"truth\",\"label\":\"truth\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"multiple\":false,\"choices\":[false,true]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.numbers\",\"wire_name\":\"numbers\",\"label\":\"numbers\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.dates\",\"wire_name\":\"dates\",\"label\":\"dates\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.times\",\"wire_name\":\"times\",\"label\":\"times\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.instants\",\"wire_name\":\"instants\",\"label\":\"instants\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.objects\",\"wire_name\":\"objects\",\"label\":\"objects\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.siblings\",\"wire_name\":\"siblings\",\"label\":\"siblings\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.favorite\",\"wire_name\":\"favorite\",\"label\":\"favorite\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":{\"city\":\"Default\"}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.created\",\"wire_name\":\"created\",\"label\":\"created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Matrix.token\",\"wire_name\":\"token\",\"label\":\"token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[],\"operations\":[]}],\"fieldsets\":[{\"id\":\"MatrixAddress\",\"label\":\"MatrixAddress\",\"fields\":[{\"id\":\"MatrixAddress.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"MatrixAddress.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"PH\"},\"validation\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Matrix", path)?;
            }
        }
        Ok(())
    }
}
