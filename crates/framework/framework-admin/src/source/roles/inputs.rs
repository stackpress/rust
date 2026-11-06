//! Source input adapters retain metadata in the admin layer and shared UI behavior in Leptos.
use super::props::{appearance, content, prop};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::FieldControl;
use stackpress_leptos::components::source::{SourceFileInput, SourceSmallInput, SourceUpload};

/// Optional root-local upload hook supplied by an application for generated file/image fields.
#[derive(Clone)]
pub struct SourceUploader(pub Callback<SourceUpload>);

/// Source file/image role behavior, including the caller-owned upload boundary.
#[component]
pub fn FileField(
    /// Selected file/image/list role and mount-only URL seed.
    context: RoleContext,
) -> impl IntoView {
    let binding = context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role);
    let name = binding
        .map(|binding| binding.component.as_str())
        .unwrap_or("");
    let multiple = name.ends_with("filelist") || name.ends_with("imagelist");
    let image = name.ends_with("image") || name.ends_with("imagelist");
    let initial = match context.value.get_untracked() {
        Presence::Value(Value::Array(values)) => {
            values.iter().map(super::metadata_string).collect()
        }
        Presence::Value(Value::String(value)) if !value.is_empty() => vec![value],
        _ => Vec::new(),
    };
    let draft = context.value;
    let uploader = use_context::<SourceUploader>().map(|uploader| uploader.0);
    view! {<FieldControl for_id=context.id.clone() label=context.field.label.clone()>
        <SourceFileInput id=context.id.clone() name=super::props::form_name(&context) initial multiple image on_upload=uploader
            uploading=context.text_prop("uploading","Uploading...") required=context.input_required() appearance=appearance(&context)
            on_update=Callback::new(move|urls:Vec<String>|draft.set(Presence::Value(if multiple{Value::Array(urls.into_iter().map(Value::String).collect())}else{urls.first().cloned().map(Value::String).unwrap_or(Value::Null)})))/>
    </FieldControl>}
}

/// Fixed 0..9 integer control through Frui's live small-number behavior.
#[component]
pub fn SmallField(
    /// Caller-owned numeric draft and source appearance.
    context: RoleContext,
) -> impl IntoView {
    let value = context.value;
    let source = context.clone();
    view! {<FieldControl for_id=context.id.clone() label=context.field.label.clone()>
        <SourceSmallInput id=context.id.clone() name=super::props::form_name(&context) value=Signal::derive(move||content(&source))
            required=context.input_required() appearance=appearance(&context) on_update=Callback::new(move|number:f64|{
                value.set(Presence::Value(serde_json::Number::from_f64(number).map(Value::Number).unwrap_or(Value::Null)));
            })/>
    </FieldControl>}
}

/// Native shared radio with Frui's authored checked/defaultChecked, label and visual flags.
#[component]
pub fn RadioField(
    /// Caller-owned source value and presentation metadata.
    context: RoleContext,
) -> impl IntoView {
    let value = content(&context);
    let checked = prop(&context, "checked")
        .or_else(|| prop(&context, "defaultChecked"))
        .is_some_and(super::editor::source_truthy);
    let selected = RwSignal::new(checked.then(|| value.clone()));
    let draft = context.value;
    view! {<stackpress_leptos::components::source::SourceRadio id=context.id.clone() name=super::props::form_name(&context) value selected
    label=context.text_prop("label","") aria_label=context.field.label.clone() check=context.bool_prop("check",false) square=context.bool_prop("square",false) rounded=context.bool_prop("rounded",false) blue=context.bool_prop("blue",false)
    orange=prop(&context,"orange").is_some_and(super::editor::source_truthy) required=context.input_required() appearance=appearance(&context)
    on_update=Callback::new(move|(value,_checked):(String,bool)|draft.set(Presence::Value(Value::String(value))))/ >}
}

/// Source suggestions keep authored local options; remote is a passthrough, not fetch.
#[component]
pub fn SuggestField(
    /// Source draft, options and ordinary appearance.
    context: RoleContext,
) -> impl IntoView {
    let initial = content(&context);
    let value = RwSignal::new(initial);
    let draft = context.value;
    let options = prop(&context, "options")
        .and_then(Value::as_array)
        .map(|values| values.iter().map(super::metadata_string).collect())
        .unwrap_or_default();
    view! {<FieldControl for_id=context.id.clone() label=context.field.label.clone()>
        <stackpress_leptos::components::source::SourceSuggestInput id=context.id.clone() name=super::props::form_name(&context) label=context.field.label.clone() value options remote=context.text_prop("remote","") required=context.input_required() appearance=appearance(&context)
            on_update=Callback::new(move|text:String|draft.set(Presence::Value(Value::String(text))))/>
    </FieldControl>}
}
