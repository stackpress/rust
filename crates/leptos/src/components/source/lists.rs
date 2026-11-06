//! Frui's index-stable, tombstoned fieldset collections; there is no Typed row/byte budget.
use super::SourceAppearance;
use crate::components::icons::{IconKind, icon};
use leptos::prelude::*;

/// Original source fieldset row storage: removals leave holes and new rows append.
#[derive(Clone, Debug, PartialEq)]
pub struct SourceRows<T> {
    /// Original indices, retained until an external value replacement.
    pub rows: Vec<Option<T>>,
}
impl<T: Clone> SourceRows<T> {
    /// Clone the source clean callback value, omitting tombstones.
    pub fn clean(&self) -> Vec<T> {
        self.rows.iter().filter_map(Clone::clone).collect()
    }
    /// Append the exact emptyValue; existing row identities remain unchanged.
    pub fn add(&mut self, value: T) {
        self.rows.push(Some(value));
    }
    /// Tombstone a row without shifting subsequent names/controls.
    pub fn remove(&mut self, index: usize) {
        if let Some(row) = self.rows.get_mut(index) {
            *row = None;
        }
    }
    /// A user edit only changes a still-mounted row.
    pub fn edit(&mut self, index: usize, value: T) {
        if let Some(row @ Some(_)) = self.rows.get_mut(index) {
            *row = Some(value);
        }
    }
}

/// Shared dynamic fieldset factory used by text lists and generated nested fieldsets.
#[component]
pub fn SourceFieldset<T>(
    /// Root-local tombstoned rows.
    rows: RwSignal<SourceRows<T>>,
    /// Application row renderer; remove/add actions remain shared.
    render: Callback<usize, AnyView>,
    /// Exact emptyValue supplied by the authored fieldset factory.
    empty_value: T,
    /// Original optional count limit; zero means unlimited.
    #[prop(optional)]
    limit: usize,
    /// Original default action text.
    #[prop(default="Add".into(),into)]
    add: String,
    /// Authored attributes decorate the Add button, matching Frui Fieldset.make.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView
where
    T: Clone + Send + Sync + 'static,
{
    view!{<div class="sp-source-fieldset">
        <For each=move||rows.with(|rows|rows.rows.iter().enumerate().filter_map(|(index,row)|row.as_ref().map(|_|index)).collect::<Vec<_>>()) key=|index|*index children=move|index|render.run(index)/>
        <Show when=move||limit==0||rows.with(|rows|rows.rows.iter().filter(|row|row.is_some()).count())<limit>
            <button type="button" class=appearance.class_name.clone() style=appearance.style.clone() on:click={let empty_value=empty_value.clone();move|_|rows.update(|rows|rows.add(empty_value.clone()))}>{icon(IconKind::Plus)}{add.clone()}</button>
        </Show>
    </div>}.into_any()
}

/// Source TextList with required ordinary text controls, repeated hidden names and source callbacks.
/// The date/time/number metadata aliases still render text: upstream TextList ignores type.
#[component]
pub fn SourceTextList(
    /// Unique native identity prefix.
    #[prop(into)]
    id: String,
    /// Repeated hidden form key; visible fields remain unnamed.
    #[prop(into)]
    name: String,
    /// Initial source values; holes are retained while editing.
    #[prop(optional)]
    initial: Vec<Option<String>>,
    /// Source list callback, with holes removed.
    #[prop(optional)]
    on_change: Option<Callback<Vec<String>>>,
    /// Called after on_change for each changed list.
    #[prop(optional)]
    on_update: Option<Callback<Vec<String>>>,
    /// Original optional list count limit.
    #[prop(optional)]
    limit: usize,
    /// Original optional Add label; an empty string uses Add.
    #[prop(optional, into)]
    add: String,
    /// Source placeholder on every visible text control.
    #[prop(optional, into)]
    placeholder: String,
    /// Add button classes/styles, as in Frui Fieldset.make.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let rows = RwSignal::new(SourceRows { rows: initial });
    let previous = StoredValue::new(rows.get_untracked());
    Effect::new(move |_| {
        let next = rows.get();
        if next != previous.get_value() {
            previous.set_value(next.clone());
            let clean = next.clean();
            if let Some(callback) = on_change {
                callback.run(clean.clone());
            }
            if let Some(callback) = on_update {
                callback.run(clean);
            }
        }
    });
    let render = Callback::new(move |index: usize| {
        let value =
            move || rows.with(|rows| rows.rows.get(index).cloned().flatten().unwrap_or_default());
        view!{<div class="sp-source-text-row">
            <label class="sp-sr-only" for=format!("{id}-{index}")>{format!("Value {}",index+1)}</label>
            <input class="sp-input" type="text" id=format!("{id}-{index}") placeholder=placeholder.clone() required=true value=value prop:value=value
                on:input=move|event|rows.update(|rows|rows.edit(index,event_target_value(&event)))/>
            <button class="sp-icon-action" type="button" aria-label=format!("Remove row {}",index+1) on:click=move|_|rows.update(|rows|rows.remove(index))>{icon(IconKind::X)}</button>
            <input type="hidden" name=name.clone() value=value prop:value=value/>
        </div>}.into_any()
    });
    view! {<SourceFieldset rows render empty_value=String::new() limit add=if add.is_empty(){"Add".into()}else{add} appearance/>}
}

/// Shared source fieldset table for single-object label rows or multiple-object heading columns.
#[component]
pub fn SourceFieldsetDisplay(
    /// Authored child labels, in selected role order.
    labels: Vec<String>,
    /// Already selected shared child views; the source adapter owns value/presence semantics.
    rows: Vec<Vec<AnyView>>,
    /// Use column headings for an array, or label/value rows for a single object.
    #[prop(optional)]
    multiple: bool,
) -> impl IntoView {
    if multiple {
        if rows.is_empty() {
            return ().into_any();
        }
        view!{<table class="sp-source-table"><thead><tr>{labels.into_iter().map(|label|view!{<th>{label}</th>}).collect_view()}</tr></thead><tbody>{rows.into_iter().map(|row|view!{<tr>{row.into_iter().map(|cell|view!{<td>{cell}</td>}).collect_view()}</tr>}).collect_view()}</tbody></table>}.into_any()
    } else {
        let row = rows.into_iter().next().unwrap_or_default();
        view!{<table class="sp-source-table"><tbody>{labels.into_iter().zip(row).map(|(label,cell)|view!{<tr><th>{label}</th><td>{cell}</td></tr>}).collect_view()}</tbody></table>}.into_any()
    }
}
