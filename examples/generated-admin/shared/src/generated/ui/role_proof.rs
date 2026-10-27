//! Generated public role wrappers; handwritten adapters remain outside this partition.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str("[{\"id\":\"RoleProof.id\",\"wire_name\":\"id\",\"label\":\"id\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[],\"relation\":null},{\"id\":\"RoleProof.text\",\"wire_name\":\"text\",\"label\":\"text\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":false,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":true,\"default\":{\"Literal\":\"fixture default\"},\"validation\":[],\"filterable\":true,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"filter\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"span\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.flag\",\"wire_name\":\"flag\",\"label\":\"flag\",\"value_type\":{\"value\":{\"kind\":\"Boolean\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":false},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.switch\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"list\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.yesno\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"RoleProof.zero\",\"wire_name\":\"zero\",\"label\":\"zero\",\"value_type\":{\"value\":{\"kind\":\"Integer\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":false,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":{\"Literal\":0},\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"list\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}]").map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
}
/// Render the explicitly selected field controls for one caller-owned instance.
pub fn field(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let mut controls = Vec::new();
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "text")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("text").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-field-{}", instance, "text"),
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
            .find(|field| field.wire_name == "flag")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("flag").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-field-{}", instance, "flag"),
            field,
            role: Role::Field,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::SwitchField context/> }.into_any(),
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
            .find(|field| field.wire_name == "text")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("text").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-filter-{}", instance, "text"),
            field,
            role: Role::Filter,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected list controls for one caller-owned instance.
pub fn list(
    value: &super::super::contracts::public::PublicRoleProof,
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
            .find(|field| field.wire_name == "text")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("text").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "text"),
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
            .find(|field| field.wire_name == "flag")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("flag").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "flag"),
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
            .find(|field| field.wire_name == "zero")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("zero").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-list-{}", instance, "zero"),
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
            .find(|field| field.wire_name == "text")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("text").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-span-{}", instance, "text"),
            field,
            role: Role::Span,
            value,
        };
        controls.push(
            view! { <stackpress_framework_admin::components::TextField context/> }.into_any(),
        );
    }
    Ok(controls.collect_view().into_any())
}
/// Render the explicitly selected view controls for one caller-owned instance.
pub fn view(
    value: &super::super::contracts::public::PublicRoleProof,
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
            .find(|field| field.wire_name == "text")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("text").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "text"),
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
            .find(|field| field.wire_name == "flag")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("flag").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "flag"),
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
            .find(|field| field.wire_name == "zero")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("zero").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "zero"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
