//! Native keyboard mask adapter; pattern parsing stays in the pure mask module.
use super::{SourceAppearance, source_mask_value};
use leptos::prelude::*;

/// G55 common-pattern mask input; incomplete required slots use Inputmask's underscore spelling.
#[component]
pub fn SourceMaskInput(
    /// Native input identity.
    #[prop(into)]
    id: String,
    /// Native successful form key.
    #[prop(into)]
    name: String,
    /// Original mask string. See source_mask for the supported grammar.
    #[prop(into)]
    mask: String,
    /// Caller-owned text, retaining masked form values.
    value: RwSignal<String>,
    /// Original onUpdate callback.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Native requirement.
    #[prop(optional)]
    required: bool,
    /// Native appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let mask = StoredValue::new(mask);
    let focused = RwSignal::new(false);
    let node = NodeRef::<leptos::html::Input>::new();
    let initial = source_mask_value(&mask.get_value(), &value.get_untracked(), false);
    value.set(initial);
    view! {<input type="text" node_ref=node id=id name=name class=format!("sp-input {}",appearance.class_name) style=appearance.style required=required value=move||value.get() prop:value=move||value.get()
    on:focus=move|_|{focused.set(true);value.update(|value|*value=source_mask_value(&mask.get_value(),value,true));}
    on:blur=move|_|{focused.set(false);value.update(|value|{if !value.chars().any(char::is_alphanumeric){value.clear();}});}
    on:keydown=move|event:leptos::ev::KeyboardEvent|{
        #[cfg(target_arch="wasm32")]
        if !event.is_composing()&&matches!(event.key().as_str(),"Backspace"|"Delete")&&let Some(node)=node.get(){
            let start=node.selection_start().ok().flatten().unwrap_or(0) as usize;let end=node.selection_end().ok().flatten().unwrap_or(start as u32) as usize;
            if let Some((text,cursor))=super::mask::source_mask_delete(&mask.get_value(),&node.value(),start,end,event.key()=="Backspace"){
                event.prevent_default();node.set_value(&text);value.set(text.clone());let _=node.set_selection_range(cursor as u32,cursor as u32);if let Some(callback)=on_update{callback.run(text);}
            }
        }
        #[cfg(not(target_arch="wasm32"))] let _=event;
    }
    on:input=move|event|{let text=source_mask_value(&mask.get_value(),&event_target_value(&event),focused.get());value.set(text.clone());if let Some(callback)=on_update{callback.run(text);}}/ >}
}
