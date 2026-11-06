//! Source-select presentation uses the existing popover, without Typed choice limits.
use super::SourceAppearance;
use crate::components::{
    Popover, PopupRole,
    icons::{IconKind, icon},
};
use leptos::prelude::*;

/// One unbounded source option, retaining source order and independent search fields.
#[derive(Clone, Debug)]
pub struct SourceChoice {
    /// Submitted source identity.
    pub value: String,
    /// Escaped display text.
    pub label: String,
    /// Independent original fields; matching never crosses a field boundary.
    pub search: Vec<String>,
}

/// Native keyboard-selectable source popup; shared by country and currency adapters.
#[component]
pub fn SourceSelect(
    /// Trigger and option identity prefix.
    #[prop(into)]
    id: String,
    /// Hidden successful form key.
    #[prop(into)]
    name: String,
    /// Original selector value, including an unknown code.
    #[prop(optional_no_strip)]
    initial: Option<String>,
    /// Source options, in original order.
    choices: Vec<SourceChoice>,
    /// Include the source keyword input.
    #[prop(optional)]
    searchable: bool,
    /// Search control hint.
    #[prop(default="Search...".into(),into)]
    search_placeholder: String,
    /// Empty selection text.
    #[prop(optional, into)]
    placeholder: String,
    /// Selected identity callback; callers map it to the original data record.
    #[prop(optional)]
    on_update: Option<Callback<String>>,
    /// Root appearance.
    #[prop(optional)]
    appearance: SourceAppearance,
    /// Trigger appearance.
    #[prop(optional)]
    display: SourceAppearance,
    /// Popup appearance.
    #[prop(optional)]
    dropdown: SourceAppearance,
    /// Option appearance.
    #[prop(optional)]
    option: SourceAppearance,
) -> impl IntoView {
    let selected = RwSignal::new(initial);
    let keyword = RwSignal::new(String::new());
    let open = RwSignal::new(false);
    let active = RwSignal::new(0usize);
    let choices = StoredValue::new(choices);
    let placeholder = StoredValue::new(placeholder);
    let popup = format!("{id}-options");
    let matching = move || {
        let query = keyword.get().to_lowercase().trim().to_owned();
        choices.with_value(|choices| {
            choices
                .iter()
                .filter(|choice| {
                    choice
                        .search
                        .iter()
                        .any(|field| field.to_lowercase().contains(&query))
                })
                .cloned()
                .collect::<Vec<_>>()
        })
    };
    let choose = Callback::new(move |choice: String| {
        selected.set(Some(choice.clone()));
        open.set(false);
        if let Some(callback) = on_update {
            callback.run(choice);
        }
    });
    let navigation = Callback::new(move |event: leptos::ev::KeyboardEvent| {
        let choices = matching();
        let count = choices.len();
        match event.key().as_str() {
            "ArrowDown" | "ArrowUp" if count > 0 => {
                event.prevent_default();
                if !open.get_untracked() {
                    open.set(true);
                    active.set(0);
                } else {
                    active.set(if event.key() == "ArrowDown" {
                        (active.get_untracked() + 1) % count
                    } else {
                        (active.get_untracked() + count - 1) % count
                    });
                }
            }
            "Enter" if open.get_untracked() => {
                if let Some(choice) = choices.get(active.get_untracked()) {
                    event.prevent_default();
                    choose.run(choice.value.clone());
                }
            }
            "Escape" => open.set(false),
            _ => {}
        }
    });
    view! {<div class=format!("sp-source-select {}",appearance.class_name) style=appearance.style>
        <button type="button" id=id.clone() class=display.class_name style=display.style aria-haspopup="listbox" aria-controls=popup.clone() aria-expanded=move||open.get().to_string()
            on:click=move|_|open.update(|open|*open = !*open) on:keydown=move|event|navigation.run(event)>
            {move||choices.with_value(|choices|choices.iter().find(|choice|selected.get().as_deref()==Some(choice.value.as_str())).map(|choice|choice.label.clone())).unwrap_or_else(||placeholder.get_value())}
        </button>
        <Show when=move||selected.with(Option::is_some)>
            <button type="button" class="sp-icon-action" aria-label="Clear selection" on:click=move|_|{selected.set(None);open.set(false);}>{icon(IconKind::X)}</button>
            <input type="hidden" name=name.clone() value=move||selected.get().unwrap_or_default() prop:value=move||selected.get().unwrap_or_default()/>
        </Show>
        <Popover id=popup anchor=id label="Choices" open role=PopupRole::Listbox>
            <div class=dropdown.class_name style=dropdown.style>
                {searchable.then(||view!{<input class="sp-input" type="text" aria-label="Search choices" placeholder=search_placeholder value=move||keyword.get() prop:value=move||keyword.get()
                    on:input=move|event|{keyword.set(event_target_value(&event));active.set(0);} on:keydown=move|event|navigation.run(event)/ >})}
                {move||matching().into_iter().enumerate().map({let option=option.clone();move|(index,choice)|{let selected_code=choice.value.clone();let code=choice.value;
                    view!{<button type="button" role="option" class=option.class_name.clone() style=option.style.clone() aria-selected=move||(selected.get().as_deref()==Some(selected_code.as_str())).to_string() data-active=move||(active.get()==index).to_string()
                        on:click=move|_|choose.run(code.clone())>{choice.label}</button>}
                }}).collect_view()}
            </div>
        </Popover>
    </div>}
}
