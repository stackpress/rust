//! Source-specific role glue; every editable field delegates to an existing Leptos control.
use crate::RoleContext;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::{
    Combobox, ComboboxState, FieldControl, Input, SelectOption, SelectOptions, Switch,
};
use std::collections::BTreeMap;

mod countries;
mod display;
mod fieldset;
mod inputs;
mod lists;
mod media;
mod props;
mod rich;
/// Existing source stringlist retains its established shared collection contract.
pub use crate::components::TextListField;
pub use countries::{ColorField, CountryField, CountryView};
pub use fieldset::{FieldsetField, FieldsetView, fieldset_context};
pub use inputs::{FileField, RadioField, SmallField, SourceUploader, SuggestField};
pub use lists::SourceTextListField;
pub use media::MediaView;
pub use rich::{FormulaView, RichField, RichView};
mod editor;
pub use display::{
    ColorView, EmailView, ListView, RatingView, RelativeView, SpreadView, TabularView,
    TransformView,
};
pub use editor::{EditorField, HtmlView};

/// A source relation choice, loaded through the configured model search event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    /// Actual stored identity, submitted unchanged.
    pub id: String,
    /// Source template label.
    pub label: String,
    /// Original joined row, retained for source display templates.
    pub row: Value,
}
/// Request/root-local joined rows and relation choices; never a caller identity slot.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Relations(pub BTreeMap<String, Vec<Choice>>);
/// Host/browser substitution boundary for a source relation search URL.
#[derive(Clone)]
pub struct RelationLoader(pub Callback<RelationRequest>);
/// One borrowing-free relation search intent and its root-local completion callback.
#[derive(Clone)]
pub struct RelationRequest {
    /// Original metadata search URL with query marker substituted by the adapter.
    pub url: String,
    /// Destination field, so simultaneous selectors retain distinct results.
    pub field: String,
    /// Owned completion returns choices to only this mounted adapter.
    pub complete: Callback<Vec<Choice>>,
}
/// Original text/url input type, using shared Input and FieldControl rather than a new field.
#[component]
pub fn InputField(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
) -> impl IntoView {
    let text = props::input_content(&context);
    let value = RwSignal::new(text);
    let draft = context.value;
    let kind = context.text_prop("type", "text");
    // Earlier non-text source adapters emitted native inputs without empty Typed associations.
    let omit_empty_associations = kind != "text";
    let id = context.id.clone();
    let appearance = props::appearance(&context);
    // The shared native Input already owns type, value, IME and successful-form behavior.
    // Forward source appearance through explicit props so SSR never emits duplicate attributes.
    let control = view! {<Input input_type=kind omit_empty_associations id=id.clone() name=props::form_name(&context) value required=context.input_required()
    class_name=appearance.class_name style=(!appearance.style.is_empty()).then_some(appearance.style)
    on:input=move|event|draft.set(Presence::Value(Value::String(event_target_value(&event))))/>};
    view! {<FieldControl for_id=id label=context.field.label.clone()>{control}</FieldControl>}
}
/// Original Boolean filter: one switch, without a generated presence-action selector.
#[component]
pub fn SwitchField(
    /// Source filter field and its root-owned omission/Boolean value.
    context: RoleContext,
) -> impl IntoView {
    let value = context.value;
    let checked = RwSignal::new(matches!(
        value.get_untracked(),
        Presence::Value(Value::Bool(true))
    ));
    // Missing remains omitted until the user operates the switch, like the source control.
    let name = if context.role == stackpress_framework_schema::Role::Filter {
        format!("eq[{}]", context.field.wire_name)
    } else {
        context.field.wire_name.clone()
    };
    view! {<FieldControl for_id=context.id.clone() label=context.field.label>
        <Switch id=context.id name=name checked on:change=move |event| {
            value.set(Presence::Value(Value::Bool(event_target_checked(&event))));
        }/>
    </FieldControl>}
}
/// Source relation selection: actual search URL/template metadata and shared Combobox state.
#[component]
pub fn RelationField(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
) -> impl IntoView {
    let source = use_context::<Relations>().unwrap_or_default();
    let field = context.field.wire_name.clone();
    let choices = source.0.get(&field).cloned().unwrap_or_default();
    let selected = context.value;
    let current = match selected.get_untracked() {
        Presence::Value(Value::String(value)) => Some(value),
        _ => None,
    };
    let Ok(entries) = relation_options(choices, current.as_deref()) else {
        return view! {<p role="alert">"Relation choices exceed the UI representation budget"</p>}
            .into_any();
    };
    let Ok(state) = ComboboxState::new(entries, current) else {
        return view! {<p role="alert">"Relation selection is unavailable"</p>}.into_any();
    };
    let search = context.text_prop("search", "");
    let loader = use_context::<RelationLoader>();
    let error = RwSignal::new(String::new());
    // One controller owns one mounted input. Lookup completion replaces only choices, so
    // the native focus, query and open popup survive asynchronous search responses.
    let lookup = Callback::new(move |query: String| {
        if state.edit(query.clone()).is_err() {
            return;
        }
        if let Some(loader) = &loader {
            // Encode text as a single query value; application code owns fetch/cancellation.
            let mut carrier = stackpress_lib::Request::new().url;
            carrier.set_query(None);
            carrier.query_pairs_mut().append_pair("q", &query);
            let encoded = carrier
                .query()
                .unwrap_or("")
                .trim_start_matches("q=")
                .to_owned();
            let url = if search.contains("{{query}}") {
                search.replace("{{query}}", &encoded)
            } else {
                format!("{search}&q={encoded}")
            };
            loader.0.run(RelationRequest {
                url,
                field: field.clone(),
                complete: Callback::new(move |rows| {
                    // A lookup never deletes an existing selected ID merely because the
                    // response omits it. Representation failures leave the old choices intact.
                    let current = state.selected();
                    let replaced = relation_options(rows, current.as_deref())
                        .and_then(|entries| state.replace_options(entries));
                    error.set(if replaced.is_ok() {
                        String::new()
                    } else {
                        "Relation choices exceed the UI representation budget".into()
                    });
                }),
            });
        }
    });
    // Missing stays omitted until a real edit or selection; mounting never rewrites the draft.
    Effect::watch(
        move || state.selected(),
        move |value, _, _| {
            selected.set(value.clone().map_or(Presence::Missing, |value| {
                Presence::Value(Value::String(value))
            }))
        },
        false,
    );
    view! {
        <div class="relation">
            <Combobox id=context.id.clone() label=context.field.label.clone()
                name=props::form_name(&context) state lookup/>
            <p role="alert">{move || error.get()}</p>
        </div>
    }
    .into_any()
}

