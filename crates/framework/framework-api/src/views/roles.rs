//! Source-specific role glue; every editable field delegates to an existing Leptos control.
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use stackpress_framework_admin::RoleContext;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::{
    Combobox, ComboboxState, FieldControl, Input, SelectOption, SelectOptions, Switch,
};
use std::collections::BTreeMap;

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
    let text = match context.value.get_untracked() {
        Presence::Missing | Presence::Null => String::new(),
        Presence::Value(value) => value
            .as_str()
            .map(str::to_owned)
            .unwrap_or_else(|| value.to_string()),
    };
    let value = RwSignal::new(text);
    let draft = context.value;
    let kind = context.text_prop("type", "text");
    let id = context.id.clone();
    // Shared Input fixes type=text. A URL needs a native typed element inside the same shared
    // frame: spreading a second type attribute makes the browser keep the first text type.
    let control = if kind == "text" {
        view!{<Input id=id.clone() name=context.field.wire_name.clone() value required=context.input_required() on:input=move |event|draft.set(Presence::Value(Value::String(event_target_value(&event))))/>}.into_any()
    } else {
        view!{<input class="sp-input" type=kind id=id.clone() name=context.field.wire_name.clone() value=move ||value.get() prop:value=move ||value.get() required=context.input_required() on:input=move |event| {let text=event_target_value(&event);value.set(text.clone());draft.set(Presence::Value(Value::String(text)));}/>}.into_any()
    };
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
    let options = RwSignal::new(choices);
    let selected = context.value;
    let label = context.field.label.clone();
    let id = context.id.clone();
    let search = context.text_prop("search", "");
    let loader = use_context::<RelationLoader>();
    // Re-keying only the finite choice widget permits refreshed options; the draft value stays
    // root-owned. A failed/late lookup never changes the selected stored ID by itself.
    let revision = RwSignal::new(0_u64);
    view! {
        <div class="relation" on:input=move |event| {
            if let Some(loader) = &loader {
                // Encode the user's text as one URLSearchParams value before substituting
                // the source template. The host owns fetch/cancellation, not this field.
                let query = event_target_value(&event);
                let mut carrier = stackpress_lib::Request::new().url;
                carrier.set_query(None);
                carrier.query_pairs_mut().append_pair("q", &query);
                let encoded = carrier.query().unwrap_or("").trim_start_matches("q=").to_owned();
                let url = if search.contains("{{query}}") {
                    search.replace("{{query}}", &encoded)
                } else {
                    format!("{search}&q={encoded}")
                };
                loader.0.run(RelationRequest {
                    url,
                    field: field.clone(),
                    complete: Callback::new(move |rows| {
                        options.set(rows);
                        revision.update(|revision| *revision += 1);
                    }),
                });
            }
        }>
            {move || {
                // Refresh only the finite choice widget; retain the selected stored ID even
                // when a search response no longer includes it. No automatic draft rewrite.
                let _ = revision.get();
                let choices = options.get();
                let current = match selected.get_untracked() {
                    Presence::Value(Value::String(value)) => Some(value),
                    _ => None,
                };
                let mut entries = choices.iter().map(|choice| SelectOption {
                    value: choice.id.clone(), label: choice.label.clone(), disabled: false,
                }).collect::<Vec<_>>();
                if let Some(current) = &current
                    && !entries.iter().any(|choice| &choice.value == current)
                {
                    entries.push(SelectOption {
                        value: current.clone(), label: current.clone(), disabled: false,
                    });
                }
                // Representation failures remain visible and leave the root draft alone.
                let Ok(entries) = SelectOptions::new(entries) else {
                    return view!{<p role="alert">"Relation choices exceed the UI representation budget"</p>}.into_any();
                };
                let Ok(state) = ComboboxState::new(entries, current) else {
                    return view!{<p role="alert">"Relation selection is unavailable"</p>}.into_any();
                };
                // Subscribe without an immediate write: Missing must remain omitted until
                // the user makes a selection, preserving native optional-field behavior.
                Effect::watch(
                    move || state.selected(),
                    move |value, _, _| selected.set(value.clone().map_or(
                        Presence::Missing, |value| Presence::Value(Value::String(value)),
                    )),
                    false,
                );
                view!{<Combobox id=id.clone() label=label.clone() name=context.field.wire_name.clone() state/>}.into_any()
            }}
        </div>
    }
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
) -> impl IntoView {
    let source = use_context::<Relations>().unwrap_or_default();
    let id = context.text();
    let row = source
        .0
        .get(&context.field.wire_name)
        .and_then(|rows| rows.iter().find(|row| row.id == id));
    let value = row.map(|row| row.label.clone()).unwrap_or(id);
    view! {<span>{value}</span>}
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
    #[cfg(all(not(target_arch = "wasm32"), feature = "ui-ssr"))]
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
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "ui-ssr")))]
    {
        let _ = iso;
        None
    }
    #[cfg(any(target_arch = "wasm32", feature = "ui-ssr"))]
    {
        Some(format!(
            "{minute} {day}, {year} {}:i{}",
            if hour % 12 == 0 { 12 } else { hour % 12 },
            if hour < 12 { "AM" } else { "PM" }
        ))
    }
}
