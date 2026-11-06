//! Source small-number specialization of Frui NumberInput (fixed min=0/max=9/step=1).
use super::SourceAppearance;
use leptos::prelude::*;

/// Frui's step=1 sanitizer removes punctuation, clamps bounds and normalizes zeros.
pub fn source_small(value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    let digits = value
        .chars()
        .filter(char::is_ascii_digit)
        .collect::<String>();
    // Sanitization of minus/dot-only tokens yields zero, while unrelated letters yield empty.
    if digits.is_empty() && !value.contains(['-', '.']) {
        return String::new();
    }
    if value.contains('-') {
        return "0".into();
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        "0".into()
    } else if digits.len() > 1 {
        "9".into()
    } else {
        digits.into()
    }
}

/// Live source number control with original clamping, arrow increments and hidden form value.
#[component]
pub fn SourceSmallInput(
    /// Native control identity.
    #[prop(into)]
    id: String,
    /// Hidden successful form key.
    #[prop(into)]
    name: String,
    /// Caller-owned original display value.
    #[prop(into)]
    value: Signal<String>,
    /// Numeric change callback; empty value is NaN, like Frui.
    #[prop(optional)]
    on_update: Option<Callback<f64>>,
    /// Native field requirement.
    #[prop(optional)]
    required: bool,
    /// Authored classes/styles on the visible input.
    #[prop(optional)]
    appearance: SourceAppearance,
) -> impl IntoView {
    let current = RwSignal::new(source_small(&value.get_untracked()));
    Effect::watch(
        move || value.get(),
        move |value, _, _| current.set(source_small(value)),
        false,
    );
    let publish = Callback::new(move |text: String| {
        let next = source_small(&text);
        if next != current.get_untracked() {
            current.set(next.clone());
            if let Some(callback) = on_update {
                callback.run(next.parse().unwrap_or(f64::NAN));
            }
        }
    });
    view! {<input id=id class=format!("sp-input {}",appearance.class_name) style=appearance.style type="text" min="0" max="9" step="1"
    required=required value=move||current.get() prop:value=move||current.get()
    on:input=move|event|publish.run(event_target_value(&event))
    on:keydown=move|event:leptos::ev::KeyboardEvent| {
        if event.key()=="ArrowUp" || event.key()=="ArrowDown" {event.prevent_default();let base=current.get_untracked().parse::<f64>().unwrap_or(0.0);publish.run((base+if event.key()=="ArrowUp"{1.0}else{-1.0}).clamp(0.0,9.0).to_string());}
    }/>
    <input type="hidden" name=name value=move||current.get() prop:value=move||current.get()/ >}
}
