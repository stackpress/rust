//! Native controlled inputs. Applications own labels, validation and submission.
//!
//! Text edits are published verbatim on native input events, including intermediate
//! IME text; no trimming, parsing or validation runs during composition. Use
//! FieldControl for deterministic label/help/error association. Select options are
//! bounded immutable data; searchable/dynamic option controllers belong to M2.
use leptos::prelude::*;
use std::collections::BTreeSet;

/// Invalid or excessive ordinary-control data, without reflecting input values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ControlError;
impl std::fmt::Display for ControlError {
    /// Public diagnostics never echo application values.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Invalid control options")
    }
}
impl std::error::Error for ControlError {}

/// One native select option; its value remains distinct from its visible label.
#[derive(Clone, Debug)]
pub struct SelectOption {
    /// Exact application value, including an explicitly selected empty string.
    pub value: String,
    /// Escaped visible label.
    pub label: String,
    /// Native disabled option policy.
    pub disabled: bool,
}
/// Validated options with unique values and a bounded render footprint.
#[derive(Clone, Debug)]
pub struct SelectOptions(Vec<SelectOption>);
/// Reserved internal placeholder is disabled and never submitted as user data.
const UNSELECTED: &str = "__stackpress_unselected__";
impl SelectOptions {
    /// Admit at most 100 unique options, each value/label at most 1024 bytes.
    ///
    /// # Errors
    /// Rejects duplicate/reserved values or excessive input before rendering.
    pub fn new(options: Vec<SelectOption>) -> Result<Self, ControlError> {
        let mut values = BTreeSet::new();
        if options.len() > 100
            || options.iter().any(|option| {
                option.value == UNSELECTED
                    || option.value.len() > 1024
                    || option.label.len() > 1024
                    || !values.insert(option.value.clone())
            })
        {
            return Err(ControlError);
        }
        Ok(Self(options))
    }
    /// Borrow the admitted immutable option data.
    pub fn entries(&self) -> &[SelectOption] {
        &self.0
    }
    /// Admit only known enabled values from a native change event, in option order.
    pub fn admit(&self, values: &[String]) -> Vec<String> {
        self.0
            .iter()
            .filter(|option| !option.disabled && values.contains(&option.value))
            .map(|option| option.value.clone())
            .collect()
    }
}
/// Immutable selection mode over explicit caller-owned state.
#[derive(Clone, Copy)]
pub enum SelectValue {
    /// None is unselected; Some("") is an explicitly selected empty value.
    Single(RwSignal<Option<String>>),
    /// Native multiple selection; empty remains distinct from a selected empty value.
    Multiple(RwSignal<Vec<String>>),
}
impl SelectValue {
    /// Read the exact controlled values without a truthiness conversion.
    fn selected(self, key: &str) -> bool {
        match self {
            Self::Single(value) => value.get().as_deref() == Some(key),
            Self::Multiple(value) => value.get().iter().any(|value| value == key),
        }
    }
    /// An unknown single value is visibly unselected, never silently the first option.
    fn unselected(self, options: &SelectOptions) -> bool {
        match self {
            Self::Single(value) => value
                .get()
                .is_none_or(|value| !options.0.iter().any(|option| option.value == value)),
            Self::Multiple(_) => false,
        }
    }
    /// Publish only known enabled selections, preserving the immutable mode.
    pub fn update(self, options: &SelectOptions, values: &[String]) {
        let admitted = options.admit(values);
        match self {
            Self::Single(value) => value.set(admitted.into_iter().next()),
            Self::Multiple(value) => value.set(admitted),
        }
    }
}

