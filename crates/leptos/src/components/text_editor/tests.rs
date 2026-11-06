//! SSR contracts and shared publication order; browser behavior has a real WASM proof.
use super::*;
use leptos::tachys::html::attribute::{any_attribute::IntoAnyAttribute, custom::custom_attribute};

/// Scope every render so node references and callbacks cannot escape their owner.
fn render(build: impl FnOnce() -> AnyView) -> String {
    let owner = Owner::new();
    let html = owner.with(|| build().to_html());
    owner.cleanup();
    html
}

/// Default toolbars are empty, HTML stays verbatim, and only the hidden field is named.
#[test]
fn editor_ssr_defaults_form_value_and_html_insertion() {
    let html = render(|| {
        view! { <TextEditor name="contents" id="hidden-id" editor_id="visible-id"
        class_name="hidden-class" value="<p><em>Raw &amp; stored</em></p>"
        attributes=vec![custom_attribute("data-proof", "exact").into_any_attr()]/> }
        .into_any()
    });
    assert!(!html.contains("<button"));
    assert!(!html.contains("<select"));
    assert!(html.contains("<p><em>Raw &amp; stored</em></p>"));
    assert!(html.contains("name=\"contents\""));
    assert!(html.contains("data-proof=\"exact\""));
    assert!(html.contains("id=\"visible-id\""));
    assert!(html.contains("class=\"hidden-class\""));
    assert_eq!(html.matches("name=").count(), 1);
    assert!(html.contains("aria-label=\"HTML Source\""));
    // HTML is a distinct raw insertion API; application-authored event attributes survive.
    let raw = render(|| {
        view! { <HTML value="<span data-kept='yes' onclick='app()'>HTML</span>"/> }.into_any()
    });
    assert!(raw.contains("<span data-kept='yes' onclick='app()'>HTML</span>"));
}

/// All source options select real controls; explicit named props can turn options back off.
#[test]
fn editor_ssr_complete_toolbar_and_explicit_overrides() {
    let options = TextEditorOptions {
        history: true,
        font: true,
        size: true,
        format: true,
        paragraph: true,
        blockquote: true,
        style: true,
        color: true,
        highlight: true,
        text: true,
        text_style: true,
        remove: true,
        indent: true,
        align: true,
        rule: true,
        list: true,
        lineheight: true,
        table: true,
        link: true,
        image: true,
        image_gallery: true,
        video: true,
        audio: true,
        math: true,
        fullscreen: true,
        showblocks: true,
        code: true,
        preview: true,
        print: true,
        save: true,
        template: true,
    };
    let html = render(|| view! { <TextEditor options dir="rtl"/> }.into_any());
    for label in [
        "Undo",
        "Redo",
        "Font",
        "Font Size",
        "Block Format",
        "Paragraph",
        "Blockquote",
        "Bold",
        "Italic",
        "Underline",
        "Text Color",
        "Highlight",
        "Strikethrough",
        "Subscript",
        "Superscript",
        "Text Style",
        "Remove Format",
        "Indent",
        "Outdent",
        "Align Left",
        "Align Center",
        "Align Right",
        "Horizontal Rule",
        "Ordered List",
        "Unordered List",
        "Line Height",
        "Insert Table",
        "Insert Link",
        "Insert Image",
        "Insert from Gallery",
        "Insert Video",
        "Insert Audio",
        "Insert Math",
        "Toggle Fullscreen",
        "Toggle Block Visibility",
        "Switch to Code View",
        "Preview Content",
        "Print",
        "Save as HTML",
        "Insert Template",
        "Toggle Text Direction",
        "Left to Right",
        "Right to Left",
    ] {
        assert!(html.contains(&format!("aria-label=\"{label}\"")), "{label}");
    }
    assert!(html.contains("sp-text-editor-rtl"));
    assert!(html.contains("value=\"72px\""));
    assert!(html.contains("value=\"h6\""));
    assert!(html.contains("value=\"1.15\""));
    assert_eq!(html.matches("type=\"file\"").count(), 2);
    assert_eq!(html.matches("multiple").count(), 1);
    assert!(html.contains("viewBox=\"0 0 24 24\""));
    let overridden =
        render(|| view! { <TextEditor options history=false style=false dir="ltr"/> }.into_any());
    assert!(!overridden.contains("aria-label=\"Undo\""));
    assert!(!overridden.contains("aria-label=\"Bold\""));
    assert!(overridden.contains("aria-label=\"Insert Audio\""));
}

/// Source initial state prefers a nonempty default, even when value is also supplied.
#[test]
fn editor_ssr_default_value_and_hidden_input_attributes() {
    let html = render(|| {
        view! { <TextEditor value="<b>controlled</b>" default_value="<i>initial</i>"
        disabled=true required=true error="reserved"/> }
        .into_any()
    });
    assert!(html.contains("<i>initial</i>"));
    assert!(!html.contains("<b>controlled</b>"));
    assert!(html.contains(" disabled"));
    assert!(html.contains(" required"));
    assert!(!html.contains("reserved"));
}

/// Command updates never duplicate onChange; exact HTML and action order reach callers.
#[test]
fn editor_shared_publication_order_and_empty_values() {
    let owner = Owner::new();
    owner.with(|| {
        let events = RwSignal::new(Vec::<String>::new());
        let editor = Editor {
            value: RwSignal::new(String::new()),
            code: RwSignal::new(false),
            editable: NodeRef::new(),
            textarea: NodeRef::new(),
            hidden: NodeRef::new(),
            file: NodeRef::new(),
            gallery: NodeRef::new(),
            on_change: Some(Callback::new(move |value: String| {
                events.update(|events| events.push(format!("change:{value}")))
            })),
            on_update: Some(Callback::new(move |update: TextEditorUpdate| {
                events.update(|events| events.push(format!("{}:{}", update.action, update.value)))
            })),
        };
        editor.publish("<b>x</b>".into(), "input");
        editor.updated("<b>x</b>".into(), "bold");
        editor.publish(String::new(), "codeViewToggle");
        assert_eq!(
            events.get_untracked(),
            [
                "change:<b>x</b>",
                "input:<b>x</b>",
                "bold:<b>x</b>",
                "change:",
                "codeViewToggle:"
            ]
        );
        assert_eq!(editor.value.get_untracked(), "");
    });
    owner.cleanup();
}
