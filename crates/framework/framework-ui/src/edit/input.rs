//! Shared semantic text control; native edit/undo and bounded pure state stay separate.
use super::{EditPolicy, EditState};
use leptos::prelude::*;

/// Native text editor for a checked decimal, ISO temporal or explicit-region phone policy.
///
/// Wraps label/help/error/committed hidden value. The visible draft has no form name:
/// submission uses only the last valid canonical value. Invalid drafts remain visible.
/// Typing never rewrites native value/undo. Blur commits without rewriting native text; Enter commits and normalizes with a native edit command; Escape cancels. During
/// composition blur defers until the turn following compositionend/final input.
#[component]
pub fn FormattedInput<P: EditPolicy>(
    /// Stable unique control and association prefix.
    #[prop(into)]
    id: String,
    /// Canonical successful-field name, distinct from the raw visible draft.
    #[prop(into)]
    name: String,
    /// Escaped accessible label.
    #[prop(into)]
    label: String,
    /// Shared root-owned pure edit state.
    state: RwSignal<EditState<P>>,
    /// Empty commit clears the value only when explicitly enabled.
    #[prop(optional)]
    nullable: bool,
    /// Application action emitted once per changed committed value, outside state guards.
    #[prop(optional)]
    on_commit: Option<Callback<Option<P::Value>>>,
    /// Native disabled policy.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned help text; policy/domain details are supplied by the application.
    #[prop(optional, into)]
    help: String,
) -> impl IntoView {
    let node = NodeRef::<leptos::html::Input>::new();
    let error_id = format!("{id}-error");
    let help_id = format!("{id}-help");
    let described = format!("{help_id} {error_id}");
    let initial = state.with_untracked(|s| s.initial().to_owned());
    let max = state.with_untracked(EditState::max_bytes);
    #[cfg(target_arch = "wasm32")]
    let adapter = super::dom::install(node, state, nullable, on_commit);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (nullable, on_commit);
    view! {<div class="sp-edit"><label for=id.clone()>{label}</label>
        <input id=id node_ref=node type="text" class="sp-input" value=initial maxlength=max autocomplete="off" spellcheck="false"
            required= !nullable
            disabled=move ||disabled.get() aria-describedby=described aria-invalid=move ||state.with(|s|s.error().is_some()).to_string()
            on:input=move |_event|{#[cfg(target_arch="wasm32")]super::dom::read(node,state,adapter.syncing);}
            on:select=move |_event|{#[cfg(target_arch="wasm32")]super::dom::select(node,state,adapter.syncing);}
            on:keyup=move |_event|{#[cfg(target_arch="wasm32")]super::dom::select(node,state,adapter.syncing);}
            on:paste=move |_event:leptos::ev::ClipboardEvent|{#[cfg(target_arch="wasm32")]super::dom::paste(&_event,node,state);}
            on:compositionstart=move |_event|{#[cfg(target_arch="wasm32")] {super::dom::read(node,state,adapter.syncing);state.update(EditState::composition_start);}}
            on:compositionend=move |_event|{#[cfg(target_arch="wasm32")] {super::dom::read(node,state,adapter.syncing);state.update(EditState::composition_end);adapter.settle.run(());}}
            on:blur=move |_event|{#[cfg(target_arch="wasm32")] {super::dom::read(node,state,adapter.syncing);super::dom::commit(state,nullable,on_commit,false);}}
            on:keydown=move |_event:leptos::ev::KeyboardEvent|{#[cfg(target_arch="wasm32")] {
                if _event.key()=="Enter" && !_event.is_composing() && !state.with(EditState::composing){_event.prevent_default();super::dom::read(node,state,adapter.syncing);super::dom::commit(state,nullable,on_commit,true);}
                else if _event.key()=="Escape" && !_event.is_composing() && state.with(|s|s.dirty()||s.composing()||s.error().is_some()){_event.prevent_default();state.update(|s|if s.composing(){s.composition_cancel()}else{s.cancel()});}
            }}/>
        <p id=help_id>{help}</p><p id=error_id role="alert">{move ||state.with(|s|s.error().map(|e|e.to_string()).unwrap_or_default())}</p>
        <input type="hidden" name=name disabled=move ||disabled.get()||state.with(|s|s.value().is_none()) value=move ||state.with(EditState::canonical).unwrap_or_default() prop:value=move ||state.with(EditState::canonical).unwrap_or_default()/>
    </div>}
}
