//! Generated portable source roles; regenerate from the application's composed Idea input.
use leptos::prelude::*;
use stackpress_framework_admin::RoleContext;
use stackpress_framework_schema::Role;
use stackpress_framework_schema::{SchemaResult, UiField, UiModel};

use stackpress_framework_admin::Draft;
use stackpress_framework_schema::__private::serde_json;
/// Only portable presentation metadata; no native descriptors or providers.
pub fn model() -> UiModel {
    serde_json::from_str("{\"id\":\"Catalog\",\"label\":\"Catalog\",\"fields\":[{\"id\":\"Catalog.categoryId\",\"wire_name\":\"categoryId\",\"label\":\"categoryId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Catalog.articleId\",\"wire_name\":\"articleId\",\"label\":\"articleId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Catalog.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Catalog.active\",\"wire_name\":\"active\",\"label\":\"active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":false,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null}],\"key\":[\"categoryId\",\"articleId\"],\"query_fields\":[],\"default_selection\":[],\"operations\":[]}").expect("generated source UI descriptor")
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
        let field = fields
            .iter()
            .find(|field| field.wire_name == "title")
            .expect("generated role field")
            .clone();
        let value = draft.get("title").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "title"),
            field,
            role: Role::Field,
            value,
        };
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
        let field = fields
            .iter()
            .find(|field| field.wire_name == "categoryId")
            .expect("generated role field")
            .clone();
        let value = draft.get("categoryId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "categoryId"),
            field,
            role: Role::List,
            value,
        };
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "articleId")
            .expect("generated role field")
            .clone();
        let value = draft.get("articleId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "articleId"),
            field,
            role: Role::List,
            value,
        };
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "title")
            .expect("generated role field")
            .clone();
        let value = draft.get("title").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "title"),
            field,
            role: Role::List,
            value,
        };
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
        let field = fields
            .iter()
            .find(|field| field.wire_name == "categoryId")
            .expect("generated role field")
            .clone();
        let value = draft.get("categoryId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "categoryId"),
            field,
            role: Role::View,
            value,
        };
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "articleId")
            .expect("generated role field")
            .clone();
        let value = draft.get("articleId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "articleId"),
            field,
            role: Role::View,
            value,
        };
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "title")
            .expect("generated role field")
            .clone();
        let value = draft.get("title").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "title"),
            field,
            role: Role::View,
            value,
        };
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    Ok(controls.collect_view().into_any())
}
