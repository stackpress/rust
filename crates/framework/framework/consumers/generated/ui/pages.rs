//! Generated public page adapters; imports register nothing.
use leptos::prelude::*;
use stackpress_framework_admin::{
    Draft, RoleContext,
    pages::{Action, Form, PageSpec},
};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult};

/// Typed adapters for one explicitly selected public model.
pub mod note {
    use super::*;
    /// Decode only the approved revision-3 model specification.
    pub fn spec() -> SchemaResult<PageSpec> {
        serde_json::from_str("{\"module\":\"note\",\"model\":{\"id\":\"Note\",\"label\":\"Note\",\"fields\":[{\"id\":\"Note.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Note.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"id\"],\"query_fields\":[\"id\"],\"default_selection\":[\"id\",\"title\"],\"operations\":[]},\"soft_remove\":false}").map_err(codec)
    }
    /// Decode through the actual generated public create/patch DTOs.
    pub fn input(action: Action, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        if action == Action::Update {
            serde_json::to_value(
                serde_json::from_value::<super::super::super::contracts::public::PublicPatchNote>(
                    value,
                )
                .map_err(codec)?,
            )
            .map_err(codec)
        } else {
            serde_json::to_value(
                serde_json::from_value::<super::super::super::contracts::public::PublicCreateNote>(
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
        let input: super::super::super::contracts::public::PublicCreateNote =
            serde_json::from_value(value).map_err(codec)?;
        Ok(input.validate_create(context)?.issues)
    }
    /// Validate the actual generated query type before sending its envelope.
    pub fn query(value: serde_json::Value) -> SchemaResult<serde_json::Value> {
        let query: super::super::super::contracts::public::NoteQuery =
            serde_json::from_value(value).map_err(codec)?;
        query.validate()?;
        serde_json::to_value(query).map_err(codec)
    }
    /// Compose selected field adapters; updates cannot edit the stored identity.
    pub fn form(form: Form) -> SchemaResult<AnyView> {
        let draft = &form.draft;
        let instance = &form.instance;
        let mut fields = super::super::note::descriptors()?;
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
        Ok(controls.collect_view().into_any())
    }
    /// This page has no selected filter controls.
    pub fn filters(_: &Draft, _: &str) -> SchemaResult<AnyView> {
        Ok(Vec::<AnyView>::new().collect_view().into_any())
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
            let typed: super::super::super::contracts::public::PublicNote =
                serde_json::from_value(row.clone()).map_err(codec)?;
            rows.push((key.to_string(), typed));
        }
        super::super::note::table(&rows, instance)
    }
    /// Render actual public detail through its typed DTO and selected view adapters.
    pub fn detail(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
        let typed: super::super::super::contracts::public::PublicNote =
            serde_json::from_value(value.clone()).map_err(codec)?;
        super::super::note::view(&typed, instance)
    }
    /// Keep the concrete codec cause private under one adapter context.
    fn codec(cause: serde_json::Error) -> SchemaError {
        SchemaError::new(SchemaErrorKind::Codec, "public admin codec failed").with_cause(cause)
    }
}
/// Explicit compiled page modules in navigation order.
pub const MODELS: &[&str] = &["note"];
/// Resolve only compiler-owned static model identities.
pub fn identity(module: &str) -> Option<(&'static str, &'static str)> {
    match module {
        "note" => Some(("note", "Note")),
        _ => None,
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn spec(module: &str) -> SchemaResult<PageSpec> {
    match module {
        "note" => note::spec(),
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
        "note" => note::input(action, value),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn query(module: &str, value: serde_json::Value) -> SchemaResult<serde_json::Value> {
    match module {
        "note" => note::query(value),
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
        "note" => note::validate_import(value, context),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn form(module: &str, form: Form) -> SchemaResult<AnyView> {
    match module {
        "note" => note::form(form),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn filters(module: &str, draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "note" => note::filters(draft, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn table(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "note" => note::table(value, instance),
        _ => Err(unavailable()),
    }
}
/// Dispatch only to an explicitly generated typed adapter.
pub fn detail(module: &str, value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    match module {
        "note" => note::detail(value, instance),
        _ => Err(unavailable()),
    }
}
/// Missing model/role is unavailable; never substitute another operation.
fn unavailable() -> SchemaError {
    SchemaError::new(SchemaErrorKind::Invalid, "admin model or role unavailable")
}
