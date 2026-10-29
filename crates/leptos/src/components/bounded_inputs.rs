//! Bounded native range/rating controls and explicit slug editing policies.
use super::icons::{IconKind, icon};
use super::{ControlError, Input};
use leptos::prelude::*;

/// Finite native range scale with at most 10,000 equal steps, including both ends.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeScale {
    min: f64,
    max: f64,
    step: f64,
}
impl RangeScale {
    /// Check finite ordered bounds and an exactly representable step count tolerance.
    /// # Errors
    /// Rejects zero/negative/non-finite steps, reversed bounds or excessive/partial steps.
    pub fn new(min: f64, max: f64, step: f64) -> Result<Self, ControlError> {
        // Reject unusable grids before any rounding/clamping can disguise configuration errors.
        let count = (max - min) / step;
        if ![min, max, step, count].iter().all(|n| n.is_finite())
            || min >= max
            || step <= 0.0
            || !(1.0..=10_000.0).contains(&count)
            || (count - count.round()).abs() > 1e-8 * count.max(1.0)
        {
            return Err(ControlError);
        }
        Ok(Self { min, max, step })
    }
    /// Stable lower bound.
    pub fn min(self) -> f64 {
        self.min
    }
    /// Stable upper bound.
    pub fn max(self) -> f64 {
        self.max
    }
    /// Positive native step.
    pub fn step(self) -> f64 {
        self.step
    }
    /// Clamp and snap finite edits relative to min; invalid edits never become zero.
    pub fn snap(self, value: f64) -> Option<f64> {
        if !value.is_finite() {
            return None;
        }
        Some(
            (self.min
                + ((value.clamp(self.min, self.max) - self.min) / self.step).round() * self.step)
                .clamp(self.min, self.max),
        )
    }
    /// Initial values must be in range and already on the selected step grid.
    fn admits(self, value: f64) -> bool {
        self.snap(value).is_some_and(|snapped| {
            value >= self.min && value <= self.max && (snapped - value).abs() <= self.step * 1e-8
        })
    }
}
/// Single and ordered pair range values are explicitly different modes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeValue {
    /// One native handle.
    Single(f64),
    /// Two independently keyboard-operable ordered handles.
    Pair {
        /// Lower inclusive value.
        lower: f64,
        /// Upper inclusive value.
        upper: f64,
    },
}
/// Root-owned checked range state. Its mode/scale cannot change under mounted handles.
#[derive(Clone, Copy)]
pub struct RangeState {
    scale: RangeScale,
    value: RwSignal<RangeValue>,
}
impl RangeState {
    /// Admit initial values without silently clamping invalid configuration.
    /// # Errors
    /// Rejects non-finite/off-grid/out-of-bounds or reversed pair values.
    pub fn new(scale: RangeScale, value: RangeValue) -> Result<Self, ControlError> {
        let valid = match value {
            RangeValue::Single(value) => scale.admits(value),
            RangeValue::Pair { lower, upper } => {
                scale.admits(lower) && scale.admits(upper) && lower <= upper
            }
        };
        if !valid {
            return Err(ControlError);
        }
        Ok(Self {
            scale,
            value: RwSignal::new(value),
        })
    }
    /// Read exact controlled state.
    pub fn value(self) -> RangeValue {
        self.value.get()
    }
    /// Borrow immutable native scale.
    pub fn scale(self) -> RangeScale {
        self.scale
    }
    /// Read an existing handle. Unknown indices never alias the first handle.
    pub fn handle(self, index: usize) -> Option<f64> {
        match (self.value.get(), index) {
            (RangeValue::Single(value), 0) => Some(value),
            (RangeValue::Pair { lower, .. }, 0) => Some(lower),
            (RangeValue::Pair { upper, .. }, 1) => Some(upper),
            _ => None,
        }
    }
    /// Parse/snap one native edit and preserve pair ordering at the transition boundary.
    pub fn edit(self, index: usize, text: &str) -> bool {
        let Some(value) = text
            .parse::<f64>()
            .ok()
            .and_then(|value| self.scale.snap(value))
        else {
            return false;
        };
        // A handle crossing its sibling stops at the sibling without exchanging identities.
        let next = match (self.value.get(), index) {
            (RangeValue::Single(_), 0) => RangeValue::Single(value),
            (RangeValue::Pair { upper, .. }, 0) => RangeValue::Pair {
                lower: value.min(upper),
                upper,
            },
            (RangeValue::Pair { lower, .. }, 1) => RangeValue::Pair {
                lower,
                upper: value.max(lower),
            },
            _ => return false,
        };
        self.value.set(next);
        true
    }
}
/// Labelled native single/pair slider. Arrows/Home/End and step semantics are browser-native.
#[component]
pub fn Slider(
    /// Native field prefix.
    #[prop(into)]
    id: String,
    /// Native legend.
    #[prop(into)]
    label: String,
    /// Native form prefix: a pair uses [lower]/[upper].
    #[prop(into)]
    name: String,
    /// Root-owned checked range state.
    state: RangeState,
    /// Native disabled policy, checked again at event admission.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned error text.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    // The admitted mode fixes handle count and native form-key shape for this mount.
    let pair = matches!(state.value(), RangeValue::Pair { .. });
    let count = if pair { 2 } else { 1 };
    let error_id = format!("{id}-error");
    view! {<fieldset class="sp-range" disabled=move ||disabled.get()><legend>{label}</legend>
        {(0..count).map(|index|{let input=format!("{id}-{index}");let field=if pair{format!("{name}[{}]",if index==0{"lower"}else{"upper"})}else{name.clone()};
            view!{<label for=input.clone()>{if pair{if index==0{"Lower"}else{"Upper"}}else{"Value"}}</label>
                <input id=input.clone() name=field type="range" min=state.scale.min max=state.scale.max step=state.scale.step
                    aria-describedby=error_id.clone() aria-invalid=move ||(!error.get().is_empty()).to_string()
                    value=move ||state.handle(index).unwrap_or(state.scale.min) prop:value=move ||state.handle(index).unwrap_or(state.scale.min)
                    on:input=move |event|{if !disabled.get(){state.edit(index,&event_target_value(&event));}}/>
                <output for=input>{move ||state.handle(index).map(|value|value.to_string()).unwrap_or_default()}</output>}
        }).collect_view()}<p id=error_id.clone() role="alert">{move ||error.get()}</p>
    </fieldset>}
}
/// Bounded root-owned rating; None and the concrete rating zero remain distinct.
#[derive(Clone, Copy)]
pub struct RatingState {
    max: u8,
    value: RwSignal<Option<u8>>,
}
impl RatingState {
    /// Admit an integer maximum 1..=100 and an optional value within 0..=max.
    /// # Errors
    /// Rejects invalid bounds/initial values before rendering radio options.
    pub fn new(max: u8, value: Option<u8>) -> Result<Self, ControlError> {
        if !(1..=100).contains(&max) || value.is_some_and(|value| value > max) {
            return Err(ControlError);
        }
        Ok(Self {
            max,
            value: RwSignal::new(value),
        })
    }
    /// Exact optional controlled rating.
    pub fn value(self) -> Option<u8> {
        self.value.get()
    }
    /// Admit an in-range concrete rating or an explicit cleared value.
    pub fn set(self, value: Option<u8>) -> bool {
        if value.is_some_and(|value| value > self.max) {
            return false;
        }
        self.value.set(value);
        true
    }
}
/// Native labelled radio group with decorative stars and an explicit clear action.
/// `min` restricts rendered/admitted options without changing the shared state contract.
/// Invalid `min > max` renders no options; Clear still represents None.
#[component]
pub fn RatingInput(
    /// Native field prefix.
    #[prop(into)]
    id: String,
    /// Native legend.
    #[prop(into)]
    label: String,
    /// Shared native radio name.
    #[prop(into)]
    name: String,
    /// Controlled bounded state.
    state: RatingState,
    /// Lowest admitted option; zero remains the compatible default.
    #[prop(optional)]
    min: u8,
    /// Native disabled policy.
    #[prop(optional, into)]
    disabled: Signal<bool>,
    /// App-owned validation message.
    #[prop(optional, into)]
    error: Signal<String>,
) -> impl IntoView {
    let error_id = format!("{id}-error");
    // Native checked state drives cumulative fill even before hydration. Each radio
    // keeps its full text label; only its associated label becomes a visual target.
    let options = (min..=state.max).map(|rating| {
        let input = format!("{id}-{rating}");
        view! {<span class="sp-rating-option">
            <input class="sp-sr-only" id=input.clone() type="radio" name=name.clone() value=rating
                checked=move ||state.value()==Some(rating) prop:checked=move ||state.value()==Some(rating)
                on:change=move |event|{
                    if !disabled.get() && event_target_checked(&event) { state.set(Some(rating)); }
                }/>
            <label class="sp-rating-choice" class:sp-rating-zero=rating==0 for=input>
                <span class="sp-sr-only">{format!("{rating} of {}",state.max)}</span>
                {if rating==0 { view!{<span aria-hidden="true">"0"</span>}.into_any() } else { icon(IconKind::Star) }}
            </label>
        </span>}
    }).collect_view();
    view! {<fieldset class="sp-rating-input" aria-describedby=error_id.clone()
        aria-invalid=move ||(!error.get().is_empty()).to_string() disabled=move ||disabled.get()>
        <legend>{label}</legend><div class="sp-rating-options">{options}</div>
        <button type="button" on:click=move |_|{if !disabled.get(){state.set(None);}}>"Clear rating"</button>
        <output>{move ||state.value().map(|value|value.to_string()).unwrap_or_else(||"Unrated".into())}</output>
        <p id=error_id.clone() role="alert">{move ||error.get()}</p>
    </fieldset>}.into_any()
}
/// Explicit separator/case policy; source camelfy preserves the first word's case.
#[derive(Clone, Copy, Debug)]
pub enum SlugStyle {
    /// Lowercase words joined by dashes.
    Dash,
    /// Lowercase words joined by underscores.
    Underscore,
    /// Preserve word case, capitalizing each word after the first.
    Camel,
}
/// Decide whether non-ASCII alphanumeric characters survive; no implicit transliteration.
#[derive(Clone, Copy, Debug)]
pub enum SlugAlphabet {
    /// ASCII alphanumeric words, with explicit separator handling.
    Ascii,
    /// Unicode alphanumeric scalar admission, explicitly selected.
    Unicode,
}
/// Deterministic pure slug transform with an input byte budget.
/// # Errors
/// Rejects inputs beyond 4096 bytes before transforming; no ambient locale is consulted.
pub fn slugify(
    value: &str,
    style: SlugStyle,
    alphabet: SlugAlphabet,
) -> Result<String, ControlError> {
    if value.len() > 4096 {
        return Err(ControlError);
    }
    // Split at an explicit alphabet boundary; no implicit locale/transliteration service.
    let words: Vec<_> = value
        .trim()
        .split(|c: char| !match alphabet {
            SlugAlphabet::Ascii => c.is_ascii_alphanumeric(),
            SlugAlphabet::Unicode => c.is_alphanumeric(),
        })
        .filter(|word| !word.is_empty())
        .collect();
    Ok(match style {
        SlugStyle::Dash => words.join("-").to_lowercase(),
        SlugStyle::Underscore => words.join("_").to_lowercase(),
        SlugStyle::Camel => words
            .into_iter()
            .enumerate()
            .map(|(index, word)| {
                if index == 0 {
                    return word.to_owned();
                }
                let mut chars = word.chars();
                let first = chars.next().expect("nonempty split word");
                format!("{}{}", first.to_uppercase(), chars.as_str())
            })
            .collect(),
    })
}
/// Verbatim controlled draft plus an explicit normalization action; IME/caret is unfiltered.
/// Submission/domain validation remains with the host; typing does not execute transforms.
#[component]
pub fn SlugInput(
    /// Native input prefix.
    #[prop(into)]
    id: String,
    /// Visible label.
    #[prop(into)]
    label: String,
    /// Native draft form name.
    #[prop(into)]
    name: String,
    /// Caller-owned draft, including intermediate/invalid edits.
    value: RwSignal<String>,
    /// Explicit separator/case policy.
    style: SlugStyle,
    /// Explicit Unicode admission policy.
    alphabet: SlugAlphabet,
    /// Native disabled policy.
    #[prop(optional, into)]
    disabled: Signal<bool>,
) -> impl IntoView {
    let error = RwSignal::new(String::new());
    let error_id = format!("{id}-error");
    view! {<div><label for=id.clone()>{label}</label><Input id name value disabled described_by=error_id.clone() invalid=move ||!error.get().is_empty()/>
    <button type="button" disabled=move ||disabled.get() on:click=move |_|{if disabled.get(){return;}match slugify(&value.get(),style,alphabet){Ok(next)=>{value.set(next);error.set(String::new());},Err(_)=>error.set("Slug is too long".into())}}>"Apply slug"</button>
    <p id=error_id.clone() role="alert">{move ||error.get()}</p></div>}
}
