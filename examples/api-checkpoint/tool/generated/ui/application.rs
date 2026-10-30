//! Generated public role wrappers; handwritten adapters remain outside this partition.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str("[{\"id\":\"Application.id\",\"wire_name\":\"id\",\"label\":\"ID\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.profileId\",\"wire_name\":\"profileId\",\"label\":\"Profile\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Profile is required\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"filter\",\"component\":\"api.source.field.relation\",\"props\":{\"id\":\"id\",\"search\":\"/admin/profile/search?json&q={{query}}\",\"template\":\"{{name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.template\",\"props\":{\"template\":\"{{profile.name}}\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.name\",\"wire_name\":\"name\",\"label\":\"Name\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Name is required\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":true,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.input\",\"props\":{\"type\":\"text\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.logo\",\"wire_name\":\"logo\",\"label\":\"Image\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.image\",\"props\":{\"height\":20,\"width\":20},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.image\",\"props\":{\"height\":100,\"width\":100},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.website\",\"wire_name\":\"website\",\"label\":\"Website\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":true,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"api.source.field.url\",\"props\":{\"type\":\"url\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.link\",\"props\":{\"target\":\"blank\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.secret\",\"wire_name\":\"secret\",\"label\":\"Secret\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.scopes\",\"wire_name\":\"scopes\",\"label\":\"Scopes\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.tags\",\"props\":{\"className\":\"frui-mr-md\",\"pill\":true,\"warning\":true},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.active\",\"wire_name\":\"active\",\"label\":\"Active\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":true},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"filter\",\"component\":\"api.source.field.switch\",\"props\":{},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.expires\",\"wire_name\":\"expires\",\"label\":\"Expires\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"required\",\"args\":[],\"client_checkable\":true,\"message\":\"Expires is required\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.datetime\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.created\",\"wire_name\":\"created\",\"label\":\"Created\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.updated\",\"wire_name\":\"updated\",\"label\":\"Updated\",\"value_type\":{\"value\":{\"kind\":\"Datetime\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"HostGenerated\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":true,\"roles\":[{\"role\":\"list\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true},{\"role\":\"view\",\"component\":\"api.source.view.date\",\"props\":{\"format\":\"m d, Y h:iA\"},\"definition_revision\":\"source-1\",\"available\":true}],\"relation\":null},{\"id\":\"Application.session\",\"wire_name\":\"session\",\"label\":\"Sessions\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Session\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Session\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Application.profile\",\"wire_name\":\"profile\",\"label\":\"profile\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":false,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"profileId\",\"foreign\":\"id\",\"inverse\":null},\"lookup\":null}}]").map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
}
/// Render the explicitly selected field controls for one caller-owned instance.
pub fn field(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let mut controls = Vec::new();
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
            view! { <stackpress_framework_api::views::roles::RelationField context/> }.into_any(),
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
            view! { <stackpress_framework_admin::components::TextListField context/> }.into_any(),
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
            view! { <stackpress_framework_admin::components::TemporalField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected filter controls for one caller-owned instance.
pub fn filter(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let mut controls = Vec::new();
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
            id: format!("{}-filter-{}", instance, "profileId"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_api::views::roles::RelationField context/> }.into_any(),
        );
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "active")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("active").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-filter-{}", instance, "active"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_api::views::roles::SwitchField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected list controls for one caller-owned instance.
pub fn list(
    value: &super::super::contracts::public::PublicApplication,
    instance: &str,
) -> SchemaResult<AnyView> {
    let seed = stackpress_framework_schema::to_bounded_value(value)?;
    let fields = descriptors()?;
    let draft = Draft::new(&fields, &seed, Role::List)?;
    let mut controls = Vec::new();
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
            id: format!("{}-list-{}", instance, "id"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "profileId"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::TemplateView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "name"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "logo"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::ImageView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "website"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::LinkView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "secret")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("secret").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "secret"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "scopes"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::TagsView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "expires"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "created")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("created").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "created"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "updated")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("updated").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "updated"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected view controls for one caller-owned instance.
