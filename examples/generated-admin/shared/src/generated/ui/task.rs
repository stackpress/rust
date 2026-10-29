//! Generated public role wrappers; handwritten adapters remain outside this partition.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str("[{\"id\":\"Task.project\",\"wire_name\":\"project\",\"label\":\"project\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Project\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Project\",\"many\":false,\"mapping\":{\"local\":\"projectId\",\"foreign\":\"id\",\"inverse\":\"tasks\"},\"lookup\":null}},{\"id\":\"Task.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.projectId\",\"wire_name\":\"projectId\",\"label\":\"projectId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.done\",\"wire_name\":\"done\",\"label\":\"done\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.checkbox\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.note\",\"wire_name\":\"note\",\"label\":\"note\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.textarea\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Task.priority\",\"wire_name\":\"priority\",\"label\":\"priority\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.slider\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.slider\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}]").map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
}
/// Render the explicitly selected field controls for one caller-owned instance.
pub fn field(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
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
            id: format!("{}-field-{}", instance, "id"),
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
            view! { <stackpress_framework_admin::components::CheckboxField context/> }.into_any(),
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
            view! { <stackpress_framework_admin::components::TextareaField context/> }.into_any(),
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
            id: format!("{}-filter-{}", instance, "projectId"),
            field,
            role: Role::Filter,
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
            id: format!("{}-filter-{}", instance, "title"),
            field,
            role: Role::Filter,
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
            id: format!("{}-filter-{}", instance, "done"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::CheckboxField context/> }.into_any(),
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
            id: format!("{}-filter-{}", instance, "note"),
            field,
            role: Role::Filter,
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
            id: format!("{}-filter-{}", instance, "priority"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::SliderField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected list controls for one caller-owned instance.
pub fn list(
    value: &super::super::contracts::public::PublicTask,
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
            id: format!("{}-list-{}", instance, "title"),
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
            id: format!("{}-list-{}", instance, "done"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::BooleanView context/></div> }.into_any());
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
            id: format!("{}-list-{}", instance, "note"),
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
            id: format!("{}-list-{}", instance, "priority"),
            field,
            role: Role::List,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected span controls for one caller-owned instance.
pub fn span(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let mut controls = Vec::new();
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
            id: format!("{}-span-{}", instance, "title"),
            field,
            role: Role::Span,
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
            id: format!("{}-span-{}", instance, "done"),
            field,
            role: Role::Span,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::SwitchField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected view controls for one caller-owned instance.
pub fn view(
    value: &super::super::contracts::public::PublicTask,
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
            id: format!("{}-view-{}", instance, "title"),
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
            id: format!("{}-view-{}", instance, "done"),
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
            id: format!("{}-view-{}", instance, "note"),
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
            id: format!("{}-view-{}", instance, "priority"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
/// Render at most 100 public rows through selected list adapters.
/// Caller-supplied unique nonempty keys (at most 128 bytes) retain row identity.
/// Invalid row identities fail before constructing any controls.
pub fn table(
    rows: &[(String, super::super::contracts::public::PublicTask)],
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
                id: format!("{}-list-{}", instance, "title"),
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
                id: format!("{}-list-{}", instance, "done"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::BooleanView context/> }.into_any(),
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
                id: format!("{}-list-{}", instance, "note"),
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
                id: format!("{}-list-{}", instance, "priority"),
                field,
                role: Role::List,
                value,
            };
            controls.push(
                view! { <stackpress_framework_admin::components::TextView context/> }.into_any(),
            );
        }
        let cells = controls
            .into_iter()
            .map(|control| view! { <TableCell>{control}</TableCell> })
            .collect_view();
        rendered.push(view! { <TableRow row_key=key.clone()>{cells}</TableRow> });
    }
    let headers = vec![
        view! { <TableHeader>{"title"}</TableHeader> },
        view! { <TableHeader>{"done"}</TableHeader> },
        view! { <TableHeader>{"note"}</TableHeader> },
        view! { <TableHeader>{"priority"}</TableHeader> },
    ]
    .collect_view();
    let rows = rendered.collect_view();
    Ok(view! { <Table caption="Task"><TableHead><TableRow row_key="header">{headers}</TableRow></TableHead><TableGroup>{rows}</TableGroup></Table> }.into_any())
}
