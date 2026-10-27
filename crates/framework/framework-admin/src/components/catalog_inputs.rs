//! Local enum search and bounded numeric/slug controllers behind the role interface.
use super::{
    presence,
    sync::{Admission, connect, rejected},
};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{Input, ValueType};
use stackpress_framework_ui::components::{
    Combobox, ComboboxState, RangeScale, RangeState, RangeValue, RatingInput, RatingState,
    SelectOption, SelectOptions, Slider, SlugAlphabet, SlugInput, SlugStyle,
};

/// Admit finite on-grid numeric props; integer adapters never round fractional values.
pub(crate) fn scale(context: &RoleContext) -> Admission<RangeScale> {
    let min = context.number_prop("min", 0.0);
    let max = context.number_prop("max", 100.0);
    let step = context.number_prop("step", 1.0);
    let integer = context.field.value_type.value == ValueType::Integer;
    range_scale(min, max, step, integer)
}

/// One numeric policy is shared by build-host preflight and runtime seed admission.
pub(crate) fn range_scale(min: f64, max: f64, step: f64, integer: bool) -> Admission<RangeScale> {
    if [min, max, step]
        .iter()
        .any(|n| n.abs() > 9_007_199_254_740_991.0 || integer && n.fract() != 0.0)
    {
        return Err("Invalid numeric scale");
    }
    RangeScale::new(min, max, step).map_err(|_| "Invalid numeric scale")
}

/// Omitted/null sliders present the lower bound without publishing that presentation.
fn number(value: Input<Value>, scale: RangeScale) -> Admission<f64> {
    let number = match value {
        Input::Missing | Input::Null => scale.min(),
        Input::Value(value) => value.as_f64().ok_or("Expected a number")?,
    };
    // Validate without allocating a temporary root-owned signal on every draft transition.
    if !scale.snap(number).is_some_and(|snapped| {
        number >= scale.min()
            && number <= scale.max()
            && (snapped - number).abs() <= scale.step() * 1e-8
    }) {
        return Err("Number is outside the scale");
    }
    Ok(number)
}

/// Native single-handle slider over a finite checked scalar grid.
#[component]
pub fn SliderField(
    /// Public numeric metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let Ok(scale) = scale(&context) else {
        return rejected();
    };
    let Ok(initial) = number(context.value.get_untracked(), scale) else {
        return rejected();
    };
    let state = RangeState::new(scale, RangeValue::Single(initial)).expect("admitted scale/value");
    let integer = context.field.value_type.value == ValueType::Integer;
    let error = connect(
        context.value,
        move || state.handle(0).expect("single handle"),
        move |value| {
            state.edit(0, &value.to_string());
        },
        move |value| number(value, scale),
        move |value| {
            Ok(Input::Value(if integer {
                Value::from(value as i64)
            } else {
                Value::from(value)
            }))
        },
    );
    let presence = presence(&context);
    view! {<div class="sp-role"><Slider id=context.id.clone() label=context.text_prop("label",&context.field.label)
        name=context.field.wire_name.clone() state disabled=context.bool_prop("disabled",false) error/>{presence}</div>}.into_any()
}

/// Rating bounds are integral, finite and small enough for native radio options.
pub(crate) fn rating_max(context: &RoleContext) -> Admission<u8> {
    let max = context.number_prop("max", 5.0);
    rating_limit(max)
}

/// Match codegen and runtime rating limits without truncating fractional maxima.
pub(crate) fn rating_limit(max: f64) -> Admission<u8> {
    if !(1.0..=100.0).contains(&max) || max.fract() != 0.0 {
        return Err("Invalid rating maximum");
    }
    Ok(max as u8)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Exact numbers and finite enum choices never use truthiness or numeric string coercion.
    #[test]
    fn scalar_admission_boundaries_and_absence() {
        let scale = range_scale(-1.0, 1.0, 0.5, false).unwrap();
        assert_eq!(number(Input::Null, scale).unwrap(), -1.0);
        assert_eq!(number(Input::Value(json!(0)), scale).unwrap(), 0.0);
        for value in [json!("0"), json!(false), json!(2), json!(0.1)] {
            assert!(number(Input::Value(value), scale).is_err());
        }
        for (min, max, step, integer) in [
            (0., 1., 0.5, true),
            (0., 0., 1., false),
            (0., 1., 0., false),
            (0., f64::INFINITY, 1., false),
            (0., 1e16, 1., false),
        ] {
            assert!(range_scale(min, max, step, integer).is_err());
        }
        assert_eq!(rating(Input::Missing, 5).unwrap(), None);
        assert_eq!(rating(Input::Value(json!(0)), 5).unwrap(), Some(0));
        for value in [json!(-1), json!(6), json!(0.5), json!("0")] {
            assert!(rating(Input::Value(value), 5).is_err());
        }
        for max in [0., 101., 2.5, f64::NAN] {
            assert!(rating_limit(max).is_err());
        }
        assert_eq!(rating_limit(100.).unwrap(), 100);
        assert_eq!(slug(Input::Null).unwrap(), "");
        assert_eq!(slug(Input::Value(json!("Hello!"))).unwrap(), "Hello!");
        assert!(slug(Input::Value(json!(false))).is_err());
        let choices = vec![json!(""), json!("0")];
        assert_eq!(selection(Input::Null, &choices).unwrap(), None);
        assert_eq!(
            selection(Input::Value(json!("")), &choices).unwrap(),
            Some(String::new())
        );
        assert!(selection(Input::Value(json!(0)), &choices).is_err());
        assert!(selection(Input::Value(json!("unknown")), &choices).is_err());
    }
}

