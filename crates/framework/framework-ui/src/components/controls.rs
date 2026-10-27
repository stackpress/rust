//! Small typed HTML controls with checked navigation and explicit associations.

use crate::state::{NoticeKind, Notifications};
use leptos::prelude::*;

/// Native button behavior; links are selected explicitly with Button's href prop.
#[derive(Clone, Copy, Default)]
pub enum ButtonType {
    /// An action that does not submit an ancestor form.
    #[default]
    Button,
    /// Submit the native form, including keyboard Enter behavior.
    Submit,
    /// Reset a native form.
    Reset,
}

impl ButtonType {
    /// Return the native HTML attribute, without accepting arbitrary tokens.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Submit => "submit",
            Self::Reset => "reset",
        }
    }
}

/// Allow local relative/fragment links or explicit HTTPS/HTTP destinations.
///
/// Control characters, whitespace, backslashes, protocol-relative URLs and every
/// other scheme are rejected. This is navigation hygiene, not host authorization.
///
/// # Examples
///
/// ```
/// use stackpress_framework_ui::components::safe_href;
/// assert!(safe_href("/profiles?id=abc"));
/// assert!(!safe_href("javascript:alert(1)"));
/// ```
pub fn safe_href(href: &str) -> bool {
    // Browsers normalize these characters before scheme parsing; reject them first.
    if href.is_empty()
        || href
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        || href.contains('\\')
        || href.starts_with("//")
    {
        return false;
    }
    if href.starts_with("https://") || href.starts_with("http://") {
        // Require a nonempty authority, rather than allowing https:///host ambiguity.
        let authority = href
            .split_once("://")
            .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
            .unwrap_or_default();
        return !authority.is_empty() && !authority.contains('@');
    }
    // A colon in the first path segment could introduce an executable scheme.
    !href
        .split(['/', '?', '#'])
        .next()
        .unwrap_or_default()
        .contains(':')
}

/// Button or safe link with explicit native semantics and disabled behavior.
#[component]
pub fn Button(
    /// The native button type; default actions never submit accidentally.
    #[prop(default = ButtonType::Button)]
    button_type: ButtonType,
    /// Checked link destination, selecting anchor semantics when supplied.
    #[prop(optional)]
    href: Option<String>,
    /// Reactive disabled state; disabled anchors lose their navigation target.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Optional stable DOM identity.
    #[prop(optional)]
    id: Option<String>,
    /// Optional native form-control name.
    #[prop(optional)]
    name: Option<String>,
    /// Optional native form-control value.
    #[prop(optional)]
    value: Option<String>,
    /// Additional explicit application classes.
    #[prop(default = String::new())]
    class: String,
    /// Escaped label or deliberate child views.
    children: Children,
) -> impl IntoView {
    // Leptos props own this buffer; reuse it for the rendered class attribute.
    let mut class = class;
    class.insert_str(0, "sp-button ");
    match href {
        Some(href) => {
            let safe = safe_href(&href);
            // Removing href and tab stop prevents both pointer and keyboard navigation.
            view! { <a id=id class=class href=move || (safe && !disabled.get()).then(|| href.clone())
                aria-disabled=move || (!safe || disabled.get()).to_string()
                tabindex=move || if safe && !disabled.get() { "0" } else { "-1" }>{children()}</a> }.into_any()
        }
        None => view! { <button id=id class=class type=button_type.as_str() name=name value=value
        disabled=move || disabled.get()>{children()}</button> }
        .into_any(),
    }
}

/// Controlled text input; value changes belong to the supplied root-owned signal.
#[component]
pub fn Input(
    /// Label target and unique native DOM identifier.
    #[prop(into)]
    id: String,
    /// Native GET form key.
    #[prop(into)]
    name: String,
    /// Controlled value shared by SSR, hydration and interactive state.
    value: RwSignal<String>,
    /// IDs of help and error elements, separated by spaces.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Whether this field has a current associated validation error.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Disable native editing when the caller requires it.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required semantics; defaults preserve existing optional inputs.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Native readonly semantics; the value remains a successful form control.
    #[prop(optional, into)]
    read_only: Signal<bool>,
    /// Optional visible input hint, supplementary to the required label.
    #[prop(default = String::new())]
    placeholder: String,
) -> impl IntoView {
    // Attribute seeds SSR while property keeps subsequent browser edits controlled.
    view! { <input class="sp-input" type="text" id=id name=name
    value=move || value.get() prop:value=move || value.get()
    aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
    disabled=move || disabled.get() required=move || required.get() readonly=move || read_only.get() placeholder=placeholder
    on:input=move |event| value.set(event_target_value(&event))/> }
}

