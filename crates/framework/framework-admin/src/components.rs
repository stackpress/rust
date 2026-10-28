//! Concrete native controls behind the uniform generated [`RoleContext`] interface.
//!
//! The typed catalog limits selection to implemented controls. Values are JSON data at the
//! draft boundary; each control reads only its declared scalar family, without coercion.

use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::{Input as FieldInput, ValueType};
use stackpress_framework_ui::components::{
    Checkbox, FieldControl, Input, Select, SelectOption, SelectOptions, SelectValue, Switch,
    TextCase, TextTransform, Textarea, YesNo,
};

use crate::RoleContext;

mod catalog_inputs;
mod collection_inputs;
#[cfg(feature = "format")]
mod formatted;
mod sync;
pub use catalog_inputs::{ComboboxField, RatingField, SliderField, SlugField};
#[cfg(feature = "codegen")]
pub(crate) use catalog_inputs::{range_scale, rating_limit, rating_minimum};
pub use collection_inputs::{MetadataField, TextListField};
#[cfg(all(feature = "format", feature = "codegen"))]
pub(crate) use formatted::configuration as formatted_configuration;
#[cfg(feature = "format")]
pub use formatted::{DecimalField, FormattedView, PhoneField, TemporalField};

/// Read exact text, leaving absent/null values distinct in the source signal.
fn text_seed(context: &RoleContext) -> String {
    match context.value.get_untracked() {
        FieldInput::Value(Value::String(value)) => value,
        _ => String::new(),
    }
}

/// Reconcile programmatic draft changes without promoting omission/null to empty text.
fn text_signal(context: &RoleContext) -> RwSignal<String> {
    let text = RwSignal::new(text_seed(context));
    let value = context.value;
    // Do not overwrite Missing/Null during initial hydration just because the DOM is empty.
    Effect::watch(
        move || value.get(),
        move |input, _, _| {
            let next = match input {
                FieldInput::Value(Value::String(value)) => value.clone(),
                _ => String::new(),
            };
            if text.get_untracked() != next {
                text.set(next);
            }
        },
        false,
    );
    text
}

/// Explicit input presence selector; clearing a text box remains a supplied empty string.
fn presence(context: &RoleContext) -> AnyView {
    let patch = use_context::<crate::pages::EditorAction>()
        .is_some_and(|action| action.0 == crate::pages::Action::Update);
    let omission = context.role == stackpress_framework_schema::Role::Filter || patch;
    if !context.field.nullable && !omission {
        return ().into_any();
    }
    let state = context.value;
    let field = context.field.clone();
    let id = format!("{}-presence", context.id);
    let label = format!("{} action", context.field.label);
    let disabled = context.bool_prop("disabled", false);
    let empty_number = context.number_prop("min", 0.0);
    let missing_label = if patch {
        "Leave unchanged"
    } else if omission {
        "Any value"
    } else {
        "Not set"
    };
    view! { <label for=id.clone()>{label}</label>
        <select id=id disabled=disabled on:change=move |event| {
            if disabled { return; }
            let next = match event_target_value(&event).as_str() {
                "missing" => FieldInput::Missing,
                "null" if field.nullable => FieldInput::Null,
                _ => FieldInput::Value(if field.value_type.multiple { Value::Array(Vec::new()) } else { match field.value_type.value {
                    ValueType::Boolean => Value::Bool(false),
                    ValueType::Integer => Value::from(empty_number as i64),
                    ValueType::Number => Value::from(empty_number),
                    ValueType::Object => Value::Object(Default::default()),
                    _ => Value::String(String::new()),
                }}),
            };
            state.set(next);
        }>
            <option value="missing" selected=move || matches!(state.get(), FieldInput::Missing) prop:selected=move || matches!(state.get(), FieldInput::Missing)>{missing_label}</option>
            {context.field.nullable.then(|| view! { <option value="null" selected=move || matches!(state.get(), FieldInput::Null) prop:selected=move || matches!(state.get(), FieldInput::Null)>"Clear (empty)"</option> })}
            <option value="value" selected=move || matches!(state.get(), FieldInput::Value(_)) prop:selected=move || matches!(state.get(), FieldInput::Value(_))>"Set value"</option>
        </select>
    }.into_any()
}

/// Pair a generated control with deterministic label/help/error associations.
fn frame(context: RoleContext, control: AnyView) -> AnyView {
    let id = context.id.clone();
    let label = Signal::stored(context.text_prop("label", &context.field.label));
    let state = presence(&context);
    let error = field_error(&context);
    view! { <div class="sp-role" data-role=format!("{:?}", context.role).to_lowercase()>
        <FieldControl for_id=id label error>{control}</FieldControl>{state}
    </div> }
    .into_any()
}

