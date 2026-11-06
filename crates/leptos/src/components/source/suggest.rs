//! Source suggestions reuse the shared native popover while allowing arbitrary typed values.
use super::SourceAppearance;
use crate::components::{Input, Popover, PopupRole};
use leptos::prelude::*;

/// Source SuggestInput never filters its authored options or fetches its remote attribute.
#[component]
pub fn SourceSuggestInput(
    /// Native text input ID.
    #[prop(into)]
    id: String,
    /// Successful native text form key.
    #[prop(into)]
    name: String,
    /// Accessible popup name.
    #[prop(into)]
    label: String,
    /// Root-owned text, including values outside the options.
    value: RwSignal<String>,
    /// Options in authored order; no local filtering is imposed.
    #[prop(optional)]
    options: Vec<String>,
    /// Frui opens/query-calls only at this UTF-16 length; zero opens on every input.
    #[prop(optional)]
    chars: usize,
    /// Application-owned query callback; the component adds no transport.
    #[prop(optional)]
    on_query: Option<Callback<String>>,
    /// Text/selection callback.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Original remote string is forwarded as an ordinary native attribute.
    #[prop(optional, into)]
    remote: String,
    /// Native field requirement.
    #[prop(optional)]
    required: bool,
    /// Root appearance, matching the source Dropdown container.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let open = RwSignal::new(false);
    let active = RwSignal::new(None::<usize>);
    let popup = format!("{id}-suggestions");
    let option_prefix = popup.clone();
    let options = StoredValue::new(options);
    let remote = leptos::tachys::html::attribute::custom::custom_attribute("remote", remote);
    let publish = Callback::new(move |text: String| {
        value.set(text.clone());
        if let Some(callback) = on_update {
            callback.run(text);
        }
    });
    let choose = Callback::new(move |index: usize| {
        if let Some(option) = options.with_value(|options| options.get(index).cloned()) {
            publish.run(option);
            open.set(false);
        }
    });
    view! {<div class=format!("sp-source-suggest {}",appearance.class_name) style=appearance.style>
        <Input id=id.clone() name value required=required
            attr:role="combobox" attr:aria-autocomplete="list" attr:aria-controls=popup.clone() attr:aria-expanded=move||open.get().to_string() {..remote}
            attr:aria-activedescendant={let popup=popup.clone();move||active.get().map(|index|format!("{popup}-{index}"))}
            on:input=move|event|{let text=event_target_value(&event);publish.run(text.clone());let ready=chars==0||text.encode_utf16().count()>=chars;open.set(ready);active.set(None);if ready&&let Some(callback)=on_query{callback.run(text);}}
            on:blur=move|_|open.set(false)
            on:keydown=move|event:leptos::ev::KeyboardEvent|{
                let count=options.with_value(Vec::len);
                match event.key().as_str(){
                    "ArrowDown"|"ArrowUp" if count>0=>{event.prevent_default();open.set(true);let index=active.get_untracked();active.set(Some(if event.key()=="ArrowDown"{index.map_or(0,|index|(index+1)%count)}else{index.map_or(count-1,|index|(index+count-1)%count)}));},
                    "Enter" if open.get_untracked()&&active.get_untracked().is_some()=>{event.prevent_default();choose.run(active.get_untracked().unwrap_or(0));},
                    "Escape"=>open.set(false),_=>{}
                }
            }/>
        <Popover id=popup anchor=id label open role=PopupRole::Listbox>
            {options.get_value().into_iter().enumerate().map(move|(index,option)|view!{<button type="button" role="option" id=format!("{option_prefix}-{index}") tabindex="-1" aria-selected=move||(active.get()==Some(index)).to_string()
                on:mousedown=move|event|event.prevent_default() on:click=move|_|choose.run(index)>{option}</button>}).collect_view()}
        </Popover>
    </div>}
}