/// A response may omit the selected ID; preserve it as an explicit ordinary choice.
fn relation_options(
    choices: Vec<Choice>,
    current: Option<&str>,
) -> Result<SelectOptions, stackpress_leptos::components::ControlError> {
    let mut entries = choices
        .into_iter()
        .map(|choice| SelectOption {
            value: choice.id,
            label: choice.label,
            disabled: false,
        })
        .collect::<Vec<_>>();
    if let Some(current) = current
        && !entries.iter().any(|choice| choice.value == current)
    {
        entries.push(SelectOption {
            value: current.into(),
            label: current.into(),
            disabled: false,
        });
    }
    SelectOptions::new(entries)
}
/// Source image dimensions and URL, with normal escaped native attributes.
#[component]
pub fn ImageView(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
) -> impl IntoView {
    // Frui Image forwards dimensions, but omits src for an empty optional value.
    let src = match context.value.get() {
        Presence::Value(Value::String(value)) if !value.is_empty() => Some(value),
        _ => None,
    };
    // The original generated optional-value guard skips this component entirely when empty.
    // A nonempty URL still retains source dimensions if loading fails.
    src.map(|src|view! {<img src=src width=context.number_prop("width",100.0) height=context.number_prop("height",100.0) alt=context.field.label/>})
}
/// Source link target and text; no additional destination admission policy is introduced.
#[component]
pub fn LinkView(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
) -> impl IntoView {
    // Mirror the source generator's optional-value guard, preserving nonempty raw hrefs.
    let text = match context.value.get() {
        Presence::Value(Value::String(value)) if !value.is_empty() => Some(value),
        _ => None,
    };
    text.map(|text| {
        let href = text.clone();
        view! {<a href=href target=context.text_prop("target","")>{text}</a>}
    })
}
/// Source tag presentation reuses shared text/tag value behavior and preserves ordinary values.
#[component]
pub fn TagsView(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
) -> impl IntoView {
    let values = match context.value.get() {
        Presence::Value(Value::Array(values)) => values,
        _ => Vec::new(),
    };
    // Frui Tags makes one Badge per item; source warning/pill/margin props stay explicit.
    let classes = format!(
        "api-tag {} {} {}",
        context.text_prop("className", ""),
        if context.bool_prop("warning", false) {
            "api-tag-warning"
        } else {
            ""
        },
        if context.bool_prop("pill", false) {
            "api-tag-pill"
        } else {
            ""
        }
    );
    view! {<span class="api-tags">{values.into_iter().map(|value|view!{<span class=classes.clone()>{value.as_str().map(str::to_owned).unwrap_or_else(||value.to_string())}</span>}).collect_view()}</span>}
}
/// Resolve source joined display templates against actual related rows, never executable code.
#[component]
pub fn TemplateView(
    /// Generated field metadata, draft value and fixed source arguments.
    context: RoleContext,
    /// Source-generated displays supply their actual joined row; legacy callers may
    /// continue providing the existing relation-choice context independently.
    #[prop(optional)]
    row: Option<Value>,
) -> impl IntoView {
    if let Some(row) = row.as_ref()
        && let Some(text) = joined_template(&context.text_prop("template", ""), row)
    {
        return view! {<span>{text}</span>}.into_any();
    }
    let source = use_context::<Relations>().unwrap_or_default();
    let id = context.text();
    let row = source
        .0
        .get(&context.field.wire_name)
        .and_then(|rows| rows.iter().find(|row| row.id == id));
    let value = row.map(|row| row.label.clone()).unwrap_or(id);
    view! {<span>{value}</span>}.into_any()
}
/// Substitute only declared data paths, without evaluating expressions or HTML.
/// Missing paths retain the existing relation-choice fallback rather than inventing data.
fn joined_template(template: &str, row: &Value) -> Option<String> {
    let mut remaining = template;
    let mut text = String::new();
    while let Some((before, after)) = remaining.split_once("{{") {
        text.push_str(before);
        let (path, tail) = after.split_once("}}")?;
        let value = path
            .trim()
            .split('.')
            .try_fold(row, |value, key| value.get(key))?;
        text.push_str(
            &value
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| value.to_string()),
        );
        remaining = tail;
    }
    text.push_str(remaining);
    Some(text)
}
/// Preserve the source Moment spelling, including minute `m`, weekday `d` and literal `i`.
#[component]
pub fn DateView(
    /// Generated field metadata and canonical model Date value.
    context: RoleContext,
) -> impl IntoView {
    let iso = context.text();
    let text = source_date(&iso).unwrap_or_else(|| "Invalid date".into());
    view! {<time datetime=iso>{text}</time>}
}
/// Native local time and browser local time mirror the two original Moment execution hosts.
fn source_date(iso: &str) -> Option<String> {
    #[cfg(all(not(target_arch = "wasm32"), feature = "source-ssr"))]
    let (year, day, hour, minute) = {
        use chrono::{Datelike, Timelike};
        let date = chrono::DateTime::parse_from_rfc3339(iso)
            .ok()?
            .with_timezone(&chrono::Local);
        (
            date.year(),
            date.weekday().num_days_from_sunday(),
            date.hour(),
            date.minute(),
        )
    };
    #[cfg(target_arch = "wasm32")]
    let (year, day, hour, minute) = {
        let date = leptos::web_sys::js_sys::Date::new(&iso.into());
        if date.get_time().is_nan() {
            return None;
        }
        (
            date.get_full_year() as i32,
            date.get_day(),
            date.get_hours(),
            date.get_minutes(),
        )
    };
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "source-ssr")))]
    {
        let _ = iso;
        None
    }
    #[cfg(any(target_arch = "wasm32", feature = "source-ssr"))]
    {
        Some(format!(
            "{minute} {day}, {year} {}:i{}",
            if hour % 12 == 0 { 12 } else { hour % 12 },
            if hour < 12 { "AM" } else { "PM" }
        ))
    }
}