/// One public-rule error path shared by ordinary and formatted controls.
fn field_error(context: &RoleContext) -> Signal<String> {
    let errors = use_context::<crate::pages::FormIssues>();
    // Host errors describe the submitted values even when this editable control has moved on.
    let submitted =
        use_context::<crate::pages::EditorAction>().is_some_and(|action| action.0.writes());
    let name = context.field.wire_name.clone();
    let field = context.field.clone();
    Signal::derive(move || {
        errors
            .map(|errors| {
                errors
                    .0
                    .get()
                    .iter()
                    .filter(|issue| issue.path.first() == Some(&name))
                    .map(|issue| {
                        let message = crate::presentation::issue_message(&field, issue);
                        if submitted {
                            format!("Submitted version: {message}")
                        } else {
                            message
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            })
            .unwrap_or_default()
    })
}

/// Native text control. Public props are typed by the build-host catalog.
#[component]
pub fn TextField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    let value = text_signal(&context);
    let draft = context.value;
    let control = view! { <Input id=context.id.clone() name=context.field.wire_name.clone() value
    described_by=format!("{}-help {}-error", context.id, context.id)
    required=context.input_required() disabled=context.bool_prop("disabled", false)
    read_only=context.bool_prop("read_only", false) placeholder=context.text_prop("placeholder", "")
    on:input=move |event| draft.set(FieldInput::Value(Value::String(event_target_value(&event))))/> };
    frame(context, control.into_any())
}

/// Native multiline text, using the UI library's safe RCDATA implementation.
#[component]
pub fn TextareaField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    let value = text_signal(&context);
    let draft = context.value;
    let control = view! { <Textarea id=context.id.clone() name=context.field.wire_name.clone() value
    described_by=format!("{}-help {}-error", context.id, context.id)
    required=context.input_required() disabled=context.bool_prop("disabled", false)
    read_only=context.bool_prop("read_only", false)
    on:input=move |event| draft.set(FieldInput::Value(Value::String(event_target_value(&event))))/> };
    frame(context, control.into_any())
}

/// Shared exact Boolean bridge for checkbox and switch presentation.
fn boolean(context: RoleContext, switch: bool) -> AnyView {
    let checked = RwSignal::new(matches!(
        context.value.get_untracked(),
        FieldInput::Value(Value::Bool(true))
    ));
    let value = context.value;
    // Explicit false is a value; initial Missing and Null are not rewritten.
    Effect::watch(
        move || value.get(),
        move |input, _, _| {
            let next = matches!(input, FieldInput::Value(Value::Bool(true)));
            if checked.get_untracked() != next {
                checked.set(next);
            }
        },
        false,
    );
    let id = context.id.clone();
    let name = context.field.wire_name.clone();
    let description = format!("{id}-help {id}-error");
    let disabled = context.bool_prop("disabled", false);
    let required = context.input_required();
    let control = if switch {
        view! { <Switch id name checked described_by=description disabled required on:change=move |event| value.set(FieldInput::Value(Value::Bool(event_target_checked(&event))))/> }.into_any()
    } else {
        view! { <Checkbox id name checked described_by=description disabled required on:change=move |event| value.set(FieldInput::Value(Value::Bool(event_target_checked(&event))))/> }.into_any()
    };
    frame(context, control)
}

/// Native checkbox, preserving supplied false independently of omission.
#[component]
pub fn CheckboxField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    boolean(context, false)
}

/// Native switch with the same exact Boolean draft behavior as checkbox.
#[component]
pub fn SwitchField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    boolean(context, true)
}

/// Finite schema enum choices; no network lookup or inferred relation capability.
#[component]
pub fn SelectField(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    if context.bool_prop("searchable", false) {
        return view! { <ComboboxField context/> }.into_any();
    }
    let selected = RwSignal::new(match context.value.get_untracked() {
        FieldInput::Value(Value::String(value)) => Some(value),
        _ => None,
    });
    let value = context.value;
    Effect::watch(
        move || value.get(),
        move |input, _, _| {
            let next = match input {
                FieldInput::Value(Value::String(value)) => Some(value.clone()),
                _ => None,
            };
            if selected.get_untracked() != next {
                selected.set(next);
            }
        },
        false,
    );
    let options = SelectOptions::new(
        context
            .field
            .value_type
            .choices
            .iter()
            .filter_map(|choice| choice.as_str())
            .map(|choice| SelectOption {
                value: choice.to_owned(),
                label: choice.to_owned(),
                disabled: false,
            })
            .collect(),
    );
    // The emitter checks this bound before writing output; direct callers still fail closed.
    let Ok(options) = options else {
        return view! { <p role="alert">"Invalid choices"</p> }.into_any();
    };
    let choices = context.field.value_type.choices.clone();
    let control = view! { <Select id=context.id.clone() name=context.field.wire_name.clone()
    value=SelectValue::Single(selected) options placeholder=context.text_prop("placeholder", "Choose…")
    described_by=format!("{}-help {}-error", context.id, context.id)
    disabled=context.bool_prop("disabled", false) required=context.input_required()
    on:change=move |event| {
        let next = event_target_value(&event);
        if choices.iter().any(|choice| choice.as_str() == Some(&next)) { value.set(FieldInput::Value(Value::String(next))); }
    }/> };
    frame(context, control.into_any()).into_any()
}

/// Plain escaped display; no HTML, format strings, or executable template evaluation.
#[component]
pub fn TextView(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    view! { <span>{move || context.text()}</span> }
}

/// Exact Boolean display with explicit translated labels and separate missing/null states.
#[component]
pub fn BooleanView(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    let yes_label = context.text_prop("yes_label", "Yes");
    let no_label = context.text_prop("no_label", "No");
    move || match context.value.get() {
        FieldInput::Value(Value::Bool(value)) => {
            view! { <YesNo value yes=yes_label.clone() no=no_label.clone()/> }.into_any()
        }
        _ => view! { <span>{context.text()}</span> }.into_any(),
    }
}

/// Finite CSS casing over escaped source text, never a metadata-supplied style fragment.
#[component]
pub fn CaseView(
    /// Approved field metadata and root-owned state from the generated caller.
    context: RoleContext,
) -> impl IntoView {
    let format = match context.text_prop("case", "none").as_str() {
        "uppercase" => TextCase::Uppercase,
        "lowercase" => TextCase::Lowercase,
        "capitalize" => TextCase::Capitalize,
        _ => TextCase::None,
    };
    view! { <TextTransform value=Signal::derive(move || context.text()) format/> }
}
