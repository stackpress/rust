//! Source display metadata adaptation; rendering stays in shared Leptos components.
use super::metadata_child;
use super::props::{appearance, content, prop};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::source::{
    SourceColor, SourceEmailLink, SourceList, SourceRating, SourceSpread, SourceTable,
};
use stackpress_leptos::components::{TextCase, TextTransform};

/// Source array primitive values, preserving repeats, empty strings and zero.
fn values(context: &RoleContext, join: bool) -> Vec<String> {
    match context.value.get() {
        Presence::Value(Value::Array(values)) => values
            .iter()
            .map(|value| {
                if join {
                    if value.is_null() {
                        String::new()
                    } else {
                        super::metadata_string(value)
                    }
                } else {
                    metadata_child(value)
                }
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Raw CSS color presentation through the source-compatible shared component.
#[component]
pub fn ColorView(
    /// Selected source color and authored props.
    context: RoleContext,
) -> impl IntoView {
    let size = if context.bool_prop("sm", false) {
        8
    } else if context.bool_prop("md", false) {
        12
    } else if context.bool_prop("lg", false) {
        16
    } else {
        12
    };
    view! {<SourceColor value=content(&context) box_visible=context.bool_prop("box",true) text=context.bool_prop("text",true) size appearance=appearance(&context)/ >}
}

/// Unvalidated source mailbox link with exact title, target and style props.
#[component]
pub fn EmailView(
    /// Selected source mailbox and authored props.
    context: RoleContext,
) -> impl IntoView {
    view! {<SourceEmailLink value=content(&context) target=context.text_prop("target","") title=context.text_prop("title","") appearance=appearance(&context)/ >}
}

/// Reuse the existing shared CSS text transform; unknown formats leave text unchanged.
#[component]
pub fn TransformView(
    /// Selected source text and CSS format.
    context: RoleContext,
) -> impl IntoView {
    let format = match context.text_prop("format", "").as_str() {
        "uppercase" => TextCase::Uppercase,
        "lowercase" => TextCase::Lowercase,
        "capitalize" => TextCase::Capitalize,
        _ => TextCase::None,
    };
    view! {<TextTransform value=Signal::derive(move ||content(&context)) format/>}
}

/// Source star-only rating with JS Number conversion rather than Typed bounds.
#[component]
pub fn RatingView(
    /// Selected source rating and authored rounding/max props.
    context: RoleContext,
) -> impl IntoView {
    let value = stackpress_framework_schema::__private::source_number(&content(&context));
    let max = prop(&context, "max").and_then(Value::as_f64);
    view! {<SourceRating value max=max remainder=context.bool_prop("remainder",false) round=context.text_prop("round","")/ >}
}

/// Shared separator behavior for spread, comma and line, including fixed props.
#[component]
pub fn SpreadView(
    /// Selected source array and separator/style props.
    context: RoleContext,
) -> impl IntoView {
    view! {<SourceSpread value=values(&context,true) separator=context.text_prop("separator"," ") appearance=appearance(&context)/ >}
}

/// Shared unordered/ordered presentation for list, ol and ul.
#[component]
pub fn ListView(
    /// Selected source list and ordered flag.
    context: RoleContext,
) -> impl IntoView {
    view! {<SourceList value=values(&context,false) ordered=context.bool_prop("ordered",false) appearance=appearance(&context)/ >}
}

/// Source Object.keys(first)/Object.values(each) table, without width normalization.
#[component]
pub fn TabularView(
    /// Selected source object array and stripe/style props.
    context: RoleContext,
) -> impl IntoView {
    let rows = match context.value.get() {
        Presence::Value(Value::Array(rows)) => rows,
        _ => Vec::new(),
    };
    let columns = rows
        .first()
        .and_then(Value::as_object)
        .map(|row| row.keys().cloned().collect())
        .unwrap_or_default();
    let rows = rows
        .iter()
        .map(|row| {
            row.as_object()
                .map(|row| row.values().map(metadata_child).collect())
                .unwrap_or_default()
        })
        .collect();
    let stripes = prop(&context, "stripes")
        .and_then(Value::as_array)
        .map(|values| values.iter().map(super::metadata_string).collect())
        .unwrap_or_default();
    view! {<SourceTable columns rows stripes appearance=appearance(&context)/ >}
}

/// Native/browser Date and Moment-compatible English/short relative presentation.
#[component]
pub fn RelativeView(
    /// Exact source Date argument, preserving numeric millisecond timestamps.
    context: RoleContext,
) -> impl IntoView {
    let value = match context.value.get() {
        Presence::Value(value) => value,
        _ => Value::Null,
    };
    let seconds = relative_seconds(&value);
    let short =
        context.text_prop("format", "") == "a" || context.text_prop("locale", "en") == "short";
    stackpress_leptos::components::source::source_relative(seconds, short)
}

/// Isolate execution-host clocks and Date construction from shared threshold logic.
fn relative_seconds(value: &Value) -> f64 {
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::web_sys::js_sys::Date;
        let date = if let Some(number) = value.as_f64() {
            Date::new(&number.into())
        } else {
            Date::new(&super::metadata_string(value).into())
        };
        (date.get_time() - Date::now()) / 1000.0
    }
    #[cfg(all(not(target_arch = "wasm32"), feature = "source-ssr"))]
    {
        let millis = if value.is_null() {
            Some(0)
        } else if let Some(number) = value.as_f64() {
            Some(number as i64)
        } else {
            value.as_str().and_then(|text| {
                chrono::DateTime::parse_from_rfc3339(text)
                    .map(|date| date.timestamp_millis())
                    .ok()
                    .or_else(|| {
                        chrono::NaiveDate::parse_from_str(text, "%Y-%m-%d")
                            .ok()?
                            .and_hms_opt(0, 0, 0)
                            .map(|date| date.and_utc().timestamp_millis())
                    })
            })
        };
        millis
            .map(|millis| (millis as f64 - chrono::Utc::now().timestamp_millis() as f64) / 1000.0)
            .unwrap_or(f64::NAN)
    }
    #[cfg(all(not(target_arch = "wasm32"), not(feature = "source-ssr")))]
    {
        let _ = value;
        f64::NAN
    }
}
