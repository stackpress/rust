//! Generated public page adapters; imports register nothing.
use leptos::prelude::*;
use stackpress_framework_admin::{
    Draft, RoleContext,
    pages::{Action, Form, PageSpec},
};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult};

/// Typed adapters for one explicitly selected public model.
pub mod profile {
    use super::*;
    /// Decode only the approved revision-2 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"profile\",\"model\":{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.address\",\"wire_name\":\"address\",\"label\":\"address\",\"value_type\":{\"value\":{\"kind\":\"Fieldset\",\"name\":\"Address\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"Profile.projects\",\"wire_name\":\"projects\",\"label\":\"projects\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name </script> 🦀\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{\"placeholder\":\"Idea hint\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"name\"],\"default_selection\":[\"id\",\"name\",\"active\",\"address\"],\"operations\":[]},\"soft_remove\":true}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicPatchProfile>(value).map_err(codec)?).map_err(codec)
        } else {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicCreateProfile>(value).map_err(codec)?).map_err(codec)
        }
    }
    /// Validate all supplied import members through the actual generated create contract.
    pub fn validate_import(
        value: serde_json::Value,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
        let input: super::super::super::contracts::public::PublicCreateProfile =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::ProfileQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::profile::descriptors()?;
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

        if form.action != Action::Update {
            {
                // This exact field was approved by the app before normalization.
                let field = fields
                    .iter()
                    .find(|field| field.wire_name == "id")
                    .cloned()
                    .ok_or_else(|| {
                        SchemaError::new(
                            SchemaErrorKind::Invalid,
                            "missing generated field descriptor",
                        )
                    })?;
                let value = draft.get("id").ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
                })?;
                let context = RoleContext {
                    id: format!("{}-field-{}", instance, "id"),
                    field,
                    role: Role::Field,
                    value,
                };
                controls.push(
                    view! { <stackpress_framework_admin::components::TextField context/> }
                        .into_any(),
                );
            }
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
                view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
            );
        }
        Ok(controls.collect_view().into_any())
    }
    /// Compose the explicitly selected filter roles.
    pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
        super::super::profile::filter(draft, instance)
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
            let typed: super::super::super::contracts::public::PublicProfile =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::profile::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicProfile =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::profile::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}

/// Typed adapters for one explicitly selected public model.
pub mod project {
    use super::*;
    /// Decode only the approved revision-2 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"project\",\"model\":{\"id\":\"Project\",\"label\":\"Project\",\"fields\":[{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"ownerId\",\"title\",\"state\"],\"default_selection\":[\"id\",\"ownerId\",\"title\",\"state\",\"tags\"],\"operations\":[]},\"soft_remove\":false}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicPatchProject>(value).map_err(codec)?).map_err(codec)
        } else {
            serde_json::to_value(serde_json::from_value::<super::super::super::contracts::public::PublicCreateProject>(value).map_err(codec)?).map_err(codec)
        }
    }
    /// Validate all supplied import members through the actual generated create contract.
    pub fn validate_import(
        value: serde_json::Value,
        context: &stackpress_framework_schema::ValueContext<'_>,
    ) -> SchemaResult<Vec<stackpress_framework_schema::FieldIssue>> {
        let input: super::super::super::contracts::public::PublicCreateProject =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::ProjectQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::project::descriptors()?;
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

        if form.action != Action::Update {
            {
                // This exact field was approved by the app before normalization.
                let field = fields
                    .iter()
                    .find(|field| field.wire_name == "id")
                    .cloned()
                    .ok_or_else(|| {
                        SchemaError::new(
                            SchemaErrorKind::Invalid,
                            "missing generated field descriptor",
                        )
                    })?;
                let value = draft.get("id").ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
                })?;
                let context = RoleContext {
                    id: format!("{}-field-{}", instance, "id"),
                    field,
                    role: Role::Field,
                    value,
                };
                controls.push(
                    view! { <stackpress_framework_admin::components::TextField context/> }
                        .into_any(),
                );
            }
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "ownerId")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("ownerId").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "ownerId"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "title")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("title").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "title"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "state")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("state").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "state"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::SelectField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "tags")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("tags").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "tags"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextListField context/> }
                    .into_any(),
            );
        }
        Ok(controls.collect_view().into_any())
    }
    /// Compose the explicitly selected filter roles.
    pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
        super::super::project::filter(draft, instance)
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
            let typed: super::super::super::contracts::public::PublicProject =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::project::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicProject =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::project::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}

