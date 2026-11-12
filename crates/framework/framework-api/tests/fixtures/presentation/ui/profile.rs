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
    serde_json::from_str("{\"id\":\"Profile\",\"label\":\"Profile\",\"fields\":[{\"id\":\"Profile.id\",\"wire_name\":\"id\",\"label\":\"ID\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.source.view.overflow\",\"props\":{\"hellip\":true,\"length\":10},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.image\",\"wire_name\":\"image\",\"label\":\"Image\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.name\",\"wire_name\":\"name\",\"label\":\"Name\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":true,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.type\",\"wire_name\":\"type\",\"label\":\"Type\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"person\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.lowercase\",\"props\":{\"case\":\"lowercase\"},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.lowercase\",\"props\":{\"case\":\"lowercase\"},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.roles\",\"wire_name\":\"roles\",\"label\":\"Roles\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[\"GUEST\"]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.stringlist\",\"props\":{\"add\":\"Add Role\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.tags\",\"wire_name\":\"tags\",\"label\":\"Tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.references\",\"wire_name\":\"references\",\"label\":\"References\",\"value_type\":{\"value\":{\"kind\":\"Object\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":{}},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.source.field.metadata\",\"props\":{\"add\":\"Add Reference\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.metadata\",\"props\":{\"className\":\"frui-pt-md frui-pr-md frui-pb-md frui-pl-md\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.active\",\"wire_name\":\"active\",\"label\":\"Active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"filter\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.created\",\"wire_name\":\"created\",\"label\":\"Created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"span\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.updated\",\"wire_name\":\"updated\",\"label\":\"Updated\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"span\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Profile.auth\",\"wire_name\":\"auth\",\"label\":\"Authentications\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Auth\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":false,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Auth\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.applications\",\"wire_name\":\"applications\",\"label\":\"Applications\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Application\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":false,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Application\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Profile.sessions\",\"wire_name\":\"sessions\",\"label\":\"Sessions\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Session\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":false,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Session\",\"many\":true,\"mapping\":null,\"lookup\":null}}],\"key\":[\"id\"],\"query_fields\":[\"type\",\"active\"],\"default_selection\":[],\"operations\":[]}").expect("generated source UI descriptor")
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
            format!("{instance}-field-{}", "image"),
            &fields,
            draft,
            "image",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "name"),
            &fields,
            draft,
            "name",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "type"),
            &fields,
            draft,
            "type",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "roles"),
            &fields,
            draft,
            "roles",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TextListField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "tags"),
            &fields,
            draft,
            "tags",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::components::TextListField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-field-{}", "references"),
            &fields,
            draft,
            "references",
            Role::Field,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::MetadataField context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected filter roles for this instance.
pub fn filter(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-filter-{}", "type"),
            &fields,
            draft,
            "type",
            Role::Filter,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::InputField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-filter-{}", "active"),
            &fields,
            draft,
            "active",
            Role::Filter,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::SwitchField context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}

/// Render the source-selected span roles for this instance.
pub fn span(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-span-{}", "created"),
            &fields,
            draft,
            "created",
            Role::Span,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::components::TemporalField context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-span-{}", "updated"),
            &fields,
            draft,
            "updated",
            Role::Span,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::components::TemporalField context />}.into_any(),
        );
    }
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
            format!("{instance}-list-{}", "id"),
            &fields,
            draft,
            "id",
            Role::List,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::OverflowView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "image"),
            &fields,
            draft,
            "image",
            Role::List,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::ImageView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "name"),
            &fields,
            draft,
            "name",
            Role::List,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "type"),
            &fields,
            draft,
            "type",
            Role::List,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::CaseView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "roles"),
            &fields,
            draft,
            "roles",
            Role::List,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TagsView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "tags"),
            &fields,
            draft,
            "tags",
            Role::List,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TagsView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "created"),
            &fields,
            draft,
            "created",
            Role::List,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-list-{}", "updated"),
            &fields,
            draft,
            "updated",
            Role::List,
        )?;
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
    let fields = descriptors()?;
    let draft = Draft::new(&fields, value, Role::View)?;
    let draft = &draft;

    let mut controls = Vec::<AnyView>::new();
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "id"),
            &fields,
            draft,
            "id",
            Role::View,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "image"),
            &fields,
            draft,
            "image",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::ImageView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "name"),
            &fields,
            draft,
            "name",
            Role::View,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::TextView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "type"),
            &fields,
            draft,
            "type",
            Role::View,
        )?;
        controls
            .push(view! {<stackpress_framework_admin::components::CaseView context />}.into_any());
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "roles"),
            &fields,
            draft,
            "roles",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TagsView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "tags"),
            &fields,
            draft,
            "tags",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::TagsView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "references"),
            &fields,
            draft,
            "references",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::MetadataView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "active"),
            &fields,
            draft,
            "active",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::components::BooleanView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "created"),
            &fields,
            draft,
            "created",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    {
        let context = RoleContext::from_draft(
            format!("{instance}-view-{}", "updated"),
            &fields,
            draft,
            "updated",
            Role::View,
        )?;
        controls.push(
            view! {<stackpress_framework_admin::source::roles::DateView context />}.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