/// Source form/field.ts:139–141 applies String(val) before calling Frui Metadata.
/// Keep that presentation conversion separate from the exact native draft until an edit.
#[component]
pub fn MetadataField(
    /// Source metadata, authored add label and the root-owned original value.
    context: RoleContext,
) -> impl IntoView {
    let draft = context.value;
    let presented = RwSignal::new(metadata_entries(draft.get_untracked()));
    let published = StoredValue::new(presented.get_untracked());
    // Programmatic resets refresh the text editor without echoing a coercion into the draft.
    Effect::watch(
        move || draft.get(),
        move |value, _, _| {
            let value = metadata_entries(value.clone());
            if presented.get_untracked() != value {
                published.set_value(value.clone());
                presented.set(value);
            }
        },
        false,
    );
    // A real editor update publishes the source's string-valued object exactly once.
    Effect::watch(
        move || presented.get(),
        move |value, _, _| {
            if published.get_value() != *value {
                published.set_value(value.clone());
                draft.set(value.clone());
            }
        },
        false,
    );
    let context = RoleContext {
        value: presented,
        ..context
    };
    view! {<crate::components::MetadataField context/>}
}

/// Object.entries also accepts arrays; missing/null remain presentation-only empty states.
fn metadata_entries(value: Presence<Value>) -> Presence<Value> {
    let entries = match value {
        Presence::Value(Value::Object(values)) => values.into_iter().collect::<Vec<_>>(),
        Presence::Value(Value::Array(values)) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value))
            .collect(),
        value => return value,
    };
    Presence::Value(Value::Object(
        entries
            .into_iter()
            .map(|(key, value)| (key, Value::String(metadata_string(&value))))
            .collect(),
    ))
}

