//! Searchable selection and generation-checked suggestions; no endpoint is inferred.
use super::{ControlError, Popover, PopupRole, SelectOptions};
use leptos::prelude::*;

/// Public acquisition state; failure never silently looks like an empty result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SuggestionStatus {
    /// Current options are available.
    Ready,
    /// One explicit application acquisition is pending.
    Loading,
    /// The application acquisition failed.
    Failed,
}
#[cfg(feature = "async")]
struct AbortLease(futures::future::AbortHandle);
#[cfg(feature = "async")]
impl Drop for AbortLease {
    /// Disposing the root cancels an admitted yielding loader.
    fn drop(&mut self) {
        self.0.abort();
    }
}
/// Root-owned searchable choices; selected value and editable query are distinct.
#[derive(Clone, Copy)]
pub struct ComboboxState {
    options: RwSignal<SelectOptions>,
    query: RwSignal<String>,
    selected: RwSignal<Option<String>>,
    active: RwSignal<Option<usize>>,
    open: RwSignal<bool>,
    status: RwSignal<SuggestionStatus>,
    generation: RwSignal<u64>,
    #[cfg(feature = "async")]
    identity: StoredValue<std::sync::Arc<()>>,
    #[cfg(feature = "async")]
    abort: StoredValue<Option<AbortLease>>,
}
impl ComboboxState {
    /// Admit bounded immutable initial options and a known enabled optional selection.
    /// # Errors
    /// Rejects invalid initial selection instead of silently taking the first entry.
    pub fn new(options: SelectOptions, selected: Option<String>) -> Result<Self, ControlError> {
        // Resolve display text only from admitted enabled values; None stays distinct.
        let query = match &selected {
            Some(value) => options
                .entries()
                .iter()
                .find(|o| &o.value == value && !o.disabled)
                .map(|o| o.label.clone())
                .ok_or(ControlError)?,
            None => String::new(),
        };
        Ok(Self {
            options: RwSignal::new(options),
            query: RwSignal::new(query),
            selected: RwSignal::new(selected),
            active: RwSignal::new(None),
            open: RwSignal::new(false),
            status: RwSignal::new(SuggestionStatus::Ready),
            generation: RwSignal::new(0),
            #[cfg(feature = "async")]
            abort: StoredValue::new(None),
            #[cfg(feature = "async")]
            identity: StoredValue::new(std::sync::Arc::new(())),
        })
    }
    /// Exact query text, independent of selected machine value.
    pub fn query(self) -> String {
        self.query.get()
    }
    /// Exact optional selected value, preserving Some("") versus None.
    pub fn selected(self) -> Option<String> {
        self.selected.get()
    }
    /// Current acquisition state.
    pub fn status(self) -> SuggestionStatus {
        self.status.get()
    }
    /// Shared native popover visibility controller.
    pub fn open(self) -> RwSignal<bool> {
        self.open
    }
    /// Current active descendant index, always in immutable result order.
    pub fn active(self) -> Option<usize> {
        self.active.get()
    }
    /// Literal case-insensitive label filtering, capped by validated option data.
    pub fn matching(self) -> Vec<(usize, super::SelectOption)> {
        let query = self.query.get().to_lowercase();
        self.options
            .get()
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, option)| option.label.to_lowercase().contains(&query))
            .map(|(index, option)| (index, option.clone()))
            .collect()
    }
    /// Seed an early native draft before mounting, without opening or acquiring options.
    /// # Errors
    /// Rejects drafts beyond the same 1024-byte query budget.
    pub fn adopt_query(self, query: String) -> Result<(), ControlError> {
        if query.len() > 1024 {
            return Err(ControlError);
        }
        self.query.set(query);
        self.selected.set(None);
        Ok(())
    }
    /// Closed SSR popovers retain their full option tree while early text is adopted.
    fn rendered(self) -> Vec<(usize, super::SelectOption)> {
        if self.open.get() {
            self.matching()
        } else {
            self.options
                .get()
                .entries()
                .iter()
                .cloned()
                .enumerate()
                .collect()
        }
    }
    /// Publish verbatim bounded edits and clear stale selected machine values.
    /// # Errors
    /// Rejects queries beyond 1024 bytes without starting work.
    pub fn edit(self, query: String) -> Result<(), ControlError> {
        if query.len() > 1024 {
            return Err(ControlError);
        }
        // Invalidate previous work before exposing the new draft/result surface.
        self.cancel();
        self.query.set(query);
        self.selected.set(None);
        self.active.set(None);
        self.open.set(true);
        Ok(())
    }
    /// Reject an oversized native edit without retaining a stale submitted selection.
    /// Keep the last bounded draft, cancel work and let the component name the error.
    pub(super) fn admit_native_edit(self, query: String) -> Result<String, ControlError> {
        if query.len() > 1024 {
            // Native maxlength counts UTF-16 units; the independent byte budget still applies.
            let previous = self.query.get_untracked();
            self.edit(previous)?;
            return Err(ControlError);
        }
        Ok(query)
    }
    /// Keyboard movement among current enabled results; focus stays on the input.
    pub fn navigate(self, forward: bool, edge: bool) -> Option<usize> {
        if self.status.get() != SuggestionStatus::Ready {
            self.active.set(None);
            return None;
        }
        let enabled: Vec<_> = self
            .matching()
            .into_iter()
            .filter(|(_, o)| !o.disabled)
            .map(|(i, _)| i)
            .collect();
        if enabled.is_empty() {
            self.active.set(None);
            return None;
        }
        // Keep DOM focus on the input; only its active-descendant identity moves.
        let position = self
            .active
            .get()
            .and_then(|i| enabled.iter().position(|key| *key == i));
        let next = if edge {
            if forward { 0 } else { enabled.len() - 1 }
        } else if forward {
            position.map_or(0, |i| (i + 1) % enabled.len())
        } else {
            position.map_or(enabled.len() - 1, |i| {
                (i + enabled.len() - 1) % enabled.len()
            })
        };
        let index = enabled[next];
        self.active.set(Some(index));
        self.open.set(true);
        Some(index)
    }
    /// Select only an enabled currently matching result; unknown/disabled events fail closed.
    pub fn choose(self, index: usize) -> bool {
        if self.status.get() != SuggestionStatus::Ready {
            return false;
        }
        let option = self
            .matching()
            .into_iter()
            .find(|(i, o)| *i == index && !o.disabled)
            .map(|(_, o)| o);
        if let Some(option) = option {
            self.cancel();
            self.selected.set(Some(option.value));
            self.query.set(option.label);
            self.active.set(Some(index));
            self.open.set(false);
            true
        } else {
            false
        }
    }
    /// Cancel pending acquisition, invalidate its generation and close the surface.
    pub fn cancel(self) {
        if self.open.get_untracked() {
            self.open.set(false);
        }
        #[cfg(feature = "async")]
        {
            let mut previous = None;
            self.abort.update_value(|slot| previous = slot.take());
            drop(previous);
        }
        self.generation
            .update(|generation| *generation = generation.saturating_add(1));
        self.status.set(SuggestionStatus::Ready);
    }
    /// Explicit acquisition ticket. Completion must be published back in this root.
    /// # Errors
    /// Rejects oversized query or exhausted generation space before starting work.
    #[cfg(feature = "async")]
    pub fn request<F>(
        self,
        query: String,
        loader: F,
    ) -> Result<impl std::future::Future<Output = Option<SuggestionCompletion>>, ControlError>
    where
        F: std::future::Future<Output = Result<SelectOptions, ControlError>> + 'static,
    {
        self.edit(query)?;
        let generation = self.generation.get();
        if generation == u64::MAX {
            return Err(ControlError);
        }
        // The ticket owns the loader; the root owns only its cooperative abort handle.
        self.status.set(SuggestionStatus::Loading);
        let (handle, registration) = futures::future::AbortHandle::new_pair();
        self.abort.set_value(Some(AbortLease(handle)));
        let identity = std::sync::Arc::downgrade(&self.identity.get_value());
        Ok(async move {
            futures::future::Abortable::new(loader, registration)
                .await
                .ok()
                .map(|result| SuggestionCompletion {
                    generation,
                    identity,
                    result,
                })
        })
    }
    /// Apply a completion only if its root still exists and its generation is current.
    /// This synchronous call belongs to the root's UI context, never an arbitrary worker.
    #[cfg(feature = "async")]
    pub fn publish(self, completion: SuggestionCompletion) -> bool {
        if self.generation.try_get() != Some(completion.generation) {
            return false;
        }
        // A local generation is not a globally unique root/controller identity.
        if !std::sync::Weak::ptr_eq(
            &completion.identity,
            &std::sync::Arc::downgrade(&self.identity.get_value()),
        ) {
            return false;
        }
        match completion.result {
            Ok(options) => {
                self.options.set(options);
                self.status.set(SuggestionStatus::Ready);
            }
            Err(_) => {
                self.options
                    .set(SelectOptions::new(Vec::new()).expect("empty options"));
                self.status.set(SuggestionStatus::Failed);
            }
        }
        self.active.set(None);
        true
    }
}
/// Owned acquisition output carries no Owner/borrow; publication remains explicit.
#[cfg(feature = "async")]
pub struct SuggestionCompletion {
    identity: std::sync::Weak<()>,
    generation: u64,
    result: Result<SelectOptions, ControlError>,
}
/// Labelled searchable single selection; async lookup is an optional explicit callback.
/// The host calls state.request and later state.publish in its UI context. No initial fetch.
#[component]
pub fn Combobox(
    /// Native input identity and listbox prefix.
    #[prop(into)]
    id: String,
    /// Visible native label.
    #[prop(into)]
    label: String,
    /// Native form field key; only a selected value is submitted.
    #[prop(into)]
    name: String,
    /// Root-owned typed options/query/selection.
    state: ComboboxState,
    /// Explicit acquisition adapter, invoked on admitted user edits only.
    #[prop(optional)]
    lookup: Option<Callback<String>>,
    /// Native disabled policy.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned validation error.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    let popup = format!("{id}-list");
    let active_prefix = id.clone();
    let option_prefix = id.clone();
    // Local admission errors do not replace app-owned domain validation.
    let app_error = error;
    let admission_error = RwSignal::new(String::new());
    let error = Signal::derive(move || {
        let local = admission_error.get();
        if local.is_empty() {
            app_error.get()
        } else {
            local
        }
    });

    #[cfg(target_arch = "wasm32")]
    Effect::new(move |_| {
        let enabled = !disabled.get();
        let opened = state.open().get();
        if !enabled || !opened {
            state.cancel();
            if !enabled {
                state.open().set(false);
            }
        }
    });
    view! {<div class="sp-combobox"><label for=id.clone()>{label.clone()}</label>
        <input id=id.clone() class="sp-input" type="text" maxlength="1024" role="combobox" autocomplete="off" aria-autocomplete="list" aria-haspopup="listbox" aria-controls=popup.clone()
            aria-expanded=move ||state.open.get().to_string() aria-activedescendant=move ||if state.open.get(){state.active().map(|index|format!("{active_prefix}-option-{index}"))}else{None}
            aria-describedby=format!("{id}-error") aria-invalid=move ||(!error.get().is_empty()).to_string() disabled=move ||disabled.get()
            value=move ||state.query() prop:value=move ||state.query()
            on:input=move |event|{if disabled.get(){return;}match state.admit_native_edit(event_target_value(&event)){
                Ok(query)=>{admission_error.set(String::new());if let Some(lookup)=lookup{lookup.run(query);}else{let _=state.edit(query);}},
                Err(_)=>admission_error.set("Query exceeds the field byte budget".into()),
            }}
            on:keydown=move |event:leptos::ev::KeyboardEvent|{if disabled.get(){return;}match event.key().as_str(){
                "ArrowDown"=>{event.prevent_default();state.navigate(true,false);},"ArrowUp"=>{event.prevent_default();state.navigate(false,false);},
                "Home" if state.open.get()=>{event.prevent_default();state.navigate(true,true);},"End" if state.open.get()=>{event.prevent_default();state.navigate(false,true);},
                "Enter" if state.open.get()=>{event.prevent_default();if let Some(index)=state.active() && state.choose(index){admission_error.set(String::new());}},
                "Escape"=>{event.prevent_default();state.cancel();state.open.set(false);},"Tab"=>{state.cancel();state.open.set(false);},_=>{}
            }}/>
        <input type="hidden" name=name disabled=move ||disabled.get() ||state.selected().is_none() value=move ||state.selected().unwrap_or_default()/>
        <p id=format!("{id}-error") role="alert">{move ||error.get()}</p>
        <Popover id=popup anchor=id label=label open=state.open() role=PopupRole::Listbox>
            <span role="status">{move ||if !state.open.get(){""}else{match state.status(){SuggestionStatus::Loading=>"Loading suggestions",SuggestionStatus::Failed=>"Suggestions unavailable",SuggestionStatus::Ready if state.matching().is_empty()=>"No suggestions",SuggestionStatus::Ready=>""}}}</span>
            <div class="sp-options"><For each=move ||state.rendered() key=|(index,option)|(*index,option.value.clone(),option.label.clone(),option.disabled) children=move |(index,option)|{
                let selected=option.value.clone();let prefix=option_prefix.clone();let blocked=option.disabled;
                view!{<button id=format!("{prefix}-option-{index}") type="button" role="option" tabindex="-1" disabled=move ||blocked ||state.status()!=SuggestionStatus::Ready
                    aria-selected=move ||(state.selected().as_deref()==Some(selected.as_str())).to_string() data-active=move ||(state.active()==Some(index)).to_string()
                    on:mousedown=move |event|event.prevent_default() on:click=move |_|{if state.choose(index){admission_error.set(String::new());}}>{option.label}</button>}
            }/></div>
        </Popover>
    </div>}
}
/// Explicit suggestion input uses the same semantics with a required lookup adapter.
#[component]
pub fn SuggestInput(
    /// Input identity.
    #[prop(into)]
    id: String,
    /// Visible native label.
    #[prop(into)]
    label: String,
    /// Native selected-value form key.
    #[prop(into)]
    name: String,
    /// Root-owned query controller.
    state: ComboboxState,
    /// Required bounded application acquisition adapter.
    lookup: Callback<String>,
    /// Native disabled admission, forwarded to the underlying combobox.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned validation message.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    view! {<Combobox id label name state lookup disabled error/>}
}
