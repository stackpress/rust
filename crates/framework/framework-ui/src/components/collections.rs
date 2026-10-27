//! Explicit keyed fieldset slots and bounded editable text/tag/metadata adapters.
use super::{BoundedList, MetadataEntry};
use leptos::prelude::*;

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
    T: Clone + Send + Sync + super::CollectionValue + 'static,
{
    // Key views only by identity: text edits must not recreate their native input nodes.
    let rows = move || list.with(|list| list.rows().iter().map(|row| row.id).collect::<Vec<_>>());
    let prefix = id.clone();
    let relationship = format!("{id}-error");
    let add_id = format!("{id}-add");
    // A successful final add disables Add; focus the new row's enabled Remove control.
    let added_prefix = id.clone();
    let pending_focus = RwSignal::new(None::<String>);
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(id) = pending_focus.get() {
            super::dialog_dom::focus_id(&id);
        }
    });
    view! {<fieldset class="sp-collection" id=id disabled=move ||disabled.get() aria-describedby=relationship.clone()>
        <legend>{label}</legend><p id=relationship.clone() role="alert">{move ||error.get()}</p>
        <ol><For each=rows key=|key|*key children={move |key:u64|{
            let row_prefix=format!("{prefix}-{key}");let up=row_prefix.clone();let down=row_prefix.clone();let removed_focus=format!("{prefix}-add");
            view!{<li data-collection-key=key>{render.run(key)}<div class="sp-actions" role="group" aria-label=format!("Row {key} actions")>
                <button id=format!("{row_prefix}-up") type="button" disabled=move ||disabled.get() ||list.get().rows().first().is_some_and(|row|row.id==key)
                    on:click=move |_|{if disabled.get(){return;}list.update(|list|{if let Some(index)=list.rows().iter().position(|row|row.id==key) && index>0{let _=list.reorder(key,index-1);}});pending_focus.set(Some(format!("{up}-remove")));}>"Move up"</button>
                <button id=format!("{row_prefix}-down") type="button" disabled=move ||disabled.get() ||list.get().rows().last().is_some_and(|row|row.id==key)
                    on:click=move |_|{if disabled.get(){return;}list.update(|list|{if let Some(index)=list.rows().iter().position(|row|row.id==key) && index+1<list.rows().len(){let _=list.reorder(key,index+1);}});pending_focus.set(Some(format!("{down}-remove")));}>"Move down"</button>
                <button id=format!("{row_prefix}-remove") type="button" on:click=move |_|{if disabled.get(){return;}list.update(|list|{let _=list.remove(key);});pending_focus.set(Some(removed_focus.clone()));}>"Remove"</button>
            </div></li>}
        }}/></ol><button id=add_id.clone() type="button" disabled=move ||disabled.get() ||!list.get().can_add()
            on:click=move |_|{if disabled.get() || !list.get().can_add(){return;}let value=create.run(());let mut added=None;list.update(|list|{added=list.add(value).ok();});if let Some(key)=added{pending_focus.set(Some(format!("{added_prefix}-{key}-remove")));}}>"Add row"</button>
    </fieldset>}
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
    // Keep domain errors owned by the app and report local byte-admission failures separately.
    let app_error = error;
    let mutation_error = RwSignal::new(String::new());
    let error = Signal::derive(move || {
        let local = mutation_error.get();
        if local.is_empty() {
            app_error.get()
        } else {
            local
        }
    });
    let prefix = id.clone();
    let render = Callback::new(move |key| {
        let input = format!("{prefix}-{key}-value");
        let error_id = format!("{prefix}-error");
        view!{<label for=input.clone()>{format!("Value {key}")}</label><input id=input name=format!("{prefix}[{key}]") class="sp-input" maxlength="4096" aria-describedby=error_id
            aria-invalid=move ||(!error.get().is_empty()).to_string() value=move ||text_value(list,key) prop:value=move ||text_value(list,key)
            on:input=move |event|{if !disabled.get(){let value=event_target_value(&event);let mut result=Ok(());list.update(|list|result=list.edit(key,value));mutation_error.set(if result.is_ok(){String::new()}else{"Value exceeds the field byte budget".into()});}}/>}.into_any()
    });
    view! {<CollectionFieldset id label list create=Callback::new(|()|String::new()) render disabled error/>}
}
/// Read by stable key so edits/reorders never capture a stale cloned initial row.
fn text_value(list: RwSignal<BoundedList<String>>, key: u64) -> String {
    list.with(|list| {
        list.rows()
            .iter()
            .find(|row| row.id == key)
            .map(|row| row.value.clone())
            .unwrap_or_default()
    })
}
/// Editable tags use the same explicit keyed list contract; no comma/empty coercion.
#[component]
pub fn TagList(
    /// Shared fieldset/control prefix.
    #[prop(into)]
    id: String,
    /// Native legend.
    #[prop(into)]
    label: String,
    /// Exact bounded tag strings; duplicates remain app policy.
    list: RwSignal<BoundedList<String>>,
    /// Native disabled flag.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Application errors.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    view! {<div class="sp-tag-list"><TextList id label list disabled error/></div>}
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
    // Keep domain errors owned by the app and report local byte-admission failures separately.
    let app_error = error;
    let mutation_error = RwSignal::new(String::new());
    let error = Signal::derive(move || {
        let local = mutation_error.get();
        if local.is_empty() {
            app_error.get()
        } else {
            local
        }
    });
    let prefix = id.clone();
    let render = Callback::new(move |key| {
        [true,false].into_iter().map(|key_field|{
            let field=if key_field{"key"}else{"value"};let input=format!("{prefix}-{key}-{field}");
            let get=move ||list.with(|list|list.rows().iter().find(|row|row.id==key).map(|row|if key_field{row.value.key.clone()}else{row.value.value.clone()}).unwrap_or_default());
            view!{<label for=input.clone()>{format!("{field} {key}")}</label><input id=input name=format!("{prefix}[{key}][{field}]") class="sp-input" maxlength="4096" aria-describedby=format!("{prefix}-error") aria-invalid=move ||(!error.get().is_empty()).to_string()
                value=get prop:value=get on:input=move |event|{if disabled.get(){return;}let mut result=Ok(());list.update(|list|{
                    if let Some(mut entry)=list.rows().iter().find(|row|row.id==key).map(|row|row.value.clone()){
                        if key_field{entry.key=event_target_value(&event);}else{entry.value=event_target_value(&event);}result=list.edit(key,entry);
                    }
                });mutation_error.set(if result.is_ok(){String::new()}else{"Value exceeds the field byte budget".into()});}/>
            }
        }).collect_view().into_any()
    });
    view! {<CollectionFieldset id label list create=Callback::new(|()|MetadataEntry::default()) render disabled error/>}
}
