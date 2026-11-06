//! G55 rich-role metadata adaptation; values are never parsed/pretty-printed as JSON.
use super::props::{appearance, content, input_content, prop, style};
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::FieldControl;
use stackpress_leptos::components::source::{
    SourceAppearance, SourceCode, SourceCodeEditor, SourceMarkdown, SourceMarkdownEditor,
    SourceMaskInput, source_formula,
};

/// Code/JSON/Markdown/mask source inputs all delegate to their shared native components.
#[component]
pub fn RichField(
    /// Original source props and caller-owned draft.
    context: RoleContext,
) -> impl IntoView {
    let kind = super::props::component(&context);
    let value = RwSignal::new(if kind.ends_with("code") || kind.ends_with("json") {
        input_content(&context)
    } else {
        content(&context)
    });
    let draft = context.value;
    let callback =
        Callback::new(move |text: String| draft.set(Presence::Value(Value::String(text))));
    let id = context.id.clone();
    let name = super::props::form_name(&context);
    let required = context.input_required();
    let appearance = appearance(&context);
    let control = if kind.ends_with("markdown") {
        view!{<SourceMarkdownEditor id=id.clone() name value rows=context.number_prop("rows",9.0) as u32 required appearance on_update=callback/>}.into_any()
    } else if kind.ends_with("mask") {
        view!{<SourceMaskInput id=id.clone() name value mask=context.text_prop("mask","") required appearance on_update=callback/>}.into_any()
    } else {
        view! {<SourceCodeEditor id=id.clone() name value required appearance on_update=callback/>}
            .into_any()
    };
    view! {<FieldControl for_id=id label=context.field.label>{control}</FieldControl>}
}

/// Code/JSON use plain monospaced text under G55; Markdown uses pulldown-cmark without sanitizer.
#[component]
pub fn RichView(
    /// Source string/structured-value conversion and presentation props.
    context: RoleContext,
) -> impl IntoView {
    if super::props::component(&context).ends_with("markdown") {
        view! {<SourceMarkdown value=Signal::derive(move||content(&context))/>}.into_any()
    } else {
        let numbers = prop(&context, "showLineNumbers")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| prop(&context, "numbers").is_some_and(super::editor::source_truthy));
        let language_appearance = SourceAppearance {
            class_name: context.text_prop("langClassName", ""),
            style: style(prop(&context, "langStyle")),
        };
        view!{<SourceCode value=input_content(&context) language=context.text_prop("language","") numbers show_language=context.bool_prop("showLanguage",false) starting_line_number=context.number_prop("startingLineNumber",1.0) add_default_styles=context.bool_prop("addDefaultStyles",true) appearance=appearance(&context) language_appearance/>}.into_any()
    }
}

/// Arithmetic formulas receive the actual aggregate-generated display row for template filling.
#[component]
pub fn FormulaView(
    /// Selected formula and source value.
    context: RoleContext,
    /// Actual complete source row, supplied by generated view/list wrappers.
    #[prop(optional)]
    row: Option<Value>,
) -> impl IntoView {
    let value = match context.value.get() {
        Presence::Value(value) => number(&value),
        _ => 0.0,
    };
    let data = row
        .as_ref()
        .and_then(Value::as_object)
        .map(|row| {
            row.iter()
                .map(|(key, value)| (key.clone(), number(value)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let result = source_formula(&context.text_prop("formula", ""), value, &data);
    if result.is_nan() {
        "NaN".into()
    } else if result.is_infinite() {
        if result.is_sign_negative() {
            "-Infinity".into()
        } else {
            "Infinity".into()
        }
    } else {
        stackpress_framework_schema::__private::source_number_text(result)
    }
}

/// JavaScript Number conversion before Frui's ||0 substitution.
fn number(value: &Value) -> f64 {
    match value {
        Value::Null => 0.0,
        Value::Bool(value) => {
            if *value {
                1.0
            } else {
                0.0
            }
        }
        Value::Number(value) => value.as_f64().unwrap_or(f64::NAN),
        Value::String(value) => stackpress_framework_schema::__private::source_number(value),
        Value::Array(_) => {
            stackpress_framework_schema::__private::source_number(&super::metadata_string(value))
        }
        Value::Object(_) => f64::NAN,
    }
}
