//! Controlled disclosures and explicit tab/panel slots; no child-tree reflection.
use super::{ChoiceState, DisclosureState, choice::choice_movement};
use leptos::prelude::*;

/// A named group of caller-supplied AccordionItem slots.
#[component]
pub fn Accordion(
    /// Accessible group name.
    #[prop(into)]
    label: String,
    /// Explicit item slots sharing root-owned state.
    children: Children,
) -> impl IntoView {
    view! {<section class="sp-accordion" aria-label=label>{children()}</section>}
}
/// One controlled disclosure; unknown keys render disabled and cannot publish state.
#[component]
pub fn AccordionItem(
    /// Unique DOM prefix.
    #[prop(into)]
    id: String,
    /// Exact immutable option key.
    #[prop(into)]
    item: String,
    /// Application-owned group selection.
    state: DisclosureState,
    /// Explicit panel content, retained while collapsed.
    children: Children,
) -> impl IntoView {
    let option = state
        .options()
        .entries()
        .iter()
        .find(|o| o.value == item)
        .cloned();
    let disabled = option.as_ref().is_none_or(|o| o.disabled);
    let label = option.map(|o| o.label).unwrap_or_default();
    let panel = format!("{id}-panel");
    let button = format!("{id}-button");
    let expanded = item.clone();
    let hidden = item.clone();
    view! {<div class="sp-disclosure"><h3><button id=button.clone() type="button" aria-controls=panel.clone() disabled=disabled
    aria-expanded=move ||state.is_open(&expanded).to_string() on:click=move |_|{state.toggle(&item);}>{label}</button></h3>
    <section id=panel aria-labelledby=button hidden=move ||!state.is_open(&hidden)>{children()}</section></div>}
}
/// Native disclosure, including browser keyboard/summary behavior before hydration.
#[component]
pub fn Details(
    /// Stable native identity.
    #[prop(into)]
    id: String,
    /// Native summary label.
    #[prop(into)]
    label: String,
    /// Root-owned state; host may adopt an early native toggle.
    open: RwSignal<bool>,
    /// Retained disclosed content.
    children: Children,
) -> impl IntoView {
    view! {<details id=id class="sp-disclosure" open=move ||open.get() prop:open=move ||open.get()
    on:toggle=move |_event|{#[cfg(target_arch="wasm32")] {use wasm_bindgen::JsCast;if let Some(node)=_event.current_target().and_then(|v|v.dyn_into::<web_sys::HtmlDetailsElement>().ok()){open.try_set(node.open());}}}>
    <summary>{label}</summary>{children()}</details>}
}
/// Tab selection follows focus only when automatic activation is selected.
#[derive(Clone, Copy, Debug, Default)]
pub enum TabActivation {
    /// Space/Enter or pointer click selects.
    #[default]
    Manual,
    /// Arrows select as they move focus.
    Automatic,
}
/// Accessible tab list with one roving stop and explicit orientation/activation.
#[component]
pub fn Tabs(
    /// Shared DOM prefix used by TabPanel.
    #[prop(into)]
    id: String,
    /// Accessible tab-list name.
    #[prop(into)]
    label: String,
    /// Root-owned bounded choice controller.
    state: ChoiceState,
    /// Keyboard selection policy.
    #[prop(optional)]
    activation: TabActivation,
    /// Up/down instead of left/right.
    #[prop(optional)]
    vertical: bool,
) -> impl IntoView {
    let options = state.options();
    view! {<div class="sp-tabs" role="tablist" aria-label=label aria-orientation=if vertical{"vertical"}else{"horizontal"}>
        {options.entries().iter().enumerate().map(|(index,option)|{
            let key=option.value.clone();let selected=key.clone();let prefix=id.clone();
            let tab=format!("{id}-tab-{index}");let panel=format!("{id}-panel-{index}");
            view!{<button id=tab type="button" role="tab" aria-controls=panel disabled=option.disabled
                tabindex=move ||if state.focused()==Some(index){0}else{-1}
                aria-selected=move ||(state.selected().as_deref()==Some(selected.as_str())).to_string()
                on:focus=move |_|{state.focus(index);}
                on:click=move |_|{state.choose(&key);}
                on:keydown=move |event:leptos::ev::KeyboardEvent|{
                    let direction=choice_movement(&event.key(), vertical);
                    if let Some(direction)=direction {event.prevent_default();if let Some(next)=state.navigate(direction,matches!(activation,TabActivation::Automatic)){
                        #[cfg(target_arch="wasm32")] super::dialog_dom::focus_id(&format!("{prefix}-tab-{next}"));
                        #[cfg(not(target_arch="wasm32"))] let _=(&prefix,next);
                    }}
                }>{option.label.clone()}</button>}
        }).collect_view()}
    </div>}
}
/// Explicit associated panel; inactive content stays mounted, hidden and unfocusable.
#[component]
pub fn TabPanel(
    /// Same DOM prefix as Tabs.
    #[prop(into)]
    id: String,
    /// Option index in immutable controller data.
    index: usize,
    /// Shared choice controller.
    state: ChoiceState,
    /// Arbitrary semantic panel content.
    children: Children,
) -> impl IntoView {
    let key = state
        .options()
        .entries()
        .get(index)
        .map(|o| o.value.clone());
    view! {<section id=format!("{id}-panel-{index}") role="tabpanel" tabindex="0" aria-labelledby=format!("{id}-tab-{index}")
    hidden=move ||key.as_ref().is_none_or(|key|state.selected().as_ref()!=Some(key))>{children()}</section>}
}