/// Label, help and error association; IDs are derived deterministically from for_id.
#[component]
pub fn FieldControl(
    /// Target Input ID; help and error IDs append -help and -error respectively.
    #[prop(into)]
    for_id: String,
    /// Human-readable native label.
    #[prop(into)]
    label: Signal<String>,
    /// Optional help text, retained as an association target when empty.
    #[prop(optional, into)]
    help: Signal<String>,
    /// Current field validation error, rendered safely and announced.
    #[prop(optional, into)]
    error: Signal<String>,
    /// The actual native form control.
    children: Children,
) -> impl IntoView {
    let help_id = format!("{for_id}-help");
    let error_id = format!("{for_id}-error");
    view! { <div class="sp-field">
        <label for=for_id>{move || label.get()}</label>{children()}
        <p class="sp-help" id=help_id>{move || help.get()}</p>
        <p class="sp-error" id=error_id role="alert">{move || error.get()}</p>
    </div> }
}

/// Safe text-only status/error presentation, without HTML interpretation.
#[component]
pub fn Alert(
    /// The public message; an empty message reserves no visual space.
    #[prop(into)]
    message: Signal<String>,
    /// Controls visible severity and assistive announcement semantics.
    #[prop(default = NoticeKind::Error)]
    kind: NoticeKind,
) -> impl IntoView {
    view! { <p class="sp-alert" role=kind.role() hidden=move || message.get().is_empty()>
        <strong>{kind.label()} ": "</strong>{move || message.get()}
    </p> }
}

/// Announced text loading indicator with no mandatory animation.
#[component]
pub fn Loader(
    /// Whether work is currently pending.
    #[prop(into)]
    busy: Signal<bool>,
    /// Localizable status text.
    #[prop(default = Signal::derive(|| "Loading…".to_owned()), into)]
    label: Signal<String>,
) -> impl IntoView {
    view! { <span class="sp-loader" role="status" aria-live="polite">
        {move || if busy.get() { label.get() } else { String::new() }}
    </span> }
}

/// Keep useful children visible while announcing their in-progress replacement.
#[component]
pub fn LoaderContainer(
    /// Applied to aria-busy on the retained content region.
    #[prop(into)]
    busy: Signal<bool>,
    /// Existing useful content remains mounted while busy.
    children: Children,
) -> impl IntoView {
    view! { <section class="sp-results" aria-busy=move || busy.get().to_string()>{children()}</section> }
}

/// Safe native anchor, usable before hydration and by the host's client router.
#[component]
pub fn Link(
    /// A safe destination; an unsafe target renders a non-navigable anchor.
    #[prop(into)]
    href: String,
    /// Reactive escaped label.
    #[prop(into)]
    label: Signal<String>,
) -> impl IntoView {
    let safe = safe_href(&href);
    view! { <a href=safe.then_some(href) aria-disabled=(!safe).to_string()>{move || label.get()}</a> }
}

/// An owned safe presentation row with a stable application key.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListItem {
    /// Stable key used by Leptos For; callers supply unique keys.
    pub id: String,
    /// Escaped text label.
    pub label: String,
    /// Optional checked destination for a navigable row.
    pub href: Option<String>,
}

/// Keyed native list, with an explicit safe empty state.
#[component]
pub fn List(
    /// Ordered rows; keys must be unique within the collection.
    #[prop(into)]
    items: Signal<Vec<ListItem>>,
    /// Safe localized empty-state direction.
    #[prop(into)]
    empty: Signal<String>,
) -> impl IntoView {
    // Keyed rows retain their DOM identity, while label/href read the latest value.
    // Leptos For does not rerun its child factory when an existing key's props change.
    let current = Memo::new(move |_| {
        items
            .get()
            .into_iter()
            .map(|row| (row.id.clone(), row))
            .collect::<std::collections::BTreeMap<_, _>>()
    });
    view! {
        <p class="sp-empty" hidden=move || !items.get().is_empty()>{move || empty.get()}</p>
        <ul class="sp-list"><For each=move || items.get() key=|row| row.id.clone() children=move |row| {
            let id = row.id;
            let key = id.clone();
            view! { <li data-item-id=id>{move || current.with(|rows| rows.get(&key).cloned()).map(|row| match row.href {
                Some(href) => view! { <Link href label=row.label/> }.into_any(),
                None => row.label.into_any(),
            })}</li> }
        }/></ul>
    }
}

/// Root-scoped immediate notifications with native keyboard dismissal controls.
#[component]
pub fn NotifierContainer(
    /// No global singleton; the owner supplies this root's store.
    notifications: Notifications,
) -> impl IntoView {
    view! { <aside class="sp-notices" aria-label="Notifications" aria-live="polite">
        <For each=move || notifications.entries() key=|entry| entry.id children=move |entry| {
            view! { <div class="sp-notice" role=entry.kind.role()>
                <span><strong>{entry.kind.label()} ": "</strong>{entry.message}</span>
                <button type="button" aria-label="Dismiss notification" on:click=move |_| { notifications.dismiss(entry.id); }>"Dismiss"</button>
            </div> }
        }/>
    </aside> }
}
