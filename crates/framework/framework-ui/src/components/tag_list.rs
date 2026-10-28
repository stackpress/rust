//! Plain keyed tag chips; draft text is separate from the exact submitted collection.
use super::{
    BoundedList, ControlError,
    collections::text_value,
    icons::{IconKind, icon},
};
use leptos::prelude::*;

/// Append an explicit nonblank draft without trimming, splitting or deduplicating it.
fn append(list: &mut BoundedList<String>, draft: &str) -> Result<Option<u64>, ControlError> {
    if draft.trim().is_empty() {
        return Ok(None);
    }
    list.add(draft.to_owned()).map(Some)
}

/// Remove one identity and choose its next neighbour, or the preceding final chip.
fn remove(list: &mut BoundedList<String>, key: u64) -> Result<Option<u64>, ControlError> {
    let index = list
        .rows()
        .iter()
        .position(|row| row.id == key)
        .ok_or(ControlError)?;
    list.remove(key)?;
    Ok(list
        .rows()
        .get(index)
        .or_else(|| list.rows().last())
        .map(|row| row.id))
}

/// Adopt the add input's native property before hydration can reconcile its draft.
fn initial_draft(_id: &str) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::JsCast;
        web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id(_id))
            .and_then(|element| element.dyn_into::<web_sys::HtmlInputElement>().ok())
            .map(|input| input.value())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    String::new()
}

/// Removable text chips with a labelled add draft; order/duplicates/empty values stay exact.
/// Empty drafts are ignored. Enter adds after IME composition; empty Backspace only focuses
/// the last remove button. Hidden inputs retain stable form names. No chip editing/reorder.
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
    /// Native disabled flag, also checked by event admission.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Application errors, combined with local byte-budget feedback.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    let (mutation_error, error) = super::inputs::field_error(error);
    let input = format!("{id}-draft");
    let draft = RwSignal::new(initial_draft(&input));
    let composing = RwSignal::new(false);
    let pending_focus = RwSignal::new(None::<String>);
    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        if let Some(id) = pending_focus.get() {
            super::dialog_dom::focus_id(&id);
        }
    });
    let add_input = input.clone();
    let add = Callback::new(move |()| {
        // Capacity/disabled/composition checks precede any mutation. Rejections retain
        // both the exact draft and all existing chip identities.
        if disabled.get() || composing.get() || !list.with(BoundedList::can_add) {
            return;
        }
        let mut result = Ok(None);
        // A no-op/rejection must not notify an adapter and accidentally promote its
        // Missing/Null presentation into a supplied empty array.
        list.maybe_update(|list| {
            result = append(list, &draft.get_untracked());
            matches!(&result, Ok(Some(_)))
        });
        match result {
            Ok(Some(_)) => {
                draft.set(String::new());
                mutation_error.set(String::new());
                pending_focus.set(Some(add_input.clone()));
            }
            Ok(None) => {} // A blank draft remains visible, without adding an error.
            Err(_) => mutation_error.set("Value exceeds the field byte budget".into()),
        }
    });
    let prefix = id.clone();
    let keys = move || list.with(|list| list.rows().iter().map(|row| row.id).collect::<Vec<_>>());
    let error_id = format!("{id}-error");
    let backspace_prefix = id.clone();
    view! {
        <fieldset class="sp-collection sp-tag-list" id=id.clone() disabled=move ||disabled.get()
            aria-describedby=error_id.clone()>
            <legend>{label}</legend><p id=error_id.clone() role="alert">{move ||error.get()}</p>
            <ul class="sp-tag-chips"><For each=keys key=|key|*key children=move |key:u64| {
                let remove_prefix = prefix.clone();
                let displayed = move || {
                    let value = text_value(list, key);
                    if value.is_empty() { "(empty)".to_owned() } else { value }
                };
                view! {
                    <li class="sp-tag-chip" data-collection-key=key>
                        <span class="sp-tag-text">{displayed}</span>
                        <input type="hidden" id=format!("{prefix}-{key}-value") name=format!("{prefix}[{key}]")
                            value=move ||text_value(list,key) prop:value=move ||text_value(list,key)/>
                        <button type="button" class="sp-icon-action sp-tag-remove" id=format!("{prefix}-{key}-remove")
                            aria-label=move ||format!("Remove tag {}",displayed()) disabled=move ||disabled.get()
                            on:click=move |_| {
                                if disabled.get() { return; }
                                let mut result = Err(ControlError);
                                list.maybe_update(|list| {
                                    result = remove(list,key);
                                    result.is_ok()
                                });
                                if let Ok(neighbour) = result {
                                    mutation_error.set(String::new());
                                    let target = neighbour.map(|key| format!("{remove_prefix}-{key}-remove"))
                                        .unwrap_or_else(|| format!("{remove_prefix}-draft"));
                                    pending_focus.set(Some(target));
                                }
                            }>{icon(IconKind::X)}</button>
                    </li>
                }
            }/></ul>
            <label for=input.clone()>"Add tag"</label>
            <div class="sp-tag-add">
                <input id=input class="sp-input" maxlength="4096" aria-describedby=error_id.clone()
                    aria-invalid=move ||(!error.get().is_empty()).to_string()
                    value=move ||draft.get() prop:value=move ||draft.get()
                    on:compositionstart=move |_|composing.set(true)
                    on:compositionend=move |_|composing.set(false)
                    on:input=move |event| {
                        if !disabled.get() {
                            draft.set(event_target_value(&event));
                            mutation_error.set(String::new());
                        }
                    }
                    on:keydown=move |event| {
                        // Some IMEs expose only legacy keyCode 229 during confirmation.
                        if disabled.get() || composing.get() || event.is_composing() || event.key_code()==229 {
                            return;
                        }
                        if event.key()=="Enter" {
                            event.prevent_default(); // Never submit the enclosing form.
                            add.run(());
                        } else if event.key()=="Backspace" && draft.get_untracked().is_empty()
                            && let Some(key) = list.with(|list|list.rows().last().map(|row|row.id)) {
                            event.prevent_default(); // Focus first; removal needs its own action.
                            pending_focus.set(Some(format!("{backspace_prefix}-{key}-remove")));
                        }
                    }/>
                <button type="button" id=format!("{id}-add") disabled=move ||disabled.get() ||!list.with(BoundedList::can_add)
                    on:click=move |_|add.run(())>{icon(IconKind::Plus)}<span>"Add"</span></button>
            </div>
        </fieldset>
    }.into_any()
}