/// Native single/multiple select; no custom popup, lookup, retry or implicit coercion.
#[component]
pub fn Select(
    /// Unique native control identity and FieldControl label target.
    #[prop(into)]
    id: String,
    /// Native form key; multiple choices retain repeated keys.
    #[prop(into)]
    name: String,
    /// Caller-owned state and immutable selection mode.
    value: SelectValue,
    /// Unique bounded options, rendered identically on every target.
    options: SelectOptions,
    /// Localizable unselected label, never a valid submitted value.
    #[prop(default = "Choose…".to_owned())]
    placeholder: String,
    /// Deterministic help/error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Current application validation state.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required semantics; domain validation remains host-owned.
    #[prop(optional, into)]
    required: Signal<bool>,
) -> impl IntoView {
    let multiple = matches!(value, SelectValue::Multiple(_));
    let placeholder_attr = options.clone();
    let placeholder_property = options.clone();
    let option_views = options_for_view(value, options.clone());
    #[cfg(target_arch = "wasm32")]
    let event_options = options.clone();
    view! { <select class="sp-input" id=id name=name multiple=multiple
        aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
        disabled=move || disabled.get() required=move || required.get()
        on:change=move |_event| { #[cfg(target_arch="wasm32")] {
            // The DOM bridge supplies values; the native-tested options policy admits them.
            value.update(&event_options, &super::input_dom::selected_values(&_event));
        } }>
        {(!multiple).then(|| view! { <option value=UNSELECTED disabled hidden selected=move || value.unselected(&placeholder_attr) prop:selected=move || value.unselected(&placeholder_property)>{placeholder}</option> })}
        {option_views}
    </select> }
}

/// Native selected attributes seed SSR; properties reconcile subsequent root updates.
fn options_for_view(value: SelectValue, options: SelectOptions) -> impl IntoView {
    options.0.into_iter().map(move |option| {
        let attr = option.value.clone(); let property = attr.clone();
        view! { <option value=option.value disabled=option.disabled selected=move || value.selected(&attr) prop:selected=move || value.selected(&property)>{option.label}</option> }
    }).collect_view()
}

/// Native controlled boolean, retaining false as an explicit value.
#[component]
pub fn Checkbox(
    /// Unique native identity; pair with FieldControl's for_id.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Deterministic help and error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Application validation flag, never a domain validator.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy, including omission from FormData.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required policy; the authoritative host revalidates submissions.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Root-owned checked state, seeded identically on server and client.
    checked: RwSignal<bool>,
    /// Submitted only when checked; defaults to native HTML's "on".
    #[prop(default = "on".to_owned())]
    value: String,
) -> impl IntoView {
    // Native checkbox events preserve Space-key activation and form semantics.
    view! { <input class="sp-check" type="checkbox"  id=id name=name value=value
    checked=move || checked.get() prop:checked=move || checked.get()
    aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
    required=move || required.get() disabled=move || disabled.get()
    on:change=move |event| checked.set(event_target_checked(&event))/> }
}

/// Native controlled boolean, retaining false as an explicit value.
#[component]
pub fn Switch(
    /// Unique native identity; pair with FieldControl's for_id.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Deterministic help and error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Application validation flag, never a domain validator.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy, including omission from FormData.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required policy; the authoritative host revalidates submissions.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Root-owned checked state, seeded identically on server and client.
    checked: RwSignal<bool>,
    /// Submitted only when checked; defaults to native HTML's "on".
    #[prop(default = "on".to_owned())]
    value: String,
) -> impl IntoView {
    // Native checkbox events preserve Space-key activation and form semantics.
    view! { <input class="sp-check" type="checkbox" role="switch" aria-checked=move || checked.get().to_string() id=id name=name value=value
    checked=move || checked.get() prop:checked=move || checked.get()
    aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
    required=move || required.get() disabled=move || disabled.get()
    on:change=move |event| checked.set(event_target_checked(&event))/> }
}

/// Native radio within a caller-labelled fieldset; identical names form one group.
#[component]
pub fn Radio(
    /// Unique native identity; pair with FieldControl's for_id.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Deterministic help and error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Application validation flag, never a domain validator.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy, including omission from FormData.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required policy; the authoritative host revalidates submissions.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Shared group selection. None deliberately leaves every radio unchecked.
    selected: RwSignal<Option<String>>,
    /// This option's exact value, including a meaningful empty string.
    #[prop(into)]
    value: String,
) -> impl IntoView {
    let attr = value.clone();
    let property = value.clone();
    let next = value.clone();
    // Only checked changes select; native arrows move both focus and selection.
    view! { <input class="sp-check" type="radio" id=id name=name value=value
    checked=move || selected.get().as_deref() == Some(attr.as_str())
    prop:checked=move || selected.get().as_deref() == Some(property.as_str())
    aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
    required=move || required.get() disabled=move || disabled.get()
    on:change=move |event| { if event_target_checked(&event) { selected.set(Some(next.clone())); } }/> }
}

