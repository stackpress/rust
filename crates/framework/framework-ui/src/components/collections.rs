//! Explicit keyed fieldset slots and bounded editable text/metadata adapters.
use super::icons::{IconKind, icon};
use super::{BoundedList, CollectionValue, MetadataEntry};
use leptos::prelude::*;

/// Move one existing key to its adjacent admitted index; boundary requests are no-ops.
fn shift<T: CollectionValue>(list: &mut BoundedList<T>, key: u64, upward: bool) {
    let Some(index) = list.rows().iter().position(|row| row.id == key) else {
        return;
    };
    // Check the destination before moving, preserving both identity and neighbour order.
    let target = if upward {
        index.checked_sub(1)
    } else {
        index.checked_add(1)
    };
    if let Some(target) = target.filter(|target| *target < list.rows().len()) {
        let _ = list.reorder(key, target);
    }
}

/// Keyed fieldset with native add/remove/reorder actions; domain validation stays outside.
#[component]
pub fn CollectionFieldset<T>(
    /// Unique prefix for row and action IDs.
    #[prop(into)]
    id: String,
    /// Native legend text.
    #[prop(into)]
    label: String,
    /// Typed root-owned collection, not index-keyed values.
    list: RwSignal<BoundedList<T>>,
    /// Application factory for one empty/default typed row.
    create: Callback<(), T>,
    /// Render one stable identity with app-owned labels/errors.
    render: Callback<u64, AnyView>,
    /// Native disabled state, also enforced at event admission.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Named application validation errors.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView
where
    T: Clone + Send + Sync + CollectionValue + 'static,
{
    // Views depend only on identity: edits and reorder retain native inputs and their drafts.
    let rows = move || list.with(|list| list.rows().iter().map(|row| row.id).collect::<Vec<_>>());
    let prefix = id.clone();
    let relationship = format!("{id}-error");
    let add_id = format!("{id}-add");
    let added_prefix = id.clone();
    let pending_focus = RwSignal::new(None::<String>);
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(id) = pending_focus.get() {
            super::dialog_dom::focus_id(&id);
        }
    });
    // Keep this generic surface's concrete tuple local. Generated adapters otherwise
    // amplify its type depth when instantiating nested async hydration views.
    view! {
        <fieldset class="sp-collection" id=id disabled=move ||disabled.get() aria-describedby=relationship.clone()>
            <legend>{label}</legend><p id=relationship.clone() role="alert">{move ||error.get()}</p>
            <ol><For each=rows key=|key|*key children=move |key:u64| {
                let row_prefix=format!("{prefix}-{key}");
                let up_focus=format!("{row_prefix}-remove");
                let down_focus=up_focus.clone();
                let removed_focus=format!("{prefix}-add");
                view! {
                    <li data-collection-key=key>
                        <div class="sp-collection-fields">{render.run(key)}</div>
                        <div class="sp-actions sp-collection-actions" role="group" aria-label=format!("Row {key} actions")>
                            <button class="sp-icon-action" aria-label=format!("Move row {key} up") id=format!("{row_prefix}-up") type="button"
                                disabled=move ||disabled.get() ||list.with(|list|list.rows().first().is_some_and(|row|row.id==key))
                                on:click=move |_| {
                                    if disabled.get() { return; }
                                    list.update(|list|shift(list,key,true));
                                    pending_focus.set(Some(up_focus.clone()));
                                }>{icon(IconKind::ArrowUp)}</button>
                            <button class="sp-icon-action" aria-label=format!("Move row {key} down") id=format!("{row_prefix}-down") type="button"
                                disabled=move ||disabled.get() ||list.with(|list|list.rows().last().is_some_and(|row|row.id==key))
                                on:click=move |_| {
                                    if disabled.get() { return; }
                                    list.update(|list|shift(list,key,false));
                                    pending_focus.set(Some(down_focus.clone()));
                                }>{icon(IconKind::ArrowDown)}</button>
                            <button class="sp-icon-action" aria-label=format!("Remove row {key}") id=format!("{row_prefix}-remove") type="button"
                                on:click=move |_| {
                                    if disabled.get() { return; }
                                    list.update(|list|{let _=list.remove(key);});
                                    pending_focus.set(Some(removed_focus.clone()));
                                }>{icon(IconKind::Trash)}</button>
                        </div>
                    </li>
                }
            }/></ol>
            <button id=add_id type="button" disabled=move ||disabled.get() ||!list.with(BoundedList::can_add)
                on:click=move |_| {
                    if disabled.get() || !list.with(BoundedList::can_add) { return; }
                    // App factories can read the collection; never call one under its write guard.
                    let value=create.run(());
                    let mut added=None;
                    list.update(|list|added=list.add(value).ok());
                    if let Some(key)=added {
                        // The final add disables Add; the new Remove remains an enabled target.
                        pending_focus.set(Some(format!("{added_prefix}-{key}-remove")));
                    }
                }>"Add row"</button>
        </fieldset>
    }.into_any()
}

