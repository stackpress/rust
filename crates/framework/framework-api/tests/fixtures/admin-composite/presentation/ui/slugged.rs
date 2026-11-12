//! Generated portable source roles; regenerate from the application's composed Idea input.
use leptos::prelude::*;
use stackpress_framework_admin::RoleContext;
use stackpress_framework_schema::Role;
use stackpress_framework_schema::{SchemaResult, UiField, UiModel};

use stackpress_framework_admin::Draft;
use stackpress_framework_schema::__private::serde_json;
/// Only portable presentation metadata; no native descriptors or providers.
#[expect(
    clippy::expect_used,
    reason = "generator serializes and round-trip validates this compile-time UiModel literal"
)]
pub fn model() -> UiModel {
    serde_json::from_str("{\"id\":\"Slugged\",\"label\":\"Slugged\",\"fields\":[{\"id\":\"Slugged.slug\",\"wire_name\":\"slug\",\"label\":\"slug\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Slugged.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}],\"key\":[\"slug\"],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}").expect("generated source UI descriptor")
}
/// Source field order and props, detached from private metadata.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    Ok(model().fields)
}

/// Render the source-selected field roles for this instance.
pub fn field(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "title"),
            &fields,
            draft,
            "title",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected filter roles for this instance.
pub fn filter(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let _ = (draft, instance);

    let controls = Vec::<AnyView>::new();
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected span roles for this instance.
pub fn span(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let _ = (draft, instance);

    let controls = Vec::<AnyView>::new();
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected list roles for this instance.
pub fn list_cells(value: &serde_json::Value, instance: &str) -> SchemaResult<Vec<AnyView>> {
    let fields = descriptors()?;
    let draft = Draft::new(&fields, value, Role::List)?;
    let draft = &draft;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "slug"),
            &fields,
            draft,
            "slug",
            Role::List,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "title"),
            &fields,
            draft,
            "title",
            Role::List,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    Ok(controls)
}
/// Render source list controls outside a table.
pub fn list(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    Ok(list_cells(value, instance)?.collect_view().into_any())
}

/// Render the source-selected view roles for this instance.
pub fn view(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let draft = Draft::new(&fields, value, Role::View)?;
    let draft = &draft;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "slug"),
            &fields,
            draft,
            "slug",
            Role::View,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "title"),
            &fields,
            draft,
            "title",
            Role::View,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    Ok(controls.collect_view().into_any())
}