/// JavaScript String(val), including null, array joining and plain-object spelling.
fn metadata_string(value: &Value) -> String {
    match value {
        Value::Null => "null".into(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => stackpress_framework_schema::__private::source_number_text(
            value.as_f64().expect("JSON number"),
        ),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(|value| {
                if value.is_null() {
                    String::new()
                } else {
                    metadata_string(value)
                }
            })
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
    }
}

/// Source select options reuse the shared select with the authored choice order and labels.
#[component]
pub fn SelectField(
    /// Source options and caller-owned selected value.
    mut context: RoleContext,
) -> impl IntoView {
    if let Some(options) = context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role)
        .and_then(|binding| binding.props.get("options"))
    {
        context.field.value_type.choices = options.as_array().cloned().unwrap_or_default();
    }
    view! {<crate::components::SelectField context/>}
}

/// Source clip/overflow presentation; metadata remains text and never executable markup.
#[component]
pub fn OverflowView(
    /// Source value and finite clip arguments.
    context: RoleContext,
) -> impl IntoView {
    let text = context.text();
    let length = context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role)
        .and_then(|binding| binding.props.get("length"));
    let count = length
        .and_then(|value| {
            value.as_f64().or_else(|| {
                value
                    .as_str()
                    .map(stackpress_framework_schema::__private::source_number)
            })
        })
        .unwrap_or(0.0);
    stackpress_leptos::components::source::source_overflow(
        &text,
        count,
        context.bool_prop("words", false),
        context.bool_prop("hellip", false),
    )
}
/// Source Metadata is a two-column table in object insertion order (Frui Metadata.tsx:23-33).
#[component]
pub fn MetadataView(
    /// Source object value and ordinary styling props.
    context: RoleContext,
) -> impl IntoView {
    let values = match context.value.get() {
        Presence::Missing | Presence::Null | Presence::Value(Value::Null) => return ().into_any(),
        Presence::Value(Value::Object(values)) => values,
        Presence::Value(Value::Array(values)) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| (index.to_string(), value))
            .collect(),
        Presence::Value(Value::String(value)) => value
            .encode_utf16()
            .enumerate()
            .map(|(index, unit)| {
                (
                    index.to_string(),
                    Value::String(String::from_utf16_lossy(&[unit])),
                )
            })
            .collect(),
        Presence::Value(_) => Default::default(),
    };
    let class = context.text_prop("className", "");
    let rows = values
        .into_iter()
        .map(|(key, value)| {
            let class = class.clone();
            // The source component's declared children are strings/numbers. Like React,
            // an object child is a rendering failure, not an invented JSON display.
            let text = metadata_child(&value);
            view! {<tr><td class=class.clone()>{key}</td><td class=class>{text}</td></tr>}
        })
        .collect_view();
    view! {<table><tbody>{rows}</tbody></table>}.into_any()
}

/// React's primitive/array children are text; an object child fails source rendering.
fn metadata_child(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Number(value) => stackpress_framework_schema::__private::source_number_text(
            value.as_f64().expect("JSON number"),
        ),
        Value::Null | Value::Bool(_) => String::new(),
        Value::Array(values) => values.iter().map(metadata_child).collect(),
        Value::Object(_) => panic!("source metadata object is not a renderable child"),
    }
}
