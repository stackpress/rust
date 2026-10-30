//! Generated public page adapters; imports register nothing.
use leptos::prelude::*;
use stackpress_framework_admin::{
    Draft, RoleContext,
    pages::{Action, Form, PageSpec},
};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult};

/// Typed adapters for one explicitly selected public model.
pub mod application {
    use super::*;
    /// Decode only the approved revision-3 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"application\",\"model\":{\"id\":\"Application\",\"label\":\"App\",\"fields\":[{\"id\":\"Application.id\",\"wire_name\":\"id\",\"label\":\"ID\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.profileId\",\"wire_name\":\"profileId\",\"label\":\"Profile\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.name\",\"wire_name\":\"name\",\"label\":\"Name\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Name is required\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":true,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.logo\",\"wire_name\":\"logo\",\"label\":\"Image\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.website\",\"wire_name\":\"website\",\"label\":\"Website\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":true,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.secret\",\"wire_name\":\"secret\",\"label\":\"Secret\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.scopes\",\"wire_name\":\"scopes\",\"label\":\"Scopes\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.active\",\"wire_name\":\"active\",\"label\":\"Active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"filter\",\"component\":\"api.source.field.switch\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.expires\",\"wire_name\":\"expires\",\"label\":\"Expires\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.created\",\"wire_name\":\"created\",\"label\":\"Created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.updated\",\"wire_name\":\"updated\",\"label\":\"Updated\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.session\",\"wire_name\":\"session\",\"label\":\"Sessions\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Session\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Session\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Application.profile\",\"wire_name\":\"profile\",\"label\":\"profile\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"profileId\",\"active\"],\"default_selection\":[\"id\",\"profileId\",\"name\",\"logo\",\"website\",\"secret\",\"scopes\",\"active\",\"expires\",\"created\",\"updated\",\"profile.id\",\"profile.image\",\"profile.name\",\"profile.type\",\"profile.roles\",\"profile.tags\",\"profile.references\",\"profile.active\",\"profile.created\",\"profile.updated\"],\"operations\":[]},\"soft_remove\":true}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(
                serde_json::from_value::<
                    super::super::super::contracts::public::PublicPatchApplication,
                >(value)
                .map_err(codec)?,
            )
            .map_err(codec)
        } else {
            serde_json::to_value(
                serde_json::from_value::<
                    super::super::super::contracts::public::PublicCreateApplication,
                >(value)
                .map_err(codec)?,
            )
            .map_err(codec)
        }
    }
    /// Validate all supplied import members through the actual generated create contract.
    pub fn validate_import(
        value: serde_json::Value,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
        let input: super::super::super::contracts::public::PublicCreateApplication =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::ApplicationQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::application::descriptors()?;
        // Related scope can add a disabled constraint, never enable a forbidden field.
        if let Some(locked) = &form.locked {
            for field in &mut fields {
                if &field.wire_name == locked {
                    for binding in &mut field.roles {
                        if binding.role == Role::Field {
                            binding.props.insert("disabled".into(), true.into());
                        }
                    }
                }
            }
        }
        let mut controls = Vec::<AnyView>::new();

        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "profileId")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("profileId").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "profileId"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::RelationField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "name")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("name").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "name"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::InputField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "logo")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("logo").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "logo"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::InputField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "website")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("website").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "website"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::InputField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "scopes")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("scopes").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "scopes"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextListField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "expires")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("expires").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "expires"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TemporalField context/> }
                    .into_any(),
            );
        }
        Ok(controls.collect_view().into_any())
    }
    /// Compose the explicitly selected filter roles.
    pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
        super::super::application::filter(draft, instance)
    }

    /// Render actual public rows through their typed DTO and selected list adapters.
    pub fn table(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let spec = spec()?;
        let mut rows = Vec::new();
        for value in value
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(unavailable)?
        {
            let row = value.get("row").unwrap_or(value);
            let key = stackpress_framework_admin::pages::key(&spec, row)?;
            let typed: super::super::super::contracts::public::PublicApplication =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::application::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicApplication =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::application::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}

/// Typed adapters for one explicitly selected public model.
pub mod session {
    use super::*;
    /// Decode only the approved revision-3 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"session\",\"model\":{\"id\":\"Session\",\"label\":\"Session\",\"fields\":[{\"id\":\"Session.id\",\"wire_name\":\"id\",\"label\":\"ID\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.applicationId\",\"wire_name\":\"applicationId\",\"label\":\"App\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"App is required\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/application/search?json\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/application/search?json\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{application.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{application.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.profileId\",\"wire_name\":\"profileId\",\"label\":\"Profile\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.secret\",\"wire_name\":\"secret\",\"label\":\"Secret\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.scopes\",\"wire_name\":\"scopes\",\"label\":\"Scopes\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.active\",\"wire_name\":\"active\",\"label\":\"Active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"filter\",\"component\":\"api.source.field.switch\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.expires\",\"wire_name\":\"expires\",\"label\":\"Expires\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.created\",\"wire_name\":\"created\",\"label\":\"Created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.updated\",\"wire_name\":\"updated\",\"label\":\"Updated\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Session.application\",\"wire_name\":\"application\",\"label\":\"application\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Application\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Application\",\"many\":false,\"mapping\":{\"local\":\"applicationId\",\"foreign\":\"id\",\"inverse\":\"session\"},\"lookup\":null}},{\"id\":\"Session.profile\",\"wire_name\":\"profile\",\"label\":\"profile\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"applicationId\",\"profileId\",\"active\"],\"default_selection\":[\"id\",\"applicationId\",\"profileId\",\"secret\",\"scopes\",\"active\",\"expires\",\"created\",\"updated\",\"profile.id\",\"profile.image\",\"profile.name\",\"profile.type\",\"profile.roles\",\"profile.tags\",\"profile.references\",\"profile.active\",\"profile.created\",\"profile.updated\",\"application.id\",\"application.profileId\",\"application.name\",\"application.logo\",\"application.website\",\"application.secret\",\"application.scopes\",\"application.active\",\"application.expires\",\"application.created\",\"application.updated\"],\"operations\":[]},\"soft_remove\":true}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicPatchSession>(value).map_err(codec)?).map_err(codec)
        } else {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicCreateSession>(value).map_err(codec)?).map_err(codec)
        }
    }
    /// Validate all supplied import members through the actual generated create contract.
    pub fn validate_import(
        value: serde_json::Value,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
        let input: super::super::super::contracts::public::PublicCreateSession =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::SessionQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::session::descriptors()?;
        // Related scope can add a disabled constraint, never enable a forbidden field.
        if let Some(locked) = &form.locked {
            for field in &mut fields {
                if &field.wire_name == locked {
                    for binding in &mut field.roles {
                        if binding.role == Role::Field {
                            binding.props.insert("disabled".into(), true.into());
                        }
                    }
                }
            }
        }
        let mut controls = Vec::<AnyView>::new();

        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "applicationId")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("applicationId").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "applicationId"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::RelationField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "profileId")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("profileId").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "profileId"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::RelationField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "scopes")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("scopes").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "scopes"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextListField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "expires")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("expires").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "expires"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TemporalField context/> }
                    .into_any(),
            );
        }
        Ok(controls.collect_view().into_any())
    }
    /// Compose the explicitly selected filter roles.
    pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
        super::super::session::filter(draft, instance)
    }

    /// Render actual public rows through their typed DTO and selected list adapters.
    pub fn table(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let spec = spec()?;
        let mut rows = Vec::new();
        for value in value
            .get("rows")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(unavailable)?
        {
            let row = value.get("row").unwrap_or(value);
            let key = stackpress_framework_admin::pages::key(&spec, row)?;
            let typed: super::super::super::contracts::public::PublicSession =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::session::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicSession =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::session::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}
/// Explicit compiled page modules in navigation order.
pub const MODELS: &[&str] = &["application", "session"];
/// Resolve only compiler-owned static model identities.
pub fn identity(module: &str) -> Option<(&'static str, &'static str)> {
    match module {
        "application" => Some(("application", "Application")),
        "session" => Some(("session", "Session")),
        _ => None,
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn spec(module: &str) -> SchemaResult<PageSpec> {
    match module {
        "application" => application::spec(),
        "session" => session::spec(),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn input(
    module: &str,
    action: Action,
    value: serde_json::Value,
) -> SchemaResult<serde_json::Value> {
    match module {
        "application" => application::input(action, value),
        "session" => session::input(action, value),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn query(module: &str, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
    match module {
        "application" => application::query(value),
        "session" => session::query(value),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn validate_import(
    module: &str,
    value: serde_json::Value,
    context: &stackpress_framework_schema::ValueContext<'_>,
) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
    match module {
        "application" => application::validate_import(value, context),
        "session" => session::validate_import(value, context),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn form(module: &str, form: Form) -> SchemaResult<AnyView> {
    match module {
        "application" => application::form(form),
        "session" => session::form(form),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn filters(module: &str, draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "application" => application::filters(draft, instance),
        "session" => session::filters(draft, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn table(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "application" => application::table(value, instance),
        "session" => session::table(value, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn detail(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "application" => application::detail(value, instance),
        "session" => session::detail(value, instance),
        _ => Err(unavailable()),
    }
}
/// Missing model/role is unavailable; never substitute another operation.
fn unavailable() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "admin model or role unavailable")
}