/// Editable bounded text rows. Native names include stable keys, never positional indices.
#[component]
pub fn TextList(
    /// Shared fieldset/control ID prefix.
    #[prop(into)]
    id: String,
    /// Native legend text.
    #[prop(into)]
    label: String,
    /// Controlled typed rows, retaining empty/zero strings.
    list: RwSignal<BoundedList<String>>,
    /// Native disabled flag.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned field error text.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    // Keep domain errors app-owned and local byte admission feedback separate.
    let (mutation_error, error) = super::inputs::field_error(error);
    let prefix = id.clone();
    let render = Callback::new(move |key| {
        let input = format!("{prefix}-{key}-value");
        let error_id = format!("{prefix}-error");
        view! {
            <label class="sp-sr-only" for=input.clone()>{format!("Value {key}")}</label>
            <input id=input name=format!("{prefix}[{key}]") class="sp-input" maxlength="4096" aria-describedby=error_id
                aria-invalid=move ||(!error.get().is_empty()).to_string()
                value=move ||text_value(list,key) prop:value=move ||text_value(list,key)
                on:input=move |event| {
                    if disabled.get() { return; }
                    let value=event_target_value(&event);
                    let mut result=Ok(());
                    list.update(|list|result=list.edit(key,value));
                    // A rejected edit keeps the last admitted model value.
                    mutation_error.set(if result.is_ok(){String::new()}else{"Value exceeds the field byte budget".into()});
                }/>
        }.into_any()
    });
    view! {<div class="sp-collection-inline"><CollectionFieldset id label list create=Callback::new(|()|String::new()) render disabled error/></div>}
}

/// Read by stable key so edits/reorders never capture a stale cloned initial row.
pub(super) fn text_value(list: RwSignal<BoundedList<String>>, key: u64) -> String {
    list.with(|list| {
        list.rows()
            .iter()
            .find(|row| row.id == key)
            .map(|row| row.value.clone())
            .unwrap_or_default()
    })
}

/// Typed editable metadata rows with independent key/value fields and stable row identities.
#[component]
pub fn MetadataInput(
    /// Shared fieldset/control prefix.
    #[prop(into)]
    id: String,
    /// Native legend.
    #[prop(into)]
    label: String,
    /// Root-owned metadata entries.
    list: RwSignal<BoundedList<MetadataEntry>>,
    /// Native disabled flag.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Application-owned errors; duplicate key policy belongs here.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    let (mutation_error, error) = super::inputs::field_error(error);
    let prefix = id.clone();
    let render = Callback::new(move |key| {
        [true,false].into_iter().map(|key_field| {
            let field=if key_field{"key"}else{"value"};
            let input=format!("{prefix}-{key}-{field}");
            // Each field reads its current keyed entry; edits never replace sibling drafts.
            let get=move ||list.with(|list|list.rows().iter().find(|row|row.id==key)
                .map(|row|if key_field{row.value.key.clone()}else{row.value.value.clone()}).unwrap_or_default());
            view! {
                <label class="sp-sr-only" for=input.clone()>{format!("{field} {key}")}</label>
                <input id=input name=format!("{prefix}[{key}][{field}]") class="sp-input" maxlength="4096"
                    aria-describedby=format!("{prefix}-error") aria-invalid=move ||(!error.get().is_empty()).to_string()
                    value=get prop:value=get on:input=move |event| {
                        if disabled.get() { return; }
                        let mut result=Ok(());
                        list.update(|list| {
                            if let Some(mut entry)=list.rows().iter().find(|row|row.id==key).map(|row|row.value.clone()) {
                                // Admit the entire entry atomically; changing its key cannot lose its value.
                                if key_field { entry.key=event_target_value(&event); }
                                else { entry.value=event_target_value(&event); }
                                result=list.edit(key,entry);
                            }
                        });
                        mutation_error.set(if result.is_ok(){String::new()}else{"Value exceeds the field byte budget".into()});
                    }/>
            }
        }).collect_view().into_any()
    });
    view! {<div class="sp-collection-inline"><CollectionFieldset id label list create=Callback::new(|()|MetadataEntry::default()) render disabled error/></div>}
}