#[cfg(test)]
mod tests {
    use super::super::CollectionRow;
    use super::*;

    /// Draft admission preserves exact data, including duplicates, Unicode and empty seeds.
    #[test]
    fn exact_append_and_neighbour_removal() {
        let mut list = BoundedList::new(
            vec![CollectionRow {
                id: 4,
                value: String::new(),
            }],
            4,
        )
        .unwrap();
        for blank in ["", " \t\n", "\u{2003}"] {
            let before = list.clone();
            assert_eq!(append(&mut list, blank), Ok(None));
            assert_eq!(list, before);
        }
        assert_eq!(append(&mut list, " 0,雪 "), Ok(Some(5)));
        assert_eq!(append(&mut list, " 0,雪 "), Ok(Some(6)));
        let before = list.clone();
        assert!(append(&mut list, &"🦀".repeat(1025)).is_err());
        assert_eq!(list, before);
        assert_eq!(
            list.rows()
                .iter()
                .map(|row| row.value.as_str())
                .collect::<Vec<_>>(),
            ["", " 0,雪 ", " 0,雪 "]
        );
        assert_eq!(remove(&mut list, 5), Ok(Some(6)));
        assert_eq!(remove(&mut list, 6), Ok(Some(4)));
        assert_eq!(remove(&mut list, 4), Ok(None));
        assert!(remove(&mut list, 4).is_err());
        assert_eq!(append(&mut list, "0"), Ok(Some(7)));
    }
}
