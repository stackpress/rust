//! Generated portable source roles; regenerate from the application's composed Idea input.
use leptos::prelude::*;
use stackpress_framework_admin::RoleContext;
use stackpress_framework_schema::Role;
use stackpress_framework_schema::{SchemaResult, UiField, UiModel};

use stackpress_framework_admin::Draft;
use stackpress_framework_schema::__private::serde_json;
/// Only portable presentation metadata; no native descriptors or providers.
pub fn model() -> UiModel {
    serde_json::from_str("{\"id\":\"Auth\",\"label\":\"Auth\",\"fields\":[{\"id\":\"Auth.id\",\"wire_name\":\"id\",\"label\":\"ID\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.overflow\",\"props\":{\"hellip\":true,\"length\":10},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.profileId\",\"wire_name\":\"profileId\",\"label\":\"Profile\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.type\",\"wire_name\":\"type\",\"label\":\"Type\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"username\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.token\",\"wire_name\":\"token\",\"label\":\"Token\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.secret\",\"wire_name\":\"secret\",\"label\":\"Secret\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":false,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.password\",\"props\":{\"type\":\"password\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.verified\",\"wire_name\":\"verified\",\"label\":\"Verified\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.consumed\",\"wire_name\":\"consumed\",\"label\":\"Last Used\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.active\",\"wire_name\":\"active\",\"label\":\"Active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"filter\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.created\",\"wire_name\":\"created\",\"label\":\"Created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.updated\",\"wire_name\":\"updated\",\"label\":\"Updated\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Auth.profile\",\"wire_name\":\"profile\",\"label\":\"profile\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":false,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"profileId\",\"type\",\"verified\",\"active\"],\"default_selection\":[\"*\",\"profile.*\"],\"operations\":[]}").expect("generated source UI descriptor")
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
            .find(|field| field.wire_name == "profileId")
            .expect("generated role field")
            .clone();
        let value = draft.get("profileId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "profileId"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::RelationField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "type")
            .expect("generated role field")
            .clone();
        let value = draft.get("type").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "type"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "token")
            .expect("generated role field")
            .clone();
        let value = draft.get("token").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "token"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "secret")
            .expect("generated role field")
            .clone();
        let value = draft.get("secret").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "secret"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "verified")
            .expect("generated role field")
            .clone();
        let value = draft.get("verified").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-field-{}", "verified"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::SwitchField context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected filter roles for this instance.
pub fn filter(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;

    let mut controls = Vec::<AnyView>::new();
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "profileId")
            .expect("generated role field")
            .clone();
        let value = draft.get("profileId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-filter-{}", "profileId"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::RelationField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "type")
            .expect("generated role field")
            .clone();
        let value = draft.get("type").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-filter-{}", "type"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "verified")
            .expect("generated role field")
            .clone();
        let value = draft.get("verified").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-filter-{}", "verified"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::SwitchField context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "active")
            .expect("generated role field")
            .clone();
        let value = draft.get("active").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-filter-{}", "active"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::SwitchField context />}.into_any(),
        );
    }
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
    let value_row = value;

    let fields = descriptors()?;
    let draft = Draft::new(&fields, value, Role::List)?;
    let draft = &draft;

    let mut controls = Vec::<AnyView>::new();
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "id")
            .expect("generated role field")
            .clone();
        let value = draft.get("id").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "id"),
            field,
            role: Role::List,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::OverflowView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "profileId")
            .expect("generated role field")
            .clone();
        let value = draft.get("profileId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "profileId"),
            field,
            role: Role::List,
            value,
        };
        let row = value_row.clone();
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TemplateView context row/>}
                .into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "type")
            .expect("generated role field")
            .clone();
        let value = draft.get("type").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "type"),
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
            .find(|field| field.wire_name == "token")
            .expect("generated role field")
            .clone();
        let value = draft.get("token").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "token"),
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
            .find(|field| field.wire_name == "verified")
            .expect("generated role field")
            .clone();
        let value = draft.get("verified").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "verified"),
            field,
            role: Role::List,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::components::BooleanView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "consumed")
            .expect("generated role field")
            .clone();
        let value = draft.get("consumed").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "consumed"),
            field,
            role: Role::List,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "created")
            .expect("generated role field")
            .clone();
        let value = draft.get("created").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "created"),
            field,
            role: Role::List,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "updated")
            .expect("generated role field")
            .clone();
        let value = draft.get("updated").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-list-{}", "updated"),
            field,
            role: Role::List,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    Ok(controls)
}
/// Render source list controls outside a table.
pub fn list(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    Ok(list_cells(value, instance)?.collect_view().into_any())
}

/// Render the source-selected view roles for this instance.
pub fn view(value: &serde_json::Value, instance: &str) -> SchemaResult<AnyView> {
    let value_row = value;

    let fields = descriptors()?;
    let draft = Draft::new(&fields, value, Role::View)?;
    let draft = &draft;

    let mut controls = Vec::<AnyView>::new();
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "id")
            .expect("generated role field")
            .clone();
        let value = draft.get("id").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "id"),
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
            .find(|field| field.wire_name == "profileId")
            .expect("generated role field")
            .clone();
        let value = draft.get("profileId").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "profileId"),
            field,
            role: Role::View,
            value,
        };
        let row = value_row.clone();
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TemplateView context row/>}
                .into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "type")
            .expect("generated role field")
            .clone();
        let value = draft.get("type").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "type"),
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
            .find(|field| field.wire_name == "token")
            .expect("generated role field")
            .clone();
        let value = draft.get("token").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "token"),
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
            .find(|field| field.wire_name == "verified")
            .expect("generated role field")
            .clone();
        let value = draft.get("verified").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "verified"),
            field,
            role: Role::View,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::components::BooleanView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "consumed")
            .expect("generated role field")
            .clone();
        let value = draft.get("consumed").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "consumed"),
            field,
            role: Role::View,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "active")
            .expect("generated role field")
            .clone();
        let value = draft.get("active").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "active"),
            field,
            role: Role::View,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::components::BooleanView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "created")
            .expect("generated role field")
            .clone();
        let value = draft.get("created").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "created"),
            field,
            role: Role::View,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let field = fields
            .iter()
            .find(|field| field.wire_name == "updated")
            .expect("generated role field")
            .clone();
        let value = draft.get("updated").expect("source role admitted draft");
        let context = RoleContext {
            id: format!("{instance}-view-{}", "updated"),
            field,
            role: Role::View,
            value,
        };
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
