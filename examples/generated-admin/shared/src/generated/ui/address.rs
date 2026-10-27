//! Generated public role wrappers; handwritten adapters remain outside this partition.
use leptos::prelude::*;
use stackpress_framework_admin::{Draft, RoleContext};
use stackpress_framework_schema::{Role, SchemaError, SchemaErrorKind, SchemaResult, UiField};
/// Decode only approved field descriptors; imports and source positions stay tool-only.
pub fn descriptors() -> SchemaResult<Vec<UiField>> {
    serde_json::from_str("[{\"id\":\"Address.city\",\"wire_name\":\"city\",\"label\":\"city\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[{\"code\":\"ne\",\"args\":[],\"client_checkable\":true,\"message\":\"field validation failed\",\"definition_revision\":\"1\"}],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null},{\"id\":\"Address.country\",\"wire_name\":\"country\",\"label\":\"country\",\"value_type\":{\"value\":{\"kind\":\"String\"},\"multiple\":false,\"choices\":[]},\"required\":true,\"input_required\":true,\"readable\":true,\"writable\":true,\"sensitive\":false,\"nullable\":false,\"default\":\"None\",\"validation\":[],\"filterable\":false,\"searchable\":false,\"sortable\":false,\"roles\":[{\"role\":\"field\",\"component\":\"stackpress.ui.field.input\",\"props\":{},\"definition_revision\":\"1\",\"available\":true},{\"role\":\"view\",\"component\":\"stackpress.ui.view.text\",\"props\":{},\"definition_revision\":\"1\",\"available\":true}],\"relation\":null}]").map_err(|cause| SchemaError::new(SchemaErrorKind::Codec, "public role descriptor decoding failed").with_cause(cause))
}
/// Render the explicitly selected field controls for one caller-owned instance.
pub fn field(draft: &Draft, instance: &str) -> SchemaResult<AnyView> {
    let fields = descriptors()?;
    let mut controls = Vec::new();
    {
        // This exact field was approved by the app before normalization.
        let field = fields
            .iter()
            .find(|field| field.wire_name == "city")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("city").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-field-{}", instance, "city"),
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
            .find(|field| field.wire_name == "country")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("country").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-field-{}", instance, "country"),
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
/// Render the explicitly selected view controls for one caller-owned instance.
pub fn view(
    value: &super::super::contracts::public::PublicAddress,
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
            .find(|field| field.wire_name == "city")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("city").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "city"),
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
            .find(|field| field.wire_name == "country")
            .cloned()
            .ok_or_else(|| {
                SchemaError::new(
                    SchemaErrorKind::Invalid,
                    "missing generated field descriptor",
                )
            })?;
        let value = draft.get("country").ok_or_else(|| {
            SchemaError::new(SchemaErrorKind::Invalid, "missing generated draft field")
        })?;
        let context = RoleContext {
            id: format!("{}-view-{}", instance, "country"),
            field,
            role: Role::View,
            value,
        };
        let label = context.text_prop("label", &context.field.label);
        controls.push(view! { <div class="sp-role-value"><span class="sp-role-label">{label} ": "</span><stackpress_framework_admin::components::TextView context/></div> }.into_any());
    }
    Ok(controls.collect_view().into_any())
}
