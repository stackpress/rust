//! Source metadata adapters for the shared editor and raw HTML display.
use crate::RoleContext;
use leptos::prelude::*;
use serde_json::Value;
use stackpress_framework_schema::Input as Presence;
use stackpress_leptos::components::{FieldControl, HTML, TextEditor, TextEditorOptions};

/// Source field.editor passes its exact toolbar choices into the shared Leptos editor.
#[component]
pub fn EditorField(
    /// Selected role metadata and caller-owned HTML draft.
    context: RoleContext,
) -> impl IntoView {
    let binding = context
        .field
        .roles
        .iter()
        .find(|binding| binding.role == context.role);
    let prop = |name: &str| binding.and_then(|binding| binding.props.get(name));
    let flag = |name: &str| prop(name).is_some_and(source_truthy);
    let options = TextEditorOptions {
        history: flag("history"),
        font: flag("font"),
        size: flag("size"),
        format: flag("format"),
        paragraph: flag("paragraph"),
        blockquote: flag("blockquote"),
        style: flag("style"),
        color: flag("color"),
        highlight: flag("highlight"),
        text: flag("text"),
        text_style: flag("textStyle"),
        remove: flag("remove"),
        indent: flag("indent"),
        align: flag("align"),
        rule: flag("rule"),
        list: flag("list"),
        lineheight: flag("lineheight"),
        table: flag("table"),
        link: flag("link"),
        image: flag("image"),
        image_gallery: flag("imageGallery"),
        video: flag("video"),
        audio: flag("audio"),
        math: flag("math"),
        fullscreen: flag("fullscreen"),
        showblocks: flag("showblocks"),
        code: flag("code"),
        preview: flag("preview"),
        print: flag("print"),
        save: flag("save"),
        template: flag("template"),
    };
    // Source permits Boolean dir and CSS-object style. Frui uses truthiness for
    // controls; React omits Boolean values from the non-Boolean HTML dir attribute.
    let direction_controls = flag("dir");
    let dir = prop("dir")
        .filter(|value| !value.is_boolean())
        .map(source_string)
        .unwrap_or_default();
    let value = context.value;
    let initial = prop("value")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| source_content(context.value.get_untracked()));
    let external = RwSignal::new(initial);
    let name = context.text_prop("name", &context.field.wire_name);
    let hidden_id = context.text_prop("id", "");
    let id = context.id.clone();
    let default_value = context.text_prop("defaultValue", "");
    let class_name = context.text_prop("className", "");
    let disabled = context.bool_prop("disabled", false);
    let required = context.bool_prop("required", context.input_required());
    view! {
        <FieldControl for_id=id.clone() label=context.field.label.clone()>
            <TextEditor value=external default_value name id=hidden_id editor_id=id class_name options dir direction_controls
                disabled required on_change=Callback::new(move |html: String| {
                    // One edit updates both the public draft and a parent-owned echo signal.
                    value.set(Presence::Value(Value::String(html.clone())));
                    external.set(html);
                })/>
        </FieldControl>
    }
}

/// React conditionals use JavaScript truthiness, including truthy empty objects/arrays.
pub(super) fn source_truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        Value::String(value) => !value.is_empty(),
        Value::Array(_) | Value::Object(_) => true,
    }
}

/// Source primitive-to-attribute spelling, including legacy Boolean direction metadata.
fn source_string(value: &Value) -> String {
    match value {
        Value::String(value) => value.clone(),
        Value::Null => String::new(),
        Value::Array(values) => values
            .iter()
            .map(source_string)
            .collect::<Vec<_>>()
            .join(","),
        Value::Object(_) => "[object Object]".into(),
        value => value.to_string(),
    }
}

/// Source view.html renders stored markup verbatim through frui/view/HTML's contract.
///
/// No sanitization is performed. Applications own stored HTML/content security
/// under G43; this adapter does not route values through the optional sanitizer.
#[component]
pub fn HtmlView(
    /// Source HTML value; missing/null drafts produce empty content.
    context: RoleContext,
) -> impl IntoView {
    // The ordinary Typed text adapter supplies a null-label placeholder. Frui's
    // raw HTML component instead leaves null/undefined content empty.
    let value = Signal::derive(move || source_content(context.value.get()));
    view! { <HTML value/> }
}

/// Absent nullable content is empty, without Typed text's display placeholder.
fn source_content(value: Presence<Value>) -> String {
    match value {
        Presence::Missing | Presence::Null => String::new(),
        Presence::Value(value) => source_string(&value),
    }
}