/// Exact integer rating, with absence distinct from the concrete zero option.
fn rating(value: Input<Value>, max: u8) -> Admission<Option<u8>> {
    match value {
        Input::Missing | Input::Null => Ok(None),
        Input::Value(value) => value
            .as_u64()
            .filter(|n| *n <= u64::from(max))
            .map(|n| Some(n as u8))
            .ok_or("Expected an in-range integer rating"),
    }
}

/// Native zero-inclusive rating. Clear means omitted; nullable Null remains explicit.
#[component]
pub fn RatingField(
    /// Public integer metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let Ok(max) = rating_max(&context) else {
        return rejected();
    };
    let Ok(initial) = rating(context.value.get_untracked(), max) else {
        return rejected();
    };
    let state = RatingState::new(max, initial).expect("admitted rating");
    let error = connect(
        context.value,
        move || state.value(),
        move |next| {
            state.set(next);
        },
        move |value| rating(value, max),
        |value| Ok(value.map_or(Input::Missing, |value| Input::Value(Value::from(value)))),
    );
    let presence = presence(&context);
    view! {<div class="sp-role"><RatingInput id=context.id.clone() label=context.text_prop("label",&context.field.label)
        name=context.field.wire_name.clone() state disabled=context.bool_prop("disabled",false) error/>{presence}</div>}.into_any()
}

/// Verbatim text edits and an explicit deterministic ASCII dash slug action.
#[component]
pub fn SlugField(
    /// Public string metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let Ok(initial) = slug(context.value.get_untracked()) else {
        return rejected();
    };
    let value = RwSignal::new(initial);
    // The UI owns normalization; the bridge publishes both typing and explicit Apply actions.
    let draft = context.value;
    let error = connect(
        draft,
        move || value.get(),
        move |next| value.set(next),
        slug,
        |value| Ok(Input::Value(Value::String(value))),
    );
    let presence = presence(&context);
    view! {<div class="sp-role"><SlugInput id=context.id.clone() label=context.text_prop("label",&context.field.label)
        name=context.field.wire_name.clone() value style=SlugStyle::Dash alphabet=SlugAlphabet::Ascii disabled=context.bool_prop("disabled",false)/>
        <p role="alert">{move ||error.get()}</p>{presence}</div>}.into_any()
}

/// Keep slug text verbatim; omission and null only affect its empty presentation.
fn slug(input: Input<Value>) -> Admission<String> {
    match input {
        Input::Missing | Input::Null => Ok(String::new()),
        Input::Value(Value::String(value)) => Ok(value),
        _ => Err("Expected slug text"),
    }
}

/// Exact optional string selection, refusing an enum value absent from the finite catalog.
fn selection(input: Input<Value>, choices: &[Value]) -> Admission<Option<String>> {
    match input {
        Input::Missing | Input::Null => Ok(None),
        Input::Value(Value::String(value))
            if choices.iter().any(|choice| choice.as_str() == Some(&value)) =>
        {
            Ok(Some(value))
        }
        _ => Err("Expected a declared enum choice"),
    }
}

/// Search only the finite schema enum; no inferred endpoint or relation acquisition.
#[component]
pub fn ComboboxField(
    /// Public finite enum metadata and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let choices = context.field.value_type.choices.clone();
    let Ok(initial) = selection(context.value.get_untracked(), &choices) else {
        return rejected();
    };
    let options = SelectOptions::new(
        choices
            .iter()
            .filter_map(Value::as_str)
            .map(|choice| SelectOption {
                value: choice.into(),
                label: choice.into(),
                disabled: false,
            })
            .collect(),
    );
    let Ok(state) = options.and_then(|options| ComboboxState::new(options, initial)) else {
        return rejected();
    };
    let positions = choices.clone();
    let error = connect(
        context.value,
        move || state.selected(),
        move |next| {
            // Clear the search before resolving a known selection so filtering cannot hide it.
            let _ = state.adopt_query(String::new());
            if let Some(next) = next
                && let Some(index) = positions
                    .iter()
                    .position(|choice| choice.as_str() == Some(&next))
            {
                state.choose(index);
            }
            state.cancel();
        },
        move |input| selection(input, &choices),
        |value| Ok(value.map_or(Input::Missing, |value| Input::Value(Value::String(value)))),
    );
    let presence = presence(&context);
    view! {<div class="sp-role"><Combobox id=context.id.clone() label=context.text_prop("label",&context.field.label)
        name=context.field.wire_name.clone() state disabled=context.bool_prop("disabled",false) error/>{presence}</div>}.into_any()
}
