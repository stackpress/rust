//! Owned Matrix contract generated from finalized Idea.

/// Checked-shape owned values; no database handle or lazy relation.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matrix {
    /// Declared stored value.
    #[serde(rename = "id")]
    pub id: i64,
    /// Declared stored value.
    #[serde(rename = "text")]
    pub text: String,
    /// Declared stored value.
    #[serde(rename = "count")]
    pub count: i64,
    /// Declared stored value.
    #[serde(rename = "real")]
    pub real: stackpress_framework_schema::FiniteNumber,
    /// Declared stored value.
    #[serde(rename = "float")]
    pub float: Option<stackpress_framework_schema::FiniteNumber>,
    /// Declared stored value.
    #[serde(rename = "flag")]
    pub flag: bool,
    /// Declared stored value.
    #[serde(rename = "day")]
    pub day: stackpress_framework_schema::DateValue,
    /// Declared stored value.
    #[serde(rename = "time")]
    pub time: stackpress_framework_schema::TimeValue,
    /// Declared stored value.
    #[serde(rename = "stamp")]
    pub stamp: stackpress_framework_schema::DatetimeValue,
    /// Declared stored value.
    #[serde(rename = "object")]
    pub object: stackpress_framework_schema::JsonObject,
    /// Declared stored value.
    #[serde(rename = "choice")]
    pub choice: super::count::Count,
    /// Declared stored value.
    #[serde(rename = "truth")]
    pub truth: super::truth::Truth,
    /// Declared stored value.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
    /// Declared stored value.
    #[serde(rename = "numbers")]
    pub numbers: Vec<stackpress_framework_schema::FiniteNumber>,
    /// Declared stored value.
    #[serde(rename = "dates")]
    pub dates: Vec<stackpress_framework_schema::DateValue>,
    /// Declared stored value.
    #[serde(rename = "times")]
    pub times: Vec<stackpress_framework_schema::TimeValue>,
    /// Declared stored value.
    #[serde(rename = "instants")]
    pub instants: Vec<stackpress_framework_schema::DatetimeValue>,
    /// Declared stored value.
    #[serde(rename = "objects")]
    pub objects: Vec<stackpress_framework_schema::JsonObject>,
    /// Declared stored value.
    #[serde(rename = "address")]
    pub address: Option<super::matrix_address::MatrixAddress>,
    /// Declared stored value.
    #[serde(rename = "siblings")]
    pub siblings: Vec<super::matrix_address::MatrixAddress>,
    /// Declared stored value.
    #[serde(rename = "favorite")]
    pub favorite: super::matrix_address::MatrixAddress,
    /// Declared stored value.
    #[serde(rename = "created")]
    pub created: stackpress_framework_schema::DatetimeValue,
    /// Declared stored value.
    #[serde(rename = "token")]
    pub token: String,
    /// Declared stored value.
    #[serde(rename = "secret")]
    pub secret: Option<String>,
    /// Declared stored value.
    #[serde(rename = "updated")]
    pub updated: stackpress_framework_schema::DatetimeValue,
}
impl Matrix {
    /// Decode the bounded runtime-only record policy; private metadata is absent.
    fn record_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::RecordSchema> {
        serde_json::from_str("{\"model\":{\"name\":\"Matrix\",\"rust_name\":\"Matrix\",\"module\":\"matrix\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":true,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"cge\",\"args\":[2],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"count\",\"rust_name\":\"count\",\"column\":\"count\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"ge\",\"args\":[0],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"},{\"code\":\"le\",\"args\":[10],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"real\",\"rust_name\":\"real\",\"column\":\"real\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"float\",\"rust_name\":\"float\",\"column\":\"float\",\"value_type\":{\"kind\":\"Number\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"day\",\"rust_name\":\"day\",\"column\":\"day\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"time\",\"rust_name\":\"time\",\"column\":\"time\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"stamp\",\"rust_name\":\"stamp\",\"column\":\"stamp\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"object\",\"rust_name\":\"object\",\"column\":\"object\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"choice\",\"rust_name\":\"choice\",\"column\":\"choice\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Count\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"truth\",\"rust_name\":\"truth\",\"column\":\"truth\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"Truth\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"numbers\",\"rust_name\":\"numbers\",\"column\":\"numbers\",\"value_type\":{\"kind\":\"Number\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"dates\",\"rust_name\":\"dates\",\"column\":\"dates\",\"value_type\":{\"kind\":\"Date\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"times\",\"rust_name\":\"times\",\"column\":\"times\",\"value_type\":{\"kind\":\"Time\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"instants\",\"rust_name\":\"instants\",\"column\":\"instants\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"objects\",\"rust_name\":\"objects\",\"column\":\"objects\",\"value_type\":{\"kind\":\"Object\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"siblings\",\"rust_name\":\"siblings\",\"column\":\"siblings\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"favorite\",\"rust_name\":\"favorite\",\"column\":\"favorite\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"MatrixAddress\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"created\",\"rust_name\":\"created\",\"column\":\"created\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"token\",\"rust_name\":\"token\",\"column\":\"token\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"secret\",\"rust_name\":\"secret\",\"column\":\"secret\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"updated\",\"rust_name\":\"updated\",\"column\":\"updated\",\"value_type\":{\"kind\":\"Datetime\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":true,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"links\",\"rust_name\":\"links\",\"column\":\"links\",\"value_type\":{\"kind\":\"Model\",\"name\":\"Link\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"MatrixAddress\",\"rust_name\":\"MatrixAddress\",\"module\":\"matrix_address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zip\",\"rust_name\":\"zip\",\"column\":\"zip\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[{\"code\":\"app.approved\",\"args\":[],\"client_checkable\":false,\"message\":\"field validation failed\",\"definition_revision\":\"app-rule-1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"nonce\",\"rust_name\":\"nonce\",\"column\":\"nonce\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":true,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},{\"name\":\"Payload\",\"rust_name\":\"Payload\",\"module\":\"payload\",\"fields\":[{\"name\":\"row\",\"rust_name\":\"row\",\"column\":\"row\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"value\",\"rust_name\":\"value\",\"column\":\"value\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"Count\",\"rust_name\":\"Count\",\"module\":\"count\",\"variants\":[[\"Zero\",0],[\"One\",1]]},{\"name\":\"Truth\",\"rust_name\":\"Truth\",\"module\":\"truth\",\"variants\":[[\"No\",false],[\"Yes\",true]]}]}").map_err(|cause| stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec, "record policy decoding failed").with_cause(cause))
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
pub struct CreateMatrix {
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<i64>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub text: stackpress_framework_schema::Input<String>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "count",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub count: stackpress_framework_schema::Input<i64>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "real",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub real: stackpress_framework_schema::Input<stackpress_framework_schema::FiniteNumber>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "float",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub float: stackpress_framework_schema::Input<stackpress_framework_schema::FiniteNumber>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub flag: stackpress_framework_schema::Input<bool>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "day",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub day: stackpress_framework_schema::Input<stackpress_framework_schema::DateValue>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "time",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub time: stackpress_framework_schema::Input<stackpress_framework_schema::TimeValue>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "stamp",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub stamp: stackpress_framework_schema::Input<stackpress_framework_schema::DatetimeValue>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "object",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub object: stackpress_framework_schema::Input<stackpress_framework_schema::JsonObject>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "choice",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub choice: stackpress_framework_schema::Input<super::count::Count>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "truth",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub truth: stackpress_framework_schema::Input<super::truth::Truth>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub tags: stackpress_framework_schema::Input<Vec<String>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "numbers",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub numbers: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::FiniteNumber>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "dates",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub dates: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::DateValue>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "times",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub times: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::TimeValue>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "instants",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub instants:
        stackpress_framework_schema::Input<Vec<stackpress_framework_schema::DatetimeValue>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "objects",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub objects: stackpress_framework_schema::Input<Vec<stackpress_framework_schema::JsonObject>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub address: stackpress_framework_schema::Input<super::matrix_address::CreateMatrixAddress>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "siblings",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub siblings:
        stackpress_framework_schema::Input<Vec<super::matrix_address::CreateMatrixAddress>>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "favorite",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub favorite: stackpress_framework_schema::Input<super::matrix_address::CreateMatrixAddress>,
    /// Explicit create input; omission consults the declared host/literal source.
    #[serde(
        rename = "secret",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub secret: stackpress_framework_schema::Input<String>,
}
impl CreateMatrix {
    /// Prepare using no ambient clock or provider.
    pub fn prepare(self) -> stackpress_framework_schema::SchemaResult<Matrix> {
        self.prepare_with(&stackpress_framework_schema::ValueContext::default())
    }
    /// Fill defaults exactly once and validate all nested business rules once before effects.
    pub fn prepare_with(
        self,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> stackpress_framework_schema::SchemaResult<Matrix> {
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
    ) -> stackpress_framework_schema::SchemaResult<Matrix> {
        let _ = context;
        Ok(Matrix {
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
            text: match self.text {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("\"Café\"").map_err(|cause| {
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
                            budget.charge("text", &value)?;
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
                    budget.charge("text", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            count: match self.count {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("0").map_err(|cause| {
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
                        let value: i64 = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        {
                            budget.charge("count", &value)?;
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
                    budget.charge("count", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            real: match self.real {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("0.25").map_err(|cause| {
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
                        let value: stackpress_framework_schema::FiniteNumber =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("real", &value)?;
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
                    budget.charge("real", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            float: match self.float {
                stackpress_framework_schema::Input::Missing => None,
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("float", &value)?;
                    value
                }),
            },
            // Explicit null never falls through to a literal or host default.
            flag: match self.flag {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("false").map_err(|cause| {
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
                            budget.charge("flag", &value)?;
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
                    budget.charge("flag", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            day: match self.day {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("\"2024-02-29\"").map_err(
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
                        let value: stackpress_framework_schema::DateValue =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("day", &value)?;
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
                    budget.charge("day", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            time: match self.time {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("\"12:00:00.12\"")
                        .map_err(|cause| {
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
                        let value: stackpress_framework_schema::TimeValue =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("time", &value)?;
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
                    budget.charge("time", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            stamp: match self.stamp {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("\"2024-03-01T00:00:00Z\"")
                        .map_err(|cause| {
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
                        let value: stackpress_framework_schema::DatetimeValue =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("stamp", &value)?;
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
                    budget.charge("stamp", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            object: match self.object {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("{\"nested\":[null,true]}")
                        .map_err(|cause| {
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
                        let value: stackpress_framework_schema::JsonObject =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("object", &value)?;
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
                    budget.charge("object", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            choice: match self.choice {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("0").map_err(|cause| {
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
                        let value: super::count::Count =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("choice", &value)?;
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
                    budget.charge("choice", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            truth: match self.truth {
                stackpress_framework_schema::Input::Missing => {
                    let raw =
                        serde_json::from_str::<serde_json::Value>("false").map_err(|cause| {
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
                        let value: super::truth::Truth =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("truth", &value)?;
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
                    budget.charge("truth", &value)?;
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
            numbers: match self.numbers {
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
                        let value: Vec<stackpress_framework_schema::FiniteNumber> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("numbers", &value)?;
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
                    budget.charge("numbers", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            dates: match self.dates {
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
                        let value: Vec<stackpress_framework_schema::DateValue> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("dates", &value)?;
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
                    budget.charge("dates", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            times: match self.times {
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
                        let value: Vec<stackpress_framework_schema::TimeValue> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("times", &value)?;
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
                    budget.charge("times", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            instants: match self.instants {
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
                        let value: Vec<stackpress_framework_schema::DatetimeValue> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("instants", &value)?;
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
                    budget.charge("instants", &value)?;
                    value
                }
            },
            // Explicit null never falls through to a literal or host default.
            objects: match self.objects {
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
                        let value: Vec<stackpress_framework_schema::JsonObject> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.charge("objects", &value)?;
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
                    budget.charge("objects", &value)?;
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
            // Explicit null never falls through to a literal or host default.
            siblings: match self.siblings {
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
                        let value: Vec<super::matrix_address::CreateMatrixAddress> =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.container("siblings")?;
                            if value.len() > 1024 {
                                return Err(stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Limit,
                                    "list element budget exceeded",
                                ));
                            }
                            value
                                .into_iter()
                                .map(|input| input.prepare_bounded(context, budget))
                                .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?
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
                    budget.container("siblings")?;
                    if value.len() > 1024 {
                        return Err(stackpress_framework_schema::SchemaError::new(
                            stackpress_framework_schema::SchemaErrorKind::Limit,
                            "list element budget exceeded",
                        ));
                    }
                    value
                        .into_iter()
                        .map(|input| input.prepare_bounded(context, budget))
                        .collect::<stackpress_framework_schema::SchemaResult<Vec<_>>>()?
                }
            },
            // Explicit null never falls through to a literal or host default.
            favorite: match self.favorite {
                stackpress_framework_schema::Input::Missing => {
                    let raw = serde_json::from_str::<serde_json::Value>("{\"city\":\"Default\"}")
                        .map_err(|cause| {
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
                        let value: super::matrix_address::CreateMatrixAddress =
                            serde_json::from_value(raw).map_err(|cause| {
                                stackpress_framework_schema::SchemaError::new(
                                    stackpress_framework_schema::SchemaErrorKind::Codec,
                                    "default value decoding failed",
                                )
                                .with_cause(cause)
                            })?;
                        {
                            budget.container("favorite")?;
                            value.prepare_bounded(context, budget)?
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
                    budget.container("favorite")?;
                    value.prepare_bounded(context, budget)?
                }
            },
            // Generated values cannot be supplied by the caller.
            created: {
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
            },
            // Generated values cannot be supplied by the caller.
            token: {
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
                        budget.charge("token", &value)?;
                        value
                    }
                }
            },
            // Explicit null never falls through to a literal or host default.
            secret: match self.secret {
                stackpress_framework_schema::Input::Missing => {
                    let raw = context
                        .providers
                        .ok_or_else(|| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Unsupported,
                                "explicit host defaults are required",
                            )
                        })?
                        .generate("default:Matrix.secret", &[])?;
                    stackpress_framework_schema::check_json(&raw)?;
                    if raw.is_null() {
                        None
                    } else {
                        let value: String = serde_json::from_value(raw).map_err(|cause| {
                            stackpress_framework_schema::SchemaError::new(
                                stackpress_framework_schema::SchemaErrorKind::Codec,
                                "default value decoding failed",
                            )
                            .with_cause(cause)
                        })?;
                        Some({
                            budget.charge("secret", &value)?;
                            value
                        })
                    }
                }
                stackpress_framework_schema::Input::Null => None,
                stackpress_framework_schema::Input::Value(value) => Some({
                    budget.charge("secret", &value)?;
                    value
                }),
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
