//! Semantic table slots, native dialogs and typed form validation outcomes.

use leptos::prelude::*;

/// Optional neutral styles for the advanced components; applications opt in explicitly.
pub const ADVANCED_STYLES: &str = include_str!("advanced.css");

/// A native table in its own labelled, keyboard-scrollable region.
#[component]
pub fn Table(
    /// Native caption and overflow-region label.
    #[prop(into)]
    caption: Signal<String>,
    /// Existing rows remain mounted while the application replaces data.
    #[prop(optional, into)]
    busy: Signal<bool>,
    /// Explicit thead/tbody slots; applications own unique row keys and sort/page state.
    children: Children,
) -> impl IntoView {
    view! { <div class="sp-table-scroll" role="region" tabindex="0" aria-label=move || caption.get()>
        <table aria-busy=move || busy.get().to_string()><caption>{move || caption.get()}</caption>{children()}</table>
    </div> }
}

/// A public field error with a deterministic focus target.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldError {
    /// App-owned stable control ID, never an executable URL.
    pub field: String,
    /// Safe text-only validation direction.
    pub message: String,
}

/// An explicit app form outcome; draft/domain validation remain application-owned.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum FormState {
    /// No submission has been acknowledged yet.
    #[default]
    Editing,
    /// Retained draft with associated field errors.
    Invalid(Vec<FieldError>),
    /// A confirmed successful application submission.
    Saved,
}

impl FormState {
    /// Borrow errors without fabricating an error for a successful/untouched form.
    pub fn errors(&self) -> &[FieldError] {
        match self {
            Self::Invalid(errors) => errors,
            Self::Editing | Self::Saved => &[],
        }
    }
}

/// Native form with a typed synchronous validation/submission callback.
///
/// The callback owns domain policy. This helper does not perform writes/retries or
/// claim asynchronous submission outcomes. Invalid drafts and controls stay mounted.
#[component]
pub fn Form(
    /// Stable form identity.
    #[prop(into)]
    id: String,
    /// Root-owned validation/acknowledgement state.
    state: RwSignal<FormState>,
    /// App validation returns the explicit outcome; no hidden domain validator.
    submit: Callback<(), FormState>,
    /// Native labelled controls and submit buttons.
    children: Children,
) -> impl IntoView {
    view! { <form id=id novalidate on:submit=move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        let result = submit.run(());
        #[cfg(target_arch = "wasm32")]
        let first = result.errors().first().map(|error| error.field.clone());
        state.set(result);
        #[cfg(target_arch = "wasm32")]
        if let Some(first) = first { super::dialog_dom::focus_id(&first); }
    }>
        <div role="alert" class="sp-form-errors">{move || state.get().errors().iter().map(|error| {
            view! { <p><a href=format!("#{}", error.field)>{error.message.clone()}</a></p> }
        }).collect_view()}</div>
        {children()}
        <p role="status">{move || if state.get() == FormState::Saved { "Saved" } else { "" }}</p>
    </form> }
}

/// Native modal, nonmodal and drawer presentation share controlled lifetime rules.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogMode {
    /// Default existing modal behavior.
    #[default]
    Modal,
    /// Native nonmodal window; no background inertness or Tab trap is promised.
    Nonmodal,
    /// Modal drawer at the leading edge.
    DrawerStart,
    /// Modal drawer at the trailing edge.
    DrawerEnd,
}
impl DialogMode {
    /// Modeless closure preserves unrelated background focus; modal closure always returns it.
    #[cfg(any(target_arch = "wasm32", test))]
    pub(super) fn returns_focus(self, inside: bool, at_opener: bool, detached_body: bool) -> bool {
        self != Self::Nonmodal || inside || at_opener || detached_body
    }
    /// Stable classes; default markup remains byte-identical.
    fn classes(self) -> &'static str {
        match self {
            Self::Modal | Self::Nonmodal => "sp-dialog",
            Self::DrawerStart => "sp-dialog sp-drawer sp-drawer-start",
            Self::DrawerEnd => "sp-dialog sp-drawer sp-drawer-end",
        }
    }
}

/// Controlled native modal, initially closed on SSR and hydration.
///
/// Native showModal supplies focus containment and background inertness. Callers
/// supply unique IDs; initially-open SSR routes are outside this M0 contract.
/// Include ADVANCED_STYLES to lock background scrolling while any modal is open;
/// native focus/background inertness does not depend on those optional styles.
#[component]
pub fn Dialog(
    /// Additive native mode; modal is the unchanged default.
    #[prop(optional)]
    mode: DialogMode,
    /// Stable unique identity, also the prefix for its accessible title.
    #[prop(into)]
    id: String,
    /// Root-owned modal state; start false on SSR and hydration.
    open: RwSignal<bool>,
    /// Escaped accessible heading.
    #[prop(into)]
    title: Signal<String>,
    /// Explicit first focused control after opening; empty uses native selection.
    #[prop(default = String::new(), into)]
    initial_focus_id: String,
    /// Focus target when the opener has been removed.
    #[prop(default = String::new(), into)]
    fallback_focus_id: String,
    /// Escape dismissal is an explicit application policy.
    #[prop(default = true)]
    close_on_escape: bool,
    /// The mounted dialog content, retained while closed to preserve drafts.
    children: Children,
) -> impl IntoView {
    let element = NodeRef::<leptos::html::Dialog>::new();
    #[cfg(target_arch = "wasm32")]
    super::dialog_dom::install(element, open, initial_focus_id, fallback_focus_id, mode);
    #[cfg(not(target_arch = "wasm32"))]
    let _ = (initial_focus_id, fallback_focus_id);
    let labelled_by = format!("{id}-title");
    // Pinned Tachys has no typed closedby builder; this fixed native attribute is opt-in.
    let closed_by = leptos::tachys::html::attribute::custom::custom_attribute(
        "closedby",
        if mode == DialogMode::Nonmodal {
            Some(if close_on_escape {
                "closerequest"
            } else {
                "none"
            })
        } else {
            None
        },
    );
    view! { <dialog id=id node_ref=element class=mode.classes() aria-labelledby=labelled_by.clone()
        {..closed_by}
        on:cancel=move |event: leptos::ev::Event| { if close_on_escape { open.try_set(false); } else { event.prevent_default(); } }
        on:close=move |_| { open.try_set(false); }
        on:keydown=move |_event: leptos::ev::KeyboardEvent| { #[cfg(target_arch="wasm32")] {
            // Keep focused modeless Escape working even where native closedby is unavailable.
            if mode==DialogMode::Nonmodal && _event.key()=="Escape" && !_event.default_prevented() && close_on_escape && super::dialog_dom::from_current_dialog(&_event) {_event.prevent_default();open.try_set(false);}
            else if mode!=DialogMode::Nonmodal {super::dialog_dom::trap_tab(&_event);}
        }}>
        <h2 id=labelled_by.clone()>{move || title.get()}</h2>{children()}
    </dialog> }
}
