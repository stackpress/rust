//! Explicit application projections. Trusted stored rows must use project() before public output.
/// Application-approved owned output.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicNote {
    /// Approved public field.
    #[serde(rename = "id")]
    pub id: String,
    /// Approved public field.
    #[serde(rename = "title")]
    pub title: String,
}
impl super::note::Note {
    /// Project only fields explicitly approved by the compiled application policy.
    pub fn project(&self) -> stackpress_framework_schema::SchemaResult<PublicNote> {
        self.validate()?;
        let public = PublicNote {
            id: self.id.clone(),
            title: self.title.clone(),
        };
        let wire = stackpress_framework_schema::to_bounded_value(&public)?;
        stackpress_framework_schema::check_browser_json(&wire)?;
        Ok(public)
    }
}
/// Public create input; generated/read-only/relation members cannot be supplied.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicCreateNote {
    /// Writable public input.
    #[serde(
        rename = "id",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub id: stackpress_framework_schema::Input<String>,
    /// Writable public input.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Input::is_missing"
    )]
    pub title: stackpress_framework_schema::Input<String>,
}
impl PublicCreateNote {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Note\",\"rust_name\":\"Note\",\"module\":\"note\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
impl super::note::CreateNote {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Note\",\"rust_name\":\"Note\",\"module\":\"note\",\"fields\":[{\"name\":\"id\",\"rust_name\":\"id\",\"column\":\"id\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":true,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]},{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]},\"patch\":false}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
    ) -> stackpress_framework_schema::SchemaResult<super::note::Note> {
        self.prepare_with(context)
    }
}
/// Published partial update; the host supplies private values explicitly.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicPatchNote {
    /// Approved update value; omission never writes null.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
}
impl PublicPatchNote {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Note\",\"rust_name\":\"Note\",\"module\":\"note\",\"fields\":[{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub struct NoteKey {
    /// Key member.
    #[serde(rename = "id")]
    pub id: String,
}
/// Trusted partial update input; Unchanged never writes null.
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PatchNote {
    /// Omitted, supplied, or explicit null patch value.
    #[serde(
        rename = "title",
        default,
        skip_serializing_if = "stackpress_framework_schema::Patch::is_unchanged"
    )]
    pub title: stackpress_framework_schema::Patch<String>,
}
impl PatchNote {
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
        let policy:stackpress_framework_schema::RecordSchema=serde_json::from_str("{\"model\":{\"name\":\"Note\",\"rust_name\":\"Note\",\"module\":\"note\",\"fields\":[{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"patch policy decoding failed").with_cause(cause))?;
        policy.validate(&value, context, true, false)?.into_result()
    }
}
impl PatchNote {
    /// Decode the bounded input policy; no private literal defaults or imports are embedded.
    fn input_schema()
    -> stackpress_framework_schema::SchemaResult<stackpress_framework_schema::InputSchema> {
        serde_json::from_str("{\"record\":{\"model\":{\"name\":\"Note\",\"rust_name\":\"Note\",\"module\":\"note\",\"fields\":[{\"name\":\"title\",\"rust_name\":\"title\",\"column\":\"title\",\"value_type\":{\"kind\":\"String\"},\"required\":true,\"multiple\":false,\"key\":false,\"default\":null,\"nonempty\":false,\"active\":false,\"relation\":null,\"unique\":false,\"searchable\":false,\"sortable\":false,\"timestamp\":false,\"generated\":false,\"input_required\":true,\"value_rule\":null,\"rules\":[],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"roles\":[],\"access\":{\"readable\":false,\"writable\":false,\"sensitive\":false,\"publish_default\":false},\"attributes\":[]}],\"documentation\":{\"label\":null,\"description\":null,\"examples\":[],\"display\":null,\"icon\":null,\"labels\":[]},\"query\":[],\"attributes\":[]},\"fieldsets\":[],\"enums\":[]},\"patch\":true}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"input policy decoding failed").with_cause(cause))
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
pub enum NoteFilter {
    /// Approved typed predicate.
    #[serde(rename = "id")]
    Id(stackpress_framework_schema::Predicate<String>),
}
impl NoteFilter {
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
pub enum NoteSort {}
/// Public typed query; it performs no IO and registers no operation.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct NoteQuery {
    /// Literal text search on approved searchable fields.
    pub q: Option<String>,
    /// Finite conjunctive typed predicates.
    pub filters: Vec<NoteFilter>,
    /// Explicit allowlisted directions and null placement.
    pub sort: Vec<stackpress_framework_schema::Sort<NoteSort>>,
    /// At most sixteen public relation paths, depth three.
    pub includes: Vec<String>,
    /// Offset, at most 100,000.
    pub skip: u64,
    /// Page size, 1 through 100 (default 20).
    pub take: u16,
    /// Request a matched-base-row total in future execution.
    pub total: bool,
}
impl Default for NoteQuery {
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
impl NoteQuery {
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
            let schema:stackpress_framework_schema::UiSchema=serde_json::from_str("{\"contract_revision\":3,\"schema_fingerprint\":\"3cdd24100272d6e2022291879c27f6446d4fc277f597084324a845dc0dd5b68b\",\"models\":[{\"id\":\"Note\",\"label\":\"Note\",\"fields\":[{\"id\":\"Note.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Note.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"title\"],\"operations\":[]}],\"fieldsets\":[]}").map_err(|cause|stackpress_framework_schema::SchemaError::new(stackpress_framework_schema::SchemaErrorKind::Codec,"query projection decoding failed").with_cause(cause))?;
            for path in &self.includes {
                stackpress_framework_schema::check_include(&schema, "Note", path)?;
            }
        }
        Ok(())
    }
}
