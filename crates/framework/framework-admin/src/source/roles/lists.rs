//! Source list carriers use shared source controls rather than Typed collection budgets.
use super::props::appearance;
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::source::SourceTextList;

/// Shared source TextList behavior for all five type aliases.
#[component]
pub fn SourceTextListField(
    /// Original list values, placeholder and Add label.
    context: RoleContext,
) -> impl IntoView {
    let initial = match context.value.get_untracked() {
        Presence::Value(Value::Array(values)) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    None
                } else {
                    Some(super::metadata_string(value))
                }
            })
            .collect(),
        _ => Vec::new(),
    };
    let value = context.value;
    view! {<SourceTextList id=context.id.clone() name=super::props::form_name(&context) initial add=context.text_prop("add","") placeholder=context.text_prop("placeholder","") appearance=appearance(&context)
    on_update=Callback::new(move|values:Vec<String>|value.set(Presence::Value(Value::Array(values.into_iter().map(Value::String).collect()))))/ >}
}
