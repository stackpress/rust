//! Explicit application projections. Trusted stored rows must use project() before public output.
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicAddress {
    /// Approved public field.
    #[serde(rename = "city")]
    pub city: String,
    /// Approved public field.
    #[serde(rename = "country")]
    pub country: String,
}
impl super::address::Address {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicAddress> {
        self.validate()?;
        let public = PublicAddress {
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
pub struct PublicCreateAddress {
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
impl PublicCreateAddress {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::address::CreateAddress {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::address::Address> {
        self.prepare_with(context)
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicProfile {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "active")]
    pub active: bool,
    /// Approved public field.
    #[serde(rename = "address")]
    pub address: Option<PublicAddress>,
    /// Approved public field.
    #[serde(rename = "name")]
    pub name: String,
}
impl super::profile::Profile {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicProfile> {
        self.validate()?;
        let public = PublicProfile {
            id: self.id.clone(),
            active: self.active,
            address: self
                .address
                .as_ref()
                .map(|value| value.project())
                .transpose()?,
            name: self.name.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateProfile {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub active: stackpress_framework_schema::Input<bool>,
    /// Writable public input.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub address: stackpress_framework_schema::Input<PublicCreateAddress>,
    /// Writable public input.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub name: stackpress_framework_schema::Input<String>,
}
impl PublicCreateProfile {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::profile::CreateProfile {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":true,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::profile::Profile> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchProfile {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub active: stackpress_framework_schema::Patch<bool>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub address: stackpress_framework_schema::Patch<PublicCreateAddress>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub name: stackpress_framework_schema::Patch<String>,
}
impl PublicPatchProfile {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub struct ProfileKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchProfile {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "active",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub active: stackpress_framework_schema::Patch<bool>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "address",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub address: stackpress_framework_schema::Patch<super::address::Address>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "name",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub name: stackpress_framework_schema::Patch<String>,
}
impl PatchProfile {
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchProfile {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Profile\",\"rust_name\":\"Profile\",\"module\":\"profile\",\"fields\":[{\"name\":\"active\",\"rust_name\":\"active\",\"column\":\"active\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":true,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"address\",\"rust_name\":\"address\",\"column\":\"address\",\"value_type\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"name\",\"rust_name\":\"name\",\"column\":\"name\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub enum ProfileFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "name")]
    Name(stackpress_framework_schema::Predicate<String>),
}
impl ProfileFilter {
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
            Self::Name(predicate) => predicate.validate(
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
pub enum ProfileSort {
    /// Approved scalar ordering.
    #[serde(rename = "name")]
    Name,
}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProfileQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<ProfileFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<ProfileSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for ProfileQuery {
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
impl ProfileQuery {
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"ed7339d88345035c2d45e410c7349278ec7e85bc9880a88c691f3f3ddfccfcf3\",\"models\":[{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.projects\",\"wire_name\":\"projects\",\"label\":\"projects\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{\"placeholder\":\"Idea hint\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"name\"],\"default_selection\":[\"id\",\"name\"],\"operations\":[]},{\"id\":\"Project\",\"label\":\"Project\",\"fields\":[{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"state\"],\"default_selection\":[\"id\",\"title\",\"ownerId\"],\"operations\":[]},{\"id\":\"Task\",\"label\":\"Task\",\"fields\":[{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"done\"],\"default_selection\":[\"id\",\"title\",\"projectId\",\"done\"],\"operations\":[]},{\"id\":\"RoleProof\",\"label\":\"RoleProof\",\"fields\":[{\"id\":\"RoleProof.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"RoleProof.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":\"fixture default\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.zero\",\"wire_name\":\"zero\",\"label\":\"zero\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{\"max\":10,\"min\":0,\"step\":1},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.rows\",\"wire_name\":\"rows\",\"label\":\"Rows </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.stringlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.textlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.metadata\",\"wire_name\":\"metadata\",\"label\":\"metadata\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.metadata\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.score\",\"wire_name\":\"score\",\"label\":\"score\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.rating\",\"props\":{\"max\":5},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.slug\",\"wire_name\":\"slug\",\"label\":\"slug\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slug\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.sensitive\",\"wire_name\":\"sensitive\",\"label\":\"sensitive\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":false,\"writable\":true,\"sensitive\":true,\"nullable\":true,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"text\"],\"default_selection\":[],\"operations\":[]}],\"fieldsets\":[{\"id\":\"Address\",\"label\":\"Address\",\"fields\":[{\"id\":\"Address.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Address.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Profile", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicProject {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "ownerId")]
    pub owner_id: String,
    /// Approved public field.
    #[serde(rename = "title")]
    pub title: String,
    /// Approved public field.
    #[serde(rename = "state")]
    pub state: super::project_state::ProjectState,
    /// Approved public field.
    #[serde(rename = "tags")]
    pub tags: Vec<String>,
}
impl super::project::Project {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicProject> {
        self.validate()?;
        let public = PublicProject {
            id: self.id.clone(),
            owner_id: self.owner_id.clone(),
            title: self.title.clone(),
            state: self.state.clone(),
            tags: self.tags.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateProject {
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
    pub owner_id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub title: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "state",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub state: stackpress_framework_schema::Input<super::project_state::ProjectState>,
    /// Writable public input.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub tags: stackpress_framework_schema::Input<Vec<String>>,
}
impl PublicCreateProject {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"open\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::project::CreateProject {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":\"open\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":[],\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::project::Project> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchProject {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub owner_id: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "state",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub state: stackpress_framework_schema::Patch<super::project_state::ProjectState>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub tags: stackpress_framework_schema::Patch<Vec<String>>,
}
impl PublicPatchProject {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub struct ProjectKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchProject {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "ownerId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub owner_id: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "state",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub state: stackpress_framework_schema::Patch<super::project_state::ProjectState>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "tags",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub tags: stackpress_framework_schema::Patch<Vec<String>>,
}
impl PatchProject {
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchProject {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Project\",\"rust_name\":\"Project\",\"module\":\"project\",\"fields\":[{\"name\":\"ownerId\",\"rust_name\":\"owner_id\",\"column\":\"owner_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":true,\"sortable\":true,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"state\",\"rust_name\":\"state\",\"column\":\"state\",\"value_type\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"tags\",\"rust_name\":\"tags\",\"column\":\"tags\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub enum ProjectFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "title")]
    Title(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "state")]
    State(stackpress_framework_schema::Predicate<super::project_state::ProjectState>),
}
impl ProjectFilter {
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
            Self::Title(predicate) => predicate.validate(
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
            Self::State(predicate) => predicate.validate(
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
pub enum ProjectSort {
    /// Approved scalar ordering.
    #[serde(rename = "title")]
    Title,
}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ProjectQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<ProjectFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<ProjectSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for ProjectQuery {
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
impl ProjectQuery {
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"ed7339d88345035c2d45e410c7349278ec7e85bc9880a88c691f3f3ddfccfcf3\",\"models\":[{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.projects\",\"wire_name\":\"projects\",\"label\":\"projects\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{\"placeholder\":\"Idea hint\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"name\"],\"default_selection\":[\"id\",\"name\"],\"operations\":[]},{\"id\":\"Project\",\"label\":\"Project\",\"fields\":[{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"state\"],\"default_selection\":[\"id\",\"title\",\"ownerId\"],\"operations\":[]},{\"id\":\"Task\",\"label\":\"Task\",\"fields\":[{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"done\"],\"default_selection\":[\"id\",\"title\",\"projectId\",\"done\"],\"operations\":[]},{\"id\":\"RoleProof\",\"label\":\"RoleProof\",\"fields\":[{\"id\":\"RoleProof.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"RoleProof.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":\"fixture default\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.zero\",\"wire_name\":\"zero\",\"label\":\"zero\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{\"max\":10,\"min\":0,\"step\":1},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.rows\",\"wire_name\":\"rows\",\"label\":\"Rows </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.stringlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.textlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.metadata\",\"wire_name\":\"metadata\",\"label\":\"metadata\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.metadata\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.score\",\"wire_name\":\"score\",\"label\":\"score\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.rating\",\"props\":{\"max\":5},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.slug\",\"wire_name\":\"slug\",\"label\":\"slug\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slug\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.sensitive\",\"wire_name\":\"sensitive\",\"label\":\"sensitive\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":false,\"writable\":true,\"sensitive\":true,\"nullable\":true,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"text\"],\"default_selection\":[],\"operations\":[]}],\"fieldsets\":[{\"id\":\"Address\",\"label\":\"Address\",\"fields\":[{\"id\":\"Address.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Address.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Project", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicTask {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "projectId")]
    pub project_id: String,
    /// Approved public field.
    #[serde(rename = "title")]
    pub title: String,
    /// Approved public field.
    #[serde(rename = "done")]
    pub done: bool,
}
impl super::task::Task {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicTask> {
        self.validate()?;
        let public = PublicTask {
            id: self.id.clone(),
            project_id: self.project_id.clone(),
            title: self.title.clone(),
            done: self.done,
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateTask {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "projectId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub project_id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub title: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "done",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub done: stackpress_framework_schema::Input<bool>,
}
impl PublicCreateTask {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::task::CreateTask {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::task::Task> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchTask {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "projectId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub project_id: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "done",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub done: stackpress_framework_schema::Patch<bool>,
}
impl PublicPatchTask {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub struct TaskKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchTask {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "projectId",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub project_id: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "done",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub done: stackpress_framework_schema::Patch<bool>,
}
impl PatchTask {
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchTask {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Task\",\"rust_name\":\"Task\",\"module\":\"task\",\"fields\":[{\"name\":\"projectId\",\"rust_name\":\"project_id\",\"column\":\"project_id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"done\",\"rust_name\":\"done\",\"column\":\"done\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub enum TaskFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "title")]
    Title(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "done")]
    Done(stackpress_framework_schema::Predicate<bool>),
}
impl TaskFilter {
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
            Self::Title(predicate) => predicate.validate(
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
            Self::Done(predicate) => predicate.validate(
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
pub enum TaskSort {}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TaskQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<TaskFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<TaskSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for TaskQuery {
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
impl TaskQuery {
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"ed7339d88345035c2d45e410c7349278ec7e85bc9880a88c691f3f3ddfccfcf3\",\"models\":[{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.projects\",\"wire_name\":\"projects\",\"label\":\"projects\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{\"placeholder\":\"Idea hint\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"name\"],\"default_selection\":[\"id\",\"name\"],\"operations\":[]},{\"id\":\"Project\",\"label\":\"Project\",\"fields\":[{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"state\"],\"default_selection\":[\"id\",\"title\",\"ownerId\"],\"operations\":[]},{\"id\":\"Task\",\"label\":\"Task\",\"fields\":[{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"done\"],\"default_selection\":[\"id\",\"title\",\"projectId\",\"done\"],\"operations\":[]},{\"id\":\"RoleProof\",\"label\":\"RoleProof\",\"fields\":[{\"id\":\"RoleProof.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"RoleProof.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":\"fixture default\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.zero\",\"wire_name\":\"zero\",\"label\":\"zero\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{\"max\":10,\"min\":0,\"step\":1},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.rows\",\"wire_name\":\"rows\",\"label\":\"Rows </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.stringlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.textlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.metadata\",\"wire_name\":\"metadata\",\"label\":\"metadata\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.metadata\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.score\",\"wire_name\":\"score\",\"label\":\"score\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.rating\",\"props\":{\"max\":5},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.slug\",\"wire_name\":\"slug\",\"label\":\"slug\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slug\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.sensitive\",\"wire_name\":\"sensitive\",\"label\":\"sensitive\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":false,\"writable\":true,\"sensitive\":true,\"nullable\":true,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"text\"],\"default_selection\":[],\"operations\":[]}],\"fieldsets\":[{\"id\":\"Address\",\"label\":\"Address\",\"fields\":[{\"id\":\"Address.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Address.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Task", path)?;
            }
        }
        Ok(())
    }
}
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicRoleProof {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "text")]
    pub text: Option<String>,
    /// Approved public field.
    #[serde(rename = "flag")]
    pub flag: bool,
    /// Approved public field.
    #[serde(rename = "zero")]
    pub zero: i64,
    /// Approved public field.
    #[serde(rename = "rows")]
    pub rows: Option<Vec<String>>,
    /// Approved public field.
    #[serde(rename = "metadata")]
    pub metadata: Option<stackpress_framework_schema::JsonObject>,
    /// Approved public field.
    #[serde(rename = "score")]
    pub score: Option<i64>,
    /// Approved public field.
    #[serde(rename = "slug")]
    pub slug: Option<String>,
}
impl super::role_proof::RoleProof {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicRoleProof> {
        self.validate()?;
        let public = PublicRoleProof {
            id: self.id.clone(),
            text: self.text.clone(),
            flag: self.flag,
            zero: self.zero,
            rows: self.rows.clone(),
            metadata: self.metadata.clone(),
            score: self.score,
            slug: self.slug.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateRoleProof {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub text: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub flag: stackpress_framework_schema::Input<bool>,
    /// Writable public input.
    #[serde(
        rename = "zero",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub zero: stackpress_framework_schema::Input<i64>,
    /// Writable public input.
    #[serde(
        rename = "rows",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub rows: stackpress_framework_schema::Input<Vec<String>>,
    /// Writable public input.
    #[serde(
        rename = "metadata",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub metadata: stackpress_framework_schema::Input<stackpress_framework_schema::JsonObject>,
    /// Writable public input.
    #[serde(
        rename = "score",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub score: stackpress_framework_schema::Input<i64>,
    /// Writable public input.
    #[serde(
        rename = "slug",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub slug: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "sensitive",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub sensitive: stackpress_framework_schema::Input<String>,
}
impl PublicCreateRoleProof {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"RoleProof\",\"rust_name\":\"RoleProof\",\"module\":\"role_proof\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":\"fixture default\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zero\",\"rust_name\":\"zero\",\"column\":\"zero\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"rows\",\"rust_name\":\"rows\",\"column\":\"rows\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"metadata\",\"rust_name\":\"metadata\",\"column\":\"metadata\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"score\",\"rust_name\":\"score\",\"column\":\"score\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"slug\",\"rust_name\":\"slug\",\"column\":\"slug\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"sensitive\",\"rust_name\":\"sensitive\",\"column\":\"sensitive\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::role_proof::CreateRoleProof {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"RoleProof\",\"rust_name\":\"RoleProof\",\"module\":\"role_proof\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":\"fixture default\",\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":false,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zero\",\"rust_name\":\"zero\",\"column\":\"zero\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":0,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"rows\",\"rust_name\":\"rows\",\"column\":\"rows\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"metadata\",\"rust_name\":\"metadata\",\"column\":\"metadata\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"score\",\"rust_name\":\"score\",\"column\":\"score\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"slug\",\"rust_name\":\"slug\",\"column\":\"slug\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"sensitive\",\"rust_name\":\"sensitive\",\"column\":\"sensitive\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"hostOnly\",\"rust_name\":\"host_only\",\"column\":\"host_only\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::role_proof::RoleProof> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchRoleProof {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub text: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub flag: stackpress_framework_schema::Patch<bool>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "zero",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub zero: stackpress_framework_schema::Patch<i64>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "rows",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub rows: stackpress_framework_schema::Patch<Vec<String>>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "metadata",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub metadata: stackpress_framework_schema::Patch<stackpress_framework_schema::JsonObject>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "score",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub score: stackpress_framework_schema::Patch<i64>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "slug",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub slug: stackpress_framework_schema::Patch<String>,
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "sensitive",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub sensitive: stackpress_framework_schema::Patch<String>,
}
impl PublicPatchRoleProof {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"RoleProof\",\"rust_name\":\"RoleProof\",\"module\":\"role_proof\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zero\",\"rust_name\":\"zero\",\"column\":\"zero\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"rows\",\"rust_name\":\"rows\",\"column\":\"rows\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"metadata\",\"rust_name\":\"metadata\",\"column\":\"metadata\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"score\",\"rust_name\":\"score\",\"column\":\"score\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"slug\",\"rust_name\":\"slug\",\"column\":\"slug\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"sensitive\",\"rust_name\":\"sensitive\",\"column\":\"sensitive\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub struct RoleProofKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchRoleProof {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "text",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub text: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "flag",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub flag: stackpress_framework_schema::Patch<bool>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "zero",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub zero: stackpress_framework_schema::Patch<i64>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "rows",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub rows: stackpress_framework_schema::Patch<Vec<String>>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "metadata",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub metadata: stackpress_framework_schema::Patch<stackpress_framework_schema::JsonObject>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "score",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub score: stackpress_framework_schema::Patch<i64>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "slug",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub slug: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "sensitive",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub sensitive: stackpress_framework_schema::Patch<String>,
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "hostOnly",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub host_only: stackpress_framework_schema::Patch<String>,
}
impl PatchRoleProof {
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"RoleProof\",\"rust_name\":\"RoleProof\",\"module\":\"role_proof\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zero\",\"rust_name\":\"zero\",\"column\":\"zero\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"rows\",\"rust_name\":\"rows\",\"column\":\"rows\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"metadata\",\"rust_name\":\"metadata\",\"column\":\"metadata\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"score\",\"rust_name\":\"score\",\"column\":\"score\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"slug\",\"rust_name\":\"slug\",\"column\":\"slug\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"sensitive\",\"rust_name\":\"sensitive\",\"column\":\"sensitive\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"hostOnly\",\"rust_name\":\"host_only\",\"column\":\"host_only\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchRoleProof {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"RoleProof\",\"rust_name\":\"RoleProof\",\"module\":\"role_proof\",\"fields\":[{\"name\":\"text\",\"rust_name\":\"text\",\"column\":\"text\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"flag\",\"rust_name\":\"flag\",\"column\":\"flag\",\"value_type\":{\"kind\":\"Boolean\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"zero\",\"rust_name\":\"zero\",\"column\":\"zero\",\"value_type\":{\"kind\":\"Integer\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"rows\",\"rust_name\":\"rows\",\"column\":\"rows\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":true,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"metadata\",\"rust_name\":\"metadata\",\"column\":\"metadata\",\"value_type\":{\"kind\":\"Object\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"score\",\"rust_name\":\"score\",\"column\":\"score\",\"value_type\":{\"kind\":\"Integer\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"slug\",\"rust_name\":\"slug\",\"column\":\"slug\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"sensitive\",\"rust_name\":\"sensitive\",\"column\":\"sensitive\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"hostOnly\",\"rust_name\":\"host_only\",\"column\":\"host_only\",\"value_type\":{\"kind\":\"String\"},\"required\":false,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":false,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[{\"name\":\"Address\",\"rust_name\":\"Address\",\"module\":\"address\",\"fields\":[{\"name\":\"city\",\"rust_name\":\"city\",\"column\":\"city\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":true,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"country\",\"rust_name\":\"country\",\"column\":\"country\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]}],\"enums\":[{\"name\":\"ProjectState\",\"rust_name\":\"ProjectState\",\"module\":\"project_state\",\"variants\":[[\"Open\",\"open\"],[\"Closed\",\"closed\"]]}]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub enum RoleProofFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
    /// Approved typed predicate.
    #[serde(rename = "text")]
    Text(stackpress_framework_schema::Predicate<String>),
}
impl RoleProofFilter {
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
            Self::Text(predicate) => predicate.validate(
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
                true,
            ),
        }
    }
}
/// Approved ordering field, independent of presentation roles.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum RoleProofSort {}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct RoleProofQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<RoleProofFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<RoleProofSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for RoleProofQuery {
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
impl RoleProofQuery {
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":2,\"schema_fingerprint\":\"ed7339d88345035c2d45e410c7349278ec7e85bc9880a88c691f3f3ddfccfcf3\",\"models\":[{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.projects\",\"wire_name\":\"projects\",\"label\":\"projects\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{\"placeholder\":\"Idea hint\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"name\"],\"default_selection\":[\"id\",\"name\"],\"operations\":[]},{\"id\":\"Project\",\"label\":\"Project\",\"fields\":[{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"state\"],\"default_selection\":[\"id\",\"title\",\"ownerId\"],\"operations\":[]},{\"id\":\"Task\",\"label\":\"Task\",\"fields\":[{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"title\",\"done\"],\"default_selection\":[\"id\",\"title\",\"projectId\",\"done\"],\"operations\":[]},{\"id\":\"RoleProof\",\"label\":\"RoleProof\",\"fields\":[{\"id\":\"RoleProof.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"RoleProof.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":\"fixture default\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.zero\",\"wire_name\":\"zero\",\"label\":\"zero\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{\"max\":10,\"min\":0,\"step\":1},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.rows\",\"wire_name\":\"rows\",\"label\":\"Rows </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.stringlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.textlist\",\"props\":{\"tags\":false},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.metadata\",\"wire_name\":\"metadata\",\"label\":\"metadata\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.metadata\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.score\",\"wire_name\":\"score\",\"label\":\"score\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.rating\",\"props\":{\"max\":5},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.slug\",\"wire_name\":\"slug\",\"label\":\"slug\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slug\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.sensitive\",\"wire_name\":\"sensitive\",\"label\":\"sensitive\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":false,\"writable\":true,\"sensitive\":true,\"nullable\":true,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"text\"],\"default_selection\":[],\"operations\":[]}],\"fieldsets\":[{\"id\":\"Address\",\"label\":\"Address\",\"fields\":[{\"id\":\"Address.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Address.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "RoleProof", path)?;
            }
        }
        Ok(())
    }
}