pub fn view(
    value: &super::super::contracts::public::PublicApplication,
    instance: &str,
) -> SchemaResult<AnyView> {
    let seed = stackpress_framework_schema::to_bounded_value(value)?;
    let fields = descriptors()?;
    let draft = Draft::new(&fields, &seed, Role::View)?;
    let mut controls = Vec::new();
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
            id: format!("{}-view-{}", instance, "id"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "profileId"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::TemplateView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "name"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "logo"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::ImageView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "website"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::LinkView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "secret")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("secret").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "secret"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "scopes"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::TagsView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "active")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("active").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "active"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::BooleanView context/></div> }.into_any());
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
            id: format!("{}-view-{}", instance, "expires"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "created")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("created").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "created"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "updated")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("updated").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "updated"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_api::views::roles::DateView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
/// Render at most 100 public rows through selected list adapters.
/// Caller-supplied unique nonempty keys (at most 128 bytes) retain row identity.
/// Invalid row identities fail before constructing any controls.
pub fn table(
    rows: &[(String, super::super::contracts::public::PublicApplication)],
    instance: &str,
) -> SchemaResult<AnyView> {
    use stackpress_leptos::components::{
        Table, TableCell, TableGroup, TableHead, TableHeader, TableRow,
    };
    let mut keys = std::collections::BTreeSet::new();
    if rows.len() > 100
        || rows
            .iter()
            .any(|(key, _)| key.is_empty() || key.len() > 128 || !keys.insert(key))
    {
        return Err(SchemaError::new(
            SchemaErrorKind::Invalid,
            "invalid public table row identities",
        ));
    }
    let fields = descriptors()?;
    let mut rendered = Vec::new();
    for (key, value) in rows {
        let seed = stackpress_framework_schema::to_bounded_value(value)?;
        let draft = Draft::new(&fields, &seed, Role::List)?;
        // Hex encoding is injective and cannot collide with role/field delimiters.
        let identity: String = key
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect();
        let instance = format!("{instance}-row-{identity}");
        let mut controls = Vec::new();
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
                id: format!("{}-list-{}", instance, "id"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "profileId"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::TemplateView context/> }
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
                id: format!("{}-list-{}", instance, "name"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "logo"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::ImageView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "website"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::LinkView context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "secret")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("secret").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-list-{}", instance, "secret"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "scopes"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::TagsView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "expires"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::DateView context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "created")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("created").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-list-{}", instance, "created"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::DateView context/> }.into_any(),
            );
        }
        {
            // This exact field was approved by the app before normalization.
            let field = fields
                .iter()
                .find(|field| field.wire_name == "updated")
                .cloned()
                .ok_or_else(|| {
                    SchemaError::new(
                        SchemaErrorKind::Invalid,
                        "missing generated field descriptor",
                    )
                })?;
            let value = draft.get("updated").ok_or_else(|| {
                SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
            })?;
            let context = RoleContext {
                id: format!("{}-list-{}", instance, "updated"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_api::views::roles::DateView context/> }.into_any(),
            );
        }
        let cells = controls
            .into_iter()
            .map(|control| view! { <TableCell>{control}</TableCell> })
            .collect_view();
        rendered.push(view! { <TableRow row_key=key.clone()>{cells}</TableRow> });
    }
    let headers = vec![
        view! { <TableHeader>{"ID"}</TableHeader> },
        view! { <TableHeader>{"Profile"}</TableHeader> },
        view! { <TableHeader>{"Name"}</TableHeader> },
        view! { <TableHeader>{"Image"}</TableHeader> },
        view! { <TableHeader>{"Website"}</TableHeader> },
        view! { <TableHeader>{"Secret"}</TableHeader> },
        view! { <TableHeader>{"Scopes"}</TableHeader> },
        view! { <TableHeader>{"Expires"}</TableHeader> },
        view! { <TableHeader>{"Created"}</TableHeader> },
        view! { <TableHeader>{"Updated"}</TableHeader> },
    ]
    .collect_view();
    let rows = rendered.collect_view();
    Ok(view! { <Table caption="App"><TableHead><TableRow row_key="header">{headers}</TableRow></TableHead><TableGroup>{rows}</TableGroup></Table> }.into_any())
}