/// Typed adapters for one explicitly selected public model.
pub mod task {
    use super::*;
    /// Decode only the approved revision-2 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"task\",\"model\":{\"id\":\"Task\",\"label\":\"Task\",\"fields\":[{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.textarea\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.priority\",\"wire_name\":\"priority\",\"label\":\"priority\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.slider\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.amount\",\"wire_name\":\"amount\",\"label\":\"amount\",\"value_type\":{\"value\":{\"kind\":\"Number\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.number\",\"props\":{\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.number\",\"props\":{\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.number\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.number\",\"props\":{\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.number\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.decimal\",\"wire_name\":\"decimal\",\"label\":\"decimal\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.price\",\"props\":{\"locale\":\"fr\",\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.price\",\"props\":{\"locale\":\"fr\",\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.price\",\"props\":{\"locale\":\"fr\",\"scale\":2},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.price\",\"props\":{\"locale\":\"fr\",\"scale\":2},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.date\",\"wire_name\":\"date\",\"label\":\"date\",\"value_type\":{\"value\":{\"kind\":\"Date\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.time\",\"wire_name\":\"time\",\"label\":\"time\",\"value_type\":{\"value\":{\"kind\":\"Time\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.time\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.time\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.time\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.instant\",\"wire_name\":\"instant\",\"label\":\"instant\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.date\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.phone\",\"wire_name\":\"phone\",\"label\":\"phone\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.phone\",\"props\":{\"region\":\"US\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.phone\",\"props\":{\"region\":\"US\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.phone\",\"props\":{\"region\":\"US\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.phone\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\",\"projectId\",\"title\",\"done\",\"note\",\"priority\",\"amount\",\"decimal\",\"date\",\"time\",\"instant\",\"phone\"],\"default_selection\":[\"id\",\"projectId\",\"title\",\"done\",\"note\",\"priority\",\"amount\",\"decimal\",\"date\",\"time\",\"instant\",\"phone\"],\"operations\":[]},\"soft_remove\":false}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(
                serde_json::from_value::<super::super::super::contracts::public::PublicPatchTask>(
                    value,
                )
                .map_err(codec)?,
            )
            .map_err(codec)
        } else {
            serde_json::to_value(
                serde_json::from_value::<super::super::super::contracts::public::PublicCreateTask>(
                    value,
                )
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
        let input: super::super::super::contracts::public::PublicCreateTask =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::TaskQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::task::descriptors()?;
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

        if form.action != Action::Update {
            {
                // This exact field was approved by the app before normalization.
                let field = fields
                    .iter()
                    .find(|field| field.wire_name == "id")
                    .cloned()
                    .ok_or_else(|| {
                        SchemaError::new(
                            SchemaErrorKind::Invalid,
                            "missing generated field descriptor",
                        )
                    })?;
                let value = draft.get("id").ok_or_else(|| {
                    SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
                })?;
                let context = RoleContext {
                    id: format!("{}-field-{}", instance, "id"),
                    field,
                    role: Role::Field,
                    value,
                };
                controls.push(
                    view! { <stackpress_framework_admin::components::TextField context/> }
                        .into_any(),
                );
            }
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "projectId")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("projectId").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "projectId"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "title")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("title").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "title"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "done")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("done").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "done"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::CheckboxField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "note")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("note").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "note"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextareaField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "priority")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("priority").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "priority"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::SliderField context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "amount")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("amount").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "amount"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::DecimalField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "decimal")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("decimal").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "decimal"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::DecimalField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "date")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("date").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "date"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TemporalField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "time")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("time").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "time"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TemporalField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "instant")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("instant").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "instant"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TemporalField context/> }
                    .into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "phone")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("phone").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-field-{}", instance, "phone"),
                field,
                role: Role::Field,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::PhoneField context/> }.into_any(),
            );
        }
        Ok(controls.collect_view().into_any())
    }
    /// Compose the explicitly selected filter roles.
    pub fn filters(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
        super::super::task::filter(draft, instance)
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
            let typed: super::super::super::contracts::public::PublicTask =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::task::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicTask =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::task::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}
/// Explicit compiled page modules in navigation order.
pub const MODELS: &[&str] = &["profile", "project", "task"];
/// Resolve only compiler-owned static model identities.
pub fn identity(module: &str) -> Option<(&'static str, &'static str)> {
    match module {
        "profile" => Some(("profile", "Profile")),
        "project" => Some(("project", "Project")),
        "task" => Some(("task", "Task")),
        _ => None,
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn spec(module: &str) -> SchemaResult<PageSpec> {
    match module {
        "profile" => profile::spec(),
        "project" => project::spec(),
        "task" => task::spec(),
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
        "profile" => profile::input(action, value),
        "project" => project::input(action, value),
        "task" => task::input(action, value),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn query(module: &str, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
    match module {
        "profile" => profile::query(value),
        "project" => project::query(value),
        "task" => task::query(value),
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
        "profile" => profile::validate_import(value, context),
        "project" => project::validate_import(value, context),
        "task" => task::validate_import(value, context),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn form(module: &str, form: Form) -> SchemaResult<AnyView> {
    match module {
        "profile" => profile::form(form),
        "project" => project::form(form),
        "task" => task::form(form),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn filters(module: &str, draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => profile::filters(draft, instance),
        "project" => project::filters(draft, instance),
        "task" => task::filters(draft, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn table(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => profile::table(value, instance),
        "project" => project::table(value, instance),
        "task" => task::table(value, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn detail(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "profile" => profile::detail(value, instance),
        "project" => project::detail(value, instance),
        "task" => task::detail(value, instance),
        _ => Err(unavailable()),
    }
}
/// Missing model/role is unavailable; never substitute another operation.
fn unavailable() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "admin model or role unavailable")
}
