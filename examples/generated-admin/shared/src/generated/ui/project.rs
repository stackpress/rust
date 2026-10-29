//! Generated public role wrappers; handwritten adapters remain outside this partition.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str("[{\"id\":\"Project.owner\",\"wire_name\":\"owner\",\"label\":\"owner\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Profile\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Profile\",\"many\":false,\"mapping\":{\"local\":\"ownerId\",\"foreign\":\"id\",\"inverse\":\"projects\"},\"lookup\":null}},{\"id\":\"Project.tasks\",\"wire_name\":\"tasks\",\"label\":\"tasks\",\"value_type\":{\"value\":{\"kind\":\"Model\",\"name\":\"Task\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":{\"target\":\"Task\",\"many\":true,\"mapping\":null,\"lookup\":null}},{\"id\":\"Project.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.ownerId\",\"wire_name\":\"ownerId\",\"label\":\"ownerId\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.title\",\"wire_name\":\"title\",\"label\":\"title\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":true,\"searchable\":true,\"sortable\":true,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.state\",\"wire_name\":\"state\",\"label\":\"state\",\"value_type\":{\"value\":{\"kind\":\"Enum\",\"name\":\"ProjectState\"},\"multiple\":false,\"choices\":[\"open\",\"closed\"]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":\"open\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.select\",\"props\":{\"searchable\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Project.tags\",\"wire_name\":\"tags\",\"label\":\"tags\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":true,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":[]},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.tags\",\"props\":{\"tags\":true},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}]").map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
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
            view! { <stackpress_framework_admin::components::TextListField context/> }.into_any(),
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
            id: format!("{}-filter-{}", instance, "ownerId"),
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
            id: format!("{}-filter-{}", instance, "state"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::SelectField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected list controls for one caller-owned instance.
pub fn list(
    value: &super::super::contracts::public::PublicProject,
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
            id: format!("{}-list-{}", instance, "state"),
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
            id: format!("{}-list-{}", instance, "tags"),
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
            id: format!("{}-span-{}", instance, "tags"),
            field,
            role: Role::Span,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::TextListField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected view controls for one caller-owned instance.
pub fn view(
    value: &super::super::contracts::public::PublicProject,
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
    Ok(controls.collect_view().into_any())
}
/// Render at most 100 public rows through selected list adapters.
/// Caller-supplied unique nonempty keys (at most 128 bytes) retain row identity.
/// Invalid row identities fail before constructing any controls.
pub fn table(
    rows: &[(String, super::super::contracts::public::PublicProject)],
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
                id: format!("{}-list-{}", instance, "state"),
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
                id: format!("{}-list-{}", instance, "tags"),
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
        view! { <TableHeader>{"state"}</TableHeader> },
        view! { <TableHeader>{"tags"}</TableHeader> },
    ]
    .collect_view();
    let rows = rendered.collect_view();
    Ok(view! { <Table caption="Project"><TableHead><TableRow row_key="header">{headers}</TableRow></TableHead><TableGroup>{rows}</TableGroup></Table> }.into_any())
}