/// Verbatim controlled multiline text, with native composition and paste behavior.
#[component]
pub fn Textarea(
    /// Unique native identity; pair with FieldControl's for_id.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Deterministic help and error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Application validation flag, never a domain validator.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy, including omission from FormData.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required policy; the authoritative host revalidates submissions.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Application-owned draft, including empty/Unicode/intermediate IME text.
    value: RwSignal<String>,
    /// Native readonly semantics retain successful form submission.
    #[prop(optional, into)]
    read_only: Signal<bool>,
    /// Visible native row count, clamped to the presentation range 1..=40.
    #[prop(default = 4)]
    rows: u32,
) -> impl IntoView {
    // Leptos treats textarea children as raw RCDATA. Escape the initial HTML
    // explicitly so a hostile closing tag cannot escape the native control.
    // Later edits update only the property and never enter an HTML insertion API.
    let initial = textarea_seed(&value.get_untracked());
    view! { <textarea inner_html=initial class="sp-input" id=id name=name rows=rows.clamp(1,40)
    prop:value=move || value.get() readonly=move || read_only.get()
    aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
    required=move || required.get() disabled=move || disabled.get()
    on:input=move |event| value.set(event_target_value(&event))></textarea> }
}

/// Escape native textarea RCDATA without changing its parsed string value.
fn textarea_seed(value: &str) -> String {
    let escaped = value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    // A document parser discards one leading LF after <textarea>. Fragment
    // construction during CSR does not, so only SSR adds a compensating LF.
    #[cfg(feature = "ssr")]
    let escaped = if value.starts_with('\n') {
        format!("\n{escaped}")
    } else {
        escaped
    };
    escaped
}

/// Controlled password with an accessible, explicit reveal toggle; initially hidden.
#[component]
pub fn PasswordInput(
    /// Unique native identity; pair with FieldControl's for_id.
    #[prop(into)]
    id: String,
    /// Native form key.
    #[prop(into)]
    name: String,
    /// Deterministic help and error element IDs.
    #[prop(optional, into)]
    described_by: Signal<String>,
    /// Application validation flag, never a domain validator.
    #[prop(optional, into)]
    invalid: Signal<bool>,
    /// Native disabled policy, including omission from FormData.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// Native required policy; the authoritative host revalidates submissions.
    #[prop(optional, into)]
    required: Signal<bool>,
    /// Root-owned draft; the caller must never project secrets into public SSR seeds.
    value: RwSignal<String>,
    /// Native readonly semantics; reveal does not enable editing.
    #[prop(optional, into)]
    read_only: Signal<bool>,
    /// Localizable reveal action label.
    #[prop(default = "Show password".to_owned())]
    show_label: String,
    /// Localizable conceal action label.
    #[prop(default = "Hide password".to_owned())]
    hide_label: String,
) -> impl IntoView {
    let revealed = RwSignal::new(false);
    let target = id.clone();
    let focus = id.clone();
    // Change type on the same native node; no duplicate plaintext input or submission key.
    view! { <div class="sp-password">
        <input class="sp-input" type=move || if revealed.get() { "text" } else { "password" }
            id=id name=name value=move || value.get() prop:value=move || value.get()
            autocomplete="current-password" readonly=move || read_only.get()
            aria-describedby=move || described_by.get() aria-invalid=move || invalid.get().to_string()
            required=move || required.get() disabled=move || disabled.get()
            on:input=move |event| value.set(event_target_value(&event))/>
        <button type="button" aria-controls=target aria-pressed=move || revealed.get().to_string()
            disabled=move || disabled.get() on:click=move |_| {
                revealed.update(|value| *value = !*value);
                #[cfg(target_arch="wasm32")] super::dialog_dom::focus_id(&focus);
                #[cfg(not(target_arch="wasm32"))] let _ = &focus;
            }>{move || if revealed.get() { hide_label.clone() } else { show_label.clone() }}</button>
    </div> }
}
